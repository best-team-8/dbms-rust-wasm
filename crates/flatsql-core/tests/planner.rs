//! Differential tests of the planner and the executor: the same random
//! queries on the same rows, in a table with secondary indexes and in a table
//! without them, must give the same answers. The reference query on the table
//! without indexes has its WHERE clause written as `(clause) OR FALSE`, which
//! no access path can use, so it is a full scan evaluating the whole clause on
//! every row; it checks every access path of the indexed table (key and index
//! lookups, ranges, rows counted without being read).

use flatsql_core::{Database, Prepared, QueryResult, Value};

/// xorshift64* – deterministic randomness without dependencies.
struct Rng(u64);

impl Rng {
    fn next(&mut self) -> u64 {
        self.0 ^= self.0 >> 12;
        self.0 ^= self.0 << 25;
        self.0 ^= self.0 >> 27;
        self.0.wrapping_mul(0x2545_F491_4F6C_DD1D)
    }

    fn below(&mut self, n: u64) -> u64 {
        self.next() % n
    }

    fn chance(&mut self, percent: u64) -> bool {
        self.below(100) < percent
    }
}

const SCHEMA: &str = "CREATE TABLE t (id INT PRIMARY KEY, a INT, b TEXT, c FLOAT, d BOOL)";
const COLUMNS: [&str; 5] = ["id", "a", "b", "c", "d"];

/// Random value of column `col`; about 15% NULL outside the key, a few NaN.
fn value(rng: &mut Rng, col: usize, id: i64) -> Value {
    if col > 0 && rng.chance(15) {
        return Value::Null;
    }
    match col {
        0 => Value::Int(id),
        1 => Value::Int(rng.below(20) as i64),
        2 => Value::Text(format!("k{}", rng.below(10))),
        3 if rng.chance(3) => Value::Float(f64::NAN),
        3 => Value::Float(rng.below(40) as f64 * 0.5),
        _ => Value::Bool(rng.chance(50)),
    }
}

/// The same rows in a table without and with secondary indexes.
fn databases(rng: &mut Rng, rows: i64) -> (Database, Database) {
    let mut plain = Database::new(1);
    let mut indexed = Database::new(2);
    plain.execute(SCHEMA).unwrap();
    indexed.execute(SCHEMA).unwrap();
    for c in &COLUMNS[1..] {
        indexed.execute(&format!("CREATE INDEX t_{c} ON t ({c})")).unwrap();
    }
    let ins: Prepared = plain.prepare("INSERT INTO t VALUES (?, ?, ?, ?, ?)").unwrap();
    for id in 0..rows {
        // Gaps in the keys, so key lookups also miss.
        let id = id * 2;
        let row: Vec<Value> = (0..5).map(|c| value(rng, c, id)).collect();
        plain.execute_prepared(&ins, &row).unwrap();
        indexed.execute_prepared(&ins, &row).unwrap();
    }
    (plain, indexed)
}

/// Literal comparable with column `col` (INT and FLOAT columns get numbers
/// of both types, BOOL columns also 0 and 1).
fn literal(rng: &mut Rng, col: usize) -> String {
    if rng.chance(3) {
        return "NULL".into();
    }
    match col {
        0 => rng.below(700).to_string(),
        1 if rng.chance(20) => format!("{}.5", rng.below(20)),
        1 => rng.below(22).to_string(),
        2 => format!("'k{}'", rng.below(11)),
        3 if rng.chance(30) => rng.below(20).to_string(),
        3 => format!("{:.1}", rng.below(42) as f64 * 0.5),
        _ => ["TRUE", "FALSE", "0", "1"][rng.below(4) as usize].into(),
    }
}

/// One conjunct of a WHERE clause.
fn term(rng: &mut Rng) -> String {
    let col = rng.below(5) as usize;
    let name = COLUMNS[col];
    match rng.below(9) {
        0..=3 => {
            let op = ["=", "<", "<=", ">", ">=", "<>"][rng.below(6) as usize];
            if rng.chance(20) {
                format!("{} {op} {name}", literal(rng, col))
            } else {
                format!("{name} {op} {}", literal(rng, col))
            }
        }
        4 | 5 => format!("{name} BETWEEN {} AND {}", literal(rng, col), literal(rng, col)),
        6 | 7 => {
            let n = 1 + rng.below(4);
            let items: Vec<String> = (0..n).map(|_| literal(rng, col)).collect();
            format!("{name} IN ({})", items.join(", "))
        }
        _ => format!("{name} IS {}NULL", if rng.chance(50) { "NOT " } else { "" }),
    }
}

fn filter(rng: &mut Rng) -> String {
    let n = 1 + rng.below(3);
    (0..n).map(|_| term(rng)).collect::<Vec<_>>().join(" AND ")
}

fn run(db: &mut Database, sql: &str) -> QueryResult {
    db.execute(sql).unwrap_or_else(|e| panic!("{sql}\n{e}"))
}

/// Query templates; `{w}` is the WHERE clause.
const QUERIES: [&str; 5] = [
    "SELECT COUNT(*) FROM t WHERE {w}",
    "SELECT COUNT(*), COUNT(*) + 1 FROM t WHERE {w}",
    "SELECT id FROM t WHERE {w} ORDER BY id",
    "SELECT * FROM t WHERE {w} ORDER BY id DESC LIMIT 7",
    "SELECT SUM(a), COUNT(c), MAX(b) FROM t WHERE {w}",
];

/// Answer of the indexed table and of the full-scan reference.
fn both(plain: &mut Database, indexed: &mut Database, template: &str, w: &str) -> (QueryResult, QueryResult) {
    let reference = template.replace("{w}", &format!("({w}) OR FALSE"));
    let sql = template.replace("{w}", w);
    (run(indexed, &sql), run(plain, &reference))
}

#[test]
fn random_queries_give_the_same_answers_as_a_full_scan() {
    for seed in 1..=4u64 {
        let mut rng = Rng(seed.wrapping_mul(0x9E37_79B9_7F4A_7C15) | 1);
        let (mut plain, mut indexed) = databases(&mut rng, 300);
        for _ in 0..1500 {
            let w = filter(&mut rng);
            for template in QUERIES {
                let (got, want) = both(&mut plain, &mut indexed, template, &w);
                assert_eq!(got.rows, want.rows, "seed {seed}: {}", template.replace("{w}", &w));
            }
        }
    }
}

#[test]
fn count_by_index_reads_no_rows() {
    let mut rng = Rng(7);
    let (mut plain, mut indexed) = databases(&mut rng, 300);
    for (w, counted) in [
        ("b = 'k3'", true),
        ("a IN (1, 2, NULL)", true),
        // NULL is stored in the index, but never satisfies a comparison.
        ("a < 5", true),
        ("id BETWEEN 10 AND 100", true),
        ("TRUE", true),
        // Not guaranteed by the access path: the rows are read.
        ("b = 'k3' AND a > 2", false),
        ("a < 5 OR a > 10", false),
    ] {
        let (r, want) = both(&mut plain, &mut indexed, "SELECT COUNT(*) FROM t WHERE {w}", w);
        assert_eq!(r.rows, want.rows, "{w}");
        assert_eq!(r.rows_scanned == 0, counted, "{w}: {} rows read", r.rows_scanned);
        let plan = run(&mut indexed, &format!("EXPLAIN SELECT COUNT(*) FROM t WHERE {w}"));
        let text: Vec<String> = plan.rows.iter().map(|r| r[0].to_string()).collect();
        assert_eq!(text.iter().any(|l| l.contains("counted by the access path")), counted, "{text:?}");
    }
    // Other aggregates and groups need the rows.
    for sql in ["SELECT COUNT(a) FROM t WHERE b = 'k3'", "SELECT COUNT(*) FROM t WHERE b = 'k3' GROUP BY a"] {
        assert!(run(&mut indexed, sql).rows_scanned > 0, "{sql}");
    }
}
