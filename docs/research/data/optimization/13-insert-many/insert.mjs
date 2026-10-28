// Where the time of inserting rows one prepared INSERT at a time goes at the
// JavaScript/WebAssembly boundary, for the version of Flat-SQL in DIR
// (Node.js, V8). The benchmark table and rows of `bench/suite.js`.
//
// 1. Variants, each timed over 100 000 rows in one transaction on a new
//    database, 7 rounds in alternating order, median of the rounds:
//      rows     makeRow(i) only                 creating the rows in JS
//      select   db.execute(SELECT ?,?,?,?,?, r) parameters and a call that
//                                               writes nothing
//      execute  db.execute(INSERT, r)           the insert as in the workload
//      many     db.executeMany(INSERT, rows)    one call for all rows (when
//                                               the version has it)
// 2. The V8 sampling profiler over the `execute` loop (and `many`), samples
//    grouped as in `10-boundary-profile/boundary.mjs`, with text encoding
//    (UTF-8 encoding of string parameters) as a group of its own.
//
//   node docs/research/data/optimization/13-insert-many/insert.mjs DIR [--out FILE]

import { spawnSync } from 'node:child_process';
import { mkdtempSync, readdirSync, readFileSync, rmSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join, resolve } from 'node:path';
import { fileURLToPath, pathToFileURL } from 'node:url';

const HERE = fileURLToPath(import.meta.url);
const SUITE = pathToFileURL(resolve(fileURLToPath(new URL('.', import.meta.url)), '../../../../../bench/suite.js')).href;
const N = 100_000;

async function setup(root) {
  const { Database } = await import(pathToFileURL(join(root, 'bridge', 'node.js')).href);
  const suite = await import(SUITE);
  return { Database, ...suite };
}

/** Runs variant `name` once; returns its time in ms. */
function once(env, name) {
  const { Database, SCHEMA, INDEX, INSERT, makeRow } = env;
  const db = new Database(1);
  db.exec(SCHEMA);
  db.exec(INDEX);
  const st = db.prepare(name === 'select' ? 'SELECT ?, ?, ?, ?, ?' : INSERT);
  db.exec('BEGIN');
  const t0 = performance.now();
  if (name === 'rows') {
    let n = 0;
    for (let i = 0; i < N; i++) n += makeRow(i).length;
    if (n !== 5 * N) throw new Error('rows');
  } else if (name === 'many') {
    const rows = new Array(N);
    for (let i = 0; i < N; i++) rows[i] = makeRow(i);
    db.executeMany(st, rows);
  } else {
    for (let i = 0; i < N; i++) db.execute(st, makeRow(i));
  }
  const ms = performance.now() - t0;
  db.exec('COMMIT');
  if (name !== 'rows' && name !== 'select' && db.exec('SELECT COUNT(*) FROM items').rows[0][0] !== N) throw new Error(name);
  st.free();
  db.free();
  return ms;
}

if (process.argv[2] === '--profile') {
  const [, , , name, root] = process.argv;
  const env = await setup(root);
  for (let r = 0; r < 5; r++) once(env, name);
} else {
  const root = resolve(process.argv.slice(2).find((a, i, all) => !a.startsWith('--') && all[i - 1] !== '--out') ?? '.');
  const out = process.argv.includes('--out') ? process.argv[process.argv.indexOf('--out') + 1] : null;
  const env = await setup(root);
  const hasMany = typeof env.Database.prototype.executeMany === 'function';
  const names = ['rows', 'select', 'execute', ...(hasMany ? ['many'] : [])];
  const lines = [`Flat-SQL: ${root}`, `${N} rows per round`, '', 'variant | median of 7 rounds, ms | rounds'];
  for (const n of names) once(env, n); // warm-up
  const times = Object.fromEntries(names.map((n) => [n, []]));
  for (let round = 0; round < 7; round++) {
    for (const n of round % 2 ? [...names].reverse() : names) times[n].push(once(env, n));
  }
  const median = (v) => [...v].sort((a, b) => a - b)[3];
  for (const n of names) lines.push(`${n} | ${median(times[n]).toFixed(1)} | ${times[n].map((t) => t.toFixed(1)).join(' ')}`);

  const groups = [
    ['text encoding', (f) => /encodeInto|encodeString|passStringToWasm|^encode /.test(f)],
    ['UTF-8 decoding', (f) => /decodeUTF8|decodeText|^decode /.test(f)],
    ['glue code and transitions', (f) => /flatsql\.js|inline0\.js|wasm-to-js|js-to-wasm/.test(f)],
    ['engine (WebAssembly)', (f) => /^wasm-function/.test(f)],
    ['garbage collection', (f) => /garbage collector/.test(f)],
    ['rows and measuring loop', (f) => /insert\.mjs|suite\.js/.test(f)],
  ];
  for (const name of names.filter((n) => n === 'execute' || n === 'many')) {
    const dir = mkdtempSync(join(tmpdir(), 'flatsql-prof-'));
    spawnSync(process.execPath, ['--cpu-prof', `--cpu-prof-dir=${dir}`, HERE, '--profile', name, root], { encoding: 'utf8' });
    const profile = JSON.parse(readFileSync(join(dir, readdirSync(dir)[0]), 'utf8'));
    rmSync(dir, { recursive: true, force: true });
    const nodes = new Map(profile.nodes.map((n) => [n.id, n]));
    const shares = new Map(groups.map(([g]) => [g, 0]));
    shares.set('other', 0);
    const self = new Map();
    for (const id of profile.samples) {
      const f = nodes.get(id).callFrame;
      const label = `${f.functionName || '(anonymous)'} ${f.url.split('/').pop()}`.trim();
      self.set(label, (self.get(label) ?? 0) + 1);
      const g = groups.find(([, test]) => test(label));
      shares.set(g ? g[0] : 'other', shares.get(g ? g[0] : 'other') + 1);
    }
    const total = profile.samples.length;
    lines.push('', `profile of \`${name}\` (${total} samples, 5 rounds with table setup)`);
    for (const [g, n] of shares) lines.push(`  ${((100 * n) / total).toFixed(1).padStart(5)}%  ${g}`);
    lines.push('  top functions:');
    for (const [label, n] of [...self].sort((a, b) => b[1] - a[1]).slice(0, 10)) lines.push(`  ${((100 * n) / total).toFixed(1).padStart(5)}%  ${label}`);
  }
  console.log(lines.join('\n'));
  if (out) writeFileSync(out, `${lines.join('\n')}\n`);
}
