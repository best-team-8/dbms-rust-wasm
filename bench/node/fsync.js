// Cost of durable commits on files in Node.js (FsStore), with and without
// fsync, and the cost of a durable server: one /sync/push round trip against
// a sync server with a data directory (FsStore, fsync on) and without one
// (MemoryStore). The CLI counterpart is `flatsql bench --storage DIR`.

import { mkdtemp, rm } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { pathToFileURL } from 'node:url';

import { nodeFetchFactory } from '../network.js';
import { DURABLE, INDEX, INSERT, SCHEMA, makeRow } from '../suite.js';

const now = () => performance.now();

function stats(samples) {
  const s = [...samples].sort((a, b) => a - b);
  const pick = (p) => s[Math.min(s.length - 1, Math.round((s.length - 1) * p))] * 1000;
  return { n: s.length, meanUs: (s.reduce((a, b) => a + b, 0) / s.length) * 1000, p50Us: pick(0.5), p95Us: pick(0.95), maxUs: s[s.length - 1] * 1000 };
}

async function fsStore(flat, rows, fsync) {
  const { FlatDB, FsStore } = flat;
  const dir = await mkdtemp(join(tmpdir(), 'flatsql-fsync-'));
  const ops = {};
  try {
    const db = await FlatDB.open({ store: new FsStore(dir, { fsync }) });
    db.exec(SCHEMA);
    db.exec(INDEX);
    const ins = db.prepare(INSERT);
    let t = now();
    db.exec('BEGIN');
    for (let i = 0; i < rows; i++) db.execute(ins, makeRow(i));
    db.exec('COMMIT');
    await db.flush();
    ops.insert = { n: 1, meanUs: (now() - t) * 1000 };
    const samples = [];
    for (let i = 0; i < DURABLE.single; i++) {
      const t0 = now();
      db.execute(ins, makeRow(DURABLE.firstId + i));
      await db.flush();
      samples.push(now() - t0);
    }
    ops.commit_single = stats(samples);
    t = now();
    for (let i = 0; i < DURABLE.group; i++) db.execute(ins, makeRow(DURABLE.firstId + DURABLE.single + i));
    await db.flush();
    ops.commit_group = { n: 1, meanUs: (now() - t) * 1000, perCommitUs: ((now() - t) * 1000) / DURABLE.group };
    t = now();
    await db.checkpoint();
    ops.checkpoint = { n: 1, meanUs: (now() - t) * 1000 };
    await db.close();
    t = now();
    const again = await FlatDB.open({ store: new FsStore(dir, { fsync }) });
    ops.reopen = { n: 1, meanUs: (now() - t) * 1000, rows: again.query('SELECT COUNT(*) AS n FROM items')[0].n };
    await again.close();
  } finally {
    await rm(dir, { recursive: true, force: true });
  }
  return ops;
}

/** One change pushed to the server per round (SyncClient.syncNow: push + pull). */
async function serverPush(flat, root, durableServer) {
  const { createServer } = await import(pathToFileURL(join(root, 'server', 'server.js')).href);
  const { FlatDB, MemoryStore, SyncClient } = flat;
  const dataDir = durableServer ? await mkdtemp(join(tmpdir(), 'flatsql-server-')) : null;
  const srv = await createServer({ port: 0, quiet: true, dataDir });
  const fetch = await nodeFetchFactory();
  try {
    const db = await FlatDB.open({ store: new MemoryStore() });
    db.exec('CREATE TABLE notes (id INT PRIMARY KEY, body TEXT)');
    const client = new SyncClient(db, { url: srv.url, fetch, intervalMs: 1e9 });
    await client.syncNow();
    const samples = [];
    for (let i = 0; i < 53; i++) {
      db.exec(`INSERT INTO notes VALUES (${i}, 'note ${i}')`);
      const t0 = now();
      await client.syncNow();
      if (i >= 3) samples.push(now() - t0); // the first rounds warm up the connection
    }
    await db.close();
    return stats(samples);
  } finally {
    fetch.close();
    await srv.close();
    if (dataDir) await rm(dataDir, { recursive: true, force: true });
  }
}

export async function runFsync({ env, rows, root }) {
  const flat = await env.flatsql();
  return {
    rows,
    fsStore: { fsync: await fsStore(flat, rows, true), nofsync: await fsStore(flat, rows, false) },
    syncPush: { durableServer: await serverPush(flat, root, true), memoryServer: await serverPush(flat, root, false) },
  };
}
