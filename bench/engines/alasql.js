// AlaSQL: SQL engine written in plain JavaScript (no WebAssembly), data in
// memory. Queries are compiled once (alasql's prepared statements) and use
// SELECT MATRIX, so rows come back as arrays like in the other engines.

import { makeRow } from '../suite.js';

const SCHEMA = 'CREATE TABLE items (id INT PRIMARY KEY, category STRING, price NUMBER, qty INT, name STRING)';
const INDEX = 'CREATE INDEX items_category ON items (category)';
const VERIFY = 'SELECT MATRIX COUNT(*), SUM(qty), SUM(price) FROM items';

const matrix = (sql) => sql.replace(/^SELECT /, 'SELECT MATRIX ');

async function loadAlasql(env) {
  if (env.kind === 'node') return (await env.importPkg('alasql')).default;
  await env.script('alasql/dist/alasql.min.js');
  return globalThis.alasql;
}

export async function create(env) {
  const alasql = await loadAlasql(env);
  let db;
  return {
    async: false,
    version: alasql.version,
    open() {
      db = new alasql.Database();
      db.exec(SCHEMA);
      db.exec(INDEX);
    },
    load(rows) {
      const insert = alasql.compile('INSERT INTO items VALUES (?, ?, ?, ?, ?)', db.databaseid);
      db.exec('BEGIN TRANSACTION');
      for (let i = 0; i < rows; i++) insert(makeRow(i));
      db.exec('COMMIT TRANSACTION');
    },
    prepare(op) {
      return alasql.compile(matrix(op.sql), db.databaseid);
    },
    query(st, params) {
      return st(params);
    },
    write(op) {
      db.exec(op.sql);
    },
    verify() {
      return db.exec(VERIFY)[0];
    },
    close() {
      db = null;
    },
  };
}
