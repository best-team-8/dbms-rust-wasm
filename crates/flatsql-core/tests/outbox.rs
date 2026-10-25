//! The outbox is kept only once synchronisation is enabled. Enabling it must
//! leave the database with the outbox it would have had if every change had
//! been kept from the start, and the state must survive a restart from any
//! mix of snapshot and WAL.

use flatsql_core::sync::{decode_batch, Change};
use flatsql_core::Database;

fn logical() -> u64 {
    1_700_000_000_000
}

/// The same local history: a table, inserts, updates of some rows, deletes,
/// a second table that is dropped again and an index.
fn history(db: &mut Database) {
    db.execute("CREATE TABLE t (id INT PRIMARY KEY, v TEXT, n INT)").unwrap();
    for i in 0..40 {
        db.execute(&format!("INSERT INTO t VALUES ({i}, 'v{i}', {})", i % 7)).unwrap();
    }
    db.execute("UPDATE t SET n = n + 1 WHERE id % 3 = 0").unwrap();
    db.execute("DELETE FROM t WHERE id % 5 = 1").unwrap();
    db.execute("CREATE TABLE gone (k TEXT PRIMARY KEY)").unwrap();
    db.execute("INSERT INTO gone VALUES ('a'), ('b')").unwrap();
    db.execute("DROP TABLE gone").unwrap();
    db.execute("CREATE INDEX t_n ON t (n)").unwrap();
}

fn replica(node: u32) -> Database {
    let mut db = Database::new(node);
    db.set_clock(logical);
    db
}

/// Changes of the outbox in a canonical order: the order of entries differs
/// (rows that were pending before synchronisation are numbered in key order),
/// their content must not.
fn pending(db: &Database) -> Vec<String> {
    let (batch, _) = db.export_outbox(usize::MAX).unwrap();
    let changes: Vec<Change> = decode_batch(&batch).unwrap();
    let mut out: Vec<String> = changes.iter().map(|c| format!("{c:?}")).collect();
    out.sort();
    out
}

#[test]
fn enabling_sync_later_gives_the_outbox_of_a_database_that_kept_every_change() {
    let mut always = replica(1);
    always.enable_sync().unwrap();
    history(&mut always);

    let mut later = replica(1);
    history(&mut later);
    // Only schema changes are kept while synchronisation is off.
    assert!(!later.sync_enabled());
    assert_eq!(later.outbox_len(), 4, "two CREATE TABLE, DROP TABLE and CREATE INDEX, no rows");
    let before_enable = later.snapshot().unwrap();
    let wal_before = later.take_wal();
    assert!(later.enable_sync().unwrap());
    assert!(!later.enable_sync().unwrap(), "a second call changes nothing");
    // The same changes to send. (The outbox that kept every change also holds
    // entries of the dropped table's rows, which are never sent.)
    assert_eq!(pending(&later), pending(&always));

    // Changes made after enabling are kept as before.
    for db in [&mut always, &mut later] {
        db.execute("UPDATE t SET v = 'x' WHERE id = 2").unwrap();
        db.execute("INSERT INTO t VALUES (100, 'new', 1)").unwrap();
    }
    assert_eq!(pending(&later), pending(&always));

    // A restart from the WAL alone, from the snapshot taken before enabling
    // plus the WAL after it, and from a snapshot taken after enabling.
    let wal_after = later.take_wal();
    let mut all_wal = wal_before.clone();
    all_wal.extend_from_slice(&wal_after);
    let from_wal = Database::restore(None, &all_wal, 1).unwrap();
    let from_snapshot = Database::restore(Some(&before_enable), &wal_after, 1).unwrap();
    let after_snapshot = Database::restore(Some(&later.snapshot().unwrap()), &[], 1).unwrap();
    for (name, db) in [("WAL", &from_wal), ("snapshot + WAL", &from_snapshot), ("snapshot", &after_snapshot)] {
        assert!(db.sync_enabled(), "{name}");
        assert_eq!(db.outbox_len(), later.outbox_len(), "{name}");
        assert_eq!(pending(db), pending(&always), "{name}");
        assert_eq!(db.stats().next_seq, later.stats().next_seq, "{name}");
    }

    // Acknowledged entries leave the outbox and stay out after a restart.
    let (_, last) = later.export_outbox(10).unwrap();
    later.ack_outbox(last);
    let left = later.outbox_len();
    let restored = Database::restore(Some(&before_enable), &[wal_after, later.take_wal()].concat(), 1).unwrap();
    assert_eq!(restored.outbox_len(), left);
}

#[test]
fn a_database_without_sync_keeps_no_row_entries_in_its_snapshot() {
    let mut off = replica(1);
    history(&mut off);
    let mut on = replica(1);
    on.enable_sync().unwrap();
    history(&mut on);
    let (small, large) = (off.snapshot().unwrap(), on.snapshot().unwrap());
    assert!(small.len() < large.len(), "{} vs {} bytes", small.len(), large.len());
    // Without the outbox the snapshot holds the same rows.
    let mut back = Database::restore(Some(&small), &[], 1).unwrap();
    let dump = |db: &mut Database| db.execute("SELECT * FROM t ORDER BY id").unwrap().rows;
    assert_eq!(dump(&mut back), dump(&mut on));
    assert!(!back.sync_enabled());
}
