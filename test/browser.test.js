// Browser behaviour of FlatDB checked in Node.js against an in-memory
// IndexedDB (support/fake-indexeddb.js): durability of IndexedDB writes.
// The Service Worker of the demo app (web/sw.js) runs with stand-ins for the
// cache and the network that read files the way the server serves them.

import { test } from 'node:test';
import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import { join } from 'node:path';
import { fileURLToPath } from 'node:url';
import vm from 'node:vm';

import { FlatDB, IndexedDbStore, MemoryStore } from '../bridge/node.js';
import { installFakeIndexedDB } from './support/fake-indexeddb.js';

test('FlatDB.open passes the durability option to every IndexedDB write', async () => {
  const idb = installFakeIndexedDB();
  try {
    for (const durability of ['strict', 'relaxed', undefined]) {
      idb.transactions.length = 0;
      const db = await FlatDB.open({ store: new IndexedDbStore(`durability-${durability}`), durability });
      db.exec('CREATE TABLE t (id INT PRIMARY KEY)');
      await db.flush();
      await db.checkpoint();
      const writes = idb.transactions.filter((t) => t.mode === 'readwrite');
      const stores = writes.flatMap((t) => t.stores);
      assert.ok(stores.includes('wal') && stores.includes('snapshot'), `WAL append and checkpoint: ${stores}`);
      const want = durability ?? 'default';
      assert.deepEqual([...new Set(writes.map((t) => t.durability))], [want], `durability ${durability}`);
      assert.equal(db.stats().durability, want);
      await db.close();
    }
  } finally {
    idb.uninstall();
  }
});

test('without a store FlatDB.open uses IndexedDB in the browser, so data survives a reload', async () => {
  const idb = installFakeIndexedDB();
  try {
    const db = await FlatDB.open({ name: 'notes', durability: 'strict' });
    assert.equal(db.store.kind, 'indexeddb');
    assert.equal(db.store.name, 'notes');
    assert.equal(db.stats().durability, 'strict');
    db.exec("CREATE TABLE notes (id INT PRIMARY KEY, body TEXT); INSERT INTO notes VALUES (1, 'kept');");
    await db.close();
    // A page reload: a new FlatDB with the same name finds the data.
    const again = await FlatDB.open({ name: 'notes' });
    assert.deepEqual(again.query('SELECT body FROM notes'), [{ body: 'kept' }]);
    await again.close();
    const unnamed = await FlatDB.open();
    assert.equal(unnamed.store.name, 'default');
    await unnamed.close();
  } finally {
    idb.uninstall();
  }
});

test('without a store FlatDB.open refuses to start outside the browser', async () => {
  assert.equal(globalThis.indexedDB, undefined);
  await assert.rejects(FlatDB.open(), (e) => /MemoryStore/.test(e.message) && /FsStore/.test(e.message));
  await assert.rejects(FlatDB.open({ name: 'x' }), /MemoryStore/);
});

/** navigator.storage stand-in: persisted() reports `granted`, persist() answers `answer`. */
function fakeStorageManager({ granted = false, answer = true } = {}) {
  const storage = {
    asked: 0,
    async persisted() {
      return granted;
    },
    async persist() {
      storage.asked++;
      granted = answer;
      return answer;
    },
  };
  Object.defineProperty(globalThis.navigator, 'storage', { value: storage, configurable: true });
  storage.uninstall = () => delete globalThis.navigator.storage;
  return storage;
}

test('FlatDB.open asks the browser to keep IndexedDB data (navigator.storage.persist)', async () => {
  const idb = installFakeIndexedDB();
  try {
    for (const [setup, options, asked, persisted] of [
      [{ granted: false, answer: true }, {}, 1, true],
      [{ granted: false, answer: false }, {}, 1, false],
      [{ granted: true }, {}, 0, true],
      [{ granted: false }, { persist: false }, 0, false],
    ]) {
      const storage = fakeStorageManager(setup);
      try {
        const db = await FlatDB.open({ name: 'persist', ...options });
        assert.equal(await db.persistence, persisted, JSON.stringify({ setup, options }));
        assert.equal(storage.asked, asked, JSON.stringify({ setup, options }));
        assert.equal(db.stats().persisted, persisted);
        await db.close();
      } finally {
        storage.uninstall();
      }
    }
  } finally {
    idb.uninstall();
  }
});

test('persistence is not requested for stores outside browser storage', async () => {
  const storage = fakeStorageManager();
  try {
    const db = await FlatDB.open({ store: new MemoryStore() });
    assert.equal(await db.persistence, null);
    assert.equal(storage.asked, 0);
    await db.close();
  } finally {
    storage.uninstall();
  }
});

test('durability is validated and needs a store that supports it', async () => {
  const idb = installFakeIndexedDB();
  try {
    await assert.rejects(FlatDB.open({ store: new IndexedDbStore('bad'), durability: 'fast' }), TypeError);
    await assert.rejects(FlatDB.open({ store: new MemoryStore(), durability: 'strict' }), TypeError);
  } finally {
    idb.uninstall();
  }
});

test('IndexedDbStore keeps the snapshot as a Blob and still reads one stored as bytes', async () => {
  const idb = installFakeIndexedDB();
  try {
    const store = new IndexedDbStore('blob-snapshot');
    const db = await FlatDB.open({ store, checkpointStepRows: 7 });
    db.exec("CREATE TABLE t (id INT PRIMARY KEY, v TEXT); INSERT INTO t VALUES (1, 'a'), (2, 'b'), (3, 'c');");
    await db.checkpoint();
    await db.close();
    const raw = await rawSnapshot('blob-snapshot');
    assert.ok(raw instanceof Blob, 'stored as a Blob');
    const again = await FlatDB.open({ store: new IndexedDbStore('blob-snapshot') });
    assert.deepEqual(again.exec('SELECT v FROM t ORDER BY id').rows, [['a'], ['b'], ['c']]);
    assert.equal(again.stats().snapshotBytes, raw.size);
    await again.close();
    // A snapshot written as bytes by an older version.
    await putRawSnapshot('blob-snapshot', new Uint8Array(await raw.arrayBuffer()));
    const old = await FlatDB.open({ store: new IndexedDbStore('blob-snapshot') });
    assert.deepEqual(old.exec('SELECT v FROM t ORDER BY id').rows, [['a'], ['b'], ['c']]);
    await old.close();
  } finally {
    idb.uninstall();
  }
});

/** The stored snapshot value of IndexedDB database `name` (IndexedDbStore layout). */
async function rawSnapshot(name) {
  const db = await openRaw(name);
  const value = await new Promise((resolve, reject) => {
    const r = db.transaction(['snapshot'], 'readonly').objectStore('snapshot').get('main');
    r.onsuccess = () => resolve(r.result);
    r.onerror = () => reject(r.error);
  });
  db.close();
  return value;
}

async function putRawSnapshot(name, value) {
  const db = await openRaw(name);
  await new Promise((resolve, reject) => {
    const tx = db.transaction(['snapshot'], 'readwrite');
    tx.objectStore('snapshot').put(value, 'main');
    tx.oncomplete = resolve;
    tx.onerror = () => reject(tx.error);
  });
  db.close();
}

function openRaw(name) {
  return new Promise((resolve, reject) => {
    const r = indexedDB.open(new IndexedDbStore(name).dbName);
    r.onsuccess = () => resolve(r.result);
    r.onerror = () => reject(r.error);
  });
}

const ROOT = fileURLToPath(new URL('..', import.meta.url));
const ORIGIN = 'http://demo.test';

/** File behind a URL path, mapped as in server/server.js. */
function fileOf(path) {
  if (path === '/') path = '/index.html';
  for (const prefix of ['/pkg/', '/bridge/']) {
    if (path.startsWith(prefix)) return join(ROOT, prefix, path.slice(prefix.length));
  }
  return join(ROOT, 'web', path);
}

async function serve(url) {
  try {
    return new Response(await readFile(fileOf(new URL(url).pathname)));
  } catch {
    return new Response('not found', { status: 404 });
  }
}

/** Runs web/sw.js and its install step; returns the cached URL paths. */
async function installServiceWorker() {
  const handlers = {};
  const cached = new Set();
  const resolve = (url) => new URL(url, `${ORIGIN}/sw.js`).href;
  const cache = {
    async addAll(urls) {
      for (const url of urls) {
        const response = await serve(resolve(url));
        if (!response.ok) throw new TypeError(`addAll: ${url} gives ${response.status}`);
        cached.add(new URL(resolve(url)).pathname);
      }
    },
    async put(request) {
      cached.add(new URL(resolve(request.url ?? request)).pathname);
    },
  };
  const self = {
    location: new URL(`${ORIGIN}/sw.js`),
    addEventListener: (type, handler) => (handlers[type] = handler),
    skipWaiting: async () => {},
    clients: { claim: async () => {} },
  };
  const context = {
    self,
    caches: { open: async () => cache, keys: async () => [], delete: async () => true },
    fetch: (url) => serve(resolve(url)),
    URL,
    Response,
  };
  vm.runInNewContext(await readFile(join(ROOT, 'web', 'sw.js'), 'utf8'), context);
  let installed;
  handlers.install({ waitUntil: (promise) => (installed = promise) });
  await installed;
  return cached;
}

/** URL paths the page loads: assets of index.html and the static import graph from app.js. */
async function pageFiles() {
  const files = new Set(['/']);
  const html = await readFile(fileOf('/index.html'), 'utf8');
  const queue = [];
  for (const [, ref] of html.matchAll(/(?:src|href)="([^"]+)"/g)) {
    const path = new URL(ref, `${ORIGIN}/`).pathname;
    files.add(path);
    if (path.endsWith('.js')) queue.push(path);
  }
  while (queue.length > 0) {
    const path = queue.shift();
    const source = await readFile(fileOf(path), 'utf8');
    const refs = [
      ...source.matchAll(/^\s*(?:import|export)\b[^;]*?\bfrom\s*["']([^"']+)["']/gm),
      ...source.matchAll(/^\s*import\s*["']([^"']+)["']/gm),
      ...source.matchAll(/new URL\(\s*["']([^"']+)["']\s*,\s*import\.meta\.url\s*\)/g),
    ];
    for (const [, ref] of refs) {
      const target = new URL(ref, `${ORIGIN}${path}`).pathname;
      if (files.has(target)) continue;
      files.add(target);
      if (target.endsWith('.js')) queue.push(target);
    }
  }
  return files;
}

test('the Service Worker caches at install every file the page loads, including wasm-bindgen snippets', async () => {
  const cached = await installServiceWorker();
  const needed = await pageFiles();
  assert.ok([...needed].some((p) => p.startsWith('/pkg/snippets/')), 'the module imports snippets');
  assert.ok(needed.has('/pkg/flatsql_bg.wasm'));
  const missing = [...needed].filter((p) => !cached.has(p));
  assert.deepEqual(missing, [], 'files the page loads but the Service Worker does not cache');
});
