//! Native split of the engine cost of three Benchmark 1 weaknesses, using
//! only the public API of `flatsql-core`:
//!
//! - point lookup: whole prepared statement vs. its parts (binding and access
//!   path choice, tree lookup, row copy, column names and message),
//! - full scan and GROUP BY: engine vs. a hand-written loop over the same tree,
//! - snapshot, restore and UPDATE with the outbox of 100 000 local inserts and
//!   with an empty outbox (all entries acknowledged).
//!
//! One process, 3 rounds; an orientation measurement, not a Benchmark 1 run.
//!
//!   cargo run --release --manifest-path docs/research/data/benchmark-1/analysis/cost-split/Cargo.toml --target-dir target/analysis

use std::hint::black_box;
use std::time::Instant;

use flatsql_core::expr::Binder;
use flatsql_core::plan;
use flatsql_core::sql::{parse_one, Statement};
use flatsql_core::table::Table;
use flatsql_core::{Database, Stamp, Value};

const ROWS: i64 = 100_000;
const LOOKUPS: usize = 200_000;
const ROUNDS: usize = 3;

/// Row `i` of `bench/suite.js` (`makeRow`).
fn row(i: i64) -> Vec<Value> {
    vec![
        Value::Int(i),
        Value::Text(format!("c{}", i % 100)),
        Value::Float(((i * 7919) % 10000) as f64 / 100.0),
        Value::Int(i % 50),
        Value::Text(format!("item-{i}")),
    ]
}

/// Linear congruential generator of `bench/suite.js` (`Rng`).
struct Lcg(u64);

impl Lcg {
    fn below(&mut self, n: u64) -> u64 {
        self.0 = self.0.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        (self.0 >> 33) % n
    }
}

fn per_op_ns(t: Instant, n: usize) -> f64 {
    t.elapsed().as_nanos() as f64 / n as f64
}

fn ms(t: Instant) -> f64 {
    t.elapsed().as_secs_f64() * 1e3
}

fn main() {
    let mut db = Database::new(1);
    db.execute("CREATE TABLE items (id INTEGER PRIMARY KEY, category TEXT, price FLOAT, qty INT, name TEXT)").unwrap();
    db.execute("CREATE INDEX items_category ON items (category)").unwrap();
    let ins = db.prepare("INSERT INTO items VALUES (?, ?, ?, ?, ?)").unwrap();
    db.execute("BEGIN").unwrap();
    for i in 0..ROWS {
        db.execute_prepared(&ins, &row(i)).unwrap();
    }
    db.execute("COMMIT").unwrap();
    db.take_wal();

    // The same rows in a table outside the database, for the raw costs.
    let schema = db.table_schema("items").unwrap().clone();
    let mut t = Table::new(schema.clone(), Stamp::default());
    for i in 0..ROWS {
        t.put(row(i), Stamp::default());
    }
    t.add_index("items_category".into(), 1);

    let mut rng = Lcg(42);
    let keys: Vec<i64> = (0..LOOKUPS).map(|_| rng.below(ROWS as u64) as i64).collect();
    let star = db.prepare("SELECT * FROM items WHERE id = ?").unwrap();
    let one = db.prepare("SELECT id FROM items WHERE id = ?").unwrap();
    let Statement::Select(sel) = parse_one("SELECT * FROM items WHERE id = ?").unwrap() else { unreachable!() };
    let selection = sel.selection.unwrap();

    println!("point lookup, ns per call (SELECT * | SELECT id | binding + access path | tree lookup | lookup + row copy | names + message)");
    for round in 1..=ROUNDS {
        let t0 = Instant::now();
        for &k in &keys {
            black_box(db.execute_prepared(&star, &[Value::Int(k)]).unwrap());
        }
        let whole = per_op_ns(t0, LOOKUPS);
        let t0 = Instant::now();
        for &k in &keys {
            black_box(db.execute_prepared(&one, &[Value::Int(k)]).unwrap());
        }
        let id_only = per_op_ns(t0, LOOKUPS);
        let t0 = Instant::now();
        for &k in &keys {
            let params = [Value::Int(k)];
            let filter = Binder::new(Some(&schema), &params).bind(&selection, "WHERE").unwrap();
            black_box(plan::choose(&t, Some(&filter)).unwrap());
        }
        let planning = per_op_ns(t0, LOOKUPS);
        let t0 = Instant::now();
        for &k in &keys {
            black_box(t.rows.get(&Value::Int(k)).map(|e| e.stamp));
        }
        let lookup = per_op_ns(t0, LOOKUPS);
        let t0 = Instant::now();
        for &k in &keys {
            black_box(t.rows.get(&Value::Int(k)).map(|e| e.values.clone()));
        }
        let copy = per_op_ns(t0, LOOKUPS);
        let t0 = Instant::now();
        for _ in &keys {
            let names: Vec<String> = schema.columns.iter().map(|c| c.name.clone()).collect();
            black_box((names, format!("SELECT {}", 1)));
        }
        let names = per_op_ns(t0, LOOKUPS);
        println!("  round {round}: {whole:.0} | {id_only:.0} | {planning:.0} | {lookup:.0} | {copy:.0} | {names:.0}");
    }

    let scan = db.prepare("SELECT COUNT(*) FROM items WHERE price > 50 AND qty < 10").unwrap();
    let group = db.prepare("SELECT category, COUNT(*), AVG(price), MAX(qty) FROM items GROUP BY category").unwrap();
    println!("full scan and GROUP BY, ms (engine scan | hand loop, same filter | tree walk only | engine GROUP BY)");
    for round in 1..=ROUNDS {
        let t0 = Instant::now();
        for _ in 0..20 {
            black_box(db.execute_prepared(&scan, &[]).unwrap());
        }
        let engine = ms(t0) / 20.0;
        let t0 = Instant::now();
        for _ in 0..20 {
            let mut n = 0u64;
            for e in t.rows.values() {
                let p = matches!(e.values[2], Value::Float(p) if p > 50.0);
                let q = matches!(e.values[3], Value::Int(q) if q < 10);
                n += (p && q) as u64;
            }
            black_box(n);
        }
        let hand = ms(t0) / 20.0;
        let t0 = Instant::now();
        for _ in 0..20 {
            let mut n = 0u64;
            for e in t.rows.values() {
                n += e.stamp.time & 1;
            }
            black_box(n);
        }
        let walk = ms(t0) / 20.0;
        let t0 = Instant::now();
        for _ in 0..20 {
            black_box(db.execute_prepared(&group, &[]).unwrap());
        }
        let grouped = ms(t0) / 20.0;
        println!("  round {round}: {engine:.2} | {hand:.2} | {walk:.2} | {grouped:.2}");
    }

    let with = db.snapshot().unwrap();
    let mut acked = Database::restore(Some(&with), &[], 1).unwrap();
    let upto = acked.stats().next_seq;
    acked.ack_outbox(upto);
    let without = acked.snapshot().unwrap();
    println!(
        "outbox entries {} -> {}; snapshot {} bytes with the outbox, {} without",
        db.stats().outbox,
        acked.stats().outbox,
        with.len(),
        without.len()
    );
    println!("snapshot, restore, index rebuild and UPDATE 10%, ms (with outbox / without)");
    for round in 1..=ROUNDS {
        let t0 = Instant::now();
        black_box(db.snapshot().unwrap());
        let snap_with = ms(t0);
        let t0 = Instant::now();
        black_box(acked.snapshot().unwrap());
        let snap_without = ms(t0);
        let t0 = Instant::now();
        let mut a = Database::restore(Some(&with), &[], 1).unwrap();
        let restore_with = ms(t0);
        let t0 = Instant::now();
        let mut b = Database::restore(Some(&without), &[], 1).unwrap();
        let restore_without = ms(t0);
        let mut bare = Table::new(schema.clone(), Stamp::default());
        bare.rows = t.rows.clone();
        let t0 = Instant::now();
        bare.add_index("ix".into(), 1);
        let index = ms(t0);
        black_box(bare);
        let t0 = Instant::now();
        a.execute("UPDATE items SET qty = qty + 1 WHERE id % 10 = 0").unwrap();
        let update_with = ms(t0);
        let t0 = Instant::now();
        b.execute("UPDATE items SET qty = qty + 1 WHERE id % 10 = 0").unwrap();
        let update_without = ms(t0);
        let t0 = Instant::now();
        black_box(b.execute("SELECT COUNT(*) FROM items WHERE id % 10 = 0").unwrap());
        let filter = ms(t0);
        println!(
            "  round {round}: snapshot {snap_with:.1} / {snap_without:.1} | restore {restore_with:.1} / {restore_without:.1} | index rebuild {index:.1} | UPDATE {update_with:.1} / {update_without:.1} (its WHERE alone {filter:.1})"
        );
    }
}
