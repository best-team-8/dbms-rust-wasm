//! Native benchmark. Runs the workload of `bench/suite.js` (same data, same
//! parameter seeds, same result checksum) without WebAssembly and without a
//! JavaScript boundary, so the numbers show the cost of WebAssembly.
//!
//! `WORKLOAD` must match `WORKLOAD` in `bench/suite.js`; the runner refuses to
//! mix results of different workloads.
//!
//! With `--storage DIR` the benchmark measures durable commits on files
//! (`FileStorage`) with and without fsync instead.

use std::fmt::Write as _;
use std::path::{Path, PathBuf};
use std::time::Instant;

use flatsql_core::storage::file::FileStorage;
use flatsql_core::storage::Persistent;
use flatsql_core::{Database, Prepared, Value};

const WORKLOAD: &str = "benchmark-1";
const SCHEMA: &str = "CREATE TABLE items (id INTEGER PRIMARY KEY, category TEXT, price FLOAT, qty INT, name TEXT)";
const INDEX: &str = "CREATE INDEX items_category ON items (category)";
const INSERT: &str = "INSERT INTO items VALUES (?, ?, ?, ?, ?)";
const VERIFY: &str = "SELECT COUNT(*), SUM(qty), SUM(price) FROM items";
const HEAD: usize = 3;
const WARM_ROWS: i64 = 10_000;
const WARM_ITERS: usize = 200;

/// Linear congruential generator of `bench/suite.js` (`Rng`).
struct Lcg(u64);

impl Lcg {
    fn below(&mut self, n: u64) -> u64 {
        self.0 = self.0.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        (self.0 >> 33) % n
    }
}

pub fn row(i: i64) -> Vec<Value> {
    vec![
        Value::Int(i),
        Value::Text(format!("c{}", i % 100)),
        Value::Float(((i * 7919) % 10000) as f64 / 100.0),
        Value::Int(i % 50),
        Value::Text(format!("item-{i}")),
    ]
}

#[derive(Clone, Copy)]
enum Params {
    None,
    Id,
    Category,
    Range,
}

struct ReadOp {
    name: &'static str,
    n: usize,
    sql: &'static str,
    params: Params,
    ordered: bool,
}

const READ_OPS: [ReadOp; 7] = [
    ReadOp { name: "point_lookup", n: 10_000, sql: "SELECT * FROM items WHERE id = ?", params: Params::Id, ordered: true },
    ReadOp { name: "index_lookup", n: 500, sql: "SELECT COUNT(*) FROM items WHERE category = ?", params: Params::Category, ordered: true },
    ReadOp { name: "range_scan", n: 200, sql: "SELECT SUM(price) FROM items WHERE id BETWEEN ? AND ?", params: Params::Range, ordered: true },
    ReadOp { name: "full_scan", n: 20, sql: "SELECT COUNT(*) FROM items WHERE price > 50 AND qty < 10", params: Params::None, ordered: true },
    ReadOp {
        name: "group_by",
        n: 20,
        sql: "SELECT category, COUNT(*), AVG(price), MAX(qty) FROM items GROUP BY category",
        params: Params::None,
        ordered: false,
    },
    ReadOp { name: "top_k", n: 20, sql: "SELECT id, price FROM items ORDER BY price DESC, id ASC LIMIT 10", params: Params::None, ordered: true },
    ReadOp { name: "materialize_10k", n: 20, sql: "SELECT * FROM items WHERE id < 10000", params: Params::None, ordered: false },
];

const WRITE_OPS: [(&str, &str); 2] = [
    ("update_10pct", "UPDATE items SET qty = qty + 1 WHERE id % 10 = 0"),
    ("delete_10pct", "DELETE FROM items WHERE id % 10 = 1"),
];

fn gen_params(p: Params, rng: &mut Lcg, rows: i64) -> Vec<Value> {
    match p {
        Params::None => vec![],
        Params::Id => vec![Value::Int(rng.below(rows as u64) as i64)],
        Params::Category => vec![Value::Text(format!("c{}", rng.below(100)))],
        Params::Range => {
            let a = rng.below((rows - 1000).max(1) as u64) as i64;
            vec![Value::Int(a), Value::Int(a + 999)]
        }
    }
}

// ---- checksum (canon and Checksum of bench/suite.js) ------------------------

fn canon(v: &Value) -> String {
    match v {
        Value::Null => "N".into(),
        Value::Bool(b) => if *b { "1000000" } else { "0" }.into(),
        Value::Int(i) => (*i as i128 * 1_000_000).to_string(),
        Value::Float(f) => {
            let r = (f * 1e6).round();
            if r == 0.0 { "0".into() } else { format!("{}", r as i64) }
        }
        Value::Text(s) => format!("S{s}"),
    }
}

struct Checksum {
    h1: u32,
    h2: u32,
}

impl Checksum {
    fn new() -> Self {
        Checksum { h1: 0x811c9dc5, h2: 0x050c5d1f }
    }

    fn text(&mut self, s: &str) {
        for c in s.bytes() {
            self.h1 = (self.h1 ^ c as u32).wrapping_mul(0x01000193);
            self.h2 = (self.h2 ^ c as u32).wrapping_mul(0x5bd1e995);
        }
    }

    fn result(&mut self, rows: &[Vec<Value>], ordered: bool) {
        let mut lines: Vec<String> = rows.iter().map(|r| r.iter().map(canon).collect::<Vec<_>>().join(",")).collect();
        if !ordered {
            lines.sort();
        }
        for l in &lines {
            self.text(l);
            self.text("|");
        }
        self.text(";");
    }

    fn hex(&self) -> String {
        format!("{:08x}{:08x}", self.h1, self.h2)
    }
}

// ---- measurement --------------------------------------------------------------

struct OpStat {
    name: &'static str,
    n: usize,
    mean_us: f64,
    samples_us: Vec<f64>,
    extra: String,
}

impl OpStat {
    fn single(name: &'static str, us: f64, extra: String) -> Self {
        OpStat { name, n: 1, mean_us: us, samples_us: vec![us], extra }
    }

    fn json(&self, out: &mut String) {
        let mut s = self.samples_us.clone();
        s.sort_by(|a, b| a.total_cmp(b));
        let pick = |p: f64| s[((s.len() as f64 - 1.0) * p).round() as usize];
        let _ = write!(
            out,
            "\"{}\":{{\"n\":{},\"meanUs\":{:.3},\"p50Us\":{:.3},\"p95Us\":{:.3},\"p99Us\":{:.3},\"maxUs\":{:.3}{}}}",
            self.name,
            self.n,
            self.mean_us,
            pick(0.5),
            pick(0.95),
            pick(0.99),
            s[s.len() - 1],
            self.extra
        );
    }
}

fn us_since(t: Instant) -> f64 {
    t.elapsed().as_secs_f64() * 1e6
}

fn exec(db: &mut Database, p: &Prepared, params: &[Value]) -> Vec<Vec<Value>> {
    db.execute_prepared(p, params).expect("benchmark statement failed").rows
}

fn create(db: &mut Database) {
    db.execute(SCHEMA).unwrap();
    db.execute(INDEX).unwrap();
}

fn load(db: &mut Database, rows: i64) {
    let ins = db.prepare(INSERT).unwrap();
    db.execute("BEGIN").unwrap();
    for i in 0..rows {
        exec(db, &ins, &row(i));
    }
    db.execute("COMMIT").unwrap();
    db.take_wal();
}

fn verify(db: &mut Database) -> Vec<String> {
    db.execute(VERIFY).unwrap().rows[0].iter().map(canon).collect()
}

fn json_strings(v: &[String]) -> String {
    format!("[{}]", v.iter().map(|s| format!("\"{s}\"")).collect::<Vec<_>>().join(","))
}

fn workload(rows: i64) -> String {
    // Warm-up on a throw-away database, as in the JavaScript runners.
    {
        let mut db = Database::new(1);
        create(&mut db);
        let warm = rows.min(WARM_ROWS);
        load(&mut db, warm);
        let mut rng = Lcg(7);
        for op in &READ_OPS {
            let p = db.prepare(op.sql).unwrap();
            for _ in 0..WARM_ITERS {
                let ps = gen_params(op.params, &mut rng, warm);
                exec(&mut db, &p, &ps);
            }
        }
    }

    let mut stats: Vec<OpStat> = Vec::new();
    let mut checks = String::new();
    let mut state: Vec<(&str, Vec<String>)> = Vec::new();
    let mut db = Database::new(1);
    create(&mut db);

    let t = Instant::now();
    load(&mut db, rows);
    let us = us_since(t);
    stats.push(OpStat::single("insert", us, format!(",\"rowsPerSec\":{:.1}", rows as f64 / (us / 1e6))));
    state.push(("afterInsert", verify(&mut db)));

    for (k, op) in READ_OPS.iter().enumerate() {
        let p = db.prepare(op.sql).unwrap();
        let mut rng = Lcg(1000 + k as u64);
        let params: Vec<Vec<Value>> = (0..op.n).map(|_| gen_params(op.params, &mut rng, rows)).collect();
        for ps in params.iter().take((op.n / 10).max(1)) {
            exec(&mut db, &p, ps);
        }
        let mut samples = Vec::with_capacity(op.n);
        let mut results = Vec::with_capacity(op.n);
        let start = Instant::now();
        for ps in &params {
            let t0 = Instant::now();
            let r = exec(&mut db, &p, ps);
            samples.push(us_since(t0));
            results.push(r);
        }
        let total = us_since(start);
        let (mut head, mut full) = (Checksum::new(), Checksum::new());
        for (i, r) in results.iter().enumerate() {
            if i < HEAD {
                head.result(r, op.ordered);
            }
            full.result(r, op.ordered);
        }
        if !checks.is_empty() {
            checks.push(',');
        }
        let _ = write!(
            checks,
            "\"{}\":{{\"head\":\"{}\",\"full\":\"{}\",\"n\":{},\"rows\":{}}}",
            op.name,
            head.hex(),
            full.hex(),
            op.n,
            results.last().map_or(0, Vec::len)
        );
        stats.push(OpStat { name: op.name, n: op.n, mean_us: total / op.n as f64, samples_us: samples, extra: String::new() });
    }

    for (name, sql) in WRITE_OPS {
        let t = Instant::now();
        db.execute(sql).unwrap();
        db.take_wal();
        stats.push(OpStat::single(name, us_since(t), String::new()));
        state.push((if name == "update_10pct" { "after_update_10pct" } else { "after_delete_10pct" }, verify(&mut db)));
    }

    let t = Instant::now();
    let snap = db.snapshot().unwrap();
    stats.push(OpStat::single("snapshot", us_since(t), format!(",\"bytes\":{}", snap.len())));
    let t = Instant::now();
    let mut restored = Database::restore(Some(&snap), &[], 1).unwrap();
    stats.push(OpStat::single("restore", us_since(t), String::new()));
    state.push(("afterRestore", verify(&mut restored)));

    let mut out = String::new();
    let _ = write!(out, "{{\"engine\":\"flatsql-native\",\"workload\":\"{WORKLOAD}\",\"rows\":{rows},\"ops\":{{");
    for (i, s) in stats.iter().enumerate() {
        if i > 0 {
            out.push(',');
        }
        s.json(&mut out);
    }
    let _ = write!(out, "}},\"checks\":{{{checks}}},\"stateValues\":{{");
    for (i, (k, v)) in state.iter().enumerate() {
        if i > 0 {
            out.push(',');
        }
        let _ = write!(out, "\"{k}\":{}", json_strings(v));
    }
    out.push_str("}}");
    out
}

/// Durable commits on files: the same phases as the durable group of the
/// browser benchmark, with the default checkpoint threshold of `Persistent`.
fn durable(dir: &Path, rows: i64, fsync: bool) -> String {
    let _ = std::fs::remove_dir_all(dir);
    let open = || Persistent::open(FileStorage::open(dir).unwrap().with_fsync(fsync), 1).unwrap();
    let mut p = open();
    p.execute(SCHEMA).unwrap();
    p.execute(INDEX).unwrap();
    let ins = p.db().prepare(INSERT).unwrap();

    let mut stats = Vec::new();
    let t = Instant::now();
    p.db_mut().execute("BEGIN").unwrap();
    for i in 0..rows {
        p.db_mut().execute_prepared(&ins, &row(i)).unwrap();
    }
    p.db_mut().execute("COMMIT").unwrap();
    p.flush().unwrap();
    stats.push(OpStat::single("insert", us_since(t), String::new()));

    let mut samples = Vec::new();
    let start = Instant::now();
    for i in 0..100 {
        let t0 = Instant::now();
        p.db_mut().execute_prepared(&ins, &row(10_000_000 + i)).unwrap();
        p.flush().unwrap();
        samples.push(us_since(t0));
    }
    stats.push(OpStat { name: "commit_single", n: 100, mean_us: us_since(start) / 100.0, samples_us: samples, extra: String::new() });

    let t = Instant::now();
    for i in 0..1000 {
        p.db_mut().execute_prepared(&ins, &row(10_000_100 + i)).unwrap();
    }
    p.flush().unwrap();
    let us = us_since(t);
    stats.push(OpStat::single("commit_group", us, format!(",\"perCommitUs\":{:.3}", us / 1000.0)));

    let t = Instant::now();
    p.checkpoint().unwrap();
    stats.push(OpStat::single("checkpoint", us_since(t), String::new()));
    drop(p);

    let t = Instant::now();
    let p = open();
    stats.push(OpStat::single("reopen", us_since(t), format!(",\"rows\":{}", p.db().row_count("items").unwrap_or(0))));
    drop(p);
    let _ = std::fs::remove_dir_all(dir);

    let mut out = String::from("{");
    for (i, s) in stats.iter().enumerate() {
        if i > 0 {
            out.push(',');
        }
        s.json(&mut out);
    }
    out.push('}');
    out
}

pub fn main(args: &[String]) -> i32 {
    let mut rows: i64 = 100_000;
    let mut json_out = None;
    let mut storage: Option<PathBuf> = None;
    let mut it = args.iter();
    while let Some(a) = it.next() {
        match a.as_str() {
            "--rows" => rows = it.next().and_then(|v| v.parse().ok()).unwrap_or(rows),
            "--json" => json_out = it.next().cloned(),
            "--storage" => storage = it.next().map(PathBuf::from),
            other => {
                eprintln!("unknown bench option {other}");
                return 2;
            }
        }
    }

    let result = match &storage {
        None => workload(rows),
        Some(dir) => format!(
            "{{\"engine\":\"flatsql-native\",\"workload\":\"{WORKLOAD}\",\"rows\":{rows},\"fsync\":{},\"nofsync\":{}}}",
            durable(dir, rows, true),
            durable(dir, rows, false)
        ),
    };
    let out = format!(
        "{{\"workload\":\"{WORKLOAD}\",\"native\":true,\"os\":\"{}\",\"task\":{{\"kind\":\"{}\"}},\"result\":{result}}}\n",
        std::env::consts::OS,
        if storage.is_some() { "fsync" } else { "workload" }
    );
    match json_out {
        Some(path) => {
            if let Err(e) = std::fs::write(&path, out) {
                eprintln!("cannot write {path}: {e}");
                return 1;
            }
            println!("results written to {path}");
        }
        None => print!("{out}"),
    }
    0
}
