//! WebAssembly interface of the Flat-SQL engine.
//!
//! The JavaScript side (see `bridge/`) owns persistence and networking; this
//! module only exposes the engine: SQL execution, WAL/snapshot bytes and the
//! sync primitives. Query results can be returned either as JS objects built
//! directly from Rust (`exec`) or as a JSON string (`execJson`) – the two
//! paths are compared in the benchmarks.

use flatsql_core as core;
use flatsql_core::{QueryResult, Value};
use js_sys::{Array, Object, Reflect, Uint8Array};
use wasm_bindgen::prelude::*;

#[wasm_bindgen(start)]
pub fn start() {
    console_error_panic_hook::set_once();
}

/// Engine version.
#[wasm_bindgen]
pub fn version() -> String {
    core::VERSION.to_string()
}

/// How far ahead of the local clock a stamp from another replica may be
/// before it is rejected, in milliseconds.
#[wasm_bindgen(js_name = maxDriftMs)]
pub fn max_drift_ms() -> f64 {
    core::sync::MAX_DRIFT_MS as f64
}

fn now_ms() -> u64 {
    js_sys::Date::now() as u64
}

fn random_node_id() -> u32 {
    // Node ids only need to be unique among replicas of one dataset.
    (js_sys::Math::random() * 4_294_967_295.0) as u32 | 1
}

fn js_error(e: core::Error) -> JsValue {
    let err = js_sys::Error::new(&e.to_string());
    let _ = Reflect::set(&err, &"kind".into(), &e.kind().into());
    err.into()
}

fn set(obj: &Object, key: &str, value: &JsValue) {
    let _ = Reflect::set(obj, &JsValue::from_str(key), value);
}

fn value_to_js(v: &Value) -> JsValue {
    match v {
        Value::Null => JsValue::NULL,
        Value::Bool(b) => JsValue::from_bool(*b),
        Value::Int(i) => JsValue::from_f64(*i as f64),
        Value::Float(f) => JsValue::from_f64(*f),
        Value::Text(s) => JsValue::from_str(s),
    }
}

fn value_from_js(v: &JsValue) -> Result<Value, JsValue> {
    if v.is_null() || v.is_undefined() {
        return Ok(Value::Null);
    }
    if let Some(b) = v.as_bool() {
        return Ok(Value::Bool(b));
    }
    if let Some(f) = v.as_f64() {
        if f.fract() == 0.0 && f.abs() <= 9_007_199_254_740_991.0 {
            return Ok(Value::Int(f as i64));
        }
        return Ok(Value::Float(f));
    }
    if let Some(s) = v.as_string() {
        return Ok(Value::Text(s));
    }
    if v.is_bigint() {
        let i = i64::try_from(v.clone()).map_err(|_| JsValue::from_str("BigInt parameter out of i64 range"))?;
        return Ok(Value::Int(i));
    }
    Err(js_sys::TypeError::new("unsupported parameter type (use null, boolean, number, bigint or string)").into())
}

fn result_to_js(r: &QueryResult) -> JsValue {
    let obj = Object::new();
    let columns = Array::new_with_length(r.columns.len() as u32);
    for (i, c) in r.columns.iter().enumerate() {
        columns.set(i as u32, JsValue::from_str(c));
    }
    let rows = Array::new_with_length(r.rows.len() as u32);
    for (i, row) in r.rows.iter().enumerate() {
        let arr = Array::new_with_length(row.len() as u32);
        for (j, v) in row.iter().enumerate() {
            arr.set(j as u32, value_to_js(v));
        }
        rows.set(i as u32, arr.into());
    }
    set(&obj, "columns", &columns);
    set(&obj, "rows", &rows);
    set(&obj, "rowsAffected", &JsValue::from_f64(r.rows_affected as f64));
    set(&obj, "rowsScanned", &JsValue::from_f64(r.rows_scanned as f64));
    set(&obj, "message", &JsValue::from_str(&r.message));
    obj.into()
}

fn params_from_js(params: Option<Array>) -> Result<Vec<Value>, JsValue> {
    match params {
        None => Ok(Vec::new()),
        Some(arr) => arr.iter().map(|v| value_from_js(&v)).collect(),
    }
}

/// A parsed statement with `?` placeholders.
#[wasm_bindgen]
pub struct PreparedStatement {
    inner: core::Prepared,
}

#[wasm_bindgen]
impl PreparedStatement {
    #[wasm_bindgen(getter, js_name = paramCount)]
    pub fn param_count(&self) -> usize {
        self.inner.param_count()
    }

    /// Whether the statement only reads (SELECT, EXPLAIN, SHOW, DESCRIBE).
    #[wasm_bindgen(getter, js_name = readOnly)]
    pub fn read_only(&self) -> bool {
        self.inner.statement().is_read_only()
    }
}

/// Pending local changes encoded for the sync server.
#[wasm_bindgen]
pub struct OutboxBatch {
    bytes: Vec<u8>,
    last_seq: u64,
    count: usize,
}

#[wasm_bindgen]
impl OutboxBatch {
    #[wasm_bindgen(getter)]
    pub fn bytes(&self) -> Uint8Array {
        Uint8Array::from(&self.bytes[..])
    }

    /// Value to pass to `ackOutbox` once the server stored the batch.
    #[wasm_bindgen(getter, js_name = lastSeq)]
    pub fn last_seq(&self) -> f64 {
        self.last_seq as f64
    }

    /// Number of outbox entries covered by the batch.
    #[wasm_bindgen(getter)]
    pub fn count(&self) -> usize {
        self.count
    }
}

#[wasm_bindgen]
pub struct Database {
    inner: core::Database,
}

#[wasm_bindgen]
impl Database {
    /// Creates an empty in-memory database. `nodeId` identifies the replica
    /// for synchronisation; a random one is generated when omitted.
    #[wasm_bindgen(constructor)]
    pub fn new(node_id: Option<u32>) -> Database {
        let mut inner = core::Database::new(node_id.unwrap_or_else(random_node_id));
        inner.set_clock(now_ms);
        Database { inner }
    }

    /// Opens a database from a snapshot and/or WAL bytes (crash recovery).
    pub fn restore(snapshot: Option<Vec<u8>>, wal: Option<Vec<u8>>, node_id: Option<u32>) -> Result<Database, JsValue> {
        let node = node_id.unwrap_or_else(random_node_id);
        let mut inner =
            core::Database::restore(snapshot.as_deref(), wal.as_deref().unwrap_or(&[]), node).map_err(js_error)?;
        inner.set_clock(now_ms);
        Ok(Database { inner })
    }

    /// Executes SQL (one or more statements) and returns the last result as
    /// `{ columns, rows, rowsAffected, rowsScanned, message }`.
    pub fn exec(&mut self, sql: &str) -> Result<JsValue, JsValue> {
        let r = self.inner.execute(sql).map_err(js_error)?;
        Ok(result_to_js(&r))
    }

    /// Executes SQL and returns an array with the result of every statement.
    #[wasm_bindgen(js_name = execAll)]
    pub fn exec_all(&mut self, sql: &str) -> Result<Array, JsValue> {
        let results = self.inner.execute_all(sql).map_err(js_error)?;
        Ok(results.iter().map(result_to_js).collect())
    }

    /// Like `exec` but returns the result serialised as JSON.
    #[wasm_bindgen(js_name = execJson)]
    pub fn exec_json(&mut self, sql: &str) -> Result<String, JsValue> {
        Ok(self.inner.execute(sql).map_err(js_error)?.to_json())
    }

    pub fn prepare(&self, sql: &str) -> Result<PreparedStatement, JsValue> {
        Ok(PreparedStatement { inner: self.inner.prepare(sql).map_err(js_error)? })
    }

    /// Executes a prepared statement with positional parameters.
    pub fn run(&mut self, stmt: &PreparedStatement, params: Option<Array>) -> Result<JsValue, JsValue> {
        let params = params_from_js(params)?;
        let r = self.inner.execute_prepared(&stmt.inner, &params).map_err(js_error)?;
        Ok(result_to_js(&r))
    }

    /// Executes a prepared statement and returns only the number of affected
    /// rows – no result object is built, which matters for bulk writes.
    pub fn execute(&mut self, stmt: &PreparedStatement, params: Option<Array>) -> Result<f64, JsValue> {
        let params = params_from_js(params)?;
        let r = self.inner.execute_prepared(&stmt.inner, &params).map_err(js_error)?;
        Ok(r.rows_affected as f64)
    }

    #[wasm_bindgen(js_name = runJson)]
    pub fn run_json(&mut self, stmt: &PreparedStatement, params: Option<Array>) -> Result<String, JsValue> {
        let params = params_from_js(params)?;
        Ok(self.inner.execute_prepared(&stmt.inner, &params).map_err(js_error)?.to_json())
    }

    /// Drains the WAL bytes produced by transactions committed so far.
    #[wasm_bindgen(js_name = takeWal)]
    pub fn take_wal(&mut self) -> Vec<u8> {
        self.inner.take_wal()
    }

    /// Puts WAL bytes back (when persisting them failed).
    #[wasm_bindgen(js_name = unshiftWal)]
    pub fn unshift_wal(&mut self, bytes: Vec<u8>) {
        self.inner.unshift_wal(bytes);
    }

    #[wasm_bindgen(js_name = walPending)]
    pub fn wal_pending(&self) -> usize {
        self.inner.wal_pending()
    }

    /// Full image of the committed state (checkpoint).
    pub fn snapshot(&self) -> Result<Vec<u8>, JsValue> {
        self.inner.snapshot().map_err(js_error)
    }

    #[wasm_bindgen(js_name = inTransaction)]
    pub fn in_transaction(&self) -> bool {
        self.inner.in_transaction()
    }

    #[wasm_bindgen(getter, js_name = nodeId)]
    pub fn node_id(&self) -> u32 {
        self.inner.node_id()
    }

    #[wasm_bindgen(js_name = tableNames)]
    pub fn table_names(&self) -> Array {
        self.inner.table_names().into_iter().map(JsValue::from).collect()
    }

    pub fn stats(&self) -> JsValue {
        let s = self.inner.stats();
        let obj = Object::new();
        set(&obj, "nodeId", &JsValue::from_f64(s.node_id as f64));
        set(&obj, "tables", &JsValue::from_f64(s.tables as f64));
        set(&obj, "rows", &JsValue::from_f64(s.rows as f64));
        set(&obj, "indexes", &JsValue::from_f64(s.indexes as f64));
        set(&obj, "tombstones", &JsValue::from_f64(s.tombstones as f64));
        set(&obj, "lsn", &JsValue::from_f64(s.lsn as f64));
        set(&obj, "walPending", &JsValue::from_f64(s.wal_pending as f64));
        set(&obj, "outbox", &JsValue::from_f64(s.outbox as f64));
        set(&obj, "ackedSeq", &JsValue::from_f64(s.acked_seq as f64));
        set(&obj, "clockMs", &JsValue::from_f64((s.clock >> 16) as f64));
        set(&obj, "inTransaction", &JsValue::from_bool(s.in_transaction));
        set(&obj, "sync", &JsValue::from_bool(s.sync));
        obj.into()
    }

    /// What happened during `restore` (null for a fresh database).
    #[wasm_bindgen(js_name = recoveryReport)]
    pub fn recovery_report(&self) -> JsValue {
        let Some(r) = self.inner.recovery_report() else { return JsValue::NULL };
        let obj = Object::new();
        set(&obj, "snapshotBytes", &JsValue::from_f64(r.snapshot_bytes as f64));
        set(&obj, "snapshotLsn", &JsValue::from_f64(r.snapshot_lsn as f64));
        set(&obj, "walBytes", &JsValue::from_f64(r.wal_bytes as f64));
        set(&obj, "framesReplayed", &JsValue::from_f64(r.frames_replayed as f64));
        set(&obj, "framesSkipped", &JsValue::from_f64(r.frames_skipped as f64));
        set(&obj, "tornBytes", &JsValue::from_f64(r.torn_bytes as f64));
        set(&obj, "opsFailed", &JsValue::from_f64(r.ops_failed as f64));
        let reason = r.stop_reason.as_deref().map_or(JsValue::NULL, JsValue::from_str);
        set(&obj, "stopReason", &reason);
        obj.into()
    }

    // ---- synchronisation ---------------------------------------------

    #[wasm_bindgen(js_name = outboxLength)]
    pub fn outbox_length(&self) -> usize {
        self.inner.outbox_len()
    }

    /// Starts keeping local row changes in the outbox; every existing row
    /// becomes a pending change. Returns whether synchronisation was off.
    #[wasm_bindgen(js_name = enableSync)]
    pub fn enable_sync(&mut self) -> Result<bool, JsValue> {
        self.inner.enable_sync().map_err(js_error)
    }

    #[wasm_bindgen(getter, js_name = syncEnabled)]
    pub fn sync_enabled(&self) -> bool {
        self.inner.sync_enabled()
    }

    /// Encodes up to `max` pending local changes for the sync server.
    #[wasm_bindgen(js_name = exportOutbox)]
    pub fn export_outbox(&self, max: Option<u32>) -> Result<OutboxBatch, JsValue> {
        let max = max.map_or(usize::MAX, |m| m as usize);
        let count = self.inner.outbox_len().min(max);
        let (bytes, last_seq) = self.inner.export_outbox(max).map_err(js_error)?;
        Ok(OutboxBatch { bytes, last_seq, count })
    }

    /// Whether local changes were stamped while the clock ran more than
    /// `maxDriftMs()` ahead of the current one (see `rebaseClock`).
    #[wasm_bindgen(js_name = clockAhead)]
    pub fn clock_ahead(&self) -> bool {
        self.inner.clock_ahead()
    }

    /// Gives new stamps to unsent changes made while the clock ran ahead and
    /// moves the clock back. Returns how many changes got new stamps; the
    /// caller makes the result durable with a snapshot (checkpoint).
    #[wasm_bindgen(js_name = rebaseClock)]
    pub fn rebase_clock(&mut self) -> Result<usize, JsValue> {
        self.inner.rebase_clock().map_err(js_error)
    }

    /// Confirms delivery of outbox entries up to `upto` (an `OutboxBatch.lastSeq`).
    #[wasm_bindgen(js_name = ackOutbox)]
    pub fn ack_outbox(&mut self, upto: f64) {
        self.inner.ack_outbox(upto as u64);
    }

    /// Applies a batch of remote changes (last-writer-wins). Returns
    /// `{ applied, ignored, rejected, clockDrift, conflicts, errors, accepted }`
    /// where `clockDrift` counts the rejected changes stamped too far ahead,
    /// `conflicts` lists tables defined differently on the sender
    /// (`{ table, local, remote, additive, rows }`) and `accepted` is a batch
    /// with only the changes that took effect.
    #[wasm_bindgen(js_name = applyChanges)]
    pub fn apply_changes(&mut self, batch: &[u8]) -> Result<JsValue, JsValue> {
        let r = self.inner.apply_changes(batch).map_err(js_error)?;
        let obj = Object::new();
        set(&obj, "applied", &JsValue::from_f64(r.applied as f64));
        set(&obj, "ignored", &JsValue::from_f64(r.ignored as f64));
        set(&obj, "rejected", &JsValue::from_f64(r.rejected as f64));
        set(&obj, "clockDrift", &JsValue::from_f64(r.clock_drift as f64));
        let errors: Array = r.errors.iter().map(|e| JsValue::from_str(e)).collect();
        set(&obj, "errors", &errors);
        let conflicts: Array = r
            .conflicts
            .iter()
            .map(|c| {
                let o = Object::new();
                set(&o, "table", &JsValue::from_str(&c.table));
                set(&o, "local", &JsValue::from_str(&c.local));
                set(&o, "remote", &c.remote.as_deref().map_or(JsValue::NULL, JsValue::from_str));
                set(&o, "additive", &c.additive.map_or(JsValue::NULL, JsValue::from_bool));
                set(&o, "rows", &JsValue::from_f64(c.rows as f64));
                JsValue::from(o)
            })
            .collect();
        set(&obj, "conflicts", &conflicts);
        set(&obj, "accepted", &Uint8Array::from(&r.accepted[..]));
        Ok(obj.into())
    }

    /// The whole database as a change batch (bootstraps a new replica).
    #[wasm_bindgen(js_name = exportState)]
    pub fn export_state(&self) -> Result<Vec<u8>, JsValue> {
        self.inner.export_state().map_err(js_error)
    }
}

/// Decodes a change batch to JSON (debugging / UI).
#[wasm_bindgen(js_name = describeChanges)]
pub fn describe_changes(batch: &[u8]) -> Result<String, JsValue> {
    let changes = core::sync::decode_batch(batch).map_err(js_error)?;
    Ok(core::sync::changes_to_json(&changes))
}
