// WebAssembly linear memory of Flat-SQL with 100 000 rows, depending on how
// the rows were loaded, and the size of the same data in a snapshot.
// Each variant runs in a process of its own.
//
//   node docs/research/data/benchmark-1/analysis/memory.mjs [FLATSQL_DIR]

import { spawnSync } from 'node:child_process';
import { join, resolve } from 'node:path';
import { fileURLToPath, pathToFileURL } from 'node:url';

const REPO = fileURLToPath(new URL('../../../../../', import.meta.url));
const ROWS = 100_000;

if (process.argv[2] !== '--variant') {
  const flatsql = resolve(process.argv[2] ?? REPO);
  for (const variant of ['one-transaction', 'transactions-of-1000', 'restored-from-snapshot']) {
    const r = spawnSync(process.execPath, [fileURLToPath(import.meta.url), '--variant', variant, flatsql], { encoding: 'utf8' });
    process.stdout.write(r.stdout || r.stderr);
  }
} else {
  const [, , , variant, flatsql] = process.argv;
  const { Database, engineReady } = await import(pathToFileURL(join(flatsql, 'bridge', 'node.js')).href);
  const { SCHEMA, INDEX, INSERT, makeRow } = await import(pathToFileURL(join(REPO, 'bench', 'suite.js')).href);
  const mib = async () => ((await engineReady()).memory.buffer.byteLength / 2 ** 20).toFixed(1);
  const load = (db, batch) => {
    const st = db.prepare(INSERT);
    for (let start = 0; start < ROWS; start += batch) {
      db.exec('BEGIN');
      for (let i = start; i < Math.min(ROWS, start + batch); i++) db.execute(st, makeRow(i));
      db.exec('COMMIT');
      db.takeWal();
    }
    st.free();
  };
  let db = new Database(1);
  db.exec(SCHEMA);
  db.exec(INDEX);
  const before = await mib();
  let note = '';
  if (variant === 'one-transaction') load(db, ROWS);
  if (variant === 'transactions-of-1000') load(db, 1000);
  if (variant === 'restored-from-snapshot') {
    // The snapshot is made in a separate process, so this process holds only
    // the restored database.
    const r = spawnSync(process.execPath, ['--input-type=module', '-e', `
      const { Database } = await import(${JSON.stringify(pathToFileURL(join(flatsql, 'bridge', 'node.js')).href)});
      const { SCHEMA, INDEX, INSERT, makeRow } = await import(${JSON.stringify(pathToFileURL(join(REPO, 'bench', 'suite.js')).href)});
      const db = new Database(1); db.exec(SCHEMA); db.exec(INDEX);
      const st = db.prepare(INSERT);
      for (let s = 0; s < ${ROWS}; s += 1000) { db.exec('BEGIN'); for (let i = s; i < s + 1000; i++) db.execute(st, makeRow(i)); db.exec('COMMIT'); db.takeWal(); }
      process.stdout.write(Buffer.from(db.snapshot()).toString('base64'));
    `], { encoding: 'utf8', maxBuffer: 1 << 28 });
    const snap = Buffer.from(r.stdout, 'base64');
    db.free();
    db = Database.restore(new Uint8Array(snap), undefined, 1);
    note = `, snapshot ${(snap.length / 2 ** 20).toFixed(1)} MiB`;
  }
  const s = db.stats();
  console.log(`${variant}: linear memory ${before} MiB -> ${await mib()} MiB for ${s.rows} rows, outbox ${s.outbox} entries${note}`);
}
