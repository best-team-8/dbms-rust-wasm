// wa-sqlite (PowerSync's maintained fork of rhashimoto/wa-sqlite): SQLite
// compiled with Asyncify, so that a VFS can call asynchronous browser APIs.
// IDBBatchAtomicVFS stores the database file as pages in IndexedDB and uses
// SQLite's batch-atomic writes, so a commit writes only changed pages. It
// works in the main thread. A commit is durable when its statement resolves;
// `PRAGMA synchronous` chooses the IndexedDB durability (full → strict,
// normal → relaxed; SQLite's default is full).

import { INDEX, INSERT, SCHEMA, VERIFY_SQL, makeRow } from '../suite.js';

const NAME = 'bench-wa-sqlite';
const FILE = 'bench.db';
const BASE = '@journeyapps/wa-sqlite';

export async function create(env, { synchronous = 'full' } = {}) {
  const SQLiteESMFactory = (await env.importPkg(null, `${BASE}/dist/wa-sqlite-async.mjs`)).default;
  const SQLite = await env.importPkg(null, `${BASE}/src/sqlite-api.js`);
  const { IDBBatchAtomicVFS } = await env.importPkg(null, `${BASE}/src/examples/IDBBatchAtomicVFS.js`);
  const module = await SQLiteESMFactory({ locateFile: (f) => env.url(`${BASE}/dist/${f}`) });
  const sqlite3 = SQLite.Factory(module);
  const vfs = await IDBBatchAtomicVFS.create(NAME, module);
  sqlite3.vfs_register(vfs, true);
  let db;
  let insert;

  const prepare = async (sql) => {
    for await (const stmt of sqlite3.statements(db, sql, { unscoped: true })) return stmt;
    throw new Error(`no statement in ${sql}`);
  };
  const all = async (stmt, params) => {
    if (params?.length) sqlite3.bind_collection(stmt, params);
    const rows = [];
    while ((await sqlite3.step(stmt)) === SQLite.SQLITE_ROW) rows.push(sqlite3.row(stmt));
    await sqlite3.reset(stmt);
    return rows;
  };
  const run = async (stmt, params) => {
    sqlite3.bind_collection(stmt, params);
    await sqlite3.step(stmt);
    await sqlite3.reset(stmt);
  };
  const openDb = async (schema) => {
    db = await sqlite3.open_v2(FILE);
    await sqlite3.exec(db, `PRAGMA synchronous = ${synchronous}`);
    if (schema) {
      await sqlite3.exec(db, 'DROP TABLE IF EXISTS items');
      await sqlite3.exec(db, SCHEMA);
      await sqlite3.exec(db, INDEX);
    }
    insert = await prepare(INSERT);
  };
  const closeDb = async () => {
    await sqlite3.finalize(insert);
    await sqlite3.close(db);
  };

  return {
    async: true,
    version: sqlite3.libversion(),
    // A fresh database: the table is dropped and created again (the VFS
    // keeps its IndexedDB connection for the whole page).
    open() {
      return openDb(true);
    },
    async load(rows) {
      await sqlite3.exec(db, 'BEGIN');
      for (let i = 0; i < rows; i++) await run(insert, makeRow(i));
      await sqlite3.exec(db, 'COMMIT');
    },
    prepare(op) {
      return prepare(op.sql);
    },
    query: all,
    finalize(stmt) {
      return sqlite3.finalize(stmt);
    },
    async write(op) {
      await sqlite3.exec(db, op.sql);
    },
    async verify() {
      const st = await prepare(VERIFY_SQL);
      const rows = await all(st);
      await sqlite3.finalize(st);
      return rows[0];
    },
    commit(row) {
      return run(insert, row);
    },
    // Every autocommit transaction is made durable on its own.
    async commitGroup(rows) {
      for (const row of rows) await run(insert, row);
    },
    async reopen() {
      await closeDb();
      await openDb(false);
    },
    async close({ destroy } = {}) {
      if (destroy) await sqlite3.exec(db, 'DROP TABLE IF EXISTS items');
      await closeDb();
    },
  };
}
