//! Data shared by the `cost-split` and `allocations` programs: the rows and
//! the parameter generator of `bench/suite.js`.

use flatsql_core::{Database, Value};

pub const ROWS: i64 = 100_000;

/// Row `i` of `bench/suite.js` (`makeRow`).
pub fn row(i: i64) -> Vec<Value> {
    vec![
        Value::Int(i),
        Value::Text(format!("c{}", i % 100)),
        Value::Float(((i * 7919) % 10000) as f64 / 100.0),
        Value::Int(i % 50),
        Value::Text(format!("item-{i}")),
    ]
}

/// Linear congruential generator of `bench/suite.js` (`Rng`).
pub struct Lcg(pub u64);

impl Lcg {
    pub fn below(&mut self, n: u64) -> u64 {
        self.0 = self.0.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        (self.0 >> 33) % n
    }
}

/// Snapshot of `db` with every row pending in the outbox: the state of the
/// benchmark database before the outbox was kept only with synchronisation
/// (`core_sync`: synchronisation is enabled on a copy to get it).
pub fn with_outbox(db: &Database) -> Vec<u8> {
    #[cfg(core_sync)]
    {
        let mut copy = Database::restore(Some(&db.snapshot().unwrap()), &[], 1).unwrap();
        copy.enable_sync().unwrap();
        copy.snapshot().unwrap()
    }
    #[cfg(not(core_sync))]
    db.snapshot().unwrap()
}

/// The table of Benchmark 1 with `ROWS` rows inserted in one transaction.
pub fn load() -> Database {
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
    db
}
