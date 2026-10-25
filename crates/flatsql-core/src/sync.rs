//! Local-First synchronisation primitives.
//!
//! * [`Hlc`] – hybrid logical clock producing monotonic [`Stamp`]s,
//! * [`Change`] – a row level (or schema level) change exchanged between
//!   replicas,
//! * [`Outbox`] – local changes that have not been acknowledged by the server
//!   yet; it survives restarts because every entry is rebuilt from the WAL.
//!
//! Conflicts are resolved with last-writer-wins per primary key: a change is
//! applied only if its stamp is greater than the stamp of the row (or of its
//! tombstone). Because the stamp order is total and identical on every
//! replica, replicas converge regardless of the order in which they receive
//! changes.

use std::collections::BTreeMap;
use std::fmt;
use std::sync::Arc;

use crate::error::{Error, Result};
use crate::storage::codec::{crc32, Reader, Writer};
use crate::table::TableSchema;
use crate::value::Value;

/// Hybrid logical clock timestamp + id of the replica that produced it.
/// Ordering: by `time`, ties broken by `node`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct Stamp {
    /// `physical_ms << 16 | logical_counter`
    pub time: u64,
    pub node: u32,
}

impl Stamp {
    pub fn physical_ms(&self) -> u64 {
        self.time >> 16
    }
}

impl fmt::Display for Stamp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}@{:08x}", self.time >> 16, self.time & 0xffff, self.node)
    }
}

/// How far ahead of the local wall clock a stamp from another replica may be.
/// A replica whose clock runs further ahead would otherwise win every conflict
/// and drag every other clock along (Kulkarni et al. reject such messages).
pub const MAX_DRIFT_MS: u64 = 60_000;

/// Hybrid logical clock (Kulkarni et al.): physical milliseconds in the high
/// bits, a logical counter in the low 16 bits. It never goes backwards, even if
/// the wall clock does, and it absorbs timestamps observed from other replicas
/// unless they are more than [`MAX_DRIFT_MS`] ahead of the wall clock.
#[derive(Debug, Clone)]
pub struct Hlc {
    last: u64,
    now_ms: fn() -> u64,
}

impl Hlc {
    pub fn new(now_ms: fn() -> u64) -> Self {
        Hlc { last: 0, now_ms }
    }

    pub fn tick(&mut self) -> u64 {
        self.tick_at((self.now_ms)())
    }

    /// Like [`tick`](Self::tick) with a wall clock reading taken earlier: a
    /// statement reads the clock once and stamps all its rows with it, each
    /// stamp still greater than the previous one.
    pub fn tick_at(&mut self, now_ms: u64) -> u64 {
        let physical = now_ms.saturating_mul(1 << 16);
        self.last = physical.max(self.last + 1);
        self.last
    }

    /// Current wall clock in milliseconds.
    pub fn wall_ms(&self) -> u64 {
        (self.now_ms)()
    }

    /// Largest stamp time accepted from another replica right now.
    pub fn bound(&self) -> u64 {
        self.wall_ms().saturating_add(MAX_DRIFT_MS).saturating_mul(1 << 16) | 0xffff
    }

    /// Absorbs a stamp received from another replica. Returns `false` (and
    /// leaves the clock alone) when the stamp is beyond the drift bound.
    pub fn observe(&mut self, time: u64) -> bool {
        if time > self.bound() {
            return false;
        }
        self.last = self.last.max(time);
        true
    }

    /// Absorbs a stamp from this replica's own history (snapshot, WAL), which
    /// was accepted when it was written.
    pub fn absorb(&mut self, time: u64) {
        self.last = self.last.max(time);
    }

    /// Moves the clock to `time`, also backwards (see `Database::rebase_clock`).
    pub(crate) fn reset(&mut self, time: u64) {
        self.last = time;
    }

    pub fn last(&self) -> u64 {
        self.last
    }

    pub fn set_source(&mut self, now_ms: fn() -> u64) {
        self.now_ms = now_ms;
    }
}

/// Wall clock in milliseconds. `wasm32-unknown-unknown` has no clock, so the
/// host (JavaScript) injects `Date.now` through `Database::set_clock`.
pub fn system_now_ms() -> u64 {
    #[cfg(not(target_arch = "wasm32"))]
    {
        std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_millis() as u64).unwrap_or(0)
    }
    #[cfg(target_arch = "wasm32")]
    {
        0
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Mutation {
    CreateTable(TableSchema),
    DropTable(String),
    CreateIndex { table: String, name: String, column: String },
    DropIndex { table: String, name: String },
    Put { table: String, values: Vec<Value> },
    Delete { table: String, key: Value },
}

#[derive(Debug, Clone, PartialEq)]
pub struct Change {
    pub stamp: Stamp,
    pub mutation: Mutation,
}

const M_CREATE_TABLE: u8 = 1;
const M_DROP_TABLE: u8 = 2;
const M_CREATE_INDEX: u8 = 3;
const M_DROP_INDEX: u8 = 4;
const M_PUT: u8 = 5;
const M_DELETE: u8 = 6;

pub(crate) fn encode_put(w: &mut Writer, stamp: Stamp, table: &str, values: &[Value]) {
    w.u8(M_PUT);
    w.stamp(stamp);
    w.str(table);
    w.values(values);
}

pub(crate) fn encode_delete(w: &mut Writer, stamp: Stamp, table: &str, key: &Value) {
    w.u8(M_DELETE);
    w.stamp(stamp);
    w.str(table);
    w.value(key);
}

pub(crate) fn encode_change(w: &mut Writer, c: &Change) {
    match &c.mutation {
        Mutation::Put { table, values } => encode_put(w, c.stamp, table, values),
        Mutation::Delete { table, key } => encode_delete(w, c.stamp, table, key),
        Mutation::CreateTable(schema) => {
            w.u8(M_CREATE_TABLE);
            w.stamp(c.stamp);
            w.schema(schema);
        }
        Mutation::DropTable(name) => {
            w.u8(M_DROP_TABLE);
            w.stamp(c.stamp);
            w.str(name);
        }
        Mutation::CreateIndex { table, name, column } => {
            w.u8(M_CREATE_INDEX);
            w.stamp(c.stamp);
            w.str(table);
            w.str(name);
            w.str(column);
        }
        Mutation::DropIndex { table, name } => {
            w.u8(M_DROP_INDEX);
            w.stamp(c.stamp);
            w.str(table);
            w.str(name);
        }
    }
}

pub(crate) fn decode_change(r: &mut Reader) -> Result<Change> {
    let tag = r.u8()?;
    let stamp = r.stamp()?;
    let mutation = match tag {
        M_PUT => Mutation::Put { table: r.str()?, values: r.values()? },
        M_DELETE => Mutation::Delete { table: r.str()?, key: r.value()? },
        M_CREATE_TABLE => Mutation::CreateTable(r.schema()?),
        M_DROP_TABLE => Mutation::DropTable(r.str()?),
        M_CREATE_INDEX => Mutation::CreateIndex { table: r.str()?, name: r.str()?, column: r.str()? },
        M_DROP_INDEX => Mutation::DropIndex { table: r.str()?, name: r.str()? },
        t => return Err(Error::Storage(format!("unknown change tag {t}"))),
    };
    Ok(Change { stamp, mutation })
}

const BATCH_MAGIC: &[u8; 4] = b"FSB1";

/// Encodes changes as one batch chunk: `"FSB1" count changes... crc32`.
/// Concatenated chunks form a valid batch, so a server can relay several
/// stored batches without re-encoding them.
pub fn encode_batch(changes: &[Change]) -> Vec<u8> {
    let mut w = Writer::with_capacity(16 + changes.len() * 32);
    w.raw(BATCH_MAGIC);
    w.var(changes.len() as u64);
    for c in changes {
        encode_change(&mut w, c);
    }
    let crc = crc32(&w.buf);
    w.u32le(crc);
    w.into_inner()
}

pub fn decode_batch(data: &[u8]) -> Result<Vec<Change>> {
    let mut out = Vec::new();
    let mut r = Reader::new(data);
    while !r.is_empty() {
        let start = r.pos();
        if r.take(4)? != BATCH_MAGIC {
            return Err(Error::Storage("not a Flat-SQL change batch".into()));
        }
        let n = r.len()?;
        let mut chunk = Vec::with_capacity(n);
        for _ in 0..n {
            chunk.push(decode_change(&mut r)?);
        }
        let end = r.pos();
        let crc = r.u32le()?;
        if crc != crc32(&data[start..end]) {
            return Err(Error::Storage("change batch checksum mismatch".into()));
        }
        out.extend(chunk);
    }
    Ok(out)
}

/// JSON view of a batch, for debugging tools and the demo UI.
pub fn changes_to_json(changes: &[Change]) -> String {
    let mut s = String::from("[");
    for (i, c) in changes.iter().enumerate() {
        if i > 0 {
            s.push(',');
        }
        s.push_str("{\"stamp\":");
        crate::value::write_json_string(&c.stamp.to_string(), &mut s);
        s.push_str(",\"node\":");
        s.push_str(&c.stamp.node.to_string());
        let (op, table) = match &c.mutation {
            Mutation::CreateTable(sc) => ("createTable", sc.name.as_str()),
            Mutation::DropTable(t) => ("dropTable", t.as_str()),
            Mutation::CreateIndex { table, .. } => ("createIndex", table.as_str()),
            Mutation::DropIndex { table, .. } => ("dropIndex", table.as_str()),
            Mutation::Put { table, .. } => ("put", table.as_str()),
            Mutation::Delete { table, .. } => ("delete", table.as_str()),
        };
        s.push_str(",\"op\":\"");
        s.push_str(op);
        s.push_str("\",\"table\":");
        crate::value::write_json_string(table, &mut s);
        match &c.mutation {
            Mutation::Put { values, .. } => {
                s.push_str(",\"values\":[");
                for (j, v) in values.iter().enumerate() {
                    if j > 0 {
                        s.push(',');
                    }
                    v.write_json(&mut s);
                }
                s.push(']');
            }
            Mutation::Delete { key, .. } => {
                s.push_str(",\"key\":");
                key.write_json(&mut s);
            }
            Mutation::CreateIndex { name, column, .. } => {
                s.push_str(",\"index\":");
                crate::value::write_json_string(name, &mut s);
                s.push_str(",\"column\":");
                crate::value::write_json_string(column, &mut s);
            }
            Mutation::DropIndex { name, .. } => {
                s.push_str(",\"index\":");
                crate::value::write_json_string(name, &mut s);
            }
            _ => {}
        }
        s.push('}');
    }
    s.push(']');
    s
}

/// Outcome of applying a batch of remote changes.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct ApplyReport {
    /// Changes that won last-writer-wins and modified local state.
    pub applied: usize,
    /// Changes older than (or equal to) the local state – no-ops.
    pub ignored: usize,
    /// Changes that could not be applied (unknown table, schema mismatch,
    /// stamp beyond the drift bound...).
    pub rejected: usize,
    /// Rejected changes whose stamp was more than [`MAX_DRIFT_MS`] ahead of
    /// the local wall clock (included in `rejected`).
    pub clock_drift: usize,
    /// Batch with only the applied changes (a relay server forwards this).
    pub accepted: Vec<u8>,
    /// First few rejection reasons.
    pub errors: Vec<String>,
    /// Tables defined differently here and on the sending replica; their
    /// rows were rejected (included in `rejected`).
    pub conflicts: Vec<SchemaConflict>,
}

/// A table that another replica defines differently. Until table versions
/// exist, rows of such a table are not applied; the conflict is reported
/// once per table instead of one error per row.
#[derive(Debug, Clone, PartialEq)]
pub struct SchemaConflict {
    pub table: String,
    /// The local definition (`name (column TYPE, ...)`).
    pub local: String,
    /// The other replica's definition, `None` when only rows arrived.
    pub remote: Option<String>,
    /// `Some(true)`: one definition only adds nullable columns at the end of
    /// the other, the change allowed between application versions;
    /// `Some(false)`: incompatible (type, primary key, order or names);
    /// `None`: the other definition is unknown.
    pub additive: Option<bool>,
    /// Rejected rows and deletes of the table.
    pub rows: usize,
}

/// Entry of the outbox: either a dirty row (its current state is read when
/// the outbox is exported, which coalesces repeated updates of one row) or a
/// schema change stored verbatim.
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum OutboxItem {
    Row { table: Arc<str>, key: Value },
    Schema(Change),
}

#[derive(Debug, Clone, Default)]
pub(crate) struct Outbox {
    pub(crate) by_seq: BTreeMap<u64, OutboxItem>,
    /// Pending rows of each table (by catalog key): key -> seq of the entry.
    rows: BTreeMap<Arc<str>, BTreeMap<Value, u64>>,
}

impl Outbox {
    pub fn len(&self) -> usize {
        self.by_seq.len()
    }

    pub fn insert(&mut self, seq: u64, item: OutboxItem) {
        if let OutboxItem::Row { table, key } = &item {
            let keys = match self.rows.get_mut(&**table) {
                Some(keys) => keys,
                None => self.rows.entry(table.clone()).or_default(),
            };
            if let Some(old) = keys.insert(key.clone(), seq) {
                self.by_seq.remove(&old);
            }
        }
        self.by_seq.insert(seq, item);
    }

    /// Drops every entry with `seq <= upto`.
    pub fn ack(&mut self, upto: u64) {
        let rest = self.by_seq.split_off(&(upto.saturating_add(1)));
        let done = std::mem::replace(&mut self.by_seq, rest);
        for (seq, item) in done {
            if let OutboxItem::Row { table, key } = item {
                if let Some(keys) = self.rows.get_mut(&*table) {
                    if keys.get(&key) == Some(&seq) {
                        keys.remove(&key);
                        if keys.is_empty() {
                            self.rows.remove(&*table);
                        }
                    }
                }
            }
        }
    }

    /// Pending rows grouped by table (the snapshot stores them this way).
    pub fn rows(&self) -> impl Iterator<Item = (&Arc<str>, &BTreeMap<Value, u64>)> {
        self.rows.iter()
    }

    /// Pending schema changes in order.
    pub fn schema_changes(&self) -> impl Iterator<Item = (u64, &Change)> {
        self.by_seq.iter().filter_map(|(seq, item)| match item {
            OutboxItem::Schema(c) => Some((*seq, c)),
            OutboxItem::Row { .. } => None,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::table::Column;
    use crate::value::DataType;

    #[test]
    fn hlc_is_monotonic_even_if_wall_clock_stalls() {
        fn frozen() -> u64 {
            1000
        }
        let mut c = Hlc::new(frozen);
        let a = c.tick();
        let b = c.tick();
        assert!(b > a);
        assert!(c.observe(b + 100));
        assert!(c.tick() > b + 100);
        assert_eq!(a >> 16, 1000);
    }

    #[test]
    fn hlc_ignores_stamps_beyond_the_drift_bound() {
        fn frozen() -> u64 {
            1000
        }
        let mut c = Hlc::new(frozen);
        let ahead = (1000 + MAX_DRIFT_MS + 1) << 16;
        assert!(!c.observe(ahead));
        assert!(c.last() < ahead);
        assert!(c.observe((1000 + MAX_DRIFT_MS) << 16));
    }

    #[test]
    fn batch_roundtrip_and_concatenation() {
        let schema = TableSchema {
            name: "t".into(),
            columns: vec![Column { name: "id".into(), data_type: DataType::Int, not_null: true }],
            pk: 0,
        };
        let s = |t| Stamp { time: t, node: 7 };
        let a = vec![
            Change { stamp: s(1), mutation: Mutation::CreateTable(schema) },
            Change { stamp: s(2), mutation: Mutation::Put { table: "t".into(), values: vec![Value::Int(1)] } },
        ];
        let b = vec![Change { stamp: s(3), mutation: Mutation::Delete { table: "t".into(), key: Value::Int(1) } }];
        let mut bytes = encode_batch(&a);
        bytes.extend(encode_batch(&b));
        let decoded = decode_batch(&bytes).unwrap();
        assert_eq!(decoded.len(), 3);
        assert_eq!(decoded[..2], a[..]);
        assert_eq!(decoded[2], b[0]);
        assert!(changes_to_json(&decoded).contains("\"op\":\"delete\""));
    }

    #[test]
    fn corrupted_batch_is_rejected() {
        let c = vec![Change { stamp: Stamp::default(), mutation: Mutation::DropTable("x".into()) }];
        let mut bytes = encode_batch(&c);
        let n = bytes.len();
        bytes[n - 6] ^= 0xff;
        assert!(decode_batch(&bytes).is_err());
        assert!(decode_batch(b"nope").is_err());
    }

    #[test]
    fn outbox_coalesces_rows_and_acks_by_sequence() {
        let mut o = Outbox::default();
        let row = |k| OutboxItem::Row { table: "t".into(), key: Value::Int(k) };
        o.insert(1, row(1));
        o.insert(2, row(2));
        o.insert(3, row(1)); // row 1 modified again
        assert_eq!(o.len(), 2);
        o.ack(2);
        assert_eq!(o.len(), 1); // seq 3 (row 1) still pending
        o.ack(3);
        assert_eq!(o.len(), 0);
    }
}
