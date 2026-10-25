// Flat-SQL adapters. The engine version is chosen by the runner: env.flatsql()
// loads the bridge of the measured checkout (see bench/README.md), so the same
// adapter measures the `prototyp` tag and later versions.
//
// Only API present in the `prototyp` version is used (Database: exec, prepare,
// run, runJson, execute, takeWal, snapshot, restore; FlatDB with IndexedDbStore),
// except `insert_many`, which uses Database.executeMany where the measured
// version has it and the loop of the insert phase otherwise.

import { INDEX, INSERT, MANY_INDEX, MANY_INSERT, MANY_SCHEMA, MANY_VERIFY_SQL, SCHEMA, VERIFY_SQL, makeRow } from '../suite.js';

/**
 * In-memory Database (group "memory"). `json: true` returns results as a JSON
 * string parsed by JavaScript instead of JS objects built in Rust – the two
 * Rust↔JS interop strategies (H3).
 */
export async function create(env, { json = false, store = null, durability = 'default' } = {}) {
  const flat = await env.flatsql();
  if (store === 'idb') return durableEngine(env, flat, { durability });
  const { Database } = flat;
  let db;
  return {
    async: false,
    open() {
      db = new Database(1);
      db.exec(SCHEMA);
      db.exec(INDEX);
    },
    load(rows) {
      const st = db.prepare(INSERT);
      db.exec('BEGIN');
      for (let i = 0; i < rows; i++) db.execute(st, makeRow(i));
      db.exec('COMMIT');
      st.free();
      // WAL bytes are produced exactly as in production; without a store
      // nobody writes them, so drop them here.
      db.takeWal();
    },
    loadMany(rows) {
      db.exec(MANY_SCHEMA);
      db.exec(MANY_INDEX);
      const st = db.prepare(MANY_INSERT);
      db.exec('BEGIN');
      let method = 'execute';
      if (typeof db.executeMany === 'function') {
        const all = new Array(rows);
        for (let i = 0; i < rows; i++) all[i] = makeRow(i);
        db.executeMany(st, all);
        method = 'executeMany';
      } else {
        for (let i = 0; i < rows; i++) db.execute(st, makeRow(i));
      }
      db.exec('COMMIT');
      st.free();
      db.takeWal();
      return method;
    },
    verifyMany() {
      return db.exec(MANY_VERIFY_SQL).rows[0];
    },
    prepare(op) {
      return db.prepare(op.sql);
    },
    query: json ? (st, params) => JSON.parse(db.runJson(st, params)).rows : (st, params) => db.run(st, params).rows,
    finalize(st) {
      st.free();
    },
    write(op) {
      db.exec(op.sql);
      db.takeWal();
    },
    verify() {
      return db.exec(VERIFY_SQL).rows[0];
    },
    snapshot() {
      return db.snapshot();
    },
    restore(bytes) {
      db.free();
      db = Database.restore(bytes, undefined, 1);
    },
    explain(sql) {
      return db.exec(`EXPLAIN ${sql}`).rows;
    },
    memory: () => wasmMemory(flat),
    close() {
      db.free();
    },
  };
}

async function wasmMemory(flat) {
  const exports = await flat.engineReady?.();
  return { wasmBytes: exports?.memory?.buffer?.byteLength ?? null };
}

/**
 * FlatDB on IndexedDB with the default configuration of the bridge (automatic
 * flush, checkpoint after 1 MiB of WAL). A write is measured until it is
 * durable: `await db.flush()`.
 */
function durableEngine(env, flat, { durability }) {
  const { FlatDB, IndexedDbStore } = flat;
  const name = `bench-${durability}`;
  let db;
  let insert;
  const openDb = async () => {
    db = await FlatDB.open({ store: new IndexedDbStore(name), durability, persist: false });
    insert = db.prepare(INSERT);
  };
  return {
    async: true,
    async open({ fresh } = {}) {
      if (fresh) await new IndexedDbStore(name).destroy();
      await openDb();
      db.exec(SCHEMA);
      db.exec(INDEX);
      await db.flush();
    },
    async load(rows) {
      db.exec('BEGIN');
      for (let i = 0; i < rows; i++) db.execute(insert, makeRow(i));
      db.exec('COMMIT');
      await db.flush();
    },
    prepare(op) {
      return db.prepare(op.sql);
    },
    query(st, params) {
      return db.run(st, params).rows;
    },
    finalize(st) {
      st.free();
    },
    async write(op) {
      db.exec(op.sql);
      await db.flush();
    },
    verify() {
      return db.exec(VERIFY_SQL).rows[0];
    },
    async commit(row) {
      db.execute(insert, row);
      await db.flush();
    },
    async commitGroup(rows) {
      for (const row of rows) db.execute(insert, row);
      await db.flush();
    },
    async checkpoint() {
      await db.checkpoint();
      return { bytes: db.stats().snapshotBytes };
    },
    async reopen() {
      await db.close();
      await openDb();
      return { restoreMs: db.recovery?.ms ?? null };
    },
    memory: () => wasmMemory(flat),
    async close({ destroy } = {}) {
      await db.close();
      if (destroy) await new IndexedDbStore(name).destroy();
    },
  };
}
