// FlatDB – the Flat-SQL engine bound to a durable store.
//
// Queries run synchronously inside WebAssembly (no network, no awaiting).
// Every committed transaction leaves WAL bytes inside the engine; FlatDB
// drains them on the next microtask and appends them to the store in one
// write (group commit). `await db.flush()` waits until everything committed
// so far is durable. When the WAL grows past `checkpointBytes` a snapshot is
// written and the WAL is truncated.

import { Database } from '../pkg/flatsql.js';
import { engineReady, initEngine } from './engine.js';
import { MemoryStore } from './stores/memory.js';

function randomNodeId() {
  const a = new Uint32Array(1);
  globalThis.crypto.getRandomValues(a);
  return a[0] | 1;
}

function rowsToObjects(result) {
  const { columns, rows } = result;
  return rows.map((row) => {
    const o = {};
    for (let i = 0; i < columns.length; i++) o[columns[i]] = row[i];
    return o;
  });
}

export class FlatDB extends EventTarget {
  #db;
  #store;
  #chain = Promise.resolve();
  #flushQueued = false;
  #closed = false;

  /**
   * Opens (or creates) a database persisted in `store`.
   *
   * @param {object} [options]
   * @param {object} [options.store]  MemoryStore | IndexedDbStore | FsStore
   * @param {number} [options.nodeId] replica id (generated and remembered by default)
   * @param {number} [options.checkpointBytes] WAL size that triggers a snapshot
   * @param {'auto'|'manual'} [options.flush] flush WAL automatically after writes
   */
  static async open(options = {}) {
    const { store = new MemoryStore(), nodeId, checkpointBytes = 1 << 20, flush = 'auto' } = options;
    await (engineReady() ?? initEngine());
    const state = await store.load();
    const node = state.meta.nodeId ?? nodeId ?? randomNodeId();
    const t0 = performance.now();
    const db = Database.restore(state.snapshot ?? undefined, state.wal ?? undefined, node);
    const recoveryMs = performance.now() - t0;
    if (state.meta.nodeId === undefined) await store.setMeta('nodeId', db.nodeId);
    const fdb = new FlatDB(db, store, { checkpointBytes, flush });
    fdb.recovery = { ...db.recoveryReport(), ms: recoveryMs };
    // A torn tail must not stay in front of frames appended later.
    if (fdb.recovery.tornBytes > 0) await fdb.checkpoint();
    return fdb;
  }

  constructor(db, store, { checkpointBytes, flush }) {
    super();
    this.#db = db;
    this.#store = store;
    this.checkpointBytes = checkpointBytes;
    this.autoFlush = flush === 'auto';
  }

  get store() {
    return this.#store;
  }

  get nodeId() {
    return this.#db.nodeId;
  }

  /** Executes SQL; returns `{ columns, rows, rowsAffected, rowsScanned, message }`. */
  exec(sql) {
    const r = this.#db.exec(sql);
    this.#afterWrite();
    return r;
  }

  /** Executes SQL and returns the result of every statement. */
  execAll(sql) {
    const r = this.#db.execAll(sql);
    this.#afterWrite();
    return r;
  }

  /** Same as exec, result serialised to JSON inside WebAssembly. */
  execJson(sql) {
    const r = this.#db.execJson(sql);
    this.#afterWrite();
    return r;
  }

  /** Runs a query and returns rows as plain objects. */
  query(sql, params) {
    const r = params === undefined ? this.#db.exec(sql) : this.run(this.prepare(sql), params);
    this.#afterWrite();
    return rowsToObjects(r);
  }

  prepare(sql) {
    return this.#db.prepare(sql);
  }

  run(statement, params = []) {
    const r = this.#db.run(statement, params);
    this.#afterWrite();
    return r;
  }

  /** Executes a prepared write and returns the number of affected rows. */
  execute(statement, params = []) {
    const n = this.#db.execute(statement, params);
    this.#afterWrite();
    return n;
  }

  runJson(statement, params = []) {
    const r = this.#db.runJson(statement, params);
    this.#afterWrite();
    return r;
  }

  inTransaction() {
    return this.#db.inTransaction();
  }

  tableNames() {
    return this.#db.tableNames();
  }

  stats() {
    return {
      ...this.#db.stats(),
      store: this.#store.kind,
      walBytes: this.#store.walSize,
      snapshotBytes: this.#store.snapshotSize,
    };
  }

  // ---- durability -----------------------------------------------------

  #afterWrite() {
    if (this.#db.walPending() === 0) return;
    this.dispatchEvent(new Event('change'));
    if (this.autoFlush && !this.#flushQueued) {
      this.#flushQueued = true;
      queueMicrotask(() => {
        this.#flushQueued = false;
        this.flush().catch((error) => this.dispatchEvent(new CustomEvent('error', { detail: error })));
      });
    }
  }

  #enqueue(task) {
    const run = this.#chain.catch(() => {}).then(task);
    this.#chain = run;
    return run;
  }

  /** Resolves once every transaction committed so far is in the store. */
  flush() {
    return this.#enqueue(async () => {
      await this.#appendPending();
      if (this.#store.walSize > this.checkpointBytes && !this.#db.inTransaction()) {
        await this.#writeSnapshot();
      }
    });
  }

  /** Writes a snapshot and truncates the WAL. */
  checkpoint() {
    return this.#enqueue(async () => {
      await this.#appendPending();
      await this.#writeSnapshot();
    });
  }

  async #appendPending() {
    const bytes = this.#db.takeWal();
    if (bytes.length === 0) return;
    try {
      await this.#store.appendWal(bytes);
    } catch (err) {
      this.#db.unshiftWal(bytes);
      throw err;
    }
  }

  async #writeSnapshot() {
    const snapshot = this.#db.snapshot();
    await this.#store.writeSnapshot(snapshot);
    this.dispatchEvent(new CustomEvent('checkpoint', { detail: { bytes: snapshot.length } }));
  }

  // ---- synchronisation primitives (used by SyncClient) -----------------

  outboxLength() {
    return this.#db.outboxLength();
  }

  exportOutbox(max) {
    return this.#db.exportOutbox(max);
  }

  ackOutbox(lastSeq) {
    this.#db.ackOutbox(lastSeq);
    this.#afterWrite();
  }

  applyChanges(batch) {
    const report = this.#db.applyChanges(batch);
    if (report.applied > 0) this.dispatchEvent(new CustomEvent('remote-change', { detail: report }));
    this.#afterWrite();
    return report;
  }

  exportState() {
    return this.#db.exportState();
  }

  async close() {
    if (this.#closed) return;
    await this.flush();
    this.#closed = true;
    this.#db.free();
    await this.#store.close?.();
  }
}
