//! UPDATE changes rows in place: only the assigned columns are swapped, only
//! indexes of changed columns are touched, and a rollback puts back the
//! columns, the index entries and the stamps. Random updates on a table with
//! secondary indexes are compared with the same updates on a table without
//! them, after rollbacks and after a restart from the WAL.

use std::collections::HashSet;

use flatsql_core::sync::{decode_batch, Mutation};
use flatsql_core::{Database, QueryResult};

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

fn frozen() -> u64 {
    1_700_000_000_000
}

const SCHEMA: &str = "CREATE TABLE t (id INT PRIMARY KEY, a INT, b TEXT, c FLOAT NOT NULL)";

fn databases() -> (Database, Database) {
    let mut plain = Database::new(1);
    let mut indexed = Database::new(1);
    for db in [&mut plain, &mut indexed] {
        db.set_clock(frozen);
        db.execute(SCHEMA).unwrap();
        for i in 0..200 {
            db.execute(&format!("INSERT INTO t VALUES ({i}, {}, 'k{}', {}.5)", i % 13, i % 7, i)).unwrap();
        }
        db.take_wal();
    }
    indexed.execute("CREATE INDEX t_a ON t (a); CREATE INDEX t_b ON t (b)").unwrap();
    (plain, indexed)
}

fn assignment(rng: &mut Rng) -> String {
    match rng.below(8) {
        0 => format!("a = a + {}", rng.below(5)),
        1 => format!("a = {}", rng.below(15)),
        2 => "a = NULL".into(),
        3 => format!("b = 'k{}'", rng.below(9)),
        4 => "b = NULL".into(),
        5 => "c = c * 2".into(),
        6 => "b = 'k1', a = a - 1".into(),
        _ => "c = NULL".into(), // violates NOT NULL: the statement fails
    }
}

fn filter(rng: &mut Rng) -> String {
    match rng.below(5) {
        0 => format!("id % {} = 0", 2 + rng.below(9)),
        1 => format!("a = {}", rng.below(15)),
        2 => format!("b = 'k{}'", rng.below(9)),
        3 => format!("id BETWEEN {} AND {}", rng.below(100), 100 + rng.below(100)),
        _ => "a IS NULL".into(),
    }
}

fn run(db: &mut Database, sql: &str) -> QueryResult {
    db.execute(sql).unwrap_or_else(|e| panic!("{sql}\n{e}"))
}

/// The table and answers through each index, checked against the table
/// without indexes.
fn same(plain: &mut Database, indexed: &mut Database, step: &str) {
    let all = "SELECT * FROM t ORDER BY id";
    assert_eq!(run(indexed, all).rows, run(plain, all).rows, "{step}");
    for k in 0..15 {
        let q = format!("SELECT id FROM t WHERE a = {k} ORDER BY id");
        let reference = format!("SELECT id FROM t WHERE (a = {k}) OR FALSE ORDER BY id");
        assert_eq!(run(indexed, &q).rows, run(plain, &reference).rows, "{step}: {q}");
    }
    for k in 0..9 {
        let q = format!("SELECT COUNT(*) FROM t WHERE b = 'k{k}'");
        let reference = format!("SELECT COUNT(*) FROM t WHERE (b = 'k{k}') OR FALSE");
        assert_eq!(run(indexed, &q).rows, run(plain, &reference).rows, "{step}: {q}");
    }
}

#[test]
fn updates_in_place_keep_indexes_rollbacks_and_the_wal_consistent() {
    let mut rng = Rng(0x0BAD_5EED_1234_5678);
    let (mut plain, mut indexed) = databases();
    let mut wal = Vec::new();
    for step in 0..400 {
        let sql = format!("UPDATE t SET {} WHERE {}", assignment(&mut rng), filter(&mut rng));
        let rollback = rng.below(4) == 0;
        // Rows, tombstones and stamps (the clock itself never goes back).
        let before = indexed.export_state().unwrap();
        for db in [&mut plain, &mut indexed] {
            if rollback {
                db.execute("BEGIN").unwrap();
            }
            let a = db.execute(&sql).map(|r| r.rows_affected);
            if rollback {
                db.execute("ROLLBACK").unwrap();
            }
            let _ = a;
        }
        if rollback {
            assert_eq!(indexed.export_state().unwrap(), before, "step {step}: {sql} rolled back");
        }
        wal.extend(indexed.take_wal());
        plain.take_wal();
        same(&mut plain, &mut indexed, &format!("step {step}: {sql}"));
    }
    // The WAL carries whole rows, so a restart rebuilds the same table,
    // stamps included, and the indexes from it.
    let mut restored = Database::restore(Some(&databases().1.snapshot().unwrap()), &wal, 1).unwrap();
    assert_eq!(restored.export_state().unwrap(), indexed.export_state().unwrap());
    same(&mut plain, &mut restored, "after restart");
}

#[test]
fn one_clock_reading_per_statement_still_gives_every_row_its_own_stamp() {
    let (_, mut db) = databases();
    run(&mut db, "UPDATE t SET a = a + 1");
    run(&mut db, "DELETE FROM t WHERE id % 3 = 0");
    let changes = decode_batch(&db.export_state().unwrap()).unwrap();
    let stamps: Vec<_> = changes
        .iter()
        .filter(|c| matches!(c.mutation, Mutation::Put { .. } | Mutation::Delete { .. }))
        .map(|c| c.stamp)
        .collect();
    assert_eq!(stamps.len(), 200);
    assert_eq!(stamps.iter().collect::<HashSet<_>>().len(), stamps.len(), "stamps are unique");
}
