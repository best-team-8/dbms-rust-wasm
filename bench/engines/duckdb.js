// DuckDB-Wasm: column-oriented analytical engine, database in memory.
// The synchronous ("blocking") bundles run DuckDB in the calling thread: in
// Node.js and in the browser's main thread (group "memory") or inside the
// benchmark's Worker (group "worker"; the browser's usual AsyncDuckDB puts the
// same engine in a Worker of its own). Results are Apache Arrow tables; the
// adapter converts them to rows like every other engine, and that conversion
// is part of the measured time.

import { INSERT, VERIFY_SQL, makeRow } from '../suite.js';

const SCHEMA = 'CREATE TABLE items (id INTEGER PRIMARY KEY, category VARCHAR, price DOUBLE, qty INTEGER, name VARCHAR)';
const INDEX = 'CREATE INDEX items_category ON items (category)';

function toRows(table) {
  const cols = table.numCols;
  const vectors = Array.from({ length: cols }, (_, i) => table.getChildAt(i));
  const out = new Array(table.numRows);
  for (let r = 0; r < out.length; r++) {
    const row = new Array(cols);
    for (let c = 0; c < cols; c++) row[c] = vectors[c].get(r);
    out[r] = row;
  }
  return out;
}

async function loadDuckDB(env) {
  if (env.kind === 'node') {
    const duckdb = await env.importPkg('@duckdb/duckdb-wasm/dist/duckdb-node-blocking.cjs');
    const lib = duckdb.default ?? duckdb;
    const bundles = {
      mvp: { mainModule: env.path('@duckdb/duckdb-wasm/dist/duckdb-mvp.wasm'), mainWorker: '' },
      eh: { mainModule: env.path('@duckdb/duckdb-wasm/dist/duckdb-eh.wasm'), mainWorker: '' },
    };
    return lib.createDuckDB(bundles, new lib.VoidLogger(), lib.NODE_RUNTIME);
  }
  const lib = await env.importPkg(null, 'vendor:duckdb-blocking');
  const bundles = {
    mvp: { mainModule: env.url('@duckdb/duckdb-wasm/dist/duckdb-mvp.wasm'), mainWorker: '' },
    eh: { mainModule: env.url('@duckdb/duckdb-wasm/dist/duckdb-eh.wasm'), mainWorker: '' },
  };
  return lib.createDuckDB(bundles, new lib.VoidLogger(), lib.BROWSER_RUNTIME);
}

export async function create(env) {
  const duck = await loadDuckDB(env);
  await duck.instantiate();
  let conn;
  return {
    async: false,
    version: duck.getVersion(),
    open() {
      conn = duck.connect();
      conn.query('DROP TABLE IF EXISTS items');
      conn.query(SCHEMA);
      conn.query(INDEX);
    },
    load(rows) {
      const insert = conn.prepare(INSERT);
      conn.query('BEGIN TRANSACTION');
      for (let i = 0; i < rows; i++) insert.query(...makeRow(i));
      conn.query('COMMIT');
      insert.close();
    },
    prepare(op) {
      return conn.prepare(op.sql);
    },
    query(st, params) {
      return toRows(st.query(...params));
    },
    finalize(st) {
      st.close();
    },
    write(op) {
      conn.query(op.sql);
    },
    verify() {
      return toRows(conn.query(VERIFY_SQL))[0];
    },
    explain(sql) {
      return toRows(conn.query(`EXPLAIN ${sql.replace(/\?/g, '1')}`));
    },
    close() {
      conn.query('DROP TABLE IF EXISTS items');
      conn.close();
    },
  };
}
