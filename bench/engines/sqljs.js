// sql.js: SQLite compiled with Emscripten, database in memory. Its only way
// to durability is exporting the whole database file; the "idb" variant does
// that after every commit and stores the file in IndexedDB. export() frees
// every prepared statement of the database, so writes prepare their own.

import { INDEX, INSERT, SCHEMA, VERIFY_SQL, makeRow } from '../suite.js';
import { idbDelete, idbGet, idbOpen, idbPut } from './idb.js';

async function loadSqlJs(env) {
  if (env.kind === 'node') return (await env.importPkg('sql.js')).default();
  await env.script('sql.js/dist/sql-wasm-browser.js');
  return globalThis.initSqlJs({ locateFile: (f) => env.url(`sql.js/dist/${f}`) });
}

export async function create(env, { store = null } = {}) {
  const SQL = await loadSqlJs(env);
  let db;
  const base = {
    prepare(op) {
      return db.prepare(op.sql);
    },
    query(st, params) {
      if (params.length) st.bind(params);
      const rows = [];
      while (st.step()) rows.push(st.get());
      st.reset();
      return rows;
    },
    finalize(st) {
      st.free();
    },
    verify() {
      return db.exec(VERIFY_SQL)[0].values[0];
    },
    explain(sql) {
      return db.exec(`EXPLAIN QUERY PLAN ${sql}`)[0]?.values ?? [];
    },
  };
  const fill = (rows) => {
    const st = db.prepare(INSERT);
    db.exec('BEGIN');
    for (let i = 0; i < rows; i++) st.run(makeRow(i));
    db.exec('COMMIT');
    st.free();
  };
  const fresh = () => {
    db = new SQL.Database();
    db.exec(SCHEMA);
    db.exec(INDEX);
  };

  if (store !== 'idb') {
    return {
      ...base,
      async: false,
      open: fresh,
      load: fill,
      write(op) {
        db.exec(op.sql);
      },
      snapshot() {
        return db.export();
      },
      restore(bytes) {
        db.close();
        db = new SQL.Database(bytes);
      },
      close() {
        db.close();
      },
    };
  }

  const NAME = 'bench-sqljs';
  let idb;
  const persist = () => idbPut(idb, 'db', db.export());
  return {
    ...base,
    async: true,
    async open({ fresh: clean } = {}) {
      if (clean) await idbDelete(NAME);
      idb = await idbOpen(NAME);
      fresh();
      await persist();
    },
    async load(rows) {
      fill(rows);
      await persist();
    },
    async write(op) {
      db.exec(op.sql);
      await persist();
    },
    async commit(row) {
      db.run(INSERT, row);
      await persist();
    },
    async commitGroup(rows) {
      const st = db.prepare(INSERT);
      for (const row of rows) st.run(row);
      st.free();
      await persist();
    },
    /** The sql.js counterpart of a checkpoint: export and store the file. */
    async checkpoint() {
      const bytes = db.export();
      await idbPut(idb, 'db', bytes);
      return { bytes: bytes.length };
    },
    async reopen() {
      db.close();
      idb.close();
      idb = await idbOpen(NAME);
      db = new SQL.Database(await idbGet(idb, 'db'));
    },
    async close({ destroy } = {}) {
      db.close();
      idb.close();
      if (destroy) await idbDelete(NAME);
    },
  };
}
