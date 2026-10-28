// Main-thread work of one checkpoint of the benchmark table (100 000 rows)
// in Node.js, split into the calls a checkpoint makes: snapshotBegin, every
// snapshotStep (longest step and all of them), snapshotFinish (CRC and the
// copy of the bytes to JavaScript), and the one-call snapshot for comparison.
// Median of 7 rounds.
//
//   node docs/research/data/optimization/15-checkpoint-steps/stages.mjs [DIR] [--rows N]

import { join, resolve } from 'node:path';
import { fileURLToPath, pathToFileURL } from 'node:url';

const args = process.argv.slice(2);
const root = resolve(args.find((a, i) => !a.startsWith('--') && args[i - 1] !== '--rows') ?? '.');
const stepRows = args.includes('--rows') ? Number(args[args.indexOf('--rows') + 1]) : 4096;
const { Database } = await import(pathToFileURL(join(root, 'bridge', 'node.js')).href);
const suite = await import(pathToFileURL(resolve(fileURLToPath(new URL('.', import.meta.url)), '../../../../../bench/suite.js')).href);

const db = new Database(1);
db.exec(suite.SCHEMA);
db.exec(suite.INDEX);
const ins = db.prepare(suite.INSERT);
db.exec('BEGIN');
for (let i = 0; i < 100_000; i++) db.execute(ins, suite.makeRow(i));
db.exec('COMMIT');
db.takeWal();

const rounds = { snapshot: [], begin: [], maxStep: [], steps: [], finish: [], count: [] };
for (let r = 0; r < 7; r++) {
  let t = performance.now();
  const whole = db.snapshot();
  rounds.snapshot.push(performance.now() - t);
  if (typeof db.snapshotBegin !== 'function') continue;
  t = performance.now();
  db.snapshotBegin();
  rounds.begin.push(performance.now() - t);
  let max = 0;
  let sum = 0;
  let n = 0;
  for (;;) {
    t = performance.now();
    const done = db.snapshotStep(stepRows);
    const d = performance.now() - t;
    max = Math.max(max, d);
    sum += d;
    n++;
    if (done) break;
  }
  rounds.maxStep.push(max);
  rounds.steps.push(sum);
  rounds.count.push(n);
  t = performance.now();
  const bytes = db.snapshotFinish();
  rounds.finish.push(performance.now() - t);
  if (bytes.length !== whole.length) throw new Error('different snapshot');
}
const median = (v) => [...v].sort((a, b) => a - b)[Math.floor(v.length / 2)];
console.log(`Flat-SQL: ${root}, ${stepRows} rows per step`);
for (const [k, v] of Object.entries(rounds)) if (v.length) console.log(`${k.padEnd(9)} ${median(v).toFixed(2)}${k === 'count' ? '' : ' ms'}`);
