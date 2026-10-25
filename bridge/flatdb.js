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
import { IndexedDbStore } from './stores/indexeddb.js';

const DURABILITY = ['default', 'strict', 'relaxed'];

// Without an explicit store the browser keeps the database in IndexedDB.
// Elsewhere there is no durable default, and silently keeping the data in
// memory would lose it when the process ends, so the caller has to choose.
function defaultStore(name) {
  if (typeof indexedDB === 'undefined') {
    throw new Error(
      'FlatDB.open needs a store outside the browser: { store: new FsStore(dir) } from bridge/node.js ' +
        'keeps the database in files, { store: new MemoryStore() } only in memory',
    );
  }
  return new IndexedDbStore(name);
}

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
  // Prepared statements of exec(sql) and query(sql, params) by SQL text, the
  // least recently used first; null for SQL that is not one statement.
  #statements = new Map();
  // SQL texts executed once by exec(sql): a statement is prepared and kept
  // when its text comes again, so SQL with literals inside does not fill the
  // cache.
  #seen = new Map();
  #cacheSize;
  // Statements that only read: no WAL to look for after them.
  #readOnly = new WeakSet();

  /**
   * Opens (or creates) a database persisted in `store`.
   *
   * @param {object} [options]
   * @param {object} [options.store] MemoryStore | IndexedDbStore | FsStore;
   *   default in the browser: IndexedDbStore(name), required elsewhere
   * @param {string} [options.name] name of the default IndexedDB store ('default')
   * @param {number} [options.nodeId] replica id (generated and remembered by default)
   * @param {number} [options.checkpointBytes] WAL size that triggers a snapshot
   * @param {'auto'|'manual'} [options.flush] flush WAL automatically after writes
   * @param {'default'|'strict'|'relaxed'} [options.durability] durability of
   *   IndexedDB writes: 'strict' completes a flush only after the data reached
   *   the disk; 'default' leaves it to the browser (relaxed in Chrome 121+)
   * @param {boolean} [options.persist] ask the browser not to evict the
   *   IndexedDB data (navigator.storage.persist, default true); Firefox asks
   *   the user, so an application may pass false and call
   *   requestPersistence() later
   * @param {number} [options.statementCache] number of prepared statements
   *   kept for exec(sql) and query(sql, params) by SQL text (default 64, 0
   *   turns the cache off)
   */
  static async open(options = {}) {
    const { name, nodeId, checkpointBytes = 1 << 20, flush = 'auto', durability, persist = true, statementCache = 64 } = options;
    if (name !== undefined && options.store) throw new TypeError('name applies to the default IndexedDB store only');
    const store = options.store ?? defaultStore(name ?? 'default');
    if (durability !== undefined) {
      if (!DURABILITY.includes(durability)) throw new TypeError(`durability must be one of: ${DURABILITY.join(', ')}`);
      if (!('durability' in store)) {
        throw new TypeError(`durability applies to IndexedDbStore, not to the ${store.kind} store`);
      }
      store.durability = durability;
    }
    await (engineReady() ?? initEngine());
    const state = await store.load();
    const node = state.meta.nodeId ?? nodeId ?? randomNodeId();
    const t0 = performance.now();
    const db = Database.restore(state.snapshot ?? undefined, state.wal ?? undefined, node);
    const recoveryMs = performance.now() - t0;
    if (state.meta.nodeId === undefined) await store.setMeta('nodeId', db.nodeId);
    const fdb = new FlatDB(db, store, { checkpointBytes, flush, statementCache });
    fdb.recovery = { ...db.recoveryReport(), ms: recoveryMs };
    // A torn tail must not stay in front of frames appended later.
    if (fdb.recovery.tornBytes > 0) await fdb.checkpoint();
    // Browser storage is "best effort" and may be evicted together with the
    // unsent outbox. Not awaited: Firefox waits for the user's answer.
    fdb.persistence = store.kind === 'indexeddb' ? fdb.#persistence(persist) : Promise.resolve(null);
    return fdb;
  }

  constructor(db, store, { checkpointBytes, flush, statementCache = 64 }) {
    super();
    this.#db = db;
    this.#store = store;
    this.#cacheSize = statementCache;
    this.checkpointBytes = checkpointBytes;
    this.autoFlush = flush === 'auto';
    /** true: the browser will not evict the data; false: it may; null: unknown or not applicable. */
    this.persisted = null;
    this.persistence = Promise.resolve(null);
  }

  get store() {
    return this.#store;
  }

  get nodeId() {
    return this.#db.nodeId;
  }

  /** Executes SQL; returns `{ columns, rows, rowsAffected, rowsScanned, message }`. */
  exec(sql) {
    const st = this.#cached(sql, false);
    if (st) return this.run(st);
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
    if (params === undefined) return rowsToObjects(this.exec(sql));
    return rowsToObjects(this.run(this.#cached(sql, true) ?? this.prepare(sql), params));
  }

  prepare(sql) {
    const statement = this.#db.prepare(sql);
    if (statement.readOnly) this.#readOnly.add(statement);
    return statement;
  }

  run(statement, params = []) {
    const r = this.#db.run(statement, params);
    if (!this.#readOnly.has(statement)) this.#afterWrite();
    return r;
  }

  /**
   * Prepared statement of `sql` from the cache, or prepared now and kept;
   * null when the cache is off or `sql` is not a single statement. exec(sql)
   * keeps a statement from the second time its text is executed, query with
   * parameters from the first.
   */
  #cached(sql, now) {
    if (this.#cacheSize <= 0) return null;
    let statement = this.#statements.get(sql);
    if (statement !== undefined) {
      this.#statements.delete(sql);
      this.#statements.set(sql, statement);
      return statement;
    }
    if (!now && !this.#remember(sql)) return null;
    try {
      statement = this.prepare(sql);
    } catch {
      statement = null; // several statements, or an error exec reports
    }
    this.#statements.set(sql, statement);
    if (this.#statements.size > this.#cacheSize) {
      const [oldest, evicted] = this.#statements.entries().next().value;
      this.#statements.delete(oldest);
      evicted?.free();
    }
    return statement;
  }

  /** Whether `sql` was executed before; remembers it otherwise. */
  #remember(sql) {
    if (this.#seen.delete(sql)) return true;
    this.#seen.set(sql, true);
    if (this.#seen.size > 4 * this.#cacheSize) this.#seen.delete(this.#seen.keys().next().value);
    return false;
  }

  /** Executes a prepared write and returns the number of affected rows. */
  execute(statement, params = []) {
    const n = this.#db.execute(statement, params);
    this.#afterWrite();
    return n;
  }

  /**
   * Executes a prepared INSERT, UPDATE or DELETE once for every array of
   * parameters in `rows`, as one statement (all rows or none); returns the
   * number of affected rows.
   */
  executeMany(statement, rows) {
    const n = this.#db.executeMany(statement, rows);
    this.#afterWrite();
    return n;
  }

  runJson(statement, params = []) {
    const r = this.#db.runJson(statement, params);
    if (!this.#readOnly.has(statement)) this.#afterWrite();
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
      durability: this.#store.durability ?? null,
      persisted: this.persisted,
      walBytes: this.#store.walSize,
      snapshotBytes: this.#store.snapshotSize,
    };
  }

  // ---- durability -----------------------------------------------------

  /**
   * Asks the browser to keep the site's storage (navigator.storage.persist).
   * Resolves to true when granted, false when refused, null without the API.
   */
  requestPersistence() {
    this.persistence = this.#persistence(true);
    return this.persistence;
  }

  async #persistence(ask) {
    const storage = globalThis.navigator?.storage;
    if (!storage?.persisted) return null;
    try {
      this.persisted = await storage.persisted();
      if (!this.persisted && ask && storage.persist) this.persisted = await storage.persist();
    } catch {
      this.persisted = false;
    }
    this.dispatchEvent(new CustomEvent('persistence', { detail: { persisted: this.persisted } }));
    return this.persisted;
  }

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

  /**
   * Starts keeping local changes for synchronisation (SyncClient does it);
   * until then the outbox holds only schema changes. Every existing row
   * becomes a pending change. Returns whether synchronisation was off.
   */
  enableSync() {
    const changed = this.#db.enableSync();
    this.#afterWrite();
    return changed;
  }

  get syncEnabled() {
    return this.#db.syncEnabled;
  }

  exportOutbox(max) {
    return this.#db.exportOutbox(max);
  }

  ackOutbox(lastSeq) {
    this.#db.ackOutbox(lastSeq);
    this.#afterWrite();
  }

  /** True when local changes were stamped by a clock running ahead (see rebaseClock). */
  clockAhead() {
    return this.#db.clockAhead();
  }

  /**
   * Gives new stamps to unsent changes made while the device clock ran ahead
   * (once it is right again) and makes them durable with a checkpoint.
   * Resolves to the number of re-stamped changes.
   */
  rebaseClock() {
    return this.#enqueue(async () => {
      if (!this.#db.clockAhead()) return 0;
      const restamped = this.#db.rebaseClock();
      await this.#appendPending();
      await this.#writeSnapshot();
      return restamped;
    });
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
    for (const statement of this.#statements.values()) statement?.free();
    this.#statements.clear();
    this.#db.free();
    await this.#store.close?.();
  }
}
