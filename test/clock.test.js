// A device whose clock is wrong. The sync server runs in its own process with
// the real clock; in this process Date.now (used by the engine for stamps and
// by SyncClient) is shifted to simulate the device clock.

import { test } from 'node:test';
import assert from 'node:assert/strict';
import { spawn } from 'node:child_process';
import { fileURLToPath } from 'node:url';

import { ClockError, FlatDB, MemoryStore, SyncClient } from '../bridge/node.js';

const HOUR = 3_600_000;
const realNow = Date.now;

async function startServer() {
  const script = fileURLToPath(new URL('../server/server.js', import.meta.url));
  const child = spawn(process.execPath, [script, '--port', '0'], { stdio: ['ignore', 'pipe', 'inherit'] });
  const url = await new Promise((resolve, reject) => {
    let out = '';
    child.stdout.on('data', (chunk) => {
      out += chunk;
      const m = out.match(/listening on (http:\/\/\S+)/);
      if (m) resolve(m[1]);
    });
    child.once('exit', (code) => reject(new Error(`server exited with ${code}`)));
  });
  const api = async (path, body) => {
    const res = await fetch(`${url}${path}`, body === undefined ? {} : { method: 'POST', body });
    return res.json();
  };
  return { url, api, stop: () => child.kill() };
}

const setDeviceClock = (offsetMs) => {
  Date.now = () => realNow() + offsetMs;
};

test('a device with its clock ahead pauses sync and sends its changes once the clock is fixed', async () => {
  const srv = await startServer();
  try {
    const db = await FlatDB.open({ store: new MemoryStore() });
    const sync = new SyncClient(db, { url: srv.url });
    const clockEvents = [];
    sync.addEventListener('clock', (e) => clockEvents.push(e.detail));

    db.exec("CREATE TABLE notes (id INT PRIMARY KEY, body TEXT); INSERT INTO notes VALUES (9, 'clock right');");
    await sync.drain();

    // The device clock jumps an hour ahead.
    setDeviceClock(HOUR);
    db.exec("INSERT INTO notes VALUES (1, 'clock ahead'), (2, 'clock ahead too')");
    const r = await sync.syncNow();
    assert.ok(r.error instanceof ClockError, String(r.error));
    assert.equal(sync.status, 'clock');
    assert.equal(clockEvents[0].rejected, 2);
    assert.ok(Math.abs(clockEvents[0].skewMs - HOUR) < 5000, `skew ${clockEvents[0].skewMs}`);
    assert.equal(db.outboxLength(), 2, 'rejected changes stay in the outbox');

    // Paused: further rounds only check the clock, nothing is pushed.
    const pushes = (await srv.api('/api/status')).stats.pushes;
    for (let i = 0; i < 3; i++) assert.ok((await sync.syncNow()).error instanceof ClockError);
    const status = await srv.api('/api/status');
    assert.equal(status.stats.pushes, pushes, 'no pushes while paused');
    assert.ok(Math.abs(status.db.clockMs - realNow()) < 5000, 'the server clock was not dragged ahead');
    assert.equal((await srv.api('/api/query', 'SELECT COUNT(*) AS n FROM notes')).rows[0][0], 1);

    // The clock is fixed: the changes get new stamps and go through.
    setDeviceClock(0);
    const done = await sync.syncNow();
    assert.equal(done.error, undefined, String(done.error));
    assert.equal(done.restamped, 2);
    assert.equal(done.pushed, 2);
    assert.equal(db.outboxLength(), 0);
    assert.ok(Math.abs(db.stats().clockMs - realNow()) < 5000, 'the device clock is back');
    const rows = await srv.api('/api/query', 'SELECT id, body FROM notes ORDER BY id');
    assert.deepEqual(rows.rows, [[1, 'clock ahead'], [2, 'clock ahead too'], [9, 'clock right']]);
    await db.close();
  } finally {
    setDeviceClock(0);
    srv.stop();
  }
});

test('a device with its clock behind pauses before sending anything', async () => {
  const srv = await startServer();
  try {
    setDeviceClock(-HOUR);
    const db = await FlatDB.open({ store: new MemoryStore() });
    const sync = new SyncClient(db, { url: srv.url });
    db.exec("CREATE TABLE notes (id INT PRIMARY KEY, body TEXT); INSERT INTO notes VALUES (1, 'clock behind');");
    const r = await sync.syncNow();
    assert.ok(r.error instanceof ClockError, String(r.error));
    assert.ok(Math.abs(sync.clockSkewMs + HOUR) < 5000, `skew ${sync.clockSkewMs}`);
    assert.equal((await srv.api('/api/status')).stats.pushes, 0, 'nothing was sent');

    setDeviceClock(0);
    const done = await sync.syncNow();
    assert.equal(done.error, undefined, String(done.error));
    assert.equal(done.pushed, 2);
    await db.close();
  } finally {
    setDeviceClock(0);
    srv.stop();
  }
});
