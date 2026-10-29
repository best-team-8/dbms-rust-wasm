// WebAssembly linear memory of Flat-SQL while 100 000 rows of the benchmark
// table are loaded: inside one transaction (every 25 000 rows, before and
// after COMMIT), in transactions of 10 000 and 1000 rows, and after a restore
// from a snapshot. Each variant runs twice, in processes of their own; linear
// memory grows in whole pages, so both runs must give the same sizes.
//
//   node docs/research/data/tests/memory-peak/memory-peak.mjs [FLATSQL_DIR]

import { spawnSync } from 'node:child_process';
import { join, resolve } from 'node:path';
import { fileURLToPath, pathToFileURL } from 'node:url';

const REPO = fileURLToPath(new URL('../../../../../', import.meta.url));
const ROWS = 100_000;
const VARIANTS = ['one-transaction', 'transactions-of-10000', 'transactions-of-1000', 'restored-from-snapshot'];

if (process.argv[2] !== '--variant') {
  const flatsql = resolve(process.argv[2] ?? REPO);
  for (const variant of VARIANTS) {
    const runs = [1, 2].map(() => spawnSync(process.execPath, [fileURLToPath(import.meta.url), '--variant', variant, flatsql], { encoding: 'utf8' }));
    const out = runs.map((r) => r.stdout || r.stderr);
    process.stdout.write(out[0]);
    if (out[0] !== out[1]) process.stdout.write(`  second run differs: ${out[1]}`);
  }
} else {
  const [, , , variant, flatsql] = process.argv;
  const bridge = pathToFileURL(join(flatsql, 'bridge', 'node.js')).href;
  const suite = pathToFileURL(join(REPO, 'bench', 'suite.js')).href;
  const { Database, engineReady } = await import(bridge);
  const { SCHEMA, INDEX, INSERT, makeRow } = await import(suite);
  const memory = (await engineReady()).memory;
  const mib = () => (memory.buffer.byteLength / 2 ** 20).toFixed(1);
  const marks = [];
  const load = (db, batch) => {
    const st = db.prepare(INSERT);
    for (let start = 0; start < ROWS; start += batch) {
      db.exec('BEGIN');
      for (let i = start; i < start + batch; i++) {
        db.execute(st, makeRow(i));
        if (batch === ROWS && (i + 1) % 25_000 === 0) marks.push(`${i + 1} rows ${mib()}`);
      }
      db.exec('COMMIT');
      if (batch === ROWS) marks.push(`after COMMIT ${mib()}`);
      const wal = db.takeWal();
      if (batch === ROWS) marks.push(`WAL of the transaction ${(wal.length / 2 ** 20).toFixed(1)}`);
    }
    st.free();
  };
  let db = new Database(1);
  db.exec(SCHEMA);
  db.exec(INDEX);
  db.takeWal();
  const before = mib();
  if (variant === 'one-transaction') load(db, ROWS);
  if (variant === 'transactions-of-10000') load(db, 10_000);
  if (variant === 'transactions-of-1000') load(db, 1000);
  if (variant === 'restored-from-snapshot') {
    // The snapshot is made in a separate process, so this process holds only
    // the restored database.
    const r = spawnSync(process.execPath, ['--input-type=module', '-e', `
      const { Database } = await import(${JSON.stringify(bridge)});
      const { SCHEMA, INDEX, INSERT, makeRow } = await import(${JSON.stringify(suite)});
      const db = new Database(1); db.exec(SCHEMA); db.exec(INDEX);
      const st = db.prepare(INSERT);
      for (let s = 0; s < ${ROWS}; s += 1000) { db.exec('BEGIN'); for (let i = s; i < s + 1000; i++) db.execute(st, makeRow(i)); db.exec('COMMIT'); db.takeWal(); }
      process.stdout.write(Buffer.from(db.snapshot()).toString('base64'));
    `], { encoding: 'utf8', maxBuffer: 1 << 28 });
    const snap = Buffer.from(r.stdout, 'base64');
    db.free();
    db = Database.restore(new Uint8Array(snap), undefined, 1);
    marks.push(`snapshot ${(snap.length / 2 ** 20).toFixed(1)}`);
  }
  const rows = db.stats().rows;
  console.log(`${variant}: ${before} -> ${mib()} MiB, ${rows} rows${marks.length ? ` (${marks.join(', ')})` : ''}`);
}
