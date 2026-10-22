//! Snapshot (checkpoint) format: a full image of the committed state.
//!
//! ```text
//! "FLATSQL1"  format  node_id  clock  lsn  next_seq  acked_seq
//! tables:  schema created_stamp indexes[] rows[] tombstones[]
//! outbox:  seq kind payload
//! crc32 (u32 LE) of everything above
//! ```

use std::collections::BTreeMap;

use crate::db::{catalog_key, Database};
use crate::error::{Error, Result};
use crate::storage::codec::{crc32, Reader, Writer};
use crate::sync::{decode_change, encode_change, Outbox, OutboxItem};
use crate::table::{RowEntry, Table};

const MAGIC: &[u8; 8] = b"FLATSQL1";
const FORMAT: u64 = 1;
const OUTBOX_ROW: u8 = 1;
const OUTBOX_SCHEMA: u8 = 2;

pub(crate) fn encode(db: &Database) -> Vec<u8> {
    let rows: usize = db.tables.values().map(|t| t.rows.len()).sum();
    let mut w = Writer::with_capacity(64 + rows * 24);
    w.raw(MAGIC);
    w.var(FORMAT);
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
    if format != FORMAT {
        return Err(Error::Storage(format!("unsupported snapshot format {format}")));
    }
    let node_id = u32::try_from(r.var()?).map_err(|_| Error::Storage("node id out of range".into()))?;
    let mut db = Database::new(node_id);
    db.clock.observe(r.var()?);
    db.lsn = r.var()?;
    db.next_seq = r.var()?;
    db.acked_seq = r.var()?;

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
    for _ in 0..r.len()? {
        let seq = r.var()?;
        let item = match r.u8()? {
            OUTBOX_ROW => OutboxItem::Row { table: r.str()?.into(), key: r.value()? },
            OUTBOX_SCHEMA => OutboxItem::Schema(decode_change(&mut r)?),
            t => return Err(Error::Storage(format!("unknown outbox entry kind {t}"))),
        };
        outbox.insert(seq, item);
    }
    db.outbox = outbox;
    if !r.is_empty() {
        return Err(Error::Storage("trailing bytes in snapshot".into()));
    }
    Ok(db)
}
