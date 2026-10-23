// Official SQLite WebAssembly build (@sqlite.org/sqlite-wasm), oo1 API.
//   memory       – ':memory:' database, synchronous API, any thread;
//   opfs-sahpool – database file in the Origin Private File System through the
//                  "opfs-sahpool" VFS: needs a Worker, no COOP/COEP headers;
//                  every transaction is durable when COMMIT returns.

import { INDEX, INSERT, SCHEMA, VERIFY_SQL, makeRow } from '../suite.js';

const FILE = '/bench.sqlite3';

export async function create(env, { vfs = 'memory' } = {}) {
  const sqlite3InitModule = (await env.importPkg('@sqlite.org/sqlite-wasm', '@sqlite.org/sqlite-wasm/dist/index.mjs')).default;
  const sqlite3 = await sqlite3InitModule();
  const { capi, oo1, wasm } = sqlite3;
  const pool = vfs === 'opfs-sahpool' ? await sqlite3.installOpfsSAHPoolVfs({ name: 'bench-sahpool' }) : null;
  let db;
  let insert;
  const openDb = (schema) => {
    db = pool ? new pool.OpfsSAHPoolDb(FILE) : new oo1.DB(':memory:');
    if (schema) {
      db.exec(SCHEMA);
      db.exec(INDEX);
    }
    insert = db.prepare(INSERT);
  };
  const fill = (rows) => {
    db.exec('BEGIN');
    for (let i = 0; i < rows; i++) insert.bind(makeRow(i)).stepReset();
    db.exec('COMMIT');
  };
  const closeDb = () => {
    insert.finalize();
    db.close();
  };

  const engine = {
    async: !!pool,
    version: sqlite3.version.libVersion,
    async open() {
      if (pool) await pool.wipeFiles();
      openDb(true);
    },
    load: fill,
    prepare(op) {
      return db.prepare(op.sql);
    },
    query(st, params) {
      if (params.length) st.bind(params);
      const rows = [];
      while (st.step()) rows.push(st.get([]));
      st.reset(true);
      return rows;
    },
    finalize(st) {
      st.finalize();
    },
    write(op) {
      db.exec(op.sql);
    },
    verify() {
      return db.selectArray(VERIFY_SQL);
    },
    explain(sql) {
      return db.selectArrays(`EXPLAIN QUERY PLAN ${sql}`);
    },
    memory: () => ({ wasmBytes: wasm.heap8().byteLength }),
    async close() {
      closeDb();
      if (pool) await pool.wipeFiles();
    },
  };

  if (!pool) {
    // Synchronous API: plain calls, nothing to await.
    engine.open = () => openDb(true);
    engine.close = closeDb;
    engine.snapshot = () => capi.sqlite3_js_db_export(db);
    engine.restore = (bytes) => {
      closeDb();
      db = new oo1.DB(':memory:');
      const p = wasm.allocFromTypedArray(bytes);
      const rc = capi.sqlite3_deserialize(
        db.pointer,
        'main',
        p,
        bytes.length,
        bytes.length,
        capi.SQLITE_DESERIALIZE_FREEONCLOSE | capi.SQLITE_DESERIALIZE_RESIZEABLE,
      );
      db.checkRc(rc);
      insert = db.prepare(INSERT);
      // the restored database owns the buffer (FREEONCLOSE)
    };
    return engine;
  }

  return {
    ...engine,
    commit(row) {
      insert.bind(row).stepReset();
    },
    // Every autocommit transaction is synced on its own; SQLite has no
    // group commit across transactions.
    commitGroup(rows) {
      for (const row of rows) insert.bind(row).stepReset();
    },
    reopen() {
      closeDb();
      openDb(false);
    },
  };
}
