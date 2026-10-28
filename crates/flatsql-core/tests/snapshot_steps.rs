//! A snapshot encoded in steps while the database changes between the steps
//! is the snapshot of the state at its start: the same bytes as `snapshot()`
//! called then. Restored with the WAL written after the start it gives the
//! current state, also when a schema change or a clock rebase makes the job
//! start again.

use std::sync::atomic::{AtomicU64, Ordering::Relaxed};

use flatsql_core::{Database, Value};

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
}

fn run(db: &mut Database, sql: &str) {
    db.execute(sql).unwrap_or_else(|e| panic!("{sql}\n{e}"));
}

fn text(rng: &mut Rng) -> String {
    let n = rng.below(40);
    if n % 3 == 0 {
        format!("t{n} – a text longer than fits in place")
    } else {
        format!("t{n}")
    }
}

/// Two tables with an index, tombstones and outbox entries.
fn database(seed: u64) -> (Database, Rng) {
    let mut rng = Rng(seed.wrapping_mul(0x9E37_79B9_7F4A_7C15) | 1);
    let mut db = Database::new(1);
    run(&mut db, "CREATE TABLE a (id INT PRIMARY KEY, s TEXT, f FLOAT)");
    run(&mut db, "CREATE INDEX a_s ON a (s)");
    run(&mut db, "CREATE TABLE b (k TEXT PRIMARY KEY, n INT)");
    let ins = db.prepare("INSERT INTO a VALUES (?, ?, ?)").unwrap();
    for i in 0..400 {
        let s = text(&mut rng);
        db.execute_prepared(&ins, &[Value::Int(i * 3), Value::from(s), Value::Float(i as f64 / 2.0)]).unwrap();
    }
    for i in 0..60 {
        run(&mut db, &format!("INSERT INTO b VALUES ('k{i}', {i})"));
    }
    run(&mut db, "DELETE FROM a WHERE id % 7 = 0");
    db.enable_sync().unwrap();
    run(&mut db, "UPDATE b SET n = n + 1 WHERE n < 10");
    db.take_wal();
    (db, rng)
}

/// A random change of every kind the hooks cover.
fn change(db: &mut Database, other: &mut Database, rng: &mut Rng) {
    let id = rng.below(1300) as i64;
    let s = text(rng);
    match rng.below(9) {
        0 => {
            let _ = db.execute(&format!("INSERT INTO a VALUES ({id}, '{s}', 1.5)"));
        }
        1 => run(db, &format!("INSERT OR REPLACE INTO a VALUES ({id}, '{s}', 2.5)")),
        2 => run(db, &format!("UPDATE a SET s = '{s}', f = f + 1 WHERE id BETWEEN {id} AND {}", id + 20)),
        3 => {
            let _ = db.execute(&format!("UPDATE a SET id = id + 1 WHERE id = {id}"));
        }
        4 => run(db, &format!("DELETE FROM a WHERE id BETWEEN {id} AND {}", id + 10)),
        5 => {
            run(db, "BEGIN");
            run(db, &format!("DELETE FROM a WHERE id < {id}"));
            run(db, &format!("INSERT OR REPLACE INTO b VALUES ('k{}', -1)", id % 70));
            run(db, &format!("UPDATE a SET f = 0 WHERE id > {id}"));
            run(db, if rng.below(2) == 0 { "ROLLBACK" } else { "COMMIT" });
        }
        6 => {
            // Changes from another replica (last writer wins).
            run(other, &format!("INSERT OR REPLACE INTO b VALUES ('k{}', {id})", id % 80));
            let batch = other.export_state().unwrap();
            db.apply_changes(&batch).unwrap();
        }
        7 => db.ack_outbox(rng.below(200)),
        _ => run(db, &format!("INSERT OR REPLACE INTO b VALUES ('k{}', {id})", id % 90)),
    }
}

fn other_replica() -> Database {
    let mut other = Database::new(2);
    run(&mut other, "CREATE TABLE b (k TEXT PRIMARY KEY, n INT)");
    other
}

#[test]
fn a_snapshot_in_steps_is_the_snapshot_of_its_start() {
    for seed in 1..=4 {
        for budget in [1, 3, 64, 1000, usize::MAX] {
            let (mut db, mut rng) = database(seed);
            let mut other = other_replica();
            let want = db.snapshot().unwrap();
            db.snapshot_begin().unwrap();
            let mut steps = 0;
            while !db.snapshot_step(budget).unwrap() {
                for _ in 0..rng.below(4) {
                    change(&mut db, &mut other, &mut rng);
                }
                steps += 1;
            }
            let got = db.snapshot_finish().unwrap();
            assert!(got == want, "seed {seed}, budget {budget}, {steps} steps: different bytes");
            // The snapshot and the WAL written since give the current state.
            let wal = db.take_wal();
            let restored = Database::restore(Some(&got), &wal, 1).unwrap();
            assert!(restored.export_state().unwrap() == db.export_state().unwrap(), "seed {seed}, budget {budget}");
        }
    }
}

#[test]
fn a_schema_change_starts_the_snapshot_again_after_the_transaction() {
    let (mut db, mut rng) = database(9);
    let mut other = other_replica();
    db.snapshot_begin().unwrap();
    assert!(!db.snapshot_step(50).unwrap());
    change(&mut db, &mut other, &mut rng);
    run(&mut db, "BEGIN");
    run(&mut db, "CREATE TABLE c (id INT PRIMARY KEY)");
    run(&mut db, "INSERT INTO c VALUES (1)");
    // The job cannot start again inside the transaction: no progress.
    assert!(!db.snapshot_step(usize::MAX).unwrap());
    assert!(db.snapshot_finish().is_err());
    run(&mut db, "COMMIT");
    run(&mut db, "CREATE INDEX a_f ON a (f)");
    while !db.snapshot_step(70).unwrap() {
        change(&mut db, &mut other, &mut rng);
    }
    let snap = db.snapshot_finish().unwrap();
    change(&mut db, &mut other, &mut rng);
    // WAL frames older than the snapshot are skipped on restore.
    let wal = db.take_wal();
    let restored = Database::restore(Some(&snap), &wal, 1).unwrap();
    assert!(restored.export_state().unwrap() == db.export_state().unwrap());
}

static WALL: AtomicU64 = AtomicU64::new(1_800_000_000_000);

fn wall() -> u64 {
    WALL.load(Relaxed)
}

#[test]
fn a_clock_rebase_starts_the_snapshot_again() {
    let mut db = Database::new(1);
    db.set_clock(wall);
    run(&mut db, "CREATE TABLE t (id INT PRIMARY KEY, v TEXT)");
    db.enable_sync().unwrap();
    run(&mut db, "INSERT INTO t VALUES (1, 'a')");
    // The clock jumps an hour ahead and back; the rows written meanwhile get
    // new stamps from the rebase, which are not in the WAL.
    WALL.fetch_add(3_600_000, Relaxed);
    for i in 2..300 {
        run(&mut db, &format!("INSERT INTO t VALUES ({i}, 'v{i}')"));
    }
    WALL.fetch_sub(3_595_000, Relaxed);
    db.take_wal();
    db.snapshot_begin().unwrap();
    assert!(!db.snapshot_step(100).unwrap());
    assert!(db.clock_ahead());
    assert!(db.rebase_clock().unwrap() > 0);
    while !db.snapshot_step(100).unwrap() {}
    let snap = db.snapshot_finish().unwrap();
    assert!(snap == db.snapshot().unwrap(), "the snapshot has the new stamps");
}

#[test]
fn steps_need_a_snapshot_begun_outside_a_transaction() {
    let (mut db, _) = database(3);
    assert!(db.snapshot_step(10).is_err());
    assert!(db.snapshot_finish().is_err());
    run(&mut db, "BEGIN");
    assert!(db.snapshot_begin().is_err());
    run(&mut db, "ROLLBACK");
    db.snapshot_begin().unwrap();
    assert!(db.snapshot_finish().is_err(), "not complete yet");
    while !db.snapshot_step(1000).unwrap() {}
    db.snapshot_finish().unwrap();
    assert!(db.snapshot_step(1).is_err(), "the job ended");
}
