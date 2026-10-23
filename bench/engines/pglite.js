// PGlite: PostgreSQL compiled to WebAssembly, asynchronous API, one
// connection. PGlite has no client-side prepared statement handle; every
// query goes through the extended protocol with parameters ($1, $2, …).
//   memory – data directory in memory (any environment);
//   idb    – data directory mirrored to IndexedDB (browser, any thread);
//   opfs   – data directory in OPFS through access handles (Worker only).
// In idb and opfs mode a query returns after its changes were written to the
// storage (relaxedDurability: false, the default).

import { INSERT, VERIFY_SQL, makeRow } from '../suite.js';

const SCHEMA = 'CREATE TABLE items (id INTEGER PRIMARY KEY, category TEXT, price DOUBLE PRECISION, qty INTEGER, name TEXT)';
const INDEX = 'CREATE INDEX items_category ON items (category)';
const pg$ = (sql) => {
  let n = 0;
  return sql.replace(/\?/g, () => `$${++n}`);
};
const ARRAY = { rowMode: 'array' };

export async function create(env, { fs = 'memory' } = {}) {
  const { PGlite } = await env.importPkg('@electric-sql/pglite', '@electric-sql/pglite/dist/index.js');
  const OpfsAhpFS = fs === 'opfs' ? (await env.importPkg('@electric-sql/pglite/opfs-ahp', '@electric-sql/pglite/dist/fs/opfs-ahp.js')).OpfsAhpFS : null;
  const dataDir = { memory: undefined, idb: 'idb://bench-pglite', opfs: undefined }[fs];
  const insertSql = pg$(INSERT);
  let pg;
  const openDb = async (options = {}) => {
    pg = await PGlite.create(dataDir, OpfsAhpFS ? { fs: new OpfsAhpFS('bench-pglite'), ...options } : options);
  };
  const wipe = async () => {
    if (fs === 'idb') {
      await new Promise((resolve) => {
        const req = indexedDB.deleteDatabase('/pglite/bench-pglite');
        req.onsuccess = req.onerror = req.onblocked = () => resolve();
      });
    } else if (fs === 'opfs') {
      const root = await navigator.storage.getDirectory();
      await root.removeEntry('bench-pglite', { recursive: true }).catch(() => {});
    }
  };
  // One transaction; inside transaction() PGlite writes the data directory
  // to its storage once, at COMMIT, instead of after every statement.
  const fill = (rows) =>
    pg.transaction(async (tx) => {
      for (let i = 0; i < rows; i++) await tx.query(insertSql, makeRow(i));
    });

  const engine = {
    async: true,
    async open({ fresh } = {}) {
      if (fresh) await wipe();
      await openDb();
      await pg.exec(SCHEMA);
      await pg.exec(INDEX);
    },
    load: fill,
    prepare(op) {
      return pg$(op.sql);
    },
    async query(sql, params) {
      return (await pg.query(sql, params, ARRAY)).rows;
    },
    async write(op) {
      await pg.exec(op.sql);
    },
    async verify() {
      return (await pg.query(VERIFY_SQL, [], ARRAY)).rows[0];
    },
    async close({ destroy } = {}) {
      await pg.close();
      if (destroy) await wipe();
    },
  };

  if (fs === 'memory') {
    // PGlite's own way of saving and loading a database: a tarball of the
    // data directory.
    engine.snapshot = async () => new Uint8Array(await (await pg.dumpDataDir('none')).arrayBuffer());
    engine.restore = async (bytes) => {
      await pg.close();
      await openDb({ loadDataDir: new Blob([bytes]) });
    };
    return engine;
  }

  return {
    ...engine,
    async commit(row) {
      await pg.query(insertSql, row);
    },
    // Every autocommit transaction is made durable on its own.
    async commitGroup(rows) {
      for (const row of rows) await pg.query(insertSql, row);
    },
    async reopen() {
      await pg.close();
      await openDb();
    },
  };
}
