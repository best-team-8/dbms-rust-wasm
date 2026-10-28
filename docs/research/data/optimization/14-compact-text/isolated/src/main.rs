use std::hint::black_box;
use std::time::Instant;

use flatsql_core::{Database, Value};

fn row(i: i64) -> Vec<Value> {
    vec![
        Value::Int(i),
        Value::Text(format!("c{}", i % 100).into()),
        Value::Float(((i * 7919) % 10000) as f64 / 100.0),
        Value::Int(i % 50),
        Value::Text(format!("item-{i}").into()),
    ]
}

fn main() {
    let mut db = Database::new(1);
    db.execute("CREATE TABLE items (id INTEGER PRIMARY KEY, category TEXT, price FLOAT, qty INT, name TEXT)").unwrap();
    db.execute("CREATE INDEX items_category ON items (category)").unwrap();
    let ins = db.prepare("INSERT INTO items VALUES (?, ?, ?, ?, ?)").unwrap();
    db.execute("BEGIN").unwrap();
    for i in 0..100_000 {
        db.execute_prepared(&ins, &row(i)).unwrap();
    }
    db.execute("COMMIT").unwrap();
    db.take_wal();
    let queries = [
        ("top_k", "SELECT id, price FROM items ORDER BY price DESC, id ASC LIMIT 10"),
        ("top_k id only", "SELECT id FROM items ORDER BY id DESC LIMIT 10"),
        ("order by text", "SELECT id FROM items ORDER BY name DESC LIMIT 10"),
        ("group_by", "SELECT category, COUNT(*), AVG(price), MAX(qty) FROM items GROUP BY category"),
        ("group_by int", "SELECT qty, COUNT(*), AVG(price) FROM items GROUP BY qty"),
        ("full_scan", "SELECT COUNT(*) FROM items WHERE price > 50 AND qty < 10"),
    ];
    for (name, sql) in queries {
        let p = db.prepare(sql).unwrap();
        let mut best = f64::MAX;
        for _ in 0..7 {
            let t = Instant::now();
            for _ in 0..20 {
                black_box(db.execute_prepared(&p, &[]).unwrap());
            }
            best = best.min(t.elapsed().as_secs_f64() * 1e3 / 20.0);
        }
        println!("{name:14} {best:6.2} ms");
    }
}
