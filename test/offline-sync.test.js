// Local-First validation: replicas keep working while the server is down,
// survive restarts with pending changes and converge after reconnecting.

import { test } from 'node:test';
import assert from 'node:assert/strict';
import { mkdtemp, rm } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { join } from 'node:path';

import { FlatDB, MemoryStore, SyncClient } from '../bridge/node.js';
import { createServer } from '../server/server.js';

const dump = (db, table) => db.exec(`SELECT * FROM ${table} ORDER BY 1`).rows;

async function replica(url, store = new MemoryStore()) {
  const db = await FlatDB.open({ store });
  const sync = new SyncClient(db, { url, intervalMs: 50 });
  return { db, sync, store };
}

test('two replicas exchange changes through the server', async () => {
  const srv = await createServer({ port: 0, quiet: true });
  try {
    const a = await replica(srv.url);
    const b = await replica(srv.url);
    a.db.exec('CREATE TABLE inventory (id INT PRIMARY KEY, item TEXT, qty INT)');
    a.db.exec("INSERT INTO inventory VALUES (1, 'sword', 1), (2, 'potion', 5)");
    const pushed = await a.sync.drain();
    assert.equal(pushed.pushed, 3);
    await b.sync.drain();
    assert.deepEqual(dump(b.db, 'inventory'), dump(a.db, 'inventory'));

    b.db.exec('UPDATE inventory SET qty = qty - 1 WHERE id = 2');
    await b.sync.drain();
    await a.sync.drain();
    assert.equal(a.db.query('SELECT qty FROM inventory WHERE id = 2')[0].qty, 4);
    assert.deepEqual(dump(srv.db, 'inventory'), dump(a.db, 'inventory'));
  } finally {
    await srv.close();
  }
});

test('offline: local work continues, outbox survives restart, replicas converge', async () => {
  const dataDir = await mkdtemp(join(tmpdir(), 'flatsql-server-'));
  let srv = await createServer({ port: 0, dataDir, quiet: true });
  try {
    const storeA = new MemoryStore();
    let a = await replica(srv.url, storeA);
    const b = await replica(srv.url);

    a.db.exec('CREATE TABLE tasks (id INT PRIMARY KEY, title TEXT, done BOOL)');
    a.db.exec("INSERT INTO tasks VALUES (1, 'write thesis', FALSE), (2, 'run benchmarks', FALSE)");
    await a.sync.drain();
    await b.sync.drain();

    // ---- the server goes away ------------------------------------------
    await srv.close();

    const t0 = performance.now();
    for (let i = 3; i <= 52; i++) a.db.exec(`INSERT INTO tasks VALUES (${i}, 'offline task ${i}', FALSE)`);
    a.db.exec('UPDATE tasks SET done = TRUE WHERE id = 1');
    b.db.exec("UPDATE tasks SET title = 'run benchmarks (WASM vs native)' WHERE id = 2");
    const localMs = performance.now() - t0;
    assert.ok(localMs < 1000, `local writes do not wait for the network (${localMs.toFixed(1)} ms)`);
    assert.equal(a.db.query('SELECT COUNT(*) AS n FROM tasks')[0].n, 52, 'reads work offline');

    const failed = await a.sync.syncNow();
    assert.ok(failed.error, 'sync round fails while the server is down');
    assert.equal(a.sync.status, 'offline');
    assert.equal(a.sync.pending, 51);

    // The application is closed and reopened while still offline.
    await a.db.close();
    a = await replica(srv.url, storeA);
    assert.equal(a.db.outboxLength(), 51, 'pending changes were persisted with the WAL');
    assert.equal(a.db.query('SELECT done FROM tasks WHERE id = 1')[0].done, true);

    // ---- the server comes back (new process, same data directory) -------
    srv = await createServer({ port: 0, dataDir, quiet: true });
    a.sync.url = srv.url;
    b.sync.url = srv.url;
    await a.sync.drain();
    await b.sync.drain();
    await a.sync.drain();

    assert.equal(a.db.outboxLength(), 0);
    assert.equal(b.db.outboxLength(), 0);
    assert.deepEqual(dump(a.db, 'tasks'), dump(b.db, 'tasks'));
    assert.deepEqual(dump(srv.db, 'tasks'), dump(a.db, 'tasks'));
    assert.equal(b.db.query('SELECT COUNT(*) AS n FROM tasks')[0].n, 52);
    assert.equal(a.db.query('SELECT title FROM tasks WHERE id = 2')[0].title, 'run benchmarks (WASM vs native)');
  } finally {
    await srv.close();
    await rm(dataDir, { recursive: true, force: true });
  }
});

test('conflicting offline edits: the later write wins on every replica', async () => {
  const srv = await createServer({ port: 0, quiet: true });
  try {
    const a = await replica(srv.url);
    const b = await replica(srv.url);
    a.db.exec('CREATE TABLE profile (id INT PRIMARY KEY, nick TEXT)');
    a.db.exec("INSERT INTO profile VALUES (1, 'start')");
    await a.sync.drain();
    await b.sync.drain();

    a.sync.setSimulatedOffline(true);
    b.sync.setSimulatedOffline(true);
    a.db.exec("UPDATE profile SET nick = 'from A' WHERE id = 1");
    await new Promise((r) => setTimeout(r, 5));
    b.db.exec("UPDATE profile SET nick = 'from B' WHERE id = 1");
    assert.ok((await a.sync.syncNow()).error);

    // A reconnects last, but its edit is older: B's value must win.
    b.sync.setSimulatedOffline(false);
    await b.sync.drain();
    a.sync.setSimulatedOffline(false);
    await a.sync.drain();
    await b.sync.drain();
    for (const db of [a.db, b.db, srv.db]) {
      assert.equal(db.query('SELECT nick FROM profile')[0].nick, 'from B');
    }
  } finally {
    await srv.close();
  }
});

test('cloud API writes reach local replicas through sync', async () => {
  const srv = await createServer({ port: 0, quiet: true });
  try {
    const a = await replica(srv.url);
    a.db.exec('CREATE TABLE flags (name TEXT PRIMARY KEY, enabled BOOL)');
    await a.sync.drain();
    const res = await fetch(`${srv.url}/api/query`, {
      method: 'POST',
      body: "INSERT INTO flags VALUES ('new-checkout', TRUE)",
    });
    assert.equal(res.status, 200);
    assert.equal((await res.json()).rowsAffected, 1);
    const bad = await fetch(`${srv.url}/api/query`, { method: 'POST', body: 'SELECT * FROM nope' });
    assert.equal(bad.status, 400);
    assert.equal((await bad.json()).kind, 'schema');

    await a.sync.drain();
    assert.deepEqual(a.db.query('SELECT * FROM flags'), [{ name: 'new-checkout', enabled: true }]);
  } finally {
    await srv.close();
  }
});

test('background sync pushes local writes automatically', async () => {
  const srv = await createServer({ port: 0, quiet: true });
  try {
    const a = await replica(srv.url);
    a.sync.debounceMs = 10;
    await a.sync.start();
    a.db.exec('CREATE TABLE events (id INT PRIMARY KEY, kind TEXT)');
    a.db.exec("INSERT INTO events VALUES (1, 'login')");
    const synced = await new Promise((resolve) => {
      a.sync.addEventListener('synced', function check() {
        if (a.db.outboxLength() === 0) {
          a.sync.removeEventListener('synced', check);
          resolve(true);
        }
      });
    });
    assert.ok(synced);
    a.sync.stop();
    assert.equal(srv.db.query('SELECT kind FROM events')[0].kind, 'login');
  } finally {
    await srv.close();
  }
});

test('static files and wasm are served with correct types', async () => {
  const srv = await createServer({ port: 0, quiet: true });
  try {
    const wasm = await fetch(`${srv.url}/pkg/flatsql_bg.wasm`);
    assert.equal(wasm.headers.get('content-type'), 'application/wasm');
    const bridge = await fetch(`${srv.url}/bridge/index.js`);
    assert.equal(bridge.status, 200);
    for (const path of ['/..%2f..%2fCargo.toml', '/pkg/..%2fCargo.toml', '/bridge/..%2f..%2fpackage.json', '/vendor/sql.js/..%2f..%2f..%2fpackage.json']) {
      const traversal = await fetch(`${srv.url}${path}`);
      assert.notEqual(traversal.status, 200, path);
    }
    const app = await fetch(`${srv.url}/`);
    assert.match(await app.text(), /Flat-SQL/);
  } finally {
    await srv.close();
  }
});
