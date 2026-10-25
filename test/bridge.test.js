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

test('statements kept by exec and query follow schema changes and are freed on close', async () => {
  for (const statementCache of [64, 1, 0]) {
    const db = await FlatDB.open({ store: new MemoryStore(), statementCache });
    db.exec('CREATE TABLE t (id INT PRIMARY KEY, v TEXT)');
    db.exec("INSERT INTO t VALUES (1, 'a'), (2, 'b')");
    for (let i = 0; i < 3; i++) {
      assert.deepEqual(db.exec('SELECT v FROM t ORDER BY id').rows, [['a'], ['b']]);
      assert.deepEqual(db.query('SELECT v FROM t WHERE id = ?', [2]), [{ v: 'b' }]);
    }
    // The cached statements are compiled again for the new definition.
    db.exec('DROP TABLE t; CREATE TABLE t (v TEXT, id INT PRIMARY KEY)');
    db.exec("INSERT INTO t VALUES ('x', 2)");
    assert.deepEqual(db.exec('SELECT v FROM t ORDER BY id').rows, [['x']]);
    assert.deepEqual(db.query('SELECT v FROM t WHERE id = ?', [2]), [{ v: 'x' }]);
    // Scripts and errors still go through exec.
    assert.equal(db.exec("INSERT INTO t VALUES ('y', 3); SELECT COUNT(*) FROM t").rows[0][0], 2);
    assert.throws(() => db.exec('SELEC 1'), (e) => e.kind === 'parse');
    assert.throws(() => db.exec('SELEC 1'), (e) => e.kind === 'parse');
    await db.close();
  }
});

test('reads do not look for WAL bytes, writes do', async () => {
  const db = await FlatDB.open({ store: new MemoryStore(), flush: 'manual' });
  db.exec('CREATE TABLE t (id INT PRIMARY KEY, v INT)');
  let changes = 0;
  db.addEventListener('change', () => changes++);
  const read = db.prepare('SELECT v FROM t WHERE id = ?');
  const write = db.prepare('INSERT INTO t VALUES (?, ?)');
  assert.equal(read.readOnly, true);
  assert.equal(write.readOnly, false);
  db.run(write, [1, 10]);
  assert.equal(changes, 1);
  assert.deepEqual(db.run(read, [1]).rows, [[10]]);
  db.query('SELECT v FROM t WHERE id = ?', [1]);
  assert.equal(changes, 1);
  db.query('INSERT INTO t VALUES (?, ?)', [2, 20]);
  assert.equal(changes, 2);
  await db.close();
});

test('results of a prepared statement get their own column names and follow schema changes', () => {
  const db = new Database(1);
  db.exec('CREATE TABLE t (id INT PRIMARY KEY, v TEXT)');
  db.exec("INSERT INTO t VALUES (1, 'a')");
  const st = db.prepare('SELECT * FROM t');
  const first = db.run(st, []);
  first.columns.push('changed by the application');
  const second = db.run(st, []);
  assert.deepEqual(second.columns, ['id', 'v'], 'the names are copied for every result');
  assert.deepEqual(Object.keys(second), ['columns', 'rows', 'rowsAffected', 'rowsScanned', 'message']);
  assert.equal(second.message, '');
  db.exec('DROP TABLE t; CREATE TABLE t (k INT PRIMARY KEY, a TEXT, b TEXT)');
  db.exec("INSERT INTO t VALUES (1, 'x', 'y')");
  assert.deepEqual(db.run(st, []).columns, ['k', 'a', 'b'], 'names of the new definition');
  const insert = db.prepare('INSERT INTO t VALUES (?, ?, ?)');
  assert.equal(db.run(insert, [2, 'p', 'q']).message, 'INSERT 1');
  insert.free();
  st.free();
  db.free();
});

test('values of every type come through the binary result format unchanged', () => {
  const db = new Database(1);
  db.exec('CREATE TABLE v (id INT PRIMARY KEY, t TEXT, f FLOAT, b BOOL, i INT)');
  const rows = [
    [1, '', 0.5, true, 0],
    // The first text of the result starts with U+FEFF, which a decoder
    // could take for a byte order mark.
    [2, '﻿zażółć gęślą jaźń', 0.25, false, -7],
    [3, 'emoji 😀 and 𝄞 outside the basic plane', Number.NaN, null, 9007199254740991],
    [4, null, Number.POSITIVE_INFINITY, true, null],
    [5, 'a\u0000b"\\', Number.NEGATIVE_INFINITY, false, 1],
  ];
  const ins = db.prepare('INSERT INTO v VALUES (?, ?, ?, ?, ?)');
  for (const r of rows) db.run(ins, r);
  const st = db.prepare('SELECT * FROM v ORDER BY id');
  for (const result of [db.run(st, []), db.exec('SELECT * FROM v ORDER BY id')]) {
    assert.equal(result.rows.length, rows.length);
    result.rows.forEach((got, i) => {
      rows[i].forEach((want, j) => assert.ok(Object.is(got[j], want), `row ${i} column ${j}: ${got[j]} != ${want}`));
    });
    assert.deepEqual(result.columns, ['id', 't', 'f', 'b', 'i']);
    assert.equal(result.rowsScanned, 5);
  }
  // A larger result compared with the JSON path.
  db.exec('BEGIN');
  for (let i = 10; i < 3000; i++) db.run(ins, [i, `name ${i} ł`, i / 7, i % 3 === 0, i % 11 === 0 ? null : i]);
  db.exec('COMMIT');
  const many = db.prepare('SELECT * FROM v WHERE id >= 10 ORDER BY id DESC');
  assert.deepEqual(db.run(many, []).rows, JSON.parse(db.runJson(many, [])).rows);
  const update = db.run(db.prepare('UPDATE v SET i = ? WHERE id < ?'), [3, 100]);
  assert.deepEqual([update.rowsAffected, update.message, update.rows], [95, 'UPDATE 95', []]);
  // execAll gives every result; a result larger than the kept buffer.
  const all = db.execAll("SELECT t FROM v WHERE id = 2; SELECT COUNT(*) FROM v");
  assert.deepEqual(all.map((r) => r.rows), [[[rows[1][1]]], [[2995]]]);
  const long = 'ą'.repeat(700_000);
  db.run(ins, [9000, long, 0, true, 0]);
  assert.equal(db.run(db.prepare('SELECT t FROM v WHERE id = ?'), [9000]).rows[0][0], long);
  assert.equal(db.exec('SELECT t FROM v WHERE id = 2').rows[0][0], rows[1][1]);
  db.free();
});
