//! Native benchmark. Runs the same workload as `bench/suite.js` so the
//! numbers show the cost of WebAssembly compared to native code.

use std::time::Instant;

use flatsql_core::{Database, Prepared, Value};

struct Lcg(u64);

impl Lcg {
    fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        self.0 >> 33
    }

    fn below(&mut self, n: u64) -> u64 {
        self.next() % n
    }
}

/// Produces the parameters of the next benchmark iteration.
type ParamGen = Box<dyn FnMut(&mut Lcg) -> Vec<Value>>;

struct Stat {
    name: &'static str,
    samples_us: Vec<f64>,
    note: String,
}

impl Stat {
    fn percentile(sorted: &[f64], p: f64) -> f64 {
        if sorted.is_empty() {
            return 0.0;
        }
        let idx = ((sorted.len() as f64 - 1.0) * p).round() as usize;
        sorted[idx]
    }

    fn summary(&self) -> (f64, f64, f64, f64, f64) {
        let mut s = self.samples_us.clone();
        s.sort_by(|a, b| a.total_cmp(b));
        let total: f64 = s.iter().sum();
        let mean = total / s.len().max(1) as f64;
        (mean, Self::percentile(&s, 0.5), Self::percentile(&s, 0.95), Self::percentile(&s, 0.99), total / 1000.0)
    }
}

fn time<T>(f: impl FnOnce() -> T) -> (T, f64) {
    let t = Instant::now();
    let r = f();
    (r, t.elapsed().as_secs_f64() * 1e6)
}

fn run(db: &mut Database, p: &Prepared, params: &[Value]) {
    db.execute_prepared(p, params).expect("benchmark statement failed");
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

pub fn main(args: &[String]) -> i32 {
    let mut rows: i64 = 100_000;
    let mut json_out = None;
    let mut it = args.iter();
    while let Some(a) = it.next() {
        match a.as_str() {
            "--rows" => rows = it.next().and_then(|v| v.parse().ok()).unwrap_or(rows),
            "--json" => json_out = it.next().cloned(),
            other => {
                eprintln!("unknown bench option {other}");
                return 2;
            }
        }
    }

    println!("Flat-SQL native benchmark – {rows} rows");
    let mut rng = Lcg(42);
    let mut stats: Vec<Stat> = Vec::new();
    let mut db = Database::new(1);
    db.execute(
        "CREATE TABLE items (id INT PRIMARY KEY, category TEXT, price FLOAT, qty INT, name TEXT);
         CREATE INDEX items_category ON items (category);",
    )
    .unwrap();

    // insert: one transaction, prepared statement per row
    let ins = db.prepare("INSERT INTO items VALUES (?, ?, ?, ?, ?)").unwrap();
    let (_, us) = time(|| {
        db.execute("BEGIN").unwrap();
        for i in 0..rows {
            run(&mut db, &ins, &row(i));
        }
        db.execute("COMMIT").unwrap();
    });
    let wal = db.take_wal().len();
    stats.push(Stat {
        name: "insert",
        samples_us: vec![us],
        note: format!("{:.0} rows/s, WAL {:.1} MB", rows as f64 / (us / 1e6), wal as f64 / 1e6),
    });

    let mut measure = |db: &mut Database, name: &'static str, n: usize, sql: &str, mut params: ParamGen| {
        let p = db.prepare(sql).unwrap();
        // warm-up
        for _ in 0..(n / 10).max(1) {
            let ps = params(&mut rng);
            run(db, &p, &ps);
        }
        let mut samples = Vec::with_capacity(n);
        for _ in 0..n {
            let ps = params(&mut rng);
            let ((), us) = time(|| run(db, &p, &ps));
            samples.push(us);
        }
        stats.push(Stat { name, samples_us: samples, note: String::new() });
    };

    measure(
        &mut db,
        "point_lookup",
        10_000,
        "SELECT * FROM items WHERE id = ?",
        Box::new(move |r| vec![Value::Int(r.below(rows as u64) as i64)]),
    );
    measure(
        &mut db,
        "index_lookup",
        500,
        "SELECT COUNT(*) FROM items WHERE category = ?",
        Box::new(|r| vec![Value::Text(format!("c{}", r.below(100)))]),
    );
    measure(
        &mut db,
        "range_scan",
        200,
        "SELECT SUM(price) FROM items WHERE id BETWEEN ? AND ?",
        Box::new(move |r| {
            let a = r.below((rows - 1000).max(1) as u64) as i64;
            vec![Value::Int(a), Value::Int(a + 999)]
        }),
    );
    measure(&mut db, "full_scan", 20, "SELECT COUNT(*) FROM items WHERE price > 50 AND qty < 10", Box::new(|_| vec![]));
    measure(
        &mut db,
        "group_by",
        20,
        "SELECT category, COUNT(*), AVG(price), MAX(qty) FROM items GROUP BY category",
        Box::new(|_| vec![]),
    );
    measure(&mut db, "top_k", 20, "SELECT id, price FROM items ORDER BY price DESC LIMIT 10", Box::new(|_| vec![]));

    let (_, us) = time(|| db.execute("UPDATE items SET qty = qty + 1 WHERE id % 10 = 0").unwrap());
    stats.push(Stat { name: "update_10pct", samples_us: vec![us], note: String::new() });
    let (_, us) = time(|| db.execute("DELETE FROM items WHERE id % 10 = 1").unwrap());
    stats.push(Stat { name: "delete_10pct", samples_us: vec![us], note: String::new() });
    db.take_wal();

    let mut snap = Vec::new();
    let mut snap_samples = Vec::new();
    for _ in 0..5 {
        let (s, us) = time(|| db.snapshot().unwrap());
        snap = s;
        snap_samples.push(us);
    }
    stats.push(Stat { name: "snapshot", samples_us: snap_samples, note: format!("{:.1} MB", snap.len() as f64 / 1e6) });
    let mut restore_samples = Vec::new();
    for _ in 0..5 {
        let (_, us) = time(|| Database::restore(Some(&snap), &[], 1).unwrap());
        restore_samples.push(us);
    }
    stats.push(Stat { name: "restore", samples_us: restore_samples, note: String::new() });

    println!(
        "{:<14} {:>8} {:>12} {:>12} {:>12} {:>12} {:>12}  note",
        "operation", "n", "mean µs", "p50 µs", "p95 µs", "p99 µs", "total ms"
    );
    for s in &stats {
        let (mean, p50, p95, p99, total) = s.summary();
        println!(
            "{:<14} {:>8} {:>12.2} {:>12.2} {:>12.2} {:>12.2} {:>12.2}  {}",
            s.name,
            s.samples_us.len(),
            mean,
            p50,
            p95,
            p99,
            total,
            s.note
        );
    }

    if let Some(path) = json_out {
        let mut out = format!("{{\"env\":\"native\",\"engine\":\"flatsql-native\",\"rows\":{rows},\"results\":{{");
        for (i, s) in stats.iter().enumerate() {
            let (mean, p50, p95, p99, total) = s.summary();
            if i > 0 {
                out.push(',');
            }
            out.push_str(&format!(
                "\"{}\":{{\"n\":{},\"meanUs\":{mean:.3},\"p50Us\":{p50:.3},\"p95Us\":{p95:.3},\"p99Us\":{p99:.3},\"totalMs\":{total:.3}}}",
                s.name,
                s.samples_us.len()
            ));
        }
        out.push_str("}}\n");
        if let Err(e) = std::fs::write(&path, out) {
            eprintln!("cannot write {path}: {e}");
            return 1;
        }
        println!("results written to {path}");
    }
    0
}
