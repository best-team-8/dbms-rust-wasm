// Browser behaviour of FlatDB checked in Node.js against an in-memory
// IndexedDB (support/fake-indexeddb.js): durability of IndexedDB writes.

import { test } from 'node:test';
import assert from 'node:assert/strict';

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
