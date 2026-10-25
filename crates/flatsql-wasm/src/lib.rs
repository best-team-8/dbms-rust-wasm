//! WebAssembly interface of the Flat-SQL engine.
//!
//! The JavaScript side (see `bridge/`) owns persistence and networking; this
//! module only exposes the engine: SQL execution, WAL/snapshot bytes and the
//! sync primitives. Query results can be returned either as JS objects decoded
//! from the binary result format (`exec`) or as a JSON string (`execJson`) –
//! the two paths are compared in the benchmarks.

use std::cell::RefCell;
use std::sync::Arc;

use flatsql_core as core;
use flatsql_core::Value;
use js_sys::{Array, Object, Reflect, Uint8Array};
use wasm_bindgen::prelude::*;

#[wasm_bindgen(start)]
pub fn start() {
    console_error_panic_hook::set_once();
    set_memory(&wasm_bindgen::memory());
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

/// A JavaScript number as a value: INT when it is a whole number JavaScript
/// represents exactly, FLOAT otherwise.
fn number(f: f64) -> Value {
    if f.fract() == 0.0 && f.abs() <= 9_007_199_254_740_991.0 {
        Value::Int(f as i64)
    } else {
        Value::Float(f)
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
        return Ok(number(f));
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

// A result is written by the core in the binary result format
// (`flatsql_core::result::ResultEncoder`) and turned into the result object by
// JavaScript in one call: numbers are read straight from the bytes, every
// text of the result is decoded from UTF-8 at once and cut by the UTF-16
// lengths stored with the values, and the keys of the object are literals of
// this code, so every result has the same shape. The column names of a
// prepared statement are passed once and copied for every result. The views
// of the module's memory are created again only when the memory has grown
// (its buffer is then a new one). The decoder keeps a leading U+FEFF
// (`ignoreBOM`), which is text, not a mark.
#[wasm_bindgen(inline_js = "
const utf8 = new TextDecoder('utf-8', { ignoreBOM: true });
let memory;
let buffer;
let bytes;
let view;
export function set_memory(m) {
  memory = m;
}
export function decode_result(at, length, columns, copy) {
  if (memory.buffer !== buffer) {
    buffer = memory.buffer;
    bytes = new Uint8Array(buffer);
    view = new DataView(buffer);
  }
  const end = at + length;
  const count = view.getUint32(at, true);
  const width = view.getUint32(at + 4, true);
  const textAt = at + view.getUint32(at + 24, true);
  const messageLength = view.getUint32(at + 28, true);
  const text = textAt < end ? utf8.decode(bytes.subarray(textAt, end)) : '';
  const rows = new Array(count);
  let p = at + 32;
  let t = 0;
  for (let i = 0; i < count; i++) {
    const row = new Array(width);
    for (let j = 0; j < width; j++) {
      const tag = bytes[p++];
      if (tag === 3) {
        row[j] = view.getFloat64(p, true);
        p += 8;
      } else if (tag === 4) {
        const n = view.getUint32(p, true);
        p += 4;
        row[j] = text.substring(t, t + n);
        t += n;
      } else {
        row[j] = tag === 0 ? null : tag === 2;
      }
    }
    rows[i] = row;
  }
  return {
    columns: copy ? columns.slice() : columns,
    rows,
    rowsAffected: view.getFloat64(at + 8, true),
    rowsScanned: view.getFloat64(at + 16, true),
    message: text.substring(t, t + messageLength),
  };
}")]
extern "C" {
    fn set_memory(memory: &JsValue);
    fn decode_result(at: u32, length: u32, columns: &JsValue, copy: bool) -> JsValue;
}

fn columns_to_js(columns: &[String]) -> JsValue {
    columns.iter().map(|c| JsValue::from_str(c)).collect::<Array>().into()
}

/// A buffer larger than this is not kept for the next result.
const KEEP_BUFFER: usize = 1 << 20;

/// The result in `buf` (binary result format) as a JS object.
fn decode(buf: &mut Vec<u8>, columns: &JsValue, copy: bool) -> JsValue {
    let result = decode_result(buf.as_ptr() as u32, buf.len() as u32, columns, copy);
    if buf.capacity() > KEEP_BUFFER {
        *buf = Vec::new();
    }
    result
}

fn params_from_js(params: Option<Array>) -> Result<Vec<Value>, JsValue> {
    match params {
        None => Ok(Vec::new()),
        Some(arr) => arr.iter().map(|v| value_from_js(&v)).collect(),
    }
}

// Parameters of many rows are written by JavaScript into one buffer in the
// binary parameter format and read by Rust, so a row costs no calls across
// the boundary. A row: u32 number of values, then every value as a tag and
// data (0 NULL, 1 FALSE, 2 TRUE, 3 number: f64, 4 text: u32 length in bytes
// and UTF-8, 5 BigInt: i64), all little-endian. The rows are encoded in
// chunks of about 64 KiB, each requested when the previous one is used up,
// so a large batch does not need a buffer of its size. Values are checked as
// `value_from_js` checks them, with the same errors.
#[wasm_bindgen(inline_js = "
const utf8 = new TextEncoder();
let out = new Uint8Array(1 << 16);
let view = new DataView(out.buffer);
function reserve(p, n) {
  if (p + n <= out.length) return;
  let size = out.length * 2;
  while (size < p + n) size *= 2;
  const bigger = new Uint8Array(size);
  bigger.set(out.subarray(0, p));
  out = bigger;
  view = new DataView(out.buffer);
}
export function encode_rows(rows, start) {
  if (!Array.isArray(rows)) throw new TypeError('rows must be an array of parameter arrays');
  let p = 0;
  for (let i = start; i < rows.length && p < 1 << 16; i++) {
    const row = rows[i];
    if (!Array.isArray(row)) throw new TypeError('rows must be an array of parameter arrays');
    reserve(p, 4);
    view.setUint32(p, row.length, true);
    p += 4;
    for (let j = 0; j < row.length; j++) {
      const v = row[j];
      if (v === null || v === undefined) {
        reserve(p, 1);
        out[p++] = 0;
      } else if (typeof v === 'boolean') {
        reserve(p, 1);
        out[p++] = v ? 2 : 1;
      } else if (typeof v === 'number') {
        reserve(p, 9);
        out[p] = 3;
        view.setFloat64(p + 1, v, true);
        p += 9;
      } else if (typeof v === 'string') {
        reserve(p, 5 + 3 * v.length);
        out[p] = 4;
        let k = 0;
        for (; k < v.length; k++) {
          const c = v.charCodeAt(k);
          if (c > 0x7f) break;
          out[p + 5 + k] = c;
        }
        const n = k === v.length ? k : k + utf8.encodeInto(v.slice(k), out.subarray(p + 5 + k)).written;
        view.setUint32(p + 1, n, true);
        p += 5 + n;
      } else if (typeof v === 'bigint') {
        if (BigInt.asIntN(64, v) !== v) throw 'BigInt parameter out of i64 range';
        reserve(p, 9);
        out[p] = 5;
        view.setBigInt64(p + 1, v, true);
        p += 9;
      } else {
        throw new TypeError('unsupported parameter type (use null, boolean, number, bigint or string)');
      }
    }
  }
  const bytes = out.subarray(0, p);
  if (out.length > 1 << 20) {
    out = new Uint8Array(1 << 16);
    view = new DataView(out.buffer);
  }
  return bytes;
}")]
extern "C" {
    #[wasm_bindgen(catch)]
    fn encode_rows(rows: &JsValue, start: u32) -> Result<Uint8Array, JsValue>;
}

/// A chunk of rows in the binary parameter format, read row by row.
#[derive(Default)]
struct ParamChunk {
    bytes: Vec<u8>,
    at: usize,
}

impl ParamChunk {
    fn take<const N: usize>(&mut self) -> [u8; N] {
        let b = self.bytes[self.at..self.at + N].try_into().expect("N bytes");
        self.at += N;
        b
    }

    fn done(&self) -> bool {
        self.at == self.bytes.len()
    }

    /// Appends the values of the next row to `out`.
    fn row(&mut self, out: &mut Vec<Value>) {
        let n = u32::from_le_bytes(self.take());
        for _ in 0..n {
            let [tag] = self.take();
            out.push(match tag {
                0 => Value::Null,
                1 => Value::Bool(false),
                2 => Value::Bool(true),
                3 => number(f64::from_le_bytes(self.take())),
                4 => {
                    let len = u32::from_le_bytes(self.take()) as usize;
                    let text = String::from_utf8_lossy(&self.bytes[self.at..self.at + len]).into_owned();
                    self.at += len;
                    Value::Text(text)
                }
                _ => Value::Int(i64::from_le_bytes(self.take())),
            });
        }
    }
}

/// A parsed statement with `?` placeholders.
#[wasm_bindgen]
pub struct PreparedStatement {
    inner: core::Prepared,
    /// Column names of the last result in JavaScript, with the names they
    /// were made from (shared by every result of one compiled plan).
    columns: RefCell<Option<(Arc<[String]>, JsValue)>>,
}

impl PreparedStatement {
    /// The column names `names` in JavaScript, created once per plan.
    fn columns(&self, names: &Arc<[String]>) -> JsValue {
        let mut cached = self.columns.borrow_mut();
        match &*cached {
            Some((known, js)) if Arc::ptr_eq(known, names) => js.clone(),
            _ => {
                let js = columns_to_js(names);
                *cached = Some((names.clone(), js.clone()));
                js
            }
        }
    }
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
    /// The last result of `exec`/`run` in the binary result format.
    buf: Vec<u8>,
}

#[wasm_bindgen]
impl Database {
    /// Creates an empty in-memory database. `nodeId` identifies the replica
    /// for synchronisation; a random one is generated when omitted.
    #[wasm_bindgen(constructor)]
    pub fn new(node_id: Option<u32>) -> Database {
        let mut inner = core::Database::new(node_id.unwrap_or_else(random_node_id));
        inner.set_clock(now_ms);
        Database { inner, buf: Vec::new() }
    }

    /// Opens a database from a snapshot and/or WAL bytes (crash recovery).
    pub fn restore(snapshot: Option<Vec<u8>>, wal: Option<Vec<u8>>, node_id: Option<u32>) -> Result<Database, JsValue> {
        let node = node_id.unwrap_or_else(random_node_id);
        let mut inner =
            core::Database::restore(snapshot.as_deref(), wal.as_deref().unwrap_or(&[]), node).map_err(js_error)?;
        inner.set_clock(now_ms);
        Ok(Database { inner, buf: Vec::new() })
    }

    /// Executes SQL (one or more statements) and returns the last result as
    /// `{ columns, rows, rowsAffected, rowsScanned, message }`.
    pub fn exec(&mut self, sql: &str) -> Result<JsValue, JsValue> {
        let r = self.inner.execute(sql).map_err(js_error)?;
        r.encode(&mut self.buf);
        Ok(decode(&mut self.buf, &columns_to_js(&r.columns), false))
    }

    /// Executes SQL and returns an array with the result of every statement.
    #[wasm_bindgen(js_name = execAll)]
    pub fn exec_all(&mut self, sql: &str) -> Result<Array, JsValue> {
        let results = self.inner.execute_all(sql).map_err(js_error)?;
        Ok(results
            .iter()
            .map(|r| {
                r.encode(&mut self.buf);
                decode(&mut self.buf, &columns_to_js(&r.columns), false)
            })
            .collect())
    }

    /// Like `exec` but returns the result serialised as JSON.
    #[wasm_bindgen(js_name = execJson)]
    pub fn exec_json(&mut self, sql: &str) -> Result<String, JsValue> {
        Ok(self.inner.execute(sql).map_err(js_error)?.to_json())
    }

    pub fn prepare(&self, sql: &str) -> Result<PreparedStatement, JsValue> {
        Ok(PreparedStatement { inner: self.inner.prepare(sql).map_err(js_error)?, columns: RefCell::new(None) })
    }

    /// Executes a prepared statement with positional parameters.
    pub fn run(&mut self, stmt: &PreparedStatement, params: Option<Array>) -> Result<JsValue, JsValue> {
        let params = params_from_js(params)?;
        let names = self.inner.execute_prepared_encoded(&stmt.inner, &params, &mut self.buf).map_err(js_error)?;
        let columns = stmt.columns(&names);
        Ok(decode(&mut self.buf, &columns, true))
    }

    /// Executes a prepared statement and returns only the number of affected
    /// rows – no result object is built, which matters for bulk writes.
    pub fn execute(&mut self, stmt: &PreparedStatement, params: Option<Array>) -> Result<f64, JsValue> {
        let params = params_from_js(params)?;
        let r = self.inner.execute_prepared(&stmt.inner, &params).map_err(js_error)?;
        Ok(r.rows_affected as f64)
    }

    /// Executes a prepared INSERT, UPDATE or DELETE once for every array of
    /// parameters in `rows`, as one statement: all rows are applied or none.
    /// Returns the number of affected rows. The parameters cross into
    /// WebAssembly in one buffer, so this is the fast way to write many rows.
    #[wasm_bindgen(js_name = executeMany)]
    pub fn execute_many(&mut self, stmt: &PreparedStatement, rows: &JsValue) -> Result<f64, JsValue> {
        let mut chunk = ParamChunk::default();
        let mut next_row = 0;
        let mut thrown = None;
        let r = self.inner.execute_many(&stmt.inner, |params| {
            if chunk.done() {
                match encode_rows(rows, next_row) {
                    Ok(bytes) => {
                        chunk.bytes.resize(bytes.length() as usize, 0);
                        bytes.copy_to(&mut chunk.bytes);
                        chunk.at = 0;
                    }
                    Err(e) => {
                        // Ends the statement (it is rolled back); the
                        // JavaScript error is what the caller gets.
                        thrown = Some(e);
                        return Err(core::Error::Eval(String::new()));
                    }
                }
                if chunk.done() {
                    return Ok(false);
                }
            }
            chunk.row(params);
            next_row += 1;
            Ok(true)
        });
        match thrown {
            Some(e) => Err(e),
            None => r.map(|n| n as f64).map_err(js_error),
        }
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
