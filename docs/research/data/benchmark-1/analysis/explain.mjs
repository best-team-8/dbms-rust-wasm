// Query plans of the workload in Flat-SQL (EXPLAIN, rowsScanned) and SQLite
// (EXPLAIN QUERY PLAN, sql.js) on a 1000-row table.
//
//   node docs/research/data/benchmark-1/analysis/explain.mjs [FLATSQL_DIR]

import { join, resolve } from 'node:path';
import { fileURLToPath, pathToFileURL } from 'node:url';

const REPO = fileURLToPath(new URL('../../../../../', import.meta.url));
const flatsql = resolve(process.argv[2] ?? REPO);
const { Database } = await import(pathToFileURL(join(flatsql, 'bridge', 'node.js')).href);
const { SCHEMA, INDEX, INSERT, makeRow, READ_OPS, WRITE_OPS } = await import(pathToFileURL(join(REPO, 'bench', 'suite.js')).href);
const { default: initSqlJs } = await import('sql.js');

const db = new Database(1);
db.exec(SCHEMA);
db.exec(INDEX);
const st = db.prepare(INSERT);
db.exec('BEGIN');
for (let i = 0; i < 1000; i++) db.execute(st, makeRow(i));
db.exec('COMMIT');
db.takeWal();

const SQL = await initSqlJs();
const lite = new SQL.Database();
lite.exec(SCHEMA);
lite.exec(INDEX);

const bind = (sql) => sql.replace('id = ?', 'id = 5').replace('category = ?', "category = 'c5'").replace('BETWEEN ? AND ?', 'BETWEEN 10 AND 1009');
for (const op of [...READ_OPS, ...WRITE_OPS]) {
  const sql = bind(op.sql);
  console.log(`== ${op.name}: ${sql}`);
  console.log(db.exec(`EXPLAIN ${sql}`).rows.map((r) => `   Flat-SQL  ${r[0]}`).join('\n'));
  console.log((lite.exec(`EXPLAIN QUERY PLAN ${sql}`)[0]?.values ?? []).map((r) => `   SQLite    ${r[3]}`).join('\n'));
}
const count = db.exec("SELECT COUNT(*) FROM items WHERE category = 'c5'");
console.log(`COUNT(*) by index: ${count.rows[0][0]} rows counted, rowsScanned = ${count.rowsScanned}`);
const update = db.exec('UPDATE items SET qty = qty + 1 WHERE id % 10 = 0');
console.log(`UPDATE 10%: rowsAffected = ${update.rowsAffected}, rowsScanned = ${update.rowsScanned}`);
