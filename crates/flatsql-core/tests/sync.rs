//! Local-First synchronisation: outbox, acknowledgements, last-writer-wins and
//! convergence of replicas that worked offline.

use std::cell::Cell;

use flatsql_core::sync::{decode_batch, Mutation};
use flatsql_core::{Database, QueryResult, Value};

/// Pure logical clock: makes stamps deterministic in tests.
fn logical() -> u64 {
    0
}

fn replica(node: u32) -> Database {
    let mut db = Database::new(node);
    db.set_clock(logical);
    db
}

/// In-process stand-in for the sync server: a replica plus a log of the
/// batches it accepted (what `server/server.js` keeps in memory).
struct Server {
    db: Database,
    log: Vec<Vec<u8>>,
}

impl Server {
    fn new() -> Self {
        Server { db: replica(0), log: Vec::new() }
    }

    /// Push the client's outbox, then pull everything it has not seen yet.
    fn sync(&mut self, client: &mut Database, cursor: &mut usize) {
        let (batch, last) = client.export_outbox(usize::MAX).unwrap();
        let report = self.db.apply_changes(&batch).unwrap();
        assert_eq!(report.rejected, 0, "{:?}", report.errors);
        if report.applied > 0 {
            self.log.push(report.accepted);
        }
        client.ack_outbox(last);
        for b in &self.log[*cursor..] {
            client.apply_changes(b).unwrap();
        }
        *cursor = self.log.len();
    }
}

fn dump(db: &mut Database, sql: &str) -> QueryResult {
    db.execute(sql).unwrap()
}

#[test]
fn changes_flow_between_two_replicas() {
    let mut server = Server::new();
    let (mut a, mut b) = (replica(1), replica(2));
    let (mut ca, mut cb) = (0, 0);

    a.execute("CREATE TABLE inventory (id INT PRIMARY KEY, item TEXT, qty INT)").unwrap();
    a.execute("CREATE INDEX inv_item ON inventory (item)").unwrap();
    a.execute("INSERT INTO inventory VALUES (1, 'sword', 1), (2, 'potion', 3)").unwrap();
    assert_eq!(a.outbox_len(), 4);

    server.sync(&mut a, &mut ca);
    assert_eq!(a.outbox_len(), 0);
    server.sync(&mut b, &mut cb);

    let sql = "SELECT * FROM inventory ORDER BY id";
    assert_eq!(dump(&mut a, sql), dump(&mut b, sql));
    // Index definitions replicate too.
    let plan = dump(&mut b, "EXPLAIN SELECT * FROM inventory WHERE item = 'sword'");
    assert!(plan.rows[1][0].to_string().contains("INDEX LOOKUP inv_item"));

    b.execute("UPDATE inventory SET qty = qty - 1 WHERE id = 2").unwrap();
    server.sync(&mut b, &mut cb);
    server.sync(&mut a, &mut ca);
    assert_eq!(dump(&mut a, "SELECT qty FROM inventory WHERE id = 2").rows[0][0], Value::Int(2));
}

#[test]
fn concurrent_offline_edits_converge_with_last_writer_wins() {
    let mut server = Server::new();
    let (mut a, mut b) = (replica(1), replica(2));
    let (mut ca, mut cb) = (0, 0);
    a.execute("CREATE TABLE doc (id INT PRIMARY KEY, title TEXT)").unwrap();
    a.execute("INSERT INTO doc VALUES (1, 'draft')").unwrap();
    server.sync(&mut a, &mut ca);
    server.sync(&mut b, &mut cb);

    // Both go offline and edit the same row; B edits "later" (higher clock).
    a.execute("UPDATE doc SET title = 'from A' WHERE id = 1").unwrap();
    b.execute("UPDATE doc SET title = 'from B (1)' WHERE id = 1").unwrap();
    b.execute("UPDATE doc SET title = 'from B (2)' WHERE id = 1").unwrap();

    // B reconnects first, A second – A's older edit must lose everywhere.
    server.sync(&mut b, &mut cb);
    server.sync(&mut a, &mut ca);
    server.sync(&mut b, &mut cb);

    for db in [&mut a, &mut b, &mut server.db] {
        assert_eq!(dump(db, "SELECT title FROM doc").rows[0][0], Value::from("from B (2)"));
    }
}

#[test]
fn delete_wins_over_older_update() {
    let mut server = Server::new();
    let (mut a, mut b) = (replica(1), replica(2));
    let (mut ca, mut cb) = (0, 0);
    a.execute("CREATE TABLE t (id INT PRIMARY KEY, v INT)").unwrap();
    a.execute("INSERT INTO t VALUES (1, 0)").unwrap();
    server.sync(&mut a, &mut ca);
    server.sync(&mut b, &mut cb);

    a.execute("UPDATE t SET v = 1 WHERE id = 1").unwrap(); // older
    b.execute("UPDATE t SET v = 2 WHERE id = 1").unwrap();
    b.execute("DELETE FROM t WHERE id = 1").unwrap(); // newest
    server.sync(&mut b, &mut cb);
    server.sync(&mut a, &mut ca);
    assert!(dump(&mut a, "SELECT * FROM t").rows.is_empty(), "stale update must not resurrect the row");
    assert!(dump(&mut server.db, "SELECT * FROM t").rows.is_empty());
}

#[test]
fn outbox_coalesces_repeated_updates() {
    let mut a = replica(1);
    a.execute("CREATE TABLE t (id INT PRIMARY KEY, v INT)").unwrap();
    a.execute("INSERT INTO t VALUES (1, 0)").unwrap();
    for i in 1..=100 {
        a.execute(&format!("UPDATE t SET v = {i} WHERE id = 1")).unwrap();
    }
    assert_eq!(a.outbox_len(), 2, "CREATE TABLE + one entry for row 1");
    let (batch, _) = a.export_outbox(usize::MAX).unwrap();
    let changes = decode_batch(&batch).unwrap();
    assert_eq!(changes.len(), 2);
}

#[test]
fn outbox_survives_restart_and_ack_is_durable() {
    let mut a = replica(1);
    a.execute("CREATE TABLE t (id INT PRIMARY KEY, v TEXT)").unwrap();
    a.execute("INSERT INTO t VALUES (1, 'offline write')").unwrap();
    let mut wal = a.take_wal();

    // App closed while offline: the outbox is rebuilt from the WAL.
    let mut a = Database::restore(None, &wal, 1).unwrap();
    assert_eq!(a.outbox_len(), 2);
    let (batch, last) = a.export_outbox(usize::MAX).unwrap();
    let mut server = replica(0);
    server.apply_changes(&batch).unwrap();
    a.ack_outbox(last);
    wal.extend(a.take_wal());

    // After another restart nothing is re-sent.
    let a = Database::restore(None, &wal, 1).unwrap();
    assert_eq!(a.outbox_len(), 0);

    // Same through a snapshot.
    let snap = a.snapshot().unwrap();
    let a = Database::restore(Some(&snap), &[], 1).unwrap();
    assert_eq!(a.outbox_len(), 0);
}

#[test]
fn remote_changes_are_not_echoed_back() {
    let mut server = Server::new();
    let (mut a, mut b) = (replica(1), replica(2));
    let (mut ca, mut cb) = (0, 0);
    a.execute("CREATE TABLE t (id INT PRIMARY KEY)").unwrap();
    a.execute("INSERT INTO t VALUES (1), (2), (3)").unwrap();
    server.sync(&mut a, &mut ca);
    server.sync(&mut b, &mut cb);
    assert_eq!(b.outbox_len(), 0, "applied remote changes do not enter the outbox");
    // Re-applying the same batch is a no-op (idempotent delivery).
    let again = server.log[0].clone();
    let report = b.apply_changes(&again).unwrap();
    assert_eq!(report.applied, 0);
    assert_eq!(report.ignored, 4); // CREATE TABLE + 3 rows
}

#[test]
fn export_state_bootstraps_a_new_replica() {
    let mut a = replica(1);
    a.execute("CREATE TABLE t (id INT PRIMARY KEY, v INT); CREATE INDEX t_v ON t (v);").unwrap();
    a.execute("INSERT INTO t VALUES (1, 10), (2, 20), (3, 30); DELETE FROM t WHERE id = 2;").unwrap();
    let state = a.export_state().unwrap();
    let mut fresh = replica(9);
    let report = fresh.apply_changes(&state).unwrap();
    assert_eq!(report.rejected, 0);
    let sql = "SELECT * FROM t ORDER BY id";
    assert_eq!(dump(&mut a, sql), dump(&mut fresh, sql));
    assert_eq!(fresh.stats().tombstones, 1);
}

#[test]
fn rejected_changes_are_reported() {
    let mut a = replica(1);
    a.execute("CREATE TABLE t (id INT PRIMARY KEY, v INT)").unwrap();
    let (_, last) = a.export_outbox(usize::MAX).unwrap();
    a.ack_outbox(last);
    a.execute("INSERT INTO t VALUES (1, 1)").unwrap();
    let (batch, _) = a.export_outbox(usize::MAX).unwrap();
    // A replica that never received the table.
    let mut other = replica(2);
    let report = other.apply_changes(&batch).unwrap();
    assert_eq!((report.applied, report.ignored, report.rejected), (0, 0, 1));
    assert!(report.errors[0].contains("no such table"), "{:?}", report.errors);
    assert!(report.conflicts.is_empty());
}

#[test]
fn a_different_table_definition_is_reported_as_one_schema_conflict() {
    let mut a = replica(1);
    a.execute("CREATE TABLE t (id INT PRIMARY KEY, v INT)").unwrap();
    a.execute("INSERT INTO t VALUES (1, 1), (2, 2), (3, 3)").unwrap();
    let (batch, _) = a.export_outbox(usize::MAX).unwrap();

    // A newer version of the application added a nullable column.
    let mut newer = replica(2);
    newer.execute("CREATE TABLE t (id INT PRIMARY KEY, v INT, extra TEXT)").unwrap();
    let report = newer.apply_changes(&batch).unwrap();
    assert_eq!((report.applied, report.ignored, report.rejected), (0, 0, 4), "the definition is not ignored");
    assert_eq!(report.errors.len(), 1, "one report for the table, not one per row: {:?}", report.errors);
    assert!(report.errors[0].contains("schema conflict"), "{:?}", report.errors);
    assert!(dump(&mut newer, "SELECT * FROM t").rows.is_empty());
    let c = &report.conflicts[..];
    assert_eq!(c.len(), 1);
    assert_eq!(c[0].table, "t");
    assert_eq!(c[0].local, "t (id INT PRIMARY KEY, v INT, extra TEXT)");
    assert_eq!(c[0].remote.as_deref(), Some("t (id INT PRIMARY KEY, v INT)"));
    assert_eq!((c[0].additive, c[0].rows), (Some(true), 3), "only a nullable column added");

    // Another type of a column: not a difference between versions.
    let mut other = replica(3);
    other.execute("CREATE TABLE t (id INT PRIMARY KEY, v TEXT)").unwrap();
    let report = other.apply_changes(&batch).unwrap();
    assert_eq!((report.conflicts[0].additive, report.conflicts[0].rows), (Some(false), 3));
    assert!(dump(&mut other, "SELECT * FROM t").rows.is_empty(), "rows that happen to fit are not applied");

    // The same definition is no conflict.
    let mut same = replica(4);
    same.execute("CREATE TABLE T (ID INT PRIMARY KEY, V INT)").unwrap();
    let report = same.apply_changes(&batch).unwrap();
    assert_eq!((report.applied, report.ignored, report.rejected), (3, 1, 0));
    assert!(report.conflicts.is_empty());

    // Rows sent later, without the definition, are reported the same way.
    let (_, last) = a.export_outbox(usize::MAX).unwrap();
    a.ack_outbox(last);
    a.execute("INSERT INTO t VALUES (4, 4); DELETE FROM t WHERE id = 1;").unwrap();
    let (later, _) = a.export_outbox(usize::MAX).unwrap();
    let report = newer.apply_changes(&later).unwrap();
    assert_eq!((report.rejected, report.errors.len()), (2, 1));
    assert_eq!((report.conflicts[0].remote.as_deref(), report.conflicts[0].additive), (None, None));
    assert_eq!(report.conflicts[0].rows, 2);
}

thread_local! {
    /// Wall clocks of simulated devices (milliseconds), one per thread.
    static WALL: [Cell<u64>; 3] = Default::default();
}

fn wall0() -> u64 {
    WALL.with(|w| w[0].get())
}

fn wall1() -> u64 {
    WALL.with(|w| w[1].get())
}

fn wall2() -> u64 {
    WALL.with(|w| w[2].get())
}

fn set_wall(i: usize, ms: u64) {
    WALL.with(|w| w[i].set(ms));
}

const T: u64 = 1_700_000_000_000;
const HOUR: u64 = 3_600_000;

fn device(node: u32, clock: fn() -> u64) -> Database {
    let mut db = Database::new(node);
    db.set_clock(clock);
    db
}

#[test]
fn stamps_beyond_the_drift_bound_are_rejected_and_do_not_move_the_clock() {
    set_wall(0, T + HOUR); // clock set an hour ahead
    set_wall(1, T);
    set_wall(2, T + 30_000); // ahead, but within the bound
    let (mut future, mut b, mut near) = (device(1, wall0), device(2, wall1), device(3, wall2));
    b.execute("CREATE TABLE t (id INT PRIMARY KEY, v TEXT)").unwrap();
    let (schema, _) = b.export_outbox(usize::MAX).unwrap();
    future.apply_changes(&schema).unwrap();
    near.apply_changes(&schema).unwrap();

    future.execute("INSERT INTO t VALUES (1, 'from the future')").unwrap();
    let (batch, _) = future.export_outbox(usize::MAX).unwrap();
    let clock = b.stats().clock;
    let report = b.apply_changes(&batch).unwrap();
    assert_eq!((report.applied, report.rejected), (0, 1), "{:?}", report.errors);
    assert!(report.errors[0].contains("clock"), "{:?}", report.errors);
    assert_eq!(b.stats().clock, clock, "a rejected stamp does not move the clock");
    assert!(dump(&mut b, "SELECT * FROM t").rows.is_empty());

    near.execute("INSERT INTO t VALUES (2, 'slightly ahead')").unwrap();
    let (batch, _) = near.export_outbox(usize::MAX).unwrap();
    let report = b.apply_changes(&batch).unwrap();
    assert_eq!((report.applied, report.rejected), (1, 0));
    assert!(b.stats().clock >= (T + 30_000) << 16, "a stamp within the bound is absorbed");
}

/// Stamp of row `id` of table `t` in a replica's full state.
fn row_stamp(db: &Database, id: i64) -> Option<flatsql_core::Stamp> {
    decode_batch(&db.export_state().unwrap()).unwrap().into_iter().find_map(|c| match &c.mutation {
        Mutation::Put { table, values } if table == "t" && values[0] == Value::Int(id) => Some(c.stamp),
        _ => None,
    })
}

#[test]
fn changes_made_with_a_clock_ahead_get_new_stamps_once_it_is_fixed() {
    set_wall(0, T);
    set_wall(1, T);
    let mut server = device(100, wall1);
    let mut a = device(1, wall0);
    a.execute("CREATE TABLE t (id INT PRIMARY KEY, v TEXT)").unwrap();
    a.execute("INSERT INTO t VALUES (9, 'sent with a correct clock')").unwrap();
    let (batch, last) = a.export_outbox(usize::MAX).unwrap();
    assert_eq!(server.apply_changes(&batch).unwrap().rejected, 0);
    a.ack_outbox(last);
    let sent = row_stamp(&server, 9);

    // The device clock jumps an hour ahead; the work goes on offline.
    set_wall(0, T + HOUR);
    a.execute("CREATE TABLE u (id INT PRIMARY KEY)").unwrap();
    a.execute("INSERT INTO t VALUES (1, 'a'), (2, 'b'), (3, 'c')").unwrap();
    a.execute("UPDATE t SET v = 'b2' WHERE id = 2").unwrap();
    a.execute("DELETE FROM t WHERE id = 3").unwrap();
    a.execute("INSERT INTO u VALUES (1)").unwrap();
    let (batch, _) = a.export_outbox(usize::MAX).unwrap();
    let report = server.apply_changes(&batch).unwrap();
    assert_eq!((report.applied, report.rejected, report.clock_drift), (0, 5, 5));
    // Not acknowledged: the changes stay in the outbox.

    // The clock is fixed; the unsent changes get new stamps.
    set_wall(0, T + 5_000);
    assert!(a.clock_ahead());
    let restamped = a.rebase_clock().unwrap();
    // The host checkpoints after a rebase.
    let mut a = Database::restore(Some(&a.snapshot().unwrap()), &[], 1).unwrap();
    a.set_clock(wall0);
    let (batch, last) = a.export_outbox(usize::MAX).unwrap();
    let report = server.apply_changes(&batch).unwrap();
    assert_eq!((report.applied, report.rejected), (5, 0), "{:?}", report.errors);
    a.ack_outbox(last);
    assert_eq!(restamped, 5);
    assert!(!a.clock_ahead());
    assert!(a.stats().clock >> 16 <= T + 5_000, "the clock went back to the wall clock");

    for sql in ["SELECT * FROM t ORDER BY id", "SELECT * FROM u ORDER BY id"] {
        assert_eq!(dump(&mut a, sql), dump(&mut server, sql));
    }
    assert_eq!(server.stats().tombstones, 1);
    assert_eq!(row_stamp(&server, 9), sent, "a stamp that left the device is kept");
}

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

#[test]
fn randomized_offline_sessions_converge() {
    for seed in 1..=25u64 {
        let mut rng = Rng(seed.wrapping_mul(0x9E37_79B9_7F4A_7C15) | 1);
        let mut server = Server::new();
        let mut replicas: Vec<Database> = (1..=3).map(replica).collect();
        let mut cursors = [0usize; 3];
        replicas[0].execute("CREATE TABLE t (id INT PRIMARY KEY, v INT)").unwrap();
        server.sync(&mut replicas[0], &mut cursors[0]);
        for i in 1..3 {
            server.sync(&mut replicas[i], &mut cursors[i]);
        }

        for _step in 0..300 {
            let r = rng.below(3) as usize;
            let key = rng.below(12);
            let val = rng.below(1000);
            match rng.below(10) {
                0..=3 => {
                    replicas[r].execute(&format!("INSERT OR REPLACE INTO t VALUES ({key}, {val})")).unwrap();
                }
                4..=6 => {
                    replicas[r].execute(&format!("UPDATE t SET v = {val} WHERE id = {key}")).unwrap();
                }
                7 | 8 => {
                    replicas[r].execute(&format!("DELETE FROM t WHERE id = {key}")).unwrap();
                }
                _ => {
                    // This replica comes online for a moment.
                    server.sync(&mut replicas[r], &mut cursors[r]);
                }
            }
        }
        // Everybody reconnects (twice, so all pushes are pulled by everyone).
        for _ in 0..2 {
            for i in 0..3 {
                server.sync(&mut replicas[i], &mut cursors[i]);
            }
        }
        let sql = "SELECT * FROM t ORDER BY id";
        let expected = dump(&mut server.db, sql);
        for (i, db) in replicas.iter_mut().enumerate() {
            assert_eq!(dump(db, sql).rows, expected.rows, "seed {seed}, replica {i} diverged");
            assert_eq!(db.outbox_len(), 0);
        }
    }
}
