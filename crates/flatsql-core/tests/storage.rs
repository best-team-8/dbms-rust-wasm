//! Durability: WAL replay, snapshots, torn writes and the file backend.

use flatsql_core::storage::file::FileStorage;
use flatsql_core::storage::{MemoryStorage, Persistent, Storage};
use flatsql_core::{Database, Value};

fn count(db: &mut Database, table: &str) -> i64 {
    match db.execute(&format!("SELECT COUNT(*) FROM {table}")).unwrap().rows[0][0] {
        Value::Int(n) => n,
        ref other => panic!("{other:?}"),
    }
}

fn seeded() -> (Database, Vec<u8>) {
    let mut db = Database::new(7);
    db.execute("CREATE TABLE kv (k TEXT PRIMARY KEY, v INT); CREATE INDEX kv_v ON kv (v);").unwrap();
    for i in 0..100 {
        db.execute(&format!("INSERT INTO kv VALUES ('key{i}', {})", i % 10)).unwrap();
    }
    db.execute("UPDATE kv SET v = 99 WHERE k = 'key5'").unwrap();
    db.execute("DELETE FROM kv WHERE v = 3").unwrap();
    let wal = db.take_wal();
    (db, wal)
}

#[test]
fn wal_replay_rebuilds_identical_state() {
    let (mut original, wal) = seeded();
    let mut restored = Database::restore(None, &wal, 7).unwrap();
    let report = restored.recovery_report().unwrap().clone();
    assert_eq!(report.torn_bytes, 0);
    assert_eq!(report.ops_failed, 0);
    assert_eq!(report.frames_replayed, original.lsn());
    let sql = "SELECT k, v FROM kv ORDER BY k";
    assert_eq!(original.execute(sql).unwrap(), restored.execute(sql).unwrap());
    // The secondary index is rebuilt as well.
    let r = restored.execute("SELECT COUNT(*) FROM kv WHERE v = 99").unwrap();
    assert_eq!((r.rows[0][0].clone(), r.rows_scanned), (Value::Int(1), 1));
    assert_eq!(restored.stats().tombstones, 10);
}

#[test]
fn snapshot_roundtrip_and_wal_after_snapshot() {
    let (mut db, _) = seeded();
    let snap = db.snapshot().unwrap();
    db.execute("INSERT INTO kv VALUES ('after', 1)").unwrap();
    let wal = db.take_wal();
    let mut restored = Database::restore(Some(&snap), &wal, 0).unwrap();
    assert_eq!(restored.node_id(), 7, "node id comes from the snapshot");
    assert_eq!(count(&mut restored, "kv"), 91);
    assert_eq!(restored.execute("SELECT v FROM kv WHERE k = 'after'").unwrap().rows[0][0], Value::Int(1));
}

#[test]
fn frames_covered_by_snapshot_are_skipped() {
    // Crash between "snapshot written" and "WAL truncated": the WAL still
    // holds frames that are already part of the snapshot.
    let (db, wal) = seeded();
    let snap = db.snapshot().unwrap();
    let mut restored = Database::restore(Some(&snap), &wal, 7).unwrap();
    let report = restored.recovery_report().unwrap();
    assert_eq!(report.frames_replayed, 0);
    assert!(report.frames_skipped > 0);
    assert_eq!(count(&mut restored, "kv"), 90);
}

#[test]
fn torn_wal_tail_is_discarded() {
    let (_, wal) = seeded();
    let mut db = Database::restore(None, &wal, 7).unwrap();
    db.execute("INSERT INTO kv VALUES ('torn', 1)").unwrap();
    let last = db.take_wal();
    let mut damaged = wal.clone();
    damaged.extend_from_slice(&last[..last.len() - 3]); // power loss mid-write
    let mut restored = Database::restore(None, &damaged, 7).unwrap();
    let report = restored.recovery_report().unwrap();
    assert_eq!(report.torn_bytes, last.len() - 3);
    assert!(report.stop_reason.is_some());
    assert_eq!(count(&mut restored, "kv"), 90, "the incomplete transaction is not visible");
}

#[test]
fn corrupted_snapshot_is_rejected() {
    let (db, _) = seeded();
    let mut snap = db.snapshot().unwrap();
    let mid = snap.len() / 2;
    snap[mid] ^= 0xff;
    assert!(Database::restore(Some(&snap), &[], 7).is_err());
}

#[test]
fn every_wal_prefix_recovers_to_a_committed_state() {
    let mut db = Database::new(3);
    let mut wal = Vec::new();
    // End offset of every committed frame.
    let mut boundaries = Vec::new();
    db.execute("CREATE TABLE t (id INT PRIMARY KEY, v TEXT)").unwrap();
    wal.extend(db.take_wal());
    boundaries.push(wal.len());
    for i in 1..20 {
        db.execute(&format!("INSERT INTO t VALUES ({i}, 'v{i}')")).unwrap();
        wal.extend(db.take_wal());
        boundaries.push(wal.len());
    }
    // Cut the log at every byte: the result must always equal the state after
    // the last complete transaction before the cut.
    for cut in 0..=wal.len() {
        let restored = Database::restore(None, &wal[..cut], 3).unwrap();
        let complete = boundaries.iter().filter(|b| **b <= cut).count();
        if complete == 0 {
            assert!(restored.row_count("t").is_none(), "cut at {cut}");
        } else {
            assert_eq!(restored.row_count("t"), Some(complete - 1), "cut at {cut}");
        }
    }
}

#[test]
fn persistent_memory_storage_checkpoints() {
    let mut p = Persistent::open(MemoryStorage::default(), 1).unwrap().with_checkpoint_bytes(2_000);
    p.execute("CREATE TABLE t (id INT PRIMARY KEY, payload TEXT)").unwrap();
    for i in 0..200 {
        p.execute(&format!("INSERT INTO t VALUES ({i}, 'some payload text {i}')")).unwrap();
    }
    let storage = p.storage().clone();
    assert!(storage.snapshot.is_some(), "a checkpoint happened");
    assert!(storage.wal_size() < 2_100);
    let (_, storage) = p.into_parts();
    let mut reopened = Persistent::open(storage, 1).unwrap();
    assert_eq!(count(reopened.db_mut(), "t"), 200);
}

#[test]
fn node_id_is_stable_across_reopens() {
    let mut p = Persistent::open(MemoryStorage::default(), 11).unwrap();
    p.execute("CREATE TABLE t (id INT PRIMARY KEY)").unwrap();
    let (_, storage) = p.into_parts();
    let p = Persistent::open(storage, 99).unwrap();
    assert_eq!(p.db().node_id(), 11);
}

#[test]
fn file_storage_survives_reopen_and_torn_tail() {
    let dir = std::env::temp_dir().join(format!("flatsql-test-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    {
        let storage = FileStorage::open(&dir).unwrap().with_fsync(false);
        let mut p = Persistent::open(storage, 42).unwrap();
        p.execute("CREATE TABLE notes (id INT PRIMARY KEY, body TEXT)").unwrap();
        p.execute("INSERT INTO notes VALUES (1, 'first'), (2, 'second')").unwrap();
        p.checkpoint().unwrap();
        p.execute("INSERT INTO notes VALUES (3, 'after checkpoint')").unwrap();
    }
    // Append garbage to simulate a torn write.
    {
        use std::io::Write;
        let mut f = std::fs::OpenOptions::new().append(true).open(dir.join("wal.fsql")).unwrap();
        f.write_all(&[0x10, 0x00, 0x00, 0x00, 0xde, 0xad]).unwrap();
    }
    {
        let storage = FileStorage::open(&dir).unwrap();
        let mut p = Persistent::open(storage, 0).unwrap();
        assert_eq!(p.db().node_id(), 42);
        assert_eq!(p.recovery_report().unwrap().torn_bytes, 6);
        assert_eq!(count(p.db_mut(), "notes"), 3);
        // The torn tail was cleaned up by a checkpoint, new writes are safe.
        p.execute("INSERT INTO notes VALUES (4, 'fourth')").unwrap();
    }
    {
        let storage = FileStorage::open(&dir).unwrap();
        let mut p = Persistent::open(storage, 0).unwrap();
        assert_eq!(p.recovery_report().unwrap().torn_bytes, 0);
        assert_eq!(count(p.db_mut(), "notes"), 4);
    }
    std::fs::remove_dir_all(&dir).unwrap();
}
