// WASM-Bridge: FlatDB on top of the stores, durability and recovery.

import { test } from 'node:test';
import assert from 'node:assert/strict';
import { mkdtemp, rm, appendFile } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { join } from 'node:path';

import { FlatDB, FsStore, MemoryStore, Database, version } from '../bridge/node.js';

test('engine version is exposed', () => {
  assert.match(version(), /^\d+\.\d+\.\d+$/);
});

test('exec, query, prepared statements and JSON results', async () => {
  const db = await FlatDB.open({ store: new MemoryStore() });
  db.exec('CREATE TABLE heroes (id INT PRIMARY KEY, name TEXT, level INT, gold FLOAT)');
  db.exec("INSERT INTO heroes VALUES (1, 'Ayla', 12, 99.5), (2, 'Borin', 7, 10.0), (3, 'Cyra', 20, NULL)");

  const r = db.exec('SELECT name FROM heroes WHERE level > 10 ORDER BY level DESC');
  assert.deepEqual(r.columns, ['name']);
  assert.deepEqual(r.rows, [['Cyra'], ['Ayla']]);

  assert.deepEqual(db.query('SELECT id, gold FROM heroes WHERE id = ?', [3]), [{ id: 3, gold: null }]);

  const json = JSON.parse(db.execJson('SELECT COUNT(*) AS n, AVG(level) AS avg FROM heroes'));
  assert.deepEqual(json.columns, ['n', 'avg']);
  assert.deepEqual(json.rows, [[3, 13]]);

  const stmt = db.prepare('UPDATE heroes SET gold = ? WHERE id = ?');
  assert.equal(stmt.paramCount, 2);
  assert.equal(db.run(stmt, [5.25, 2]).rowsAffected, 1);
  assert.equal(db.query('SELECT gold FROM heroes WHERE id = 2')[0].gold, 5.25);
  await db.close();
});

test('errors carry a machine readable kind', async () => {
  const db = await FlatDB.open({ store: new MemoryStore() });
  db.exec('CREATE TABLE t (id INT PRIMARY KEY)');
  assert.throws(() => db.exec('SELECT * FROM missing'), (e) => e.kind === 'schema');
  assert.throws(() => db.exec('SELEC 1'), (e) => e.kind === 'parse');
  assert.throws(() => db.exec('INSERT INTO t VALUES (1), (1)'), (e) => e.kind === 'constraint');
  assert.throws(() => db.exec('SELECT * FROM a JOIN b'), (e) => e.kind === 'unsupported');
  assert.equal(db.query('SELECT COUNT(*) AS n FROM t')[0].n, 0, 'failed statement left no rows');
  await db.close();
});

test('writes are group-committed and survive a reopen (memory store)', async () => {
  const store = new MemoryStore();
  const db = await FlatDB.open({ store });
  db.exec('CREATE TABLE log (id INT PRIMARY KEY, msg TEXT)');
  for (let i = 0; i < 100; i++) db.exec(`INSERT INTO log VALUES (${i}, 'm${i}')`);
  await db.flush();
  assert.equal(store.walChunks.length, 1, '101 commits flushed in a single store write');
  const nodeId = db.nodeId;
  await db.close();

  const again = await FlatDB.open({ store });
  assert.equal(again.nodeId, nodeId, 'node id is remembered');
  assert.equal(again.query('SELECT COUNT(*) AS n FROM log')[0].n, 100);
  assert.equal(again.recovery.framesReplayed, 101);
  await again.close();
});

test('automatic checkpoint truncates the WAL', async () => {
  const store = new MemoryStore();
  const db = await FlatDB.open({ store, checkpointBytes: 4096 });
  let checkpoints = 0;
  db.addEventListener('checkpoint', () => checkpoints++);
  db.exec('CREATE TABLE t (id INT PRIMARY KEY, payload TEXT)');
  for (let i = 0; i < 300; i++) {
    db.exec(`INSERT INTO t VALUES (${i}, '${'x'.repeat(50)}')`);
    if (i % 50 === 0) await db.flush();
  }
  await db.flush();
  assert.ok(checkpoints > 0);
  assert.ok(store.snapshot instanceof Uint8Array);
  assert.ok(store.walSize < 4096 + 2048);
  await db.close();
  const again = await FlatDB.open({ store });
  assert.equal(again.query('SELECT COUNT(*) AS n FROM t')[0].n, 300);
});

test('transactions are flushed only on commit', async () => {
  const store = new MemoryStore();
  const db = await FlatDB.open({ store });
  db.exec('CREATE TABLE t (id INT PRIMARY KEY)');
  await db.flush();
  const before = store.walSize;
  db.exec('BEGIN');
  db.exec('INSERT INTO t VALUES (1), (2)');
  await db.flush();
  assert.equal(store.walSize, before, 'nothing durable before COMMIT');
  db.exec('COMMIT');
  await db.flush();
  assert.ok(store.walSize > before);
  db.exec('BEGIN; DELETE FROM t; ROLLBACK;');
  assert.equal(db.query('SELECT COUNT(*) AS n FROM t')[0].n, 2);
  await db.close();
});

test('file store: crash with a torn WAL tail is recovered', async () => {
  const dir = await mkdtemp(join(tmpdir(), 'flatsql-'));
  try {
    const db = await FlatDB.open({ store: new FsStore(dir) });
    db.exec('CREATE TABLE notes (id INT PRIMARY KEY, body TEXT)');
    db.exec("INSERT INTO notes VALUES (1, 'kept'), (2, 'kept too')");
    await db.flush();
    // The process "dies" here – no close(). Then half of a frame is appended.
    await appendFile(join(dir, 'wal.fsql'), new Uint8Array([0x40, 0, 0, 0, 1, 2, 3]));

    const reopened = await FlatDB.open({ store: new FsStore(dir) });
    assert.equal(reopened.recovery.tornBytes, 7);
    assert.equal(reopened.query('SELECT COUNT(*) AS n FROM notes')[0].n, 2);
    reopened.exec("INSERT INTO notes VALUES (3, 'after recovery')");
    await reopened.close();

    const third = await FlatDB.open({ store: new FsStore(dir) });
    assert.equal(third.recovery.tornBytes, 0, 'torn tail was cleaned up by a checkpoint');
    assert.equal(third.query('SELECT COUNT(*) AS n FROM notes')[0].n, 3);
    await third.close();
  } finally {
    await rm(dir, { recursive: true, force: true });
  }
});

test('low level Database API: snapshot + WAL restore', () => {
  const db = new Database(5);
  db.exec('CREATE TABLE t (k TEXT PRIMARY KEY, v INT)');
  db.exec("INSERT INTO t VALUES ('a', 1)");
  const snap = db.snapshot();
  db.exec("INSERT INTO t VALUES ('b', 2)");
  const wal = db.takeWal();
  const restored = Database.restore(snap, wal, 0);
  assert.equal(restored.nodeId, 5);
  assert.deepEqual(restored.exec('SELECT k FROM t ORDER BY k').rows, [['a'], ['b']]);
  const report = restored.recoveryReport();
  assert.equal(report.framesSkipped, 2);
  assert.equal(report.framesReplayed, 1);
  db.free();
  restored.free();
});
