// FsStore durability: crash states built from the file operations the store
// performed on a simulated file system (support/crash-fs.js) and the platform
// behaviour it relies on.

import { mock, test } from 'node:test';
import assert from 'node:assert/strict';
import fsp from 'node:fs/promises';
import { mkdtemp, rm } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { join } from 'node:path';

import { FlatDB, FsStore } from '../bridge/node.js';
import { SimFs, contentKey, crashState, forEachCrashState, minimalState, rng } from './support/crash-fs.js';

// Simulated directories never touch the disk; the paths only have to be unique.
const DB = join(tmpdir(), `flatsql-sim-${process.pid}`, 'db');
const RECOVERED = join(tmpdir(), `flatsql-sim-${process.pid}`, 'recovered');

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

/** Runs the workload on an FsStore over a simulated directory and records its operations. */
async function recordWorkload({ checkpointBytes, fsync = true } = {}) {
  const sim = new SimFs(DB).install();
  const run = { states: [null], issued: [], durable: [] };
  try {
    const db = await FlatDB.open({ store: new FsStore(DB, { fsync }), checkpointBytes, flush: 'manual' });
    let model = null;
    for (const step of WORKLOAD) {
      if (step === null) {
        await db.checkpoint();
        continue;
      }
      const [sql, apply] = step;
      run.issued.push(sim.trace.length);
      db.exec(sql);
      await db.flush();
      // A client keeps its sync cursor next to the data.
      await db.store.setMeta('sync', { cursor: run.durable.length });
      run.durable.push(sim.trace.length);
      model = apply(new Map(model ?? []));
      run.states.push(model);
    }
    await db.close();
  } finally {
    sim.uninstall();
  }
  run.trace = sim.trace;
  if (!fsync) {
    // Without fsync a transaction is durable once a checkpoint that contains
    // it has synced the directory after renaming the snapshot.
    const synced = [];
    let renamed = false;
    run.trace.forEach((op, i) => {
      if (op.op === 'rename' && op.to === 'snapshot.fsql') renamed = true;
      if (op.op === 'syncDir') {
        if (renamed) synced.push(i + 1);
        renamed = false;
      }
    });
    run.durable = run.issued.map((s) => synced.find((c) => c > s) ?? Infinity);
  }
  return run;
}

const rowsOf = (db) => (db.tableNames().includes('kv') ? new Map(db.exec('SELECT id, v FROM kv ORDER BY id').rows) : null);

/** Opens a simulated directory with these files, runs fn(db) and returns { result, files }. */
async function withDatabase(files, fn) {
  const sim = new SimFs(RECOVERED, files).install();
  try {
    const db = await FlatDB.open({ store: new FsStore(RECOVERED) });
    const result = fn(db);
    await db.close();
    return { result, files: sim.files() };
  } finally {
    sim.uninstall();
  }
}

/**
 * Opens the database left in `files` and returns { rows } (rows null: no
 * table) or { error }. The recovered database must also keep a new write: a
 * torn WAL tail left in front of it would hide it on the next open.
 */
async function recover(files) {
  try {
    const first = await withDatabase(files, (db) => {
      const rows = rowsOf(db);
      if (rows !== null) db.exec("INSERT OR REPLACE INTO kv VALUES (100, 'after recovery')");
      return rows;
    });
    const rows = first.result;
    if (rows !== null) {
      const { result: after } = await withDatabase(first.files, rowsOf);
      if (show(after) !== show(new Map(rows).set(100, 'after recovery'))) {
        throw new Error('a write made after recovery was lost on the next open');
      }
    }
    return { rows };
  } catch (err) {
    return { error: err.message };
  }
}

/** The recovered rows equal the state after j transactions, durable <= j <= issued. */
function check(run, crash, recovered) {
  if (recovered.error) return `recovery failed: ${recovered.error}`;
  const durable = run.durable.filter((d) => d <= crash).length;
  const issued = run.issued.filter((s) => s < crash).length;
  const got = recovered.rows;
  for (let j = durable; j <= issued; j++) {
    const want = run.states[j];
    if (want === null ? got === null : got !== null && show(got) === show(want)) return null;
  }
  return `got ${show(got)}, expected the state after ${durable}..${issued} transactions`;
}

const describe = (run, crash) => `crash after ${crash} of ${run.trace.length} operations (${JSON.stringify(run.trace[crash - 1]?.op)})`;

test('FsStore: flushed transactions survive a crash when only synced writes persist', async (t) => {
  const run = await recordWorkload({ checkpointBytes: 300 });
  const checkpoints = run.trace.filter((op) => op.op === 'rename' && op.to === 'snapshot.fsql').length;
  t.diagnostic(`${run.trace.length + 1} crash points, ${checkpoints} checkpoints`);
  for (let crash = 0; crash <= run.trace.length; crash++) {
    const err = check(run, crash, await recover(minimalState(run.trace, crash)));
    assert.equal(err, null, `${describe(run, crash)}: ${err}`);
  }
});

// With fsync off the number of crash states explodes (unsynced appends and
// meta.json rewrites): at most this many are drawn per crash point;
// FLATSQL_CRASH_STATES raises it for a full manual run.
const SAMPLED_STATES = Number(process.env.FLATSQL_CRASH_STATES ?? 64);

test('FsStore: every permitted crash state recovers to a committed state', async (t) => {
  for (const fsync of [true, false]) {
    const run = await recordWorkload({ checkpointBytes: 300, fsync });
    const cache = new Map();
    const random = rng(fsync ? 1 : 2);
    const max = fsync ? 4096 : SAMPLED_STATES;
    let [total, examined, sampled] = [0, 0, 0];
    for (let crash = 0; crash <= run.trace.length; crash++) {
      let failure = null;
      const r = await forEachCrashState(run.trace, crash, { max, random }, async (persisted) => {
        examined++;
        const files = crashState(run.trace.slice(0, crash), persisted);
        const key = contentKey(files);
        if (!cache.has(key)) cache.set(key, await recover(files));
        failure ??= check(run, crash, cache.get(key));
      });
      total = Math.min(total + r.total, Number.MAX_SAFE_INTEGER);
      if (r.sampled) sampled++;
      assert.equal(failure, null, `fsync ${fsync}, ${describe(run, crash)}: ${failure}`);
    }
    t.diagnostic(
      `fsync ${fsync}: ${run.trace.length + 1} crash points, ${total} crash states, ${examined} examined ` +
        `(${sampled} points sampled), ${cache.size} distinct directory contents`,
    );
  }
});

test('FsStore: fsync can be turned off for WAL appends only', async () => {
  const sim = new SimFs(DB).install();
  try {
    const store = new FsStore(DB, { fsync: false });
    assert.equal(store.fsync, false);
    const db = await FlatDB.open({ store, flush: 'manual' });
    db.exec('CREATE TABLE t (id INT PRIMARY KEY)');
    await db.flush();
    const afterAppend = sim.trace.length;
    await db.checkpoint();
    const appendOps = sim.trace.slice(0, afterAppend).filter((op) => op.op === 'sync');
    assert.equal(appendOps.length, 0, 'no fsync after a WAL append');
    const checkpointOps = sim.trace.slice(afterAppend).map((op) => op.op);
    assert.ok(checkpointOps.includes('sync') && checkpointOps.includes('syncDir'), `checkpoint is synced: ${checkpointOps}`);
    await db.close();
  } finally {
    sim.uninstall();
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

test('FsStore keeps the WAL open between checkpoints and closes it', async () => {
  const dir = await mkdtemp(join(tmpdir(), 'flatsql-handle-'));
  const opens = [];
  const original = fsp.open;
  const spy = mock.method(fsp, 'open', async (path, ...args) => {
    if (String(path).endsWith('wal.fsql')) opens.push(args[0]);
    return original.call(fsp, path, ...args);
  });
  try {
    const db = await FlatDB.open({ store: new FsStore(dir), flush: 'manual' });
    opens.length = 0; // opening a new directory creates the WAL file
    db.exec('CREATE TABLE kv (id INT PRIMARY KEY, v TEXT)');
    for (let i = 0; i < 50; i++) {
      db.exec(`INSERT INTO kv VALUES (${i}, 'v${i}')`);
      await db.flush();
    }
    assert.deepEqual(opens, ['a'], 'one handle for 50 appends');
    await db.checkpoint();
    for (let i = 50; i < 60; i++) {
      db.exec(`INSERT INTO kv VALUES (${i}, 'v${i}')`);
      await db.flush();
    }
    assert.deepEqual(opens, ['a', 'w', 'a'], 'closed for the truncation, opened again after it');
    await db.close();
    const again = await FlatDB.open({ store: new FsStore(dir) });
    assert.equal(again.exec('SELECT COUNT(*) FROM kv').rows[0][0], 60);
    await again.close();
  } finally {
    spy.mock.restore();
    // Fails on Windows if a handle was left open.
    await rm(dir, { recursive: true });
  }
});
