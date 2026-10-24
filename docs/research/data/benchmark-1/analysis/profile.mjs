// CPU profile (V8 sampling profiler) of 200 000 primary-key lookups on
// 100 000 rows in Node.js, results as JS objects (`run`) and as JSON
// (`runJson`). Prints the functions with the largest share of samples
// (self time). WebAssembly functions have no names in the release build.
//
//   node docs/research/data/benchmark-1/analysis/profile.mjs [FLATSQL_DIR]

import { spawnSync } from 'node:child_process';
import { mkdtempSync, readdirSync, readFileSync, rmSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join, resolve } from 'node:path';
import { fileURLToPath, pathToFileURL } from 'node:url';

const REPO = fileURLToPath(new URL('../../../../../', import.meta.url));

if (process.argv[2] !== '--run') {
  const flatsql = resolve(process.argv[2] ?? REPO);
  for (const mode of ['objects', 'json']) {
    const dir = mkdtempSync(join(tmpdir(), 'flatsql-prof-'));
    const r = spawnSync(process.execPath, ['--cpu-prof', `--cpu-prof-dir=${dir}`, fileURLToPath(import.meta.url), '--run', mode, flatsql], { encoding: 'utf8' });
    process.stdout.write(r.stdout || r.stderr);
    const profile = JSON.parse(readFileSync(join(dir, readdirSync(dir)[0]), 'utf8'));
    const nodes = new Map(profile.nodes.map((n) => [n.id, n]));
    const self = new Map();
    for (const id of profile.samples) {
      const f = nodes.get(id).callFrame;
      const name = `${f.functionName || '(anonymous)'} ${f.url.split('/').pop()}`.trim();
      self.set(name, (self.get(name) ?? 0) + 1);
    }
    const total = profile.samples.length;
    for (const [name, n] of [...self].sort((a, b) => b[1] - a[1]).slice(0, 12)) console.log(`  ${((100 * n) / total).toFixed(1).padStart(5)}%  ${name}`);
    rmSync(dir, { recursive: true, force: true });
  }
} else {
  const [, , , mode, flatsql] = process.argv;
  const { Database } = await import(pathToFileURL(join(flatsql, 'bridge', 'node.js')).href);
  const { SCHEMA, INDEX, INSERT, makeRow, Rng } = await import(pathToFileURL(join(REPO, 'bench', 'suite.js')).href);
  const db = new Database(1);
  db.exec(SCHEMA);
  db.exec(INDEX);
  const st = db.prepare(INSERT);
  db.exec('BEGIN');
  for (let i = 0; i < 100_000; i++) db.execute(st, makeRow(i));
  db.exec('COMMIT');
  db.takeWal();
  const q = db.prepare('SELECT * FROM items WHERE id = ?');
  const rng = new Rng(1);
  const ids = Array.from({ length: 200_000 }, () => [rng.below(100_000)]);
  const t = performance.now();
  for (const p of ids) {
    if (mode === 'json') JSON.parse(db.runJson(q, p));
    else db.run(q, p);
  }
  console.log(`${mode}: ${(((performance.now() - t) * 1000) / ids.length).toFixed(2)} µs per lookup (with the profiler running)`);
}
