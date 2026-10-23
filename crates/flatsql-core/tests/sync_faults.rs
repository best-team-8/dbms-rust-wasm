//! Randomized synchronisation under hard conditions: skewed clocks (one
//! device an hour ahead for a while), network partitions, deletes during a
//! partition, lost acknowledgements and cursors, restarts from the WAL.
//!
//! The devices follow the protocol of `bridge/sync.js`: check the clock
//! before pushing, pause while it differs from the server's by more than
//! `MAX_DRIFT_MS`, keep a batch the server rejected for the clock, re-stamp
//! after the clock is fixed and keep the cursor when a pulled change was
//! rejected for the clock.
//!
//! Oracle: for every key, the change with the greatest stamp among all
//! changes the devices sent within the drift bound. Every replica must end in
//! exactly that state, which also rules out resurrected rows.

use std::cell::Cell;
use std::collections::BTreeMap;

use flatsql_core::sync::{decode_batch, Mutation, MAX_DRIFT_MS};
use flatsql_core::{Database, Stamp, Value};

const T: u64 = 1_700_000_000_000;
const HOUR: i64 = 3_600_000;
const DEVICES: usize = 4;
const STEPS: usize = 400;

thread_local! {
    /// Simulated real time (the server clock) and each device's offset from it.
    static NOW: Cell<u64> = const { Cell::new(T) };
    static OFFSET: [Cell<i64>; DEVICES] = Default::default();
}

fn now() -> u64 {
    NOW.with(Cell::get)
}

fn offset(i: usize) -> i64 {
    OFFSET.with(|o| o[i].get())
}

fn set_offset(i: usize, ms: i64) {
    OFFSET.with(|o| o[i].set(ms));
}

fn device_clock<const I: usize>() -> u64 {
    now().saturating_add_signed(offset(I))
}

const CLOCKS: [fn() -> u64; DEVICES] = [device_clock::<0>, device_clock::<1>, device_clock::<2>, device_clock::<3>];

/// xorshift64* – deterministic randomness without dependencies.
struct Rng(u64);

impl Rng {
    fn below(&mut self, n: u64) -> u64 {
        self.0 ^= self.0 >> 12;
        self.0 ^= self.0 << 25;
        self.0 ^= self.0 >> 27;
        self.0.wrapping_mul(0x2545_F491_4F6C_DD1D) % n
    }
}

/// A device: the database, what it has on disk and its sync state.
struct Device {
    node: u32,
    clock: fn() -> u64,
    db: Database,
    snapshot: Option<Vec<u8>>,
    wal: Vec<u8>,
    cursor: usize,
    online: bool,
    paused: bool,
    /// Clock difference measured at the last contact with the server.
    known_skew: Option<i64>,
}

impl Device {
    fn new(node: u32, clock: fn() -> u64) -> Self {
        let mut db = Database::new(node);
        db.set_clock(clock);
        Device {
            node,
            clock,
            db,
            snapshot: None,
            wal: Vec::new(),
            cursor: 0,
            online: true,
            paused: false,
            known_skew: None,
        }
    }

    fn persist(&mut self) {
        let wal = self.db.take_wal();
        self.wal.extend(wal);
    }

    fn checkpoint(&mut self) {
        self.persist();
        self.snapshot = Some(self.db.snapshot().unwrap());
        self.wal.clear();
    }

    /// The application is closed and opened again from its files.
    fn restart(&mut self) {
        self.persist();
        self.db = Database::restore(self.snapshot.as_deref(), &self.wal, self.node).unwrap();
        self.db.set_clock(self.clock);
    }

    fn exec(&mut self, sql: &str) {
        self.db.execute(sql).unwrap();
        self.persist();
    }
}

#[derive(Default, Debug)]
struct Stats {
    writes: usize,
    deletes: usize,
    deletes_in_partition: usize,
    syncs: usize,
    paused_rounds: usize,
    drift_rejected_push: usize,
    drift_rejected_pull: usize,
    restamped: usize,
    lost_acks: usize,
    lost_cursors: usize,
    restarts: usize,
    partitions: usize,
    deleted_at_end: usize,
}

struct Sim {
    server: Database,
    log: Vec<Vec<u8>>,
    devices: Vec<Device>,
    /// key -> (stamp, value or None for a delete) of the newest sent change.
    oracle: BTreeMap<i64, (Stamp, Option<i64>)>,
    stats: Stats,
}

fn int(v: &Value) -> i64 {
    match v {
        Value::Int(i) => *i,
        other => panic!("{other:?}"),
    }
}

impl Sim {
    fn record(&mut self, batch: &[u8]) {
        let bound = now() + MAX_DRIFT_MS;
        for c in decode_batch(batch).unwrap() {
            if c.stamp.physical_ms() > bound {
                continue; // the server rejects it
            }
            let (key, value) = match &c.mutation {
                Mutation::Put { values, .. } => (int(&values[0]), Some(int(&values[1]))),
                Mutation::Delete { key, .. } => (int(key), None),
                _ => continue,
            };
            let e = self.oracle.entry(key).or_insert((c.stamp, value));
            if c.stamp > e.0 {
                *e = (c.stamp, value);
            }
        }
    }

    /// One round of `SyncClient` for device `i`.
    fn sync(&mut self, i: usize, rng: &mut Rng) {
        if !self.devices[i].online {
            return;
        }
        self.stats.syncs += 1;
        let skew = offset(i);
        let too_far = skew.unsigned_abs() > MAX_DRIFT_MS;
        let d = &mut self.devices[i];
        // Before the first push and while paused only the clock is checked.
        if d.paused || d.known_skew.is_none() {
            d.known_skew = Some(skew);
            if too_far {
                d.paused = true;
                self.stats.paused_rounds += 1;
                return;
            }
            d.paused = false;
        }
        if d.db.clock_ahead() {
            self.stats.restamped += d.db.rebase_clock().unwrap();
            d.checkpoint();
        }
        let (batch, last) = d.db.export_outbox(usize::MAX).unwrap();
        if last > 0 {
            self.record(&batch);
            let report = self.server.apply_changes(&batch).unwrap();
            assert_eq!(report.rejected, report.clock_drift, "{:?}", report.errors);
            if report.applied > 0 {
                self.log.push(report.accepted);
            }
            let d = &mut self.devices[i];
            d.known_skew = Some(skew);
            if report.clock_drift > 0 || too_far {
                // Not acknowledged: the batch stays in the outbox.
                d.paused = true;
                self.stats.drift_rejected_push += report.clock_drift;
                return;
            }
            if rng.below(10) == 0 {
                self.stats.lost_acks += 1; // the response was lost; the batch is sent again
            } else {
                d.db.ack_outbox(last);
                d.persist();
            }
        }
        let d = &mut self.devices[i];
        if too_far {
            d.paused = true;
            return;
        }
        let mut drift = 0;
        for b in &self.log[d.cursor..] {
            let r = d.db.apply_changes(b).unwrap();
            assert_eq!(r.rejected, r.clock_drift, "{:?}", r.errors);
            drift += r.clock_drift;
        }
        d.persist();
        if drift > 0 {
            self.stats.drift_rejected_pull += drift; // pulled again later
        } else if rng.below(10) == 0 {
            self.stats.lost_cursors += 1;
        } else {
            d.cursor = self.log.len();
        }
        assert!(
            self.server.stats().clock >> 16 <= now() + MAX_DRIFT_MS,
            "the server clock was dragged ahead of real time"
        );
    }

    fn rows(db: &mut Database) -> Vec<(i64, i64)> {
        db.execute("SELECT id, v FROM t ORDER BY id").unwrap().rows.iter().map(|r| (int(&r[0]), int(&r[1]))).collect()
    }
}

fn run(seed: u64) -> Stats {
    let mut rng = Rng(seed.wrapping_mul(0x9E37_79B9_7F4A_7C15) | 1);
    NOW.with(|n| n.set(T));
    // Two clocks within the bound but 80 s apart, one exact, one ahead later.
    for (i, o) in [0, 40_000, -40_000, 0].into_iter().enumerate() {
        set_offset(i, o);
    }
    let mut server = Database::new(1000);
    server.set_clock(now);
    let devices = (0..DEVICES).map(|i| Device::new(i as u32 + 1, CLOCKS[i])).collect();
    let mut sim = Sim { server, log: Vec::new(), devices, oracle: BTreeMap::new(), stats: Stats::default() };

    sim.devices[0].exec("CREATE TABLE t (id INT PRIMARY KEY, v INT)");
    for i in 0..DEVICES {
        sim.sync(i, &mut rng);
    }
    for step in 0..STEPS {
        NOW.with(|n| n.set(n.get() + rng.below(1500)));
        // Device 3 runs an hour ahead for a quarter of the run.
        set_offset(3, if (STEPS / 4..STEPS / 2).contains(&step) { HOUR } else { 0 });
        // Scripted in every run: the device ahead writes and tries to sync,
        // and a row is deleted while another device, cut off, changes it.
        if step == STEPS / 4 + 1 {
            sim.devices[3].online = true;
            sim.devices[3].exec("INSERT OR REPLACE INTO t VALUES (11, 1)");
            sim.sync(3, &mut rng);
        }
        if step == STEPS / 3 {
            sim.devices[2].online = false;
            sim.stats.partitions += 1;
            sim.devices[2].exec("INSERT OR REPLACE INTO t VALUES (5, 2)");
            sim.devices[0].exec("DELETE FROM t WHERE id = 5");
            sim.stats.deletes += 1;
            sim.stats.deletes_in_partition += 1;
            sim.sync(0, &mut rng);
        }
        let r = rng.below(DEVICES as u64) as usize;
        let key = rng.below(12);
        let val = rng.below(1000);
        match rng.below(20) {
            0..=6 => {
                sim.devices[r].exec(&format!("INSERT OR REPLACE INTO t VALUES ({key}, {val})"));
                sim.stats.writes += 1;
            }
            7..=9 => {
                sim.devices[r].exec(&format!("UPDATE t SET v = {val} WHERE id = {key}"));
                sim.stats.writes += 1;
            }
            10..=12 => {
                sim.devices[r].exec(&format!("DELETE FROM t WHERE id = {key}"));
                sim.stats.deletes += 1;
                if sim.devices.iter().any(|d| !d.online || d.paused) {
                    sim.stats.deletes_in_partition += 1;
                }
            }
            13..=16 => sim.sync(r, &mut rng),
            17 => {
                // Partition: the device loses (or regains) the network.
                sim.devices[r].online = !sim.devices[r].online;
                sim.stats.partitions += !sim.devices[r].online as usize;
            }
            18 => {
                sim.devices[r].restart();
                sim.stats.restarts += 1;
            }
            _ => sim.devices[r].checkpoint(),
        }
    }

    // Everybody is back online with a correct clock.
    for i in 0..DEVICES {
        set_offset(i, 0);
        sim.devices[i].online = true;
    }
    for round in 0.. {
        assert!(round < 100, "seed {seed}: synchronisation did not settle");
        for i in 0..DEVICES {
            sim.sync(i, &mut rng);
        }
        let settled = sim.devices.iter().all(|d| d.db.outbox_len() == 0 && d.cursor == sim.log.len() && !d.paused);
        if settled {
            break;
        }
    }

    let expected: Vec<(i64, i64)> = sim.oracle.iter().filter_map(|(&k, (_, v))| v.map(|v| (k, v))).collect();
    sim.stats.deleted_at_end = sim.oracle.values().filter(|(_, v)| v.is_none()).count();
    assert_eq!(Sim::rows(&mut sim.server), expected, "seed {seed}: server differs from the newest sent changes");
    for (i, d) in sim.devices.iter_mut().enumerate() {
        d.restart();
        assert_eq!(Sim::rows(&mut d.db), expected, "seed {seed}: device {i} diverged");
    }
    sim.stats
}

/// Seeds run by default; `FLATSQL_SYNC_SEEDS` sets another number.
fn seeds() -> u64 {
    std::env::var("FLATSQL_SYNC_SEEDS").ok().and_then(|v| v.parse().ok()).unwrap_or(40)
}

#[test]
fn sync_converges_despite_skewed_clocks_partitions_and_retries() {
    let mut total = Stats::default();
    let n = seeds();
    for seed in 1..=n {
        let s = run(seed);
        total.writes += s.writes;
        total.deletes += s.deletes;
        total.deletes_in_partition += s.deletes_in_partition;
        total.syncs += s.syncs;
        total.paused_rounds += s.paused_rounds;
        total.drift_rejected_push += s.drift_rejected_push;
        total.drift_rejected_pull += s.drift_rejected_pull;
        total.restamped += s.restamped;
        total.lost_acks += s.lost_acks;
        total.lost_cursors += s.lost_cursors;
        total.restarts += s.restarts;
        total.partitions += s.partitions;
        total.deleted_at_end += s.deleted_at_end;
        assert!(s.drift_rejected_push + s.paused_rounds > 0, "seed {seed}: the device ahead was never stopped");
    }
    eprintln!("{n} seeds x {STEPS} steps, {DEVICES} devices: {total:?}");
}
