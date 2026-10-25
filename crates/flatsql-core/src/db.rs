//! The database object: catalog, transactions, WAL production, crash
//! recovery and the synchronisation API.
//!
//! The engine itself never performs I/O. Every committed transaction is
//! encoded as a WAL frame into an internal buffer; the host (the native
//! `Persistent` wrapper, or JavaScript in the browser) drains it with
//! [`Database::take_wal`] and writes it to durable storage. A full image of
//! the database is produced by [`Database::snapshot`] (checkpoint).

use std::borrow::Cow;
use std::cell::RefCell;
use std::collections::{BTreeMap, HashSet};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

use crate::error::{Error, Result};
use crate::exec;
use crate::expr::missing_param;
use crate::result::QueryResult;
use crate::sql::{self, Statement};
use crate::storage::codec::Writer;
use crate::storage::snapshot;
use crate::storage::wal::{self, LogOp, WalReader, OP_ACK, OP_CHANGE, OP_SYNC};
use crate::sync::{
    decode_batch, encode_batch, encode_change, encode_delete, encode_put, system_now_ms, ApplyReport, Change, Hlc,
    Mutation, Outbox, OutboxItem, SchemaConflict, Stamp, MAX_DRIFT_MS,
};
use crate::table::{Index, RowEntry, Table, TableSchema};
use crate::value::Value;

/// Table and index names are case-insensitive. Already lower-case names
/// (the common case) are used without allocating.
pub(crate) fn catalog_key(name: &str) -> Cow<'_, str> {
    if name.chars().any(char::is_uppercase) {
        Cow::Owned(name.to_lowercase())
    } else {
        Cow::Borrowed(name)
    }
}

fn no_table(name: &str) -> Error {
    Error::Schema(format!("no such table '{name}'"))
}

/// Outcome of one remote change.
enum Remote {
    Applied,
    /// The local state is newer (or the same).
    Ignored,
    /// The change does not fit the local definition of its table; `remote`
    /// is the other definition when the change carries it.
    Conflict {
        table: String,
        remote: Option<TableSchema>,
    },
}

/// What happened while opening a database from a snapshot + WAL.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct RecoveryReport {
    pub snapshot_bytes: usize,
    pub snapshot_lsn: u64,
    pub wal_bytes: usize,
    pub frames_replayed: u64,
    /// Frames already contained in the snapshot (crash between checkpoint and
    /// WAL truncation).
    pub frames_skipped: u64,
    /// Bytes of a torn / corrupted WAL tail that were discarded.
    pub torn_bytes: usize,
    pub stop_reason: Option<String>,
    pub ops_failed: u64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DbStats {
    pub node_id: u32,
    pub tables: usize,
    pub rows: usize,
    pub indexes: usize,
    pub tombstones: usize,
    pub lsn: u64,
    pub wal_pending: usize,
    pub outbox: usize,
    pub next_seq: u64,
    pub acked_seq: u64,
    pub clock: u64,
    pub in_transaction: bool,
    /// Whether local row changes are kept in the outbox.
    pub sync: bool,
}

/// A parsed statement that can be executed many times with different
/// `?` parameters. SELECT, INSERT, UPDATE and DELETE also keep their
/// compiled form (names resolved, access path chosen or prepared as a
/// template), built when the statement is prepared and again when the
/// schema of the database it runs on has changed since.
#[derive(Debug, Clone)]
pub struct Prepared {
    stmt: Statement,
    params: usize,
    /// Compiled form and the catalog version it was built for.
    compiled: RefCell<Option<(u64, exec::Compiled)>>,
}

impl Prepared {
    pub fn param_count(&self) -> usize {
        self.params
    }

    pub fn statement(&self) -> &Statement {
        &self.stmt
    }
}

enum Undo {
    /// Restores a row (or its absence) and the tombstone that existed before.
    Row {
        table: Arc<str>,
        key: Value,
        prev: Option<RowEntry>,
        prev_tomb: Option<Stamp>,
    },
    /// Restores the columns and the stamp of a row changed in place.
    Update {
        table: Arc<str>,
        key: Value,
        cols: Vec<(usize, Value)>,
        prev_stamp: Stamp,
    },
    CreateTable(String),
    DropTable(Box<Table>),
    CreateIndex {
        table: String,
        name: String,
    },
    DropIndex {
        table: String,
        pos: usize,
        index: Index,
    },
}

#[derive(Default)]
struct Txn {
    explicit: bool,
    seq_start: u64,
    undo: Vec<Undo>,
    /// Encoded WAL ops, written as a single frame on commit.
    ops: Vec<u8>,
    op_count: u64,
    /// Outbox entries, published on commit.
    outbox: Vec<(u64, OutboxItem)>,
}

/// Savepoint used for statement level atomicity.
#[derive(Clone, Copy)]
struct Mark {
    undo: usize,
    ops: usize,
    op_count: u64,
    outbox: usize,
    next_seq: u64,
}

/// A new catalog version. The counter is shared by every database of the
/// process, so a version also identifies the database.
fn next_catalog_version() -> u64 {
    static NEXT: AtomicU64 = AtomicU64::new(1);
    NEXT.fetch_add(1, Ordering::Relaxed)
}

pub struct Database {
    pub(crate) tables: BTreeMap<String, Table>,
    /// Changes with every change of the schema (tables and indexes); a
    /// compiled statement is valid for the version it was built for.
    catalog_version: u64,
    pub(crate) node_id: u32,
    pub(crate) clock: Hlc,
    pub(crate) lsn: u64,
    pub(crate) next_seq: u64,
    pub(crate) acked_seq: u64,
    pub(crate) outbox: Outbox,
    /// Synchronisation is enabled: local row changes enter the outbox. Until
    /// then only schema changes do; enabling marks every row and tombstone as
    /// pending, which is what the outbox of a database that never synced
    /// would hold.
    pub(crate) sync: bool,
    wal_out: Vec<u8>,
    txn: Option<Txn>,
    recovery: Option<RecoveryReport>,
}

impl Database {
    /// Creates an empty database. `node_id` identifies this replica in sync
    /// stamps and should be random and unique per device.
    pub fn new(node_id: u32) -> Self {
        Database {
            tables: BTreeMap::new(),
            catalog_version: next_catalog_version(),
            node_id,
            clock: Hlc::new(system_now_ms),
            lsn: 0,
            next_seq: 1,
            acked_seq: 0,
            outbox: Outbox::default(),
            sync: false,
            wal_out: Vec::new(),
            txn: None,
            recovery: None,
        }
    }

    /// Replaces the wall clock used for stamps (required on wasm32).
    pub fn set_clock(&mut self, now_ms: fn() -> u64) {
        self.clock.set_source(now_ms);
    }

    pub fn node_id(&self) -> u32 {
        self.node_id
    }

    // ------------------------------------------------------------------
    // Query API
    // ------------------------------------------------------------------

    /// Executes one or more statements and returns the result of the last one.
    /// Statements run in autocommit mode unless wrapped in BEGIN / COMMIT.
    pub fn execute(&mut self, sql: &str) -> Result<QueryResult> {
        let mut last = QueryResult::command("OK");
        for stmt in sql::parse(sql)? {
            last = self.execute_statement(&stmt, &[])?;
        }
        Ok(last)
    }

    /// Like [`execute`](Self::execute) but returns every statement's result.
    pub fn execute_all(&mut self, sql: &str) -> Result<Vec<QueryResult>> {
        let stmts = sql::parse(sql)?;
        let mut out = Vec::with_capacity(stmts.len());
        for stmt in &stmts {
            out.push(self.execute_statement(stmt, &[])?);
        }
        Ok(out)
    }

    pub fn prepare(&self, sql: &str) -> Result<Prepared> {
        let stmt = sql::parse_one(sql)?;
        let params = sql::param_count(&stmt);
        // A statement on a table that does not exist yet is compiled when it
        // runs, which also reports the error.
        let compiled = exec::compile(self, &stmt).ok().flatten().map(|c| (self.catalog_version, c));
        Ok(Prepared { stmt, params, compiled: RefCell::new(compiled) })
    }

    pub fn execute_prepared(&mut self, p: &Prepared, params: &[Value]) -> Result<QueryResult> {
        if params.len() != p.params {
            return Err(Error::Eval(format!("statement expects {} parameter(s), {} supplied", p.params, params.len())));
        }
        let fresh = matches!(&*p.compiled.borrow(), Some((version, _)) if *version == self.catalog_version);
        if !fresh {
            match exec::compile(self, &p.stmt)? {
                Some(c) => *p.compiled.borrow_mut() = Some((self.catalog_version, c)),
                None => return self.execute_statement(&p.stmt, params),
            }
        }
        let compiled = p.compiled.borrow();
        let (_, plan) = compiled.as_ref().expect("compiled above");
        self.run_compiled(plan, params)
    }

    pub fn execute_statement(&mut self, stmt: &Statement, params: &[Value]) -> Result<QueryResult> {
        match stmt {
            Statement::Begin => {
                if self.txn.is_some() {
                    return Err(Error::Transaction("a transaction is already active".into()));
                }
                self.txn = Some(Txn { explicit: true, seq_start: self.next_seq, ..Default::default() });
                return Ok(QueryResult::command("BEGIN"));
            }
            Statement::Commit => {
                if !self.txn.as_ref().is_some_and(|t| t.explicit) {
                    return Err(Error::Transaction("no active transaction".into()));
                }
                self.commit();
                return Ok(QueryResult::command("COMMIT"));
            }
            Statement::Rollback => {
                let Some(start) = self.txn.as_ref().filter(|t| t.explicit).map(|t| t.seq_start) else {
                    return Err(Error::Transaction("no active transaction".into()));
                };
                self.rollback_to(Mark { undo: 0, ops: 0, op_count: 0, outbox: 0, next_seq: start });
                self.txn = None;
                return Ok(QueryResult::command("ROLLBACK"));
            }
            _ => {}
        }

        if params.len() < sql::param_count(stmt) {
            return Err(missing_param(params.len(), params.len()));
        }
        if let Some(c) = exec::compile(self, stmt)? {
            return self.run_compiled(&c, params);
        }
        if stmt.is_read_only() {
            return exec::run_read(self, stmt, params);
        }
        self.in_statement(|db| exec::run_write(db, stmt))
    }

    fn run_compiled(&mut self, c: &exec::Compiled, params: &[Value]) -> Result<QueryResult> {
        match c {
            exec::Compiled::Select(p) => exec::run_select(self, p, params),
            write => self.in_statement(|db| exec::run_compiled_write(db, write, params)),
        }
    }

    /// Runs a write with statement level atomicity, in a transaction of its
    /// own unless one is open.
    fn in_statement(&mut self, f: impl FnOnce(&mut Database) -> Result<QueryResult>) -> Result<QueryResult> {
        let implicit = self.txn.is_none();
        if implicit {
            self.txn = Some(Txn { seq_start: self.next_seq, ..Default::default() });
        }
        let mark = self.mark();
        match f(self) {
            Ok(r) => {
                if implicit {
                    self.commit();
                }
                Ok(r)
            }
            Err(e) => {
                // Statement level atomicity: undo everything this statement did.
                self.rollback_to(mark);
                if implicit {
                    self.txn = None;
                }
                Err(e)
            }
        }
    }

    pub fn in_transaction(&self) -> bool {
        self.txn.is_some()
    }

    // ------------------------------------------------------------------
    // Catalog access
    // ------------------------------------------------------------------

    pub(crate) fn table(&self, name: &str) -> Result<&Table> {
        self.tables.get(&*catalog_key(name)).ok_or_else(|| no_table(name))
    }

    pub fn table_names(&self) -> Vec<String> {
        self.tables.values().map(|t| t.schema.name.clone()).collect()
    }

    pub fn table_schema(&self, name: &str) -> Option<&TableSchema> {
        self.tables.get(&*catalog_key(name)).map(|t| &*t.schema)
    }

    pub fn row_count(&self, name: &str) -> Option<usize> {
        self.tables.get(&*catalog_key(name)).map(|t| t.rows.len())
    }

    /// Finds the table owning an index (index names are unique per database).
    pub(crate) fn find_index(&self, name: &str) -> Option<(String, usize)> {
        self.tables.iter().find_map(|(k, t)| t.index_named(name).map(|i| (k.clone(), i)))
    }

    pub fn stats(&self) -> DbStats {
        DbStats {
            node_id: self.node_id,
            tables: self.tables.len(),
            rows: self.tables.values().map(|t| t.rows.len()).sum(),
            indexes: self.tables.values().map(|t| t.indexes.len()).sum(),
            tombstones: self.tables.values().map(|t| t.tombstones.len()).sum(),
            lsn: self.lsn,
            wal_pending: self.wal_out.len(),
            outbox: self.outbox.len(),
            next_seq: self.next_seq,
            acked_seq: self.acked_seq,
            clock: self.clock.last(),
            in_transaction: self.txn.is_some(),
            sync: self.sync,
        }
    }

    // ------------------------------------------------------------------
    // Transactions
    // ------------------------------------------------------------------

    fn mark(&self) -> Mark {
        let t = self.txn.as_ref().expect("mark outside of a transaction");
        Mark {
            undo: t.undo.len(),
            ops: t.ops.len(),
            op_count: t.op_count,
            outbox: t.outbox.len(),
            next_seq: self.next_seq,
        }
    }

    fn rollback_to(&mut self, m: Mark) {
        let Database { txn, tables, next_seq, catalog_version, .. } = self;
        let txn = txn.as_mut().expect("rollback outside of a transaction");
        let mut schema = false;
        while txn.undo.len() > m.undo {
            let undo = txn.undo.pop().unwrap();
            schema |= !matches!(undo, Undo::Row { .. });
            apply_undo(tables, undo);
        }
        if schema {
            *catalog_version = next_catalog_version();
        }
        txn.ops.truncate(m.ops);
        txn.op_count = m.op_count;
        txn.outbox.truncate(m.outbox);
        *next_seq = m.next_seq;
    }

    fn commit(&mut self) {
        let Some(txn) = self.txn.take() else { return };
        if txn.op_count == 0 {
            return;
        }
        self.lsn += 1;
        wal::write_frame(&mut self.wal_out, self.lsn, txn.op_count, &txn.ops);
        for (seq, item) in txn.outbox {
            self.outbox.insert(seq, item);
        }
    }

    // ------------------------------------------------------------------
    // Mutations (all writes go through these, inside a transaction)
    // ------------------------------------------------------------------

    pub(crate) fn new_stamp(&mut self) -> Stamp {
        Stamp { time: self.clock.tick(), node: self.node_id }
    }

    /// Current wall clock, read once per statement for [`new_stamp_at`](Self::new_stamp_at).
    pub(crate) fn wall_ms(&self) -> u64 {
        self.clock.wall_ms()
    }

    pub(crate) fn new_stamp_at(&mut self, now_ms: u64) -> Stamp {
        Stamp { time: self.clock.tick_at(now_ms), node: self.node_id }
    }

    fn next_seq_for(&mut self, local: bool) -> u64 {
        if local {
            let s = self.next_seq;
            self.next_seq += 1;
            s
        } else {
            0
        }
    }

    /// Stores a validated row. `local == false` for changes from other replicas.
    pub(crate) fn write_put(&mut self, tkey: &str, values: Vec<Value>, stamp: Stamp, local: bool) -> Result<()> {
        let seq = self.next_seq_for(local);
        let Database { tables, txn, sync, .. } = self;
        let txn = txn.as_mut().expect("write outside of a transaction");
        let t = tables.get_mut(tkey).ok_or_else(|| no_table(tkey))?;
        let mut w = Writer { buf: std::mem::take(&mut txn.ops) };
        w.u8(OP_CHANGE);
        w.var(seq);
        encode_put(&mut w, stamp, &t.schema.name, &values);
        txn.ops = w.buf;
        txn.op_count += 1;
        let key = values[t.schema.pk].clone();
        let (prev, prev_tomb) = t.put(values, stamp);
        if local && *sync {
            txn.outbox.push((seq, OutboxItem::Row { table: t.key.clone(), key: key.clone() }));
        }
        txn.undo.push(Undo::Row { table: t.key.clone(), key, prev, prev_tomb });
        Ok(())
    }

    /// Changes columns of an existing row in place (a local UPDATE that keeps
    /// the key): one lookup, only the changed columns are swapped and kept
    /// for undo, and only indexes whose column value changed are updated. The
    /// WAL gets the whole new row, as from [`write_put`](Self::write_put).
    pub(crate) fn write_update(
        &mut self,
        tkey: &str,
        key: Value,
        changes: Vec<(usize, Value)>,
        stamp: Stamp,
    ) -> Result<()> {
        let seq = self.next_seq_for(true);
        let Database { tables, txn, sync, .. } = self;
        let txn = txn.as_mut().expect("write outside of a transaction");
        let t = tables.get_mut(tkey).ok_or_else(|| no_table(tkey))?;
        let Table { schema, key: table_key, rows, indexes, .. } = t;
        let row = rows.get_mut(&key).ok_or_else(|| Error::Storage(format!("row {} disappeared", key.to_sql())))?;
        let mut cols = Vec::with_capacity(changes.len());
        for (c, new) in changes {
            for idx in indexes.iter_mut().filter(|i| i.column == c) {
                if row.values[c] != new {
                    idx.remove(&row.values[c], &key);
                    idx.insert(&new, &key);
                }
            }
            cols.push((c, std::mem::replace(&mut row.values[c], new)));
        }
        let prev_stamp = std::mem::replace(&mut row.stamp, stamp);
        let mut w = Writer { buf: std::mem::take(&mut txn.ops) };
        w.u8(OP_CHANGE);
        w.var(seq);
        encode_put(&mut w, stamp, &schema.name, &row.values);
        txn.ops = w.buf;
        txn.op_count += 1;
        if *sync {
            txn.outbox.push((seq, OutboxItem::Row { table: table_key.clone(), key: key.clone() }));
        }
        txn.undo.push(Undo::Update { table: table_key.clone(), key, cols, prev_stamp });
        Ok(())
    }

    pub(crate) fn write_delete(&mut self, tkey: &str, key: Value, stamp: Stamp, local: bool) -> Result<()> {
        let seq = self.next_seq_for(local);
        let Database { tables, txn, sync, .. } = self;
        let txn = txn.as_mut().expect("write outside of a transaction");
        let t = tables.get_mut(tkey).ok_or_else(|| no_table(tkey))?;
        let mut w = Writer { buf: std::mem::take(&mut txn.ops) };
        w.u8(OP_CHANGE);
        w.var(seq);
        encode_delete(&mut w, stamp, &t.schema.name, &key);
        txn.ops = w.buf;
        txn.op_count += 1;
        let (prev, prev_tomb) = t.delete(&key, stamp);
        if local && *sync {
            txn.outbox.push((seq, OutboxItem::Row { table: t.key.clone(), key: key.clone() }));
        }
        txn.undo.push(Undo::Row { table: t.key.clone(), key, prev, prev_tomb });
        Ok(())
    }

    /// Applies a schema change (CREATE/DROP TABLE/INDEX).
    pub(crate) fn write_schema(&mut self, change: Change, local: bool) -> Result<()> {
        let undo = apply_schema(&mut self.tables, &change)?;
        self.catalog_version = next_catalog_version();
        let seq = self.next_seq_for(local);
        let txn = self.txn.as_mut().expect("write outside of a transaction");
        let mut w = Writer { buf: std::mem::take(&mut txn.ops) };
        w.u8(OP_CHANGE);
        w.var(seq);
        encode_change(&mut w, &change);
        txn.ops = w.buf;
        txn.op_count += 1;
        txn.undo.push(undo);
        if local {
            txn.outbox.push((seq, OutboxItem::Schema(change)));
        }
        Ok(())
    }

    // ------------------------------------------------------------------
    // Persistence
    // ------------------------------------------------------------------

    /// Drains the WAL bytes of all transactions committed since the last call.
    pub fn take_wal(&mut self) -> Vec<u8> {
        std::mem::take(&mut self.wal_out)
    }

    /// Puts WAL bytes back in front of the buffer (used when a write failed).
    pub fn unshift_wal(&mut self, mut bytes: Vec<u8>) {
        bytes.extend_from_slice(&self.wal_out);
        self.wal_out = bytes;
    }

    pub fn wal_pending(&self) -> usize {
        self.wal_out.len()
    }

    /// Last committed log sequence number.
    pub fn lsn(&self) -> u64 {
        self.lsn
    }

    /// Full binary image of the committed state (checkpoint). WAL frames with
    /// an LSN lower or equal to the snapshot's LSN are skipped on recovery.
    pub fn snapshot(&self) -> Result<Vec<u8>> {
        if self.txn.is_some() {
            return Err(Error::Transaction("cannot take a snapshot inside an open transaction".into()));
        }
        Ok(snapshot::encode(self))
    }

    /// Opens a database from an optional snapshot and a WAL. A torn or
    /// corrupted tail of the WAL is reported and ignored.
    pub fn restore(snapshot: Option<&[u8]>, wal: &[u8], node_id: u32) -> Result<Database> {
        let mut db = match snapshot {
            Some(s) if !s.is_empty() => snapshot::decode(s)?,
            _ => Database::new(node_id),
        };
        let mut report = RecoveryReport {
            snapshot_bytes: snapshot.map_or(0, |s| s.len()),
            snapshot_lsn: db.lsn,
            wal_bytes: wal.len(),
            ..Default::default()
        };
        let mut reader = WalReader::new(wal);
        for frame in reader.by_ref() {
            if frame.lsn <= db.lsn {
                report.frames_skipped += 1;
                continue;
            }
            for op in frame.ops {
                match op {
                    LogOp::Change { seq, change } => {
                        if db.replay_change(seq, change).is_err() {
                            report.ops_failed += 1;
                        }
                    }
                    LogOp::Ack(upto) => db.apply_ack(upto),
                    LogOp::Sync(first) => db.mark_all_pending(first),
                }
            }
            db.lsn = frame.lsn;
            report.frames_replayed += 1;
        }
        report.torn_bytes = reader.torn_bytes();
        report.stop_reason = reader.stop_reason.take();
        db.recovery = Some(report);
        Ok(db)
    }

    pub fn recovery_report(&self) -> Option<&RecoveryReport> {
        self.recovery.as_ref()
    }

    fn replay_change(&mut self, seq: u64, change: Change) -> Result<()> {
        let stamp = change.stamp;
        self.clock.absorb(stamp.time);
        if seq > 0 {
            self.next_seq = self.next_seq.max(seq + 1);
        }
        let item = match change.mutation {
            Mutation::Put { table, values } => {
                let tkey = catalog_key(&table);
                let t = self.tables.get_mut(&*tkey).ok_or_else(|| no_table(&table))?;
                let values = t.schema.check_row(values)?;
                let key = values[t.schema.pk].clone();
                t.put(values, stamp);
                OutboxItem::Row { table: t.key.clone(), key }
            }
            Mutation::Delete { table, key } => {
                let tkey = catalog_key(&table);
                let t = self.tables.get_mut(&*tkey).ok_or_else(|| no_table(&table))?;
                t.delete(&key, stamp);
                OutboxItem::Row { table: t.key.clone(), key }
            }
            mutation => {
                let change = Change { stamp, mutation };
                apply_schema(&mut self.tables, &change)?;
                self.catalog_version = next_catalog_version();
                OutboxItem::Schema(change)
            }
        };
        if seq > 0 && (self.sync || matches!(item, OutboxItem::Schema(_))) {
            self.outbox.insert(seq, item);
        }
        Ok(())
    }

    fn apply_ack(&mut self, upto: u64) {
        self.outbox.ack(upto);
        self.acked_seq = self.acked_seq.max(upto);
    }

    // ------------------------------------------------------------------
    // Synchronisation
    // ------------------------------------------------------------------

    /// Number of local changes not yet acknowledged by the sync server.
    pub fn outbox_len(&self) -> usize {
        self.outbox.len()
    }

    /// Whether local row changes are kept in the outbox (see
    /// [`enable_sync`](Self::enable_sync)).
    pub fn sync_enabled(&self) -> bool {
        self.sync
    }

    /// Starts keeping local row changes in the outbox. A database that has
    /// not synchronised yet keeps only its schema changes there; enabling
    /// makes every row and tombstone a pending entry, so the first push sends
    /// the whole state, as the outbox of a database that kept every change
    /// would. The step is logged, so it survives a restart. Returns whether
    /// synchronisation was off before.
    pub fn enable_sync(&mut self) -> Result<bool> {
        if self.txn.is_some() {
            return Err(Error::Transaction("cannot enable synchronisation inside an open transaction".into()));
        }
        if self.sync {
            return Ok(false);
        }
        let first = self.next_seq;
        self.mark_all_pending(first);
        let mut w = Writer::new();
        w.u8(OP_SYNC);
        w.var(first);
        self.lsn += 1;
        wal::write_frame(&mut self.wal_out, self.lsn, 1, &w.buf);
        Ok(true)
    }

    /// Enables synchronisation and makes every row and tombstone a pending
    /// outbox entry, numbered from `first` in catalog and key order (the same
    /// order when the step is replayed from the WAL).
    fn mark_all_pending(&mut self, first: u64) {
        self.sync = true;
        let mut seq = first.max(self.next_seq);
        for t in self.tables.values() {
            for key in t.rows.keys().chain(t.tombstones.keys()) {
                self.outbox.insert(seq, OutboxItem::Row { table: t.key.clone(), key: key.clone() });
                seq += 1;
            }
        }
        self.next_seq = seq;
    }

    /// Encodes up to `max` pending local changes as a batch. Returns the batch
    /// and the sequence number to pass to [`ack_outbox`](Self::ack_outbox)
    /// once the server confirmed it (0 when the outbox is empty).
    pub fn export_outbox(&self, max: usize) -> Result<(Vec<u8>, u64)> {
        if self.txn.is_some() {
            return Err(Error::Transaction("cannot export changes inside an open transaction".into()));
        }
        let mut changes = Vec::new();
        let mut last = 0;
        for (&seq, item) in self.outbox.by_seq.iter().take(max) {
            last = seq;
            match item {
                OutboxItem::Schema(c) => changes.push(c.clone()),
                OutboxItem::Row { table, key } => {
                    // The current state of the row is sent, so several updates of
                    // one row collapse into a single change.
                    let Some(t) = self.tables.get(&**table) else { continue };
                    if let Some(row) = t.rows.get(key) {
                        changes.push(Change {
                            stamp: row.stamp,
                            mutation: Mutation::Put { table: t.schema.name.clone(), values: row.values.clone() },
                        });
                    } else if let Some(ts) = t.tombstones.get(key) {
                        changes.push(Change {
                            stamp: *ts,
                            mutation: Mutation::Delete { table: t.schema.name.clone(), key: key.clone() },
                        });
                    }
                }
            }
        }
        Ok((encode_batch(&changes), last))
    }

    /// Marks outbox entries up to `upto` as delivered. The acknowledgement is
    /// logged, so delivered changes are not re-sent after a restart.
    pub fn ack_outbox(&mut self, upto: u64) {
        if upto == 0 || upto <= self.acked_seq {
            return;
        }
        self.apply_ack(upto);
        let mut w = Writer::new();
        w.u8(OP_ACK);
        w.var(upto);
        self.lsn += 1;
        wal::write_frame(&mut self.wal_out, self.lsn, 1, &w.buf);
    }

    /// Whether the clock is more than [`MAX_DRIFT_MS`] ahead of the wall clock:
    /// local changes were made while the wall clock ran ahead, and other
    /// replicas would reject their stamps (see [`rebase_clock`](Self::rebase_clock)).
    pub fn clock_ahead(&self) -> bool {
        self.clock.last() > self.clock.bound()
    }

    /// Gives new stamps to local changes made while the wall clock ran ahead,
    /// once it is right again. Only outbox entries whose stamp is beyond the
    /// drift bound get one: other replicas rejected those stamps, so no copy
    /// of them exists elsewhere. The clock goes back to the newest stamp that
    /// is kept. Returns the number of re-stamped changes; the new stamps are
    /// not logged, so the host makes them durable with a checkpoint.
    pub fn rebase_clock(&mut self) -> Result<usize> {
        if self.txn.is_some() {
            return Err(Error::Transaction("cannot rebase the clock inside an open transaction".into()));
        }
        if !self.clock_ahead() {
            return Ok(0);
        }
        let bound = self.clock.bound();
        let stamp_of = |tables: &BTreeMap<String, Table>, item: &OutboxItem| match item {
            OutboxItem::Row { table, key } => tables
                .get(&**table)
                .and_then(|t| t.rows.get(key).map(|r| r.stamp).or_else(|| t.tombstones.get(key).copied())),
            OutboxItem::Schema(c) => Some(c.stamp),
        };
        let targets: Vec<u64> = self
            .outbox
            .by_seq
            .iter()
            .filter(|(_, item)| stamp_of(&self.tables, item).is_some_and(|s| s.time > bound))
            .map(|(&seq, _)| seq)
            .collect();
        // Stamps are unique per replica, so they identify what gets re-stamped.
        let restamped: HashSet<Stamp> =
            targets.iter().filter_map(|seq| stamp_of(&self.tables, &self.outbox.by_seq[seq])).collect();
        // The newest stamp that stays: anything not re-stamped may exist on
        // another replica, and the clock must not fall behind it.
        let mut keep = self.clock.wall_ms() << 16;
        for t in self.tables.values() {
            let stamps = std::iter::once(t.created)
                .chain(t.rows.values().map(|r| r.stamp))
                .chain(t.tombstones.values().copied());
            keep = stamps.filter(|s| !restamped.contains(s)).fold(keep, |k, s| k.max(s.time));
        }
        for item in self.outbox.by_seq.values() {
            if let OutboxItem::Schema(c) = item {
                if !restamped.contains(&c.stamp) {
                    keep = keep.max(c.stamp.time);
                }
            }
        }
        self.clock.reset(keep);
        for seq in targets {
            let new = self.new_stamp();
            let Database { tables, outbox, .. } = self;
            match outbox.by_seq.get_mut(&seq) {
                Some(OutboxItem::Row { table, key }) => {
                    if let Some(t) = tables.get_mut(&**table) {
                        if let Some(row) = t.rows.get_mut(key) {
                            row.stamp = new;
                        } else if let Some(ts) = t.tombstones.get_mut(key) {
                            *ts = new;
                        }
                    }
                }
                Some(OutboxItem::Schema(change)) => {
                    if let Mutation::CreateTable(schema) = &change.mutation {
                        if let Some(t) = tables.get_mut(&*catalog_key(&schema.name)) {
                            if t.created == change.stamp {
                                t.created = new;
                            }
                        }
                    }
                    change.stamp = new;
                }
                None => {}
            }
        }
        Ok(restamped.len())
    }

    /// Applies changes received from other replicas using last-writer-wins.
    /// The whole batch is committed as one transaction.
    pub fn apply_changes(&mut self, batch: &[u8]) -> Result<ApplyReport> {
        if self.txn.is_some() {
            return Err(Error::Transaction("cannot apply remote changes inside an open transaction".into()));
        }
        let changes = decode_batch(batch)?;
        self.txn = Some(Txn { seq_start: self.next_seq, ..Default::default() });
        let mut report = ApplyReport::default();
        let mut accepted = Vec::new();
        // Tables in conflict, by catalog key -> index in `report.conflicts`.
        let mut conflicted: BTreeMap<String, usize> = BTreeMap::new();
        for change in changes {
            if !self.clock.observe(change.stamp.time) {
                // Not applied and not absorbed: a replica with a clock far
                // ahead must not win every conflict nor move other clocks.
                report.rejected += 1;
                report.clock_drift += 1;
                if report.errors.len() < 5 {
                    let ahead = change.stamp.physical_ms().saturating_sub(self.clock.wall_ms());
                    report.errors.push(format!(
                        "clock drift: stamp {} is {} s ahead of the local clock (limit {} s)",
                        change.stamp,
                        ahead / 1000,
                        MAX_DRIFT_MS / 1000
                    ));
                }
                continue;
            }
            // Rows from a replica that defines the table differently mean
            // something else, even when they happen to fit.
            let row_table = match &change.mutation {
                Mutation::Put { table, .. } | Mutation::Delete { table, .. } => Some(catalog_key(table).into_owned()),
                _ => None,
            };
            if let Some(&i) = row_table.as_ref().and_then(|t| conflicted.get(t)) {
                report.rejected += 1;
                report.conflicts[i].rows += 1;
                continue;
            }
            match self.apply_remote(&change) {
                Ok(Remote::Applied) => {
                    report.applied += 1;
                    accepted.push(change);
                }
                Ok(Remote::Ignored) => report.ignored += 1,
                Ok(Remote::Conflict { table, remote }) => {
                    report.rejected += 1;
                    let key = catalog_key(&table).into_owned();
                    let i = match conflicted.get(&key) {
                        Some(&i) => i,
                        None => {
                            let local = self.tables[&key].schema.to_string();
                            report.conflicts.push(SchemaConflict {
                                table,
                                local,
                                remote: None,
                                additive: None,
                                rows: 0,
                            });
                            conflicted.insert(key.clone(), report.conflicts.len() - 1);
                            report.conflicts.len() - 1
                        }
                    };
                    let c = &mut report.conflicts[i];
                    match remote {
                        Some(r) => {
                            c.additive = self.tables[&key].schema.compare(&r);
                            c.remote = Some(r.to_string());
                        }
                        None => c.rows += 1,
                    }
                }
                Err(e) => {
                    report.rejected += 1;
                    if report.errors.len() < 5 {
                        report.errors.push(e.to_string());
                    }
                }
            }
        }
        for c in &report.conflicts {
            if report.errors.len() < 5 {
                let remote = c.remote.as_deref().unwrap_or("unknown, rows do not fit");
                report.errors.push(format!(
                    "schema conflict on table '{}': local {}, remote {}; {} row change(s) rejected",
                    c.table, c.local, remote, c.rows
                ));
            }
        }
        self.commit();
        report.accepted = encode_batch(&accepted);
        Ok(report)
    }

    /// Applies one remote change if it wins last-writer-wins.
    fn apply_remote(&mut self, change: &Change) -> Result<Remote> {
        let stamp = change.stamp;
        // A row that does not fit the local definition of its table.
        let conflict = |t: &Table| Remote::Conflict { table: t.schema.name.clone(), remote: None };
        match &change.mutation {
            Mutation::Put { table, values } => {
                let tkey = catalog_key(table);
                let t = self.tables.get(&*tkey).ok_or_else(|| no_table(table))?;
                let values = match t.schema.check_row(values.clone()) {
                    Ok(v) => v,
                    Err(Error::Schema(_) | Error::Type(_)) => return Ok(conflict(t)),
                    Err(e) => return Err(e),
                };
                let key = &values[t.schema.pk];
                if t.rows.get(key).is_some_and(|r| r.stamp >= stamp)
                    || t.tombstones.get(key).is_some_and(|ts| *ts >= stamp)
                {
                    return Ok(Remote::Ignored);
                }
                self.write_put(&tkey, values, stamp, false)?;
                Ok(Remote::Applied)
            }
            Mutation::Delete { table, key } => {
                let tkey = catalog_key(table);
                let t = self.tables.get(&*tkey).ok_or_else(|| no_table(table))?;
                let Ok(key) = key.clone().coerce(t.schema.pk_column().data_type) else { return Ok(conflict(t)) };
                let newer_local = match t.rows.get(&key) {
                    Some(r) => r.stamp >= stamp,
                    None => t.tombstones.get(&key).is_some_and(|ts| *ts >= stamp),
                };
                if newer_local {
                    return Ok(Remote::Ignored);
                }
                self.write_delete(&tkey, key, stamp, false)?;
                Ok(Remote::Applied)
            }
            Mutation::CreateTable(schema) => {
                if let Some(t) = self.tables.get(&*catalog_key(&schema.name)) {
                    return Ok(match t.schema.compare(schema) {
                        None => Remote::Ignored,
                        Some(_) => Remote::Conflict { table: t.schema.name.clone(), remote: Some(schema.clone()) },
                    });
                }
                self.write_schema(change.clone(), false)?;
                Ok(Remote::Applied)
            }
            Mutation::DropTable(name) => match self.tables.get(&*catalog_key(name)) {
                Some(t) if t.created < stamp => {
                    self.write_schema(change.clone(), false)?;
                    Ok(Remote::Applied)
                }
                _ => Ok(Remote::Ignored),
            },
            Mutation::CreateIndex { table, name, .. } => {
                if !self.tables.contains_key(&*catalog_key(table)) {
                    return Err(no_table(table));
                }
                if self.find_index(name).is_some() {
                    return Ok(Remote::Ignored);
                }
                self.write_schema(change.clone(), false)?;
                Ok(Remote::Applied)
            }
            Mutation::DropIndex { name, .. } => {
                if self.find_index(name).is_none() {
                    return Ok(Remote::Ignored);
                }
                self.write_schema(change.clone(), false)?;
                Ok(Remote::Applied)
            }
        }
    }

    /// The whole database as a change batch (schema, rows and tombstones).
    /// Used to bootstrap a new replica or to compact a server log.
    pub fn export_state(&self) -> Result<Vec<u8>> {
        if self.txn.is_some() {
            return Err(Error::Transaction("cannot export state inside an open transaction".into()));
        }
        let mut changes = Vec::new();
        for t in self.tables.values() {
            changes.push(Change { stamp: t.created, mutation: Mutation::CreateTable((*t.schema).clone()) });
            for idx in &t.indexes {
                changes.push(Change {
                    stamp: t.created,
                    mutation: Mutation::CreateIndex {
                        table: t.schema.name.clone(),
                        name: idx.name.clone(),
                        column: t.schema.columns[idx.column].name.clone(),
                    },
                });
            }
        }
        for t in self.tables.values() {
            for row in t.rows.values() {
                changes.push(Change {
                    stamp: row.stamp,
                    mutation: Mutation::Put { table: t.schema.name.clone(), values: row.values.clone() },
                });
            }
            for (key, ts) in &t.tombstones {
                changes.push(Change {
                    stamp: *ts,
                    mutation: Mutation::Delete { table: t.schema.name.clone(), key: key.clone() },
                });
            }
        }
        Ok(encode_batch(&changes))
    }
}

/// Applies a schema change to the catalog and returns how to undo it.
fn apply_schema(tables: &mut BTreeMap<String, Table>, change: &Change) -> Result<Undo> {
    match &change.mutation {
        Mutation::CreateTable(schema) => {
            let key = catalog_key(&schema.name).into_owned();
            if tables.contains_key(&key) {
                return Err(Error::Schema(format!("table '{}' already exists", schema.name)));
            }
            tables.insert(key.clone(), Table::new(schema.clone(), change.stamp));
            Ok(Undo::CreateTable(key))
        }
        Mutation::DropTable(name) => {
            let t = tables.remove(&*catalog_key(name)).ok_or_else(|| no_table(name))?;
            Ok(Undo::DropTable(Box::new(t)))
        }
        Mutation::CreateIndex { table, name, column } => {
            if tables.values().any(|t| t.index_named(name).is_some()) {
                return Err(Error::Schema(format!("index '{name}' already exists")));
            }
            let tkey = catalog_key(table);
            let t = tables.get_mut(&*tkey).ok_or_else(|| no_table(table))?;
            let col = t.schema.require_column(column)?;
            t.add_index(name.clone(), col);
            Ok(Undo::CreateIndex { table: tkey.into_owned(), name: name.clone() })
        }
        Mutation::DropIndex { table, name } => {
            let tkey = catalog_key(table);
            let t = tables.get_mut(&*tkey).ok_or_else(|| no_table(table))?;
            let pos = t.index_named(name).ok_or_else(|| Error::Schema(format!("no such index '{name}'")))?;
            let index = t.indexes.remove(pos);
            Ok(Undo::DropIndex { table: tkey.into_owned(), pos, index })
        }
        Mutation::Put { .. } | Mutation::Delete { .. } => unreachable!("row change passed as schema change"),
    }
}

fn apply_undo(tables: &mut BTreeMap<String, Table>, undo: Undo) {
    match undo {
        Undo::Row { table, key, prev, prev_tomb } => {
            if let Some(t) = tables.get_mut(&*table) {
                t.remove_row(&key);
                if let Some(p) = prev {
                    t.restore_row(p);
                }
                t.set_tombstone(&key, prev_tomb);
            }
        }
        Undo::Update { table, key, cols, prev_stamp } => {
            if let Some(t) = tables.get_mut(&*table) {
                let Table { rows, indexes, .. } = t;
                if let Some(row) = rows.get_mut(&key) {
                    for (c, old) in cols {
                        for idx in indexes.iter_mut().filter(|i| i.column == c) {
                            if row.values[c] != old {
                                idx.remove(&row.values[c], &key);
                                idx.insert(&old, &key);
                            }
                        }
                        row.values[c] = old;
                    }
                    row.stamp = prev_stamp;
                }
            }
        }
        Undo::CreateTable(key) => {
            tables.remove(&key);
        }
        Undo::DropTable(t) => {
            tables.insert(catalog_key(&t.schema.name).into_owned(), *t);
        }
        Undo::CreateIndex { table, name } => {
            if let Some(t) = tables.get_mut(&table) {
                if let Some(i) = t.index_named(&name) {
                    t.indexes.remove(i);
                }
            }
        }
        Undo::DropIndex { table, pos, index } => {
            if let Some(t) = tables.get_mut(&table) {
                let pos = pos.min(t.indexes.len());
                t.indexes.insert(pos, index);
            }
        }
    }
}
