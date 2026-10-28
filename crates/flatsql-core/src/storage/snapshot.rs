//! Snapshot (checkpoint) format: a full image of the committed state.
//!
//! ```text
//! "FLATSQL1"  format  node_id  clock  lsn  next_seq  acked_seq  flags
//! tables:  schema created_stamp indexes[] rows[] tombstones[]
//! outbox:  schema changes[seq change]  tables[name rows[seq key]]
//! crc32 (u32 LE) of everything above
//! ```
//!
//! `flags` bit 0: synchronisation enabled. Format 1 (no flags, outbox as a
//! list of `seq kind payload` with the table name in every row entry) is
//! still read; a database in format 1 always kept its outbox.

use std::collections::BTreeMap;
use std::ops::Bound;

use crate::db::{catalog_key, Database};
use crate::error::{Error, Result};
use crate::storage::codec::{crc32, crc32_update, Reader, Writer};
use crate::sync::{decode_change, encode_change, Outbox, OutboxItem};
use crate::table::{RowEntry, Table};
use crate::value::Value;

const MAGIC: &[u8; 8] = b"FLATSQL1";
const FORMAT: u64 = 2;
const FORMAT_1: u64 = 1;
const OUTBOX_ROW: u8 = 1;
const OUTBOX_SCHEMA: u8 = 2;
const FLAG_SYNC: u8 = 1;

pub(crate) fn encode(db: &Database) -> Vec<u8> {
    let mut job = Job::begin(db);
    job.step(db, usize::MAX);
    job.finish()
}

/// A snapshot encoded in steps of a given number of rows, while the database
/// keeps changing between the steps. The snapshot is the state at
/// [`begin`](Job::begin): the header, the tombstones and the outbox are
/// encoded at once, the rows step by step in key order. A row the steps have
/// not reached yet is kept as it was before its first change
/// ([`keep`](Job::keep)), so its later changes do not enter the snapshot. A
/// table created later is not in it; a schema change or a clock rebase makes
/// the database start the job again.
pub(crate) struct Job {
    out: Writer,
    tables: Vec<JobTable>,
    /// Table being encoded, whether its header is written, the last key
    /// encoded in it.
    current: usize,
    started: bool,
    cursor: Option<Value>,
    outbox: Vec<u8>,
    /// CRC of the first `crc_at` bytes of `out`, computed step by step so
    /// that finishing does not go over the whole snapshot.
    crc: u32,
    crc_at: usize,
}

struct JobTable {
    key: String,
    /// Rows and encoded tombstones at the start.
    rows: usize,
    tombstones: Vec<u8>,
    /// Rows changed after the start before the steps reached them: their
    /// state at the start (`None`: the key did not exist).
    kept: BTreeMap<Value, Option<RowEntry>>,
    written: usize,
}

impl Job {
    pub(crate) fn begin(db: &Database) -> Job {
        let rows: usize = db.tables.values().map(|t| t.rows.len()).sum();
        let mut w = Writer::with_capacity(64 + rows * 24);
        w.raw(MAGIC);
        w.var(FORMAT);
        w.var(db.node_id as u64);
        w.var(db.clock.last());
        w.var(db.lsn);
        w.var(db.next_seq);
        w.var(db.acked_seq);
        w.u8(if db.sync { FLAG_SYNC } else { 0 });
        w.var(db.tables.len() as u64);

        let tables = db
            .tables
            .iter()
            .map(|(key, t)| {
                let mut tw = Writer::with_capacity(4 + t.tombstones.len() * 16);
                tw.var(t.tombstones.len() as u64);
                for (k, s) in &t.tombstones {
                    tw.value(k);
                    tw.stamp(*s);
                }
                JobTable {
                    key: key.clone(),
                    rows: t.rows.len(),
                    tombstones: tw.into_inner(),
                    kept: BTreeMap::new(),
                    written: 0,
                }
            })
            .collect();

        let mut ow = Writer::new();
        let schema: Vec<_> = db.outbox.schema_changes().collect();
        ow.var(schema.len() as u64);
        for (seq, change) in schema {
            ow.var(seq);
            encode_change(&mut ow, change);
        }
        let outbox_tables: Vec<_> = db.outbox.rows().collect();
        ow.var(outbox_tables.len() as u64);
        for (table, keys) in outbox_tables {
            ow.str(table);
            ow.var(keys.len() as u64);
            for (key, seq) in keys {
                ow.var(*seq);
                ow.value(key);
            }
        }
        Job { out: w, tables, current: 0, started: false, cursor: None, outbox: ow.into_inner(), crc: 0, crc_at: 0 }
    }

    /// Encodes up to `budget` rows of `db` (the database the job began on,
    /// with the same tables); returns whether every row is encoded.
    pub(crate) fn step(&mut self, db: &Database, budget: usize) -> bool {
        let done = self.encode_rows(db, budget);
        self.crc = crc32_update(self.crc, &self.out.buf[self.crc_at..]);
        self.crc_at = self.out.buf.len();
        done
    }

    fn encode_rows(&mut self, db: &Database, budget: usize) -> bool {
        let Job { out, tables, current, started, cursor, .. } = self;
        let mut left = budget;
        while let Some(jt) = tables.get_mut(*current) {
            let t = &db.tables[&jt.key];
            if !*started {
                out.schema(&t.schema);
                out.stamp(t.created);
                out.var(t.indexes.len() as u64);
                for idx in &t.indexes {
                    out.str(&idx.name);
                    out.var(idx.column as u64);
                }
                out.var(jt.rows as u64);
                *started = true;
            }
            // Rows after the cursor: the table's rows, and in place of those
            // changed since the start, their kept state.
            let from = match cursor.as_ref() {
                Some(c) => Bound::Excluded(c),
                None => Bound::Unbounded,
            };
            let mut live = t.rows.range::<Value, _>((from, Bound::Unbounded)).peekable();
            let mut kept = jt.kept.range::<Value, _>((from, Bound::Unbounded)).peekable();
            let mut last = None;
            loop {
                if left == 0 {
                    if let Some(k) = last {
                        *cursor = Some(Value::clone(k));
                    }
                    return false;
                }
                let (key, row) = match (live.peek(), kept.peek()) {
                    (None, None) => break,
                    (Some(&(lk, _)), Some(&(kk, _))) if kk <= lk => {
                        if kk == lk {
                            live.next();
                        }
                        let (k, row) = kept.next().unwrap();
                        (k, row.as_ref())
                    }
                    (Some(_), _) => {
                        let (k, row) = live.next().unwrap();
                        (k, Some(row))
                    }
                    (None, Some(_)) => {
                        let (k, row) = kept.next().unwrap();
                        (k, row.as_ref())
                    }
                };
                last = Some(key);
                if let Some(row) = row {
                    out.stamp(row.stamp);
                    for v in &row.values {
                        out.value(v);
                    }
                    jt.written += 1;
                    left -= 1;
                }
            }
            debug_assert_eq!(jt.written, jt.rows, "rows of table {} at the start", jt.key);
            out.raw(&jt.tombstones);
            *current += 1;
            *started = false;
            *cursor = None;
        }
        true
    }

    /// Whether every row is encoded.
    pub(crate) fn done(&self) -> bool {
        self.current == self.tables.len()
    }

    /// The snapshot, once [`step`](Job::step) returned true.
    pub(crate) fn finish(self) -> Vec<u8> {
        let mut w = self.out;
        w.raw(&self.outbox);
        let crc = crc32_update(self.crc, &w.buf[self.crc_at..]);
        w.u32le(crc);
        w.into_inner()
    }

    /// Called before row `key` of table `tkey` changes: keeps the row as it
    /// was at the start if the steps have not reached it yet.
    pub(crate) fn keep(&mut self, tables: &BTreeMap<String, Table>, tkey: &str, key: &Value) {
        let Some(i) = self.tables.iter().position(|t| t.key == tkey) else { return };
        let reached = i < self.current || (i == self.current && self.cursor.as_ref().is_some_and(|c| key <= c));
        if reached {
            return;
        }
        let jt = &mut self.tables[i];
        if !jt.kept.contains_key(key) {
            let row = tables.get(tkey).and_then(|t| t.rows.get(key)).cloned();
            jt.kept.insert(key.clone(), row);
        }
    }
}

pub(crate) fn decode(data: &[u8]) -> Result<Database> {
    if data.len() < MAGIC.len() + 4 || &data[..MAGIC.len()] != MAGIC {
        return Err(Error::Storage("not a Flat-SQL snapshot".into()));
    }
    let body = &data[..data.len() - 4];
    let crc = u32::from_le_bytes(data[data.len() - 4..].try_into().unwrap());
    if crc32(body) != crc {
        return Err(Error::Storage("snapshot checksum mismatch".into()));
    }
    let mut r = Reader::new(body);
    r.take(MAGIC.len())?;
    let format = r.var()?;
    if format != FORMAT && format != FORMAT_1 {
        return Err(Error::Storage(format!("unsupported snapshot format {format}")));
    }
    let node_id = u32::try_from(r.var()?).map_err(|_| Error::Storage("node id out of range".into()))?;
    let mut db = Database::new(node_id);
    db.clock.absorb(r.var()?);
    db.lsn = r.var()?;
    db.next_seq = r.var()?;
    db.acked_seq = r.var()?;
    db.sync = format == FORMAT_1 || r.u8()? & FLAG_SYNC != 0;

    let mut tables = BTreeMap::new();
    for _ in 0..r.len()? {
        let schema = r.schema()?;
        let created = r.stamp()?;
        let mut indexes = Vec::new();
        for _ in 0..r.len()? {
            let name = r.str()?;
            let column = r.var()? as usize;
            if column >= schema.columns.len() {
                return Err(Error::Storage(format!("index '{name}' refers to a missing column")));
            }
            indexes.push((name, column));
        }
        let mut table = Table::new(schema, created);
        let ncols = table.schema.columns.len();
        let pk = table.schema.pk;
        let n = r.len()?;
        let mut rows = Vec::with_capacity(n.min(1 << 20));
        for _ in 0..n {
            let stamp = r.stamp()?;
            let mut values = Vec::with_capacity(ncols);
            for _ in 0..ncols {
                values.push(r.value()?);
            }
            rows.push((values[pk].clone(), RowEntry { values, stamp }));
        }
        // Rows are stored in key order, so the tree can be bulk-built.
        table.rows = rows.into_iter().collect();
        for _ in 0..r.len()? {
            let key = r.value()?;
            let stamp = r.stamp()?;
            table.tombstones.insert(key, stamp);
        }
        // Indexes are rebuilt from rows instead of being stored.
        for (name, column) in indexes {
            table.add_index(name, column);
        }
        tables.insert(catalog_key(&table.schema.name).into_owned(), table);
    }
    db.tables = tables;

    let mut outbox = Outbox::default();
    if format == FORMAT_1 {
        for _ in 0..r.len()? {
            let seq = r.var()?;
            let item = match r.u8()? {
                OUTBOX_ROW => OutboxItem::Row { table: r.str()?.into(), key: r.value()? },
                OUTBOX_SCHEMA => OutboxItem::Schema(decode_change(&mut r)?),
                t => return Err(Error::Storage(format!("unknown outbox entry kind {t}"))),
            };
            outbox.insert(seq, item);
        }
    } else {
        for _ in 0..r.len()? {
            let seq = r.var()?;
            outbox.insert(seq, OutboxItem::Schema(decode_change(&mut r)?));
        }
        for _ in 0..r.len()? {
            let table: std::sync::Arc<str> = r.str()?.into();
            for _ in 0..r.len()? {
                let seq = r.var()?;
                outbox.insert(seq, OutboxItem::Row { table: table.clone(), key: r.value()? });
            }
        }
    }
    db.outbox = outbox;
    if !r.is_empty() {
        return Err(Error::Storage("trailing bytes in snapshot".into()));
    }
    Ok(db)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Format 1 as the previous version wrote it.
    fn encode_format_1(db: &Database) -> Vec<u8> {
        let mut w = Writer::new();
        w.raw(MAGIC);
        w.var(FORMAT_1);
        w.var(db.node_id as u64);
        w.var(db.clock.last());
        w.var(db.lsn);
        w.var(db.next_seq);
        w.var(db.acked_seq);
        w.var(db.tables.len() as u64);
        for t in db.tables.values() {
            w.schema(&t.schema);
            w.stamp(t.created);
            w.var(t.indexes.len() as u64);
            for idx in &t.indexes {
                w.str(&idx.name);
                w.var(idx.column as u64);
            }
            w.var(t.rows.len() as u64);
            for row in t.rows.values() {
                w.stamp(row.stamp);
                for v in &row.values {
                    w.value(v);
                }
            }
            w.var(t.tombstones.len() as u64);
            for (k, s) in &t.tombstones {
                w.value(k);
                w.stamp(*s);
            }
        }
        w.var(db.outbox.by_seq.len() as u64);
        for (seq, item) in &db.outbox.by_seq {
            w.var(*seq);
            match item {
                OutboxItem::Row { table, key } => {
                    w.u8(OUTBOX_ROW);
                    w.str(table);
                    w.value(key);
                }
                OutboxItem::Schema(c) => {
                    w.u8(OUTBOX_SCHEMA);
                    encode_change(&mut w, c);
                }
            }
        }
        let crc = crc32(&w.buf);
        w.u32le(crc);
        w.into_inner()
    }

    #[test]
    fn snapshot_of_format_1_is_read_with_its_outbox() {
        let mut db = Database::new(3);
        db.enable_sync().unwrap();
        db.execute("CREATE TABLE t (id INT PRIMARY KEY, v TEXT); CREATE INDEX t_v ON t (v)").unwrap();
        db.execute("INSERT INTO t VALUES (1, 'a'), (2, 'b'), (3, 'c'); DELETE FROM t WHERE id = 2").unwrap();
        let old = encode_format_1(&db);
        let read = decode(&old).unwrap();
        assert!(read.sync, "a database in format 1 kept its outbox");
        assert_eq!(read.outbox.len(), db.outbox.len());
        assert_eq!(encode(&read), encode(&db), "the same state in format 2");
    }
}
