//! Native split of the engine cost of three Benchmark 1 weaknesses, using
//! only the public API of `flatsql-core`:
//!
//! - point lookup: whole prepared statement vs. its parts (binding and access
//!   path choice, tree lookup, row copy, column names and message),
//! - a result in the binary result format of the WebAssembly binding: the
//!   collected result encoded vs. rows encoded straight from the table, for a
//!   point lookup and for 10 000 rows (core with the encoder),
//! - COUNT(*) by the secondary index vs. reading only the index, a key range
//!   with SUM vs. the same range walked by hand with and without the WHERE
//!   clause, and the time to prepare each statement of the workload,
//! - full scan and GROUP BY: engine vs. a hand-written loop over the same tree,
//! - snapshot, restore and UPDATE with the outbox of 100 000 local inserts and
//!   with an empty outbox (all entries acknowledged),
//! - 100 000 inserts in one transaction: a prepared statement executed for
//!   each row vs. executed for all rows in one call (core with that call).
//!
//! One process, 3 rounds; an orientation measurement, not a Benchmark 1 run.
//!
//!   cargo run --release --manifest-path docs/research/data/benchmark-1/analysis/cost-split/Cargo.toml --target-dir target/analysis

use std::hint::black_box;
use std::time::Instant;

use cost_split::{load, row, with_outbox, Lcg, ROWS};
use flatsql_core::expr::{eval_pred, Binder};
use flatsql_core::plan;
use flatsql_core::sql::{parse_one, Statement};
use flatsql_core::table::Table;
use flatsql_core::{Database, Stamp, Value};

const LOOKUPS: usize = 200_000;

/// Evaluation context without aggregates and parameters (the argument
/// changed from a slice of aggregate values to a context with parameters).
#[cfg(not(core_cx))]
const EMPTY: &[Value] = &[];
#[cfg(core_cx)]
const EMPTY: flatsql_core::expr::Cx<'static> = flatsql_core::expr::Cx { aggs: &[], params: &[] };
const ROUNDS: usize = 3;

fn per_op_ns(t: Instant, n: usize) -> f64 {
    t.elapsed().as_nanos() as f64 / n as f64
}

fn ms(t: Instant) -> f64 {
    t.elapsed().as_secs_f64() * 1e3
}

fn main() {
    let mut db = load();

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
        // The work done for the WHERE clause at every execution: binding with
        // the parameter as a literal and choosing the access path (core
        // without compiled statements), or filling the access path template
        // with the parameter (core with compiled statements).
        #[cfg(not(core_access_plan))]
        let t0 = {
            let t0 = Instant::now();
            for &k in &keys {
                let params = [Value::Int(k)];
                let filter = Binder::new(Some(&schema), &params).bind(&selection, "WHERE").unwrap();
                black_box(plan::choose(&t, Some(&filter)).unwrap());
            }
            t0
        };
        #[cfg(core_access_plan)]
        let t0 = {
            let terms = Binder::deferred(Some(&schema)).bind(&selection, "WHERE").unwrap().into_conjuncts();
            let access = plan::AccessPlan::new(&t, &terms).unwrap();
            let t0 = Instant::now();
            for &k in &keys {
                black_box(access.resolve(&t, &terms, &[Value::Int(k)]).unwrap());
            }
            t0
        };
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
        // Column names and command tag of every result: copied and formatted
        // (core without compiled statements), or shared and empty (core with
        // compiled statements).
        #[cfg(not(core_access_plan))]
        let t0 = {
            let t0 = Instant::now();
            for _ in &keys {
                let names: Vec<String> = schema.columns.iter().map(|c| c.name.clone()).collect();
                black_box((names, format!("SELECT {}", 1)));
            }
            t0
        };
        #[cfg(core_access_plan)]
        let t0 = {
            let shared: std::sync::Arc<[String]> = schema.columns.iter().map(|c| c.name.clone()).collect();
            let t0 = Instant::now();
            for _ in &keys {
                black_box((std::sync::Arc::clone(&shared), String::new()));
            }
            t0
        };
        let names = per_op_ns(t0, LOOKUPS);
        println!("  round {round}: {whole:.0} | {id_only:.0} | {planning:.0} | {lookup:.0} | {copy:.0} | {names:.0}");
    }

    #[cfg(core_encode)]
    {
        let many = db.prepare("SELECT * FROM items WHERE id < 10000").unwrap();
        let mut out = Vec::new();
        println!("binary result, ns per lookup (collected + encoded | encoded from the table), ms per 10 000 rows (collected | collected + encoded | encoded from the table)");
        for round in 1..=ROUNDS {
            let t0 = Instant::now();
            for &k in &keys {
                db.execute_prepared(&star, &[Value::Int(k)]).unwrap().encode(&mut out);
                black_box(&out);
            }
            let collected = per_op_ns(t0, LOOKUPS);
            let t0 = Instant::now();
            for &k in &keys {
                black_box(db.execute_prepared_encoded(&star, &[Value::Int(k)], &mut out).unwrap());
            }
            let direct = per_op_ns(t0, LOOKUPS);
            let t0 = Instant::now();
            for _ in 0..20 {
                black_box(db.execute_prepared(&many, &[]).unwrap());
            }
            let many_collected = ms(t0) / 20.0;
            let t0 = Instant::now();
            for _ in 0..20 {
                db.execute_prepared(&many, &[]).unwrap().encode(&mut out);
                black_box(&out);
            }
            let many_encoded = ms(t0) / 20.0;
            let t0 = Instant::now();
            for _ in 0..20 {
                black_box(db.execute_prepared_encoded(&many, &[], &mut out).unwrap());
            }
            let many_direct = ms(t0) / 20.0;
            println!("  round {round}: {collected:.0} | {direct:.0} | {many_collected:.2} | {many_encoded:.2} | {many_direct:.2}");
        }
    }

    // COUNT(*) by the secondary index: the engine vs. reading only the index.
    let count = db.prepare("SELECT COUNT(*) FROM items WHERE category = ?").unwrap();
    let cats: Vec<Value> = (0..2000).map(|_| Value::Text(format!("c{}", rng.below(100)).into())).collect();
    let ix = &t.indexes[0];
    println!("COUNT(*) by index, ns per call (engine | index set length | index keys + row lookups | + filter on each row)");
    for round in 1..=ROUNDS {
        let t0 = Instant::now();
        for c in &cats {
            black_box(db.execute_prepared(&count, std::slice::from_ref(c)).unwrap());
        }
        let engine = per_op_ns(t0, cats.len());
        let t0 = Instant::now();
        for c in &cats {
            black_box(ix.map.get(c).map_or(0, |keys| keys.len()));
        }
        let set_len = per_op_ns(t0, cats.len());
        let t0 = Instant::now();
        for c in &cats {
            let n = ix.map.get(c).map_or(0, |keys| keys.iter().filter(|k| t.rows.get(*k).is_some()).count());
            black_box(n);
        }
        let rows = per_op_ns(t0, cats.len());
        let t0 = Instant::now();
        for c in &cats {
            let n = ix.map.get(c).map_or(0, |keys| {
                keys.iter().filter_map(|k| t.rows.get(k)).filter(|e| e.values[1] == *c).count()
            });
            black_box(n);
        }
        let filtered = per_op_ns(t0, cats.len());
        println!("  round {round}: {engine:.0} | {set_len:.0} | {rows:.0} | {filtered:.0}");
    }

    // Key range with SUM: the engine vs. the same range walked by hand, with
    // and without the WHERE clause evaluated again on every row.
    let range = db.prepare("SELECT SUM(price) FROM items WHERE id BETWEEN ? AND ?").unwrap();
    let Statement::Select(sel) = parse_one("SELECT SUM(price) FROM items WHERE id BETWEEN ? AND ?").unwrap() else {
        unreachable!()
    };
    let between = sel.selection.unwrap();
    let starts: Vec<i64> = (0..2000).map(|_| rng.below(ROWS as u64 - 1000) as i64).collect();
    let sum = |e: &flatsql_core::table::RowEntry| match e.values[2] {
        Value::Float(p) => p,
        _ => 0.0,
    };
    println!("key range (1000 rows) + SUM, ns per call (engine | range walk + SUM | + WHERE evaluated on each row)");
    for round in 1..=ROUNDS {
        let t0 = Instant::now();
        for &a in &starts {
            black_box(db.execute_prepared(&range, &[Value::Int(a), Value::Int(a + 999)]).unwrap());
        }
        let engine = per_op_ns(t0, starts.len());
        let t0 = Instant::now();
        for &a in &starts {
            let s: f64 = t.rows.range(Value::Int(a)..=Value::Int(a + 999)).map(|(_, e)| sum(e)).sum();
            black_box(s);
        }
        let walk = per_op_ns(t0, starts.len());
        let t0 = Instant::now();
        for &a in &starts {
            let params = [Value::Int(a), Value::Int(a + 999)];
            let filter = Binder::new(Some(&schema), &params).bind(&between, "WHERE").unwrap();
            let s: f64 = t
                .rows
                .range(Value::Int(a)..=Value::Int(a + 999))
                .filter(|(_, e)| eval_pred(&filter, &e.values, EMPTY).unwrap() == Some(true))
                .map(|(_, e)| sum(e))
                .sum();
            black_box(s);
        }
        let filtered = per_op_ns(t0, starts.len());
        println!("  round {round}: {engine:.0} | {walk:.0} | {filtered:.0}");
    }

    // Preparing the statements of the workload (work done once per statement).
    let queries = [
        "SELECT * FROM items WHERE id = ?",
        "SELECT COUNT(*) FROM items WHERE category = ?",
        "SELECT SUM(price) FROM items WHERE id BETWEEN ? AND ?",
        "SELECT COUNT(*) FROM items WHERE price > 50 AND qty < 10",
        "SELECT category, COUNT(*), AVG(price), MAX(qty) FROM items GROUP BY category",
        "SELECT id, price FROM items ORDER BY price DESC, id ASC LIMIT 10",
        "SELECT * FROM items WHERE id < 10000",
        "INSERT INTO items VALUES (?, ?, ?, ?, ?)",
    ];
    println!("prepare, ns per statement (point | count | range | scan | group | top-k | 10k rows | insert)");
    for round in 1..=ROUNDS {
        let mut parts = Vec::new();
        for sql in queries {
            let t0 = Instant::now();
            for _ in 0..20_000 {
                black_box(db.prepare(sql).unwrap());
            }
            parts.push(format!("{:.0}", per_op_ns(t0, 20_000)));
        }
        println!("  round {round}: {}", parts.join(" | "));
    }

    // A point lookup executed once from its SQL text (parse, compile or plan,
    // execute), as the local query of the Local-First comparison.
    let texts: Vec<String> = keys.iter().take(50_000).map(|k| format!("SELECT * FROM items WHERE id = {k}")).collect();
    println!("one-shot point lookup from SQL text, ns per call");
    for round in 1..=ROUNDS {
        let t0 = Instant::now();
        for sql in &texts {
            black_box(db.execute(sql).unwrap());
        }
        println!("  round {round}: {:.0}", per_op_ns(t0, texts.len()));
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
    // The same loop by hand with the WHERE clause compiled into closures:
    // what the closures cost without the executor around them.
    #[cfg(core_compile)]
    {
        let Statement::Select(sel) = parse_one("SELECT COUNT(*) FROM items WHERE price > 50 AND qty < 10").unwrap() else {
            unreachable!()
        };
        let terms = Binder::deferred(Some(&schema)).bind(&sel.selection.unwrap(), "WHERE").unwrap().into_conjuncts();
        let preds: Vec<_> = terms.iter().map(|e| flatsql_core::compile::predicate(e, &schema)).collect();
        println!("full scan, ms (hand loop calling the compiled conjuncts)");
        for round in 1..=ROUNDS {
            let t0 = Instant::now();
            for _ in 0..20 {
                let mut n = 0u64;
                for e in t.rows.values() {
                    n += preds.iter().all(|p| p.test(&e.values, &[]).unwrap() == Some(true)) as u64;
                }
                black_box(n);
            }
            println!("  round {round}: {:.2}", ms(t0) / 20.0);
        }
    }

    // GROUP BY category with COUNT(*), AVG(price), MAX(qty) by hand: the
    // group of a row found in a tree of groups (comparisons of the texts),
    // in a hash table, or the groups taken from the index (a key lookup for
    // every row).
    #[derive(Default, Clone, Copy)]
    struct Acc {
        n: i64,
        sum: f64,
        max: i64,
    }
    fn add(a: &mut Acc, values: &[Value]) {
        a.n += 1;
        if let Value::Float(p) = values[2] {
            a.sum += p;
        }
        if let Value::Int(q) = values[3] {
            a.max = a.max.max(q);
        }
    }
    #[derive(Default)]
    struct Fx(u64);
    impl std::hash::Hasher for Fx {
        fn write(&mut self, bytes: &[u8]) {
            for chunk in bytes.chunks(8) {
                let mut word = [0u8; 8];
                word[..chunk.len()].copy_from_slice(chunk);
                self.0 = (self.0.rotate_left(5) ^ u64::from_le_bytes(word)).wrapping_mul(0x51_7c_c1_b7_27_22_0a_95);
            }
        }
        fn finish(&self) -> u64 {
            self.0
        }
    }
    println!("GROUP BY by hand, ms (group found in a tree | in a hash table | groups from the index, a key lookup per row)");
    for round in 1..=ROUNDS {
        let t0 = Instant::now();
        for _ in 0..20 {
            let mut groups: std::collections::BTreeMap<&Value, Acc> = std::collections::BTreeMap::new();
            for e in t.rows.values() {
                add(groups.entry(&e.values[1]).or_default(), &e.values);
            }
            black_box(groups);
        }
        let tree = ms(t0) / 20.0;
        let t0 = Instant::now();
        for _ in 0..20 {
            let mut groups: std::collections::HashMap<&str, Acc, std::hash::BuildHasherDefault<Fx>> = Default::default();
            for e in t.rows.values() {
                if let Value::Text(c) = &e.values[1] {
                    add(groups.entry(c.as_str()).or_default(), &e.values);
                }
            }
            black_box(groups);
        }
        let hashed = ms(t0) / 20.0;
        let t0 = Instant::now();
        for _ in 0..20 {
            let mut out = Vec::new();
            for (value, keys) in &t.indexes[0].map {
                let mut a = Acc::default();
                for k in keys {
                    add(&mut a, &t.rows[k].values);
                }
                out.push((value, a));
            }
            black_box(out);
        }
        let by_index = ms(t0) / 20.0;
        println!("  round {round}: {tree:.2} | {hashed:.2} | {by_index:.2}");
    }

    // Parts of the work an UPDATE of 10% of the rows did per changed row
    // before rows were changed in place, measured on the table outside the
    // database: copy of the row with validation, writing it back with the
    // secondary index and without it, a key lookup, a clock reading.
    let targets: Vec<i64> = (0..ROWS).filter(|i| i % 10 == 0).collect();
    let mut bare = Table::new(schema.clone(), Stamp::default());
    bare.rows = t.rows.clone();
    let mut clock = flatsql_core::sync::Hlc::new(flatsql_core::sync::system_now_ms);
    println!("UPDATE parts, ms for 10 000 rows (row copy + validation | write back with index | without index | key lookup | clock reading)");
    for round in 1..=ROUNDS {
        let t0 = Instant::now();
        let mut new_rows = Vec::with_capacity(targets.len());
        for &k in &targets {
            let mut v = t.rows[&Value::Int(k)].values.clone();
            v[3] = Value::Int(1);
            new_rows.push(schema.check_row(v).unwrap());
        }
        let copy = ms(t0);
        let t0 = Instant::now();
        for v in new_rows.clone() {
            black_box(t.put(v, Stamp::default()));
        }
        let with_index = ms(t0);
        let t0 = Instant::now();
        for v in new_rows {
            black_box(bare.put(v, Stamp::default()));
        }
        let without_index = ms(t0);
        let t0 = Instant::now();
        for &k in &targets {
            black_box(t.rows.get(&Value::Int(k)).map(|e| e.stamp));
        }
        let lookup = ms(t0);
        let t0 = Instant::now();
        for _ in &targets {
            black_box(clock.tick());
        }
        let ticks = ms(t0);
        println!("  round {round}: {copy:.2} | {with_index:.2} | {without_index:.2} | {lookup:.2} | {ticks:.2}");
    }

    let with = with_outbox(&db);
    let mut acked = Database::restore(Some(&with), &[], 1).unwrap();
    let upto = acked.stats().next_seq;
    acked.ack_outbox(upto);
    let without = acked.snapshot().unwrap();
    println!(
        "outbox entries {} -> {}; snapshot {} bytes with the outbox, {} without; the benchmark database: outbox {}, snapshot {} bytes",
        Database::restore(Some(&with), &[], 1).unwrap().stats().outbox,
        acked.stats().outbox,
        with.len(),
        without.len(),
        db.stats().outbox,
        db.snapshot().unwrap().len()
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

    #[cfg(core_many)]
    {
        println!("100 000 inserts in one transaction, ms (a statement for each row | one call for all rows)");
        let fresh = || {
            let mut db = Database::new(1);
            db.execute("CREATE TABLE items (id INTEGER PRIMARY KEY, category TEXT, price FLOAT, qty INT, name TEXT)").unwrap();
            db.execute("CREATE INDEX items_category ON items (category)").unwrap();
            db
        };
        let sql = "INSERT INTO items VALUES (?, ?, ?, ?, ?)";
        for round in 1..=ROUNDS {
            let mut db = fresh();
            let ins = db.prepare(sql).unwrap();
            let t0 = Instant::now();
            db.execute("BEGIN").unwrap();
            for i in 0..ROWS {
                black_box(db.execute_prepared(&ins, &row(i)).unwrap());
            }
            db.execute("COMMIT").unwrap();
            let each = ms(t0);
            let mut db = fresh();
            let ins = db.prepare(sql).unwrap();
            let t0 = Instant::now();
            db.execute("BEGIN").unwrap();
            let mut next = 0;
            db.execute_many(&ins, |params| {
                if next == ROWS {
                    return Ok(false);
                }
                params.extend(row(next));
                next += 1;
                Ok(true)
            })
            .unwrap();
            db.execute("COMMIT").unwrap();
            let many = ms(t0);
            println!("  round {round}: {each:.1} | {many:.1}");
        }
    }
}
