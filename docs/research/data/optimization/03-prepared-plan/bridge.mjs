// Cost of the bridge (FlatDB) around a point lookup, for the version of
// Flat-SQL in DIR: a prepared SELECT through FlatDB.run, FlatDB.query with
// the same SQL text and parameters, FlatDB.exec with the same SQL text, and
// FlatDB.exec with a different literal in every call. Each mode: mean of
// 20 000 calls after 2000 warm-up calls, in 10 rounds in alternating order;
// the table has 100 000 rows of the benchmark workload in a MemoryStore.
//
//   node docs/research/data/optimization/03-prepared-plan/bridge.mjs DIR [--out FILE]

import { writeFileSync } from 'node:fs';
import { join, resolve } from 'node:path';
import { pathToFileURL } from 'node:url';

import { INDEX, INSERT, Rng, SCHEMA, makeRow } from '../../../../../bench/suite.js';

const args = process.argv.slice(2);
const root = resolve(args[0] ?? '.');
const out = args.includes('--out') ? args[args.indexOf('--out') + 1] : null;
const { FlatDB, MemoryStore } = await import(pathToFileURL(join(root, 'bridge', 'node.js')).href);

const ROWS = 100_000;
const CALLS = 20_000;
const db = await FlatDB.open({ store: new MemoryStore(), flush: 'manual' });
db.exec(SCHEMA);
db.exec(INDEX);
const ins = db.prepare(INSERT);
db.exec('BEGIN');
for (let i = 0; i < ROWS; i++) db.execute(ins, makeRow(i));
db.exec('COMMIT');
await db.flush();

const lookup = db.prepare('SELECT * FROM items WHERE id = ?');
const modes = {
  'FlatDB.run (prepared)': (k) => db.run(lookup, [k]),
  'FlatDB.query (same text, parameter)': (k) => db.query('SELECT * FROM items WHERE id = ?', [k]),
  'FlatDB.exec (same text)': () => db.exec('SELECT * FROM items WHERE id = 4242'),
  'FlatDB.exec (literal in the text)': (k) => db.exec(`SELECT * FROM items WHERE id = ${k}`),
};

const rng = new Rng(5);
const keys = Array.from({ length: CALLS }, () => rng.below(ROWS));
const results = Object.fromEntries(Object.keys(modes).map((m) => [m, []]));
for (let round = 0; round < 10; round++) {
  const names = Object.keys(modes);
  if (round % 2) names.reverse();
  for (const name of names) {
    const f = modes[name];
    for (let i = 0; i < 2000; i++) f(keys[i]);
    const t0 = performance.now();
    for (let i = 0; i < CALLS; i++) f(keys[i]);
    results[name].push(((performance.now() - t0) * 1000) / CALLS);
  }
}

const median = (v) => {
  const s = [...v].sort((a, b) => a - b);
  return (s[4] + s[5]) / 2;
};
const lines = [`Flat-SQL: ${root}`, 'mode | median of 10 rounds, µs per call | rounds'];
for (const [name, v] of Object.entries(results)) lines.push(`${name} | ${median(v).toFixed(2)} | ${v.map((x) => x.toFixed(2)).join(' ')}`);
console.log(lines.join('\n'));
if (out) writeFileSync(out, `${lines.join('\n')}\n`);
await db.close();
