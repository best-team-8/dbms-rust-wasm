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

test('durability is validated and needs a store that supports it', async () => {
  const idb = installFakeIndexedDB();
  try {
    await assert.rejects(FlatDB.open({ store: new IndexedDbStore('bad'), durability: 'fast' }), TypeError);
    await assert.rejects(FlatDB.open({ store: new MemoryStore(), durability: 'strict' }), TypeError);
  } finally {
    idb.uninstall();
  }
});
