// FsStore durability: crash states built from the file operations the store
// performed (see support/crash-fs.js) and the platform behaviour it relies on.

import { test } from 'node:test';
import assert from 'node:assert/strict';
import fsp from 'node:fs/promises';
import { mkdtemp, rm } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { join } from 'node:path';

import { FlatDB, FsStore } from '../bridge/node.js';
import { Recorder, materialize, minimalState } from './support/crash-fs.js';

// Workload: autocommit statements, each flushed; null = explicit checkpoint.
const WORKLOAD = [
  ['CREATE TABLE kv (id INT PRIMARY KEY, v TEXT)', (m) => m],
  ["INSERT INTO kv VALUES (1, 'first row'), (2, 'second row')", (m) => m.set(1, 'first row').set(2, 'second row')],
  ["INSERT INTO kv VALUES (3, 'third row')", (m) => m.set(3, 'third row')],
  null,
  ["UPDATE kv SET v = 'second row, updated' WHERE id = 2", (m) => m.set(2, 'second row, updated')],
  ['DELETE FROM kv WHERE id = 1', (m) => (m.delete(1), m)],
  [
    "BEGIN; INSERT INTO kv VALUES (4, 'fourth row in a transaction'); DELETE FROM kv WHERE id = 3; COMMIT",
    (m) => (m.set(4, 'fourth row in a transaction').delete(3), m),
  ],
  ["INSERT INTO kv VALUES (5, 'fifth row'), (6, 'sixth row'), (7, 'seventh row')", (m) => m.set(5, 'fifth row').set(6, 'sixth row').set(7, 'seventh row')],
  ['DELETE FROM kv WHERE id = 5', (m) => (m.delete(5), m)],
  ["INSERT INTO kv VALUES (1, 'first row again')", (m) => m.set(1, 'first row again')],
];

const show = (m) => (m === null ? 'no table' : JSON.stringify([...m].sort((a, b) => a[0] - b[0])));

/** Runs the workload on an FsStore whose file operations are recorded. */
async function recordWorkload(rec, { checkpointBytes, storeOptions } = {}) {
  const db = await FlatDB.open({ store: new FsStore(rec.dir, storeOptions), checkpointBytes, flush: 'manual' });
  const run = { states: [null], issued: [], durable: [] };
  let model = null;
  for (const step of WORKLOAD) {
    if (step === null) {
      await db.checkpoint();
      continue;
    }
    const [sql, apply] = step;
    run.issued.push(rec.trace.length);
    db.exec(sql);
    await db.flush();
    // A client keeps its sync cursor next to the data.
    await db.store.setMeta('sync', { cursor: run.durable.length });
    run.durable.push(rec.trace.length);
    model = apply(new Map(model ?? []));
    run.states.push(model);
  }
  await db.close();
  run.trace = rec.trace.slice();
  return run;
}

/** Opens the database left in `files` and returns its rows (null: no table). */
async function recover(files, dir) {
  await materialize(files, dir);
  try {
    const db = await FlatDB.open({ store: new FsStore(dir) });
    const rows = db.tableNames().includes('kv') ? new Map(db.exec('SELECT id, v FROM kv ORDER BY id').rows) : null;
    await db.close();
    return rows;
  } finally {
    await rm(dir, { recursive: true, force: true });
  }
}

/** The recovered rows equal the state after j transactions, durable <= j <= issued. */
async function check(run, crash, files, dir) {
  const durable = run.durable.filter((d) => d <= crash).length;
  const issued = run.issued.filter((s) => s < crash).length;
  let got;
  try {
    got = await recover(files, dir);
  } catch (err) {
    return `recovery failed: ${err.message}`;
  }
  for (let j = durable; j <= issued; j++) {
    const want = run.states[j];
    if (want === null ? got === null : got !== null && show(got) === show(want)) return null;
  }
  return `got ${show(got)}, expected the state after ${durable}..${issued} transactions`;
}

test('FsStore: flushed transactions survive a crash when only synced writes persist', async (t) => {
  const root = await mkdtemp(join(tmpdir(), 'flatsql-crash-'));
  const rec = new Recorder(join(root, 'db'));
  rec.install();
  try {
    rec.recording = true;
    const run = await recordWorkload(rec, { checkpointBytes: 300 });
    rec.recording = false;
    const checkpoints = run.trace.filter((op) => op.op === 'rename' && op.to === 'snapshot.fsql').length;
    t.diagnostic(`${run.trace.length + 1} crash points, ${checkpoints} checkpoints`);
    for (let crash = 0; crash <= run.trace.length; crash++) {
      const err = await check(run, crash, minimalState(run.trace, crash), join(root, 'recovered'));
      assert.equal(err, null, `crash after ${crash} of ${run.trace.length} operations (${JSON.stringify(run.trace[crash - 1]?.op)}): ${err}`);
    }
  } finally {
    rec.uninstall();
    await rm(root, { recursive: true, force: true });
  }
});

test('FsStore: fsync can be turned off for WAL appends only', async () => {
  const root = await mkdtemp(join(tmpdir(), 'flatsql-nofsync-'));
  const rec = new Recorder(join(root, 'db'));
  rec.install();
  try {
    rec.recording = true;
    const store = new FsStore(rec.dir, { fsync: false });
    assert.equal(store.fsync, false);
    const db = await FlatDB.open({ store, flush: 'manual' });
    db.exec('CREATE TABLE t (id INT PRIMARY KEY)');
    await db.flush();
    const afterAppend = rec.trace.length;
    await db.checkpoint();
    rec.recording = false;
    const appendOps = rec.trace.slice(0, afterAppend).filter((op) => op.op === 'sync');
    assert.equal(appendOps.length, 0, 'no fsync after a WAL append');
    const checkpointOps = rec.trace.slice(afterAppend).map((op) => op.op);
    assert.ok(checkpointOps.includes('sync') && checkpointOps.includes('syncDir'), `checkpoint is synced: ${checkpointOps}`);
    await db.close();
  } finally {
    rec.uninstall();
    await rm(root, { recursive: true, force: true });
  }
});

test('platform: a directory can be synced with the open flags FsStore uses', async () => {
  const dir = await mkdtemp(join(tmpdir(), 'flatsql-dirsync-'));
  try {
    const flags = process.platform === 'win32' ? 'r+' : 'r';
    const fh = await fsp.open(dir, flags);
    await fh.sync();
    await fh.close();
    if (process.platform === 'win32') {
      // Read-only handle: FlushFileBuffers needs write access (EPERM).
      const ro = await fsp.open(dir, 'r');
      await assert.rejects(ro.sync(), (e) => e.code === 'EPERM');
      await ro.close();
      // Flag 'w' (create/truncate) is refused for a directory.
      await assert.rejects(fsp.open(dir, 'w'), (e) => e.code === 'EISDIR');
    }
  } finally {
    await rm(dir, { recursive: true, force: true });
  }
});
