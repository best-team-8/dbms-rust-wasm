// Where the time of a primary-key lookup goes at the JavaScript/WebAssembly
// boundary, for the version of Flat-SQL in DIR (Node.js, V8). Two methods:
//
// 1. Variants of the call that stop at different points, each timed over
//    200 000 lookups on the benchmark table (100 000 rows), 7 rounds in
//    alternating order, median of the rounds:
//      call    db.walPending()               an export without arguments
//      execute db.execute(q, [k])            parameters, execution, result
//                                            built in Rust, only a number back
//      run     db.run(q, [k])                result as JS objects
//      json    db.runJson(q, [k])            result serialised to a JSON string
//      json+p  JSON.parse(db.runJson(q,[k])) the JSON path as used
// 2. The V8 sampling profiler over the `run` and `json+p` loops, samples
//    grouped into: UTF-8 decoding, glue code (generated JS and the
//    WebAssembly↔JS transitions), the engine (WebAssembly functions),
//    garbage collection, the measuring loop and the rest.
//
//   node docs/research/data/optimization/10-boundary-profile/boundary.mjs DIR [--out FILE]

import { spawnSync } from 'node:child_process';
import { mkdtempSync, readdirSync, readFileSync, rmSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join, resolve } from 'node:path';
import { fileURLToPath, pathToFileURL } from 'node:url';

const HERE = fileURLToPath(import.meta.url);
const SUITE = pathToFileURL(resolve(fileURLToPath(new URL('.', import.meta.url)), '../../../../../bench/suite.js')).href;
const N = 200_000;

async function table(root) {
  const { Database } = await import(pathToFileURL(join(root, 'bridge', 'node.js')).href);
  const { SCHEMA, INDEX, INSERT, makeRow, Rng } = await import(SUITE);
  const db = new Database(1);
  db.exec(SCHEMA);
  db.exec(INDEX);
  const st = db.prepare(INSERT);
  db.exec('BEGIN');
  for (let i = 0; i < 100_000; i++) db.execute(st, makeRow(i));
  db.exec('COMMIT');
  st.free();
  db.takeWal();
  const rng = new Rng(1);
  const ids = Array.from({ length: N }, () => [rng.below(100_000)]);
  return { db, q: db.prepare('SELECT * FROM items WHERE id = ?'), ids };
}

const variants = (db, q) => ({
  call: () => db.walPending(),
  execute: (p) => db.execute(q, p),
  run: (p) => db.run(q, p),
  json: (p) => db.runJson(q, p),
  'json+p': (p) => JSON.parse(db.runJson(q, p)),
});

if (process.argv[2] === '--profile') {
  // Child process under --cpu-prof: one variant in a loop.
  const [, , , name, root] = process.argv;
  const { db, q, ids } = await table(root);
  const f = variants(db, q)[name];
  for (let r = 0; r < 3; r++) for (const p of ids) f(p);
} else {
  const root = resolve(process.argv[2] ?? '.');
  const out = process.argv.includes('--out') ? process.argv[process.argv.indexOf('--out') + 1] : null;
  const lines = [`Flat-SQL: ${root}`, '', 'variant | median of 7 rounds, µs per lookup | rounds'];
  const { db, q, ids } = await table(root);
  const fs = variants(db, q);
  const names = Object.keys(fs);
  const times = Object.fromEntries(names.map((n) => [n, []]));
  for (const p of ids.slice(0, 20_000)) for (const n of names) fs[n](p); // warm-up
  for (let round = 0; round < 7; round++) {
    const order = round % 2 ? [...names].reverse() : names;
    for (const n of order) {
      const f = fs[n];
      const t0 = performance.now();
      for (const p of ids) f(p);
      times[n].push(((performance.now() - t0) * 1000) / N);
    }
  }
  const median = (v) => [...v].sort((a, b) => a - b)[3];
  for (const n of names) lines.push(`${n} | ${median(times[n]).toFixed(2)} | ${times[n].map((t) => t.toFixed(2)).join(' ')}`);

  const groups = [
    ['UTF-8 decoding', (f) => /decodeUTF8|decodeText|^decode /.test(f)],
    ['glue code and transitions', (f) => /flatsql\.js|wasm-to-js|js-to-wasm/.test(f)],
    ['engine (WebAssembly)', (f) => /^wasm-function/.test(f)],
    ['garbage collection', (f) => /garbage collector/.test(f)],
    ['JSON.parse', (f) => /^parse |JsonParse|JSON/.test(f)],
    ['measuring loop', (f) => /boundary\.mjs/.test(f)],
  ];
  for (const name of ['run', 'json+p']) {
    const dir = mkdtempSync(join(tmpdir(), 'flatsql-prof-'));
    spawnSync(process.execPath, ['--cpu-prof', `--cpu-prof-dir=${dir}`, HERE, '--profile', name, root], { encoding: 'utf8' });
    const profile = JSON.parse(readFileSync(join(dir, readdirSync(dir)[0]), 'utf8'));
    rmSync(dir, { recursive: true, force: true });
    const nodes = new Map(profile.nodes.map((n) => [n.id, n]));
    const shares = new Map(groups.map(([g]) => [g, 0]));
    shares.set('other', 0);
    const self = new Map();
    // Only samples after the table was loaded: the lookups.
    for (const id of profile.samples) {
      const f = nodes.get(id).callFrame;
      const label = `${f.functionName || '(anonymous)'} ${f.url.split('/').pop()}`.trim();
      self.set(label, (self.get(label) ?? 0) + 1);
      const g = groups.find(([, test]) => test(label));
      shares.set(g ? g[0] : 'other', shares.get(g ? g[0] : 'other') + 1);
    }
    const total = profile.samples.length;
    lines.push('', `profile of \`${name}\` (${total} samples, includes loading the table)`);
    for (const [g, n] of shares) lines.push(`  ${((100 * n) / total).toFixed(1).padStart(5)}%  ${g}`);
    lines.push('  top functions:');
    for (const [label, n] of [...self].sort((a, b) => b[1] - a[1]).slice(0, 8)) lines.push(`  ${((100 * n) / total).toFixed(1).padStart(5)}%  ${label}`);
  }
  console.log(lines.join('\n'));
  if (out) writeFileSync(out, `${lines.join('\n')}\n`);
}
