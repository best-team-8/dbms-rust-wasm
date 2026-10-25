// Reads over many rows (fetching 10 000 rows, a key range of 1000 rows with
// SUM) in the WebAssembly engine of the version in DIR, on the benchmark
// table loaded in two ways: inserted in one transaction, as the benchmark
// does, and restored from a snapshot of the same table. Mean of 50 queries
// after 10 warm-up queries, in 5 rounds; the median of the rounds is printed.
//
//   node docs/research/data/optimization/05-outbox-with-sync/layout.mjs DIR [--out FILE]

import { writeFileSync } from 'node:fs';
import { join, resolve } from 'node:path';
import { pathToFileURL } from 'node:url';

import { INDEX, INSERT, SCHEMA, makeRow } from '../../../../../bench/suite.js';

const args = process.argv.slice(2);
const root = resolve(args[0] ?? '.');
const out = args.includes('--out') ? args[args.indexOf('--out') + 1] : null;
const { Database } = await import(pathToFileURL(join(root, 'bridge', 'node.js')).href);

function inserted() {
  const db = new Database(1);
  db.exec(SCHEMA);
  db.exec(INDEX);
  const st = db.prepare(INSERT);
  db.exec('BEGIN');
  for (let i = 0; i < 100_000; i++) db.execute(st, makeRow(i));
  db.exec('COMMIT');
  st.free();
  db.takeWal();
  return db;
}

function time(db, sql, params) {
  const st = db.prepare(sql);
  for (let i = 0; i < 10; i++) db.run(st, params);
  const t0 = performance.now();
  for (let i = 0; i < 50; i++) db.run(st, params);
  const us = ((performance.now() - t0) * 1000) / 50;
  st.free();
  return us;
}

const median = (v) => [...v].sort((a, b) => a - b)[Math.floor(v.length / 2)];
const modes = { inserted: [], restored: [] };
const results = { 'fetch 10 000 rows': structuredClone(modes), 'key range 1000 rows + SUM': structuredClone(modes) };
for (let round = 0; round < 5; round++) {
  const a = inserted();
  const b = Database.restore(a.snapshot(), undefined, 1);
  for (const [mode, db] of [['inserted', a], ['restored', b]]) {
    results['fetch 10 000 rows'][mode].push(time(db, 'SELECT * FROM items WHERE id < 10000', []));
    results['key range 1000 rows + SUM'][mode].push(time(db, 'SELECT SUM(price) FROM items WHERE id BETWEEN ? AND ?', [40000, 40999]));
  }
  a.free();
  b.free();
}
const lines = [`Flat-SQL: ${root}`, 'query | inserted in one transaction, µs | restored from a snapshot, µs (median of 5 rounds)'];
for (const [q, r] of Object.entries(results)) lines.push(`${q} | ${median(r.inserted).toFixed(1)} | ${median(r.restored).toFixed(1)}`);
console.log(lines.join('\n'));
if (out) writeFileSync(out, `${lines.join('\n')}\n`);
