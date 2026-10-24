// RxDB: reactive document database with replication, here on its free Dexie
// (IndexedDB) storage; no WebAssembly, no SQL. Primary key: zero-padded id.
// Mapping of the workload onto RxDB queries (Mango selectors):
//   point_lookup    findOne(pk)
//   index_lookup    count({ selector: { category } })        (index on category)
//   range_scan      find({ pk: { $gte, $lte } }, index pk) + sum in JS
//   full_scan       find({ price: { $gt: 50 }, qty: { $lt: 10 } }), length
//   group_by        find() + aggregation in JS                (no GROUP BY)
//   top_k           find({ sort: [price desc, id asc], limit: 10 })
//   materialize_10k find({ pk: { $lt } }, index pk)
//   update/delete   find() + bulkUpsert / bulkRemove
// RxDB returns the cached result of a repeated query while the collection is
// unchanged (QueryCache, keyed by the query's JSON). The adapter empties that
// cache before every query (an internal map, the only way without changing
// the query), so every call is executed by the storage like in the other
// engines. A query by primary key is still answered from RxDB's document
// cache when the document was read before (rx-query.js, queryCollection); in
// this workload the state check after loading reads every document, so
// point_lookup measures that cache.

import { makeRow } from '../suite.js';
import { groupBy, pad, sumPrice, toDoc, toRow, verifyDocs } from './docs.js';

const NAME = 'benchrxdb';
const SCHEMA = {
  version: 0,
  primaryKey: 'pk',
  type: 'object',
  properties: {
    pk: { type: 'string', maxLength: 8 },
    id: { type: 'number', minimum: 0, maximum: 100_000_000, multipleOf: 1 },
    category: { type: 'string', maxLength: 8 },
    price: { type: 'number', minimum: 0, maximum: 100, multipleOf: 0.01 },
    qty: { type: 'number' },
    name: { type: 'string' },
  },
  required: ['pk', 'id', 'category', 'price', 'qty', 'name'],
  indexes: ['category', ['price', 'id']],
};

// Without a hint RxDB's query planner answers a primary-key range through the
// [category, pk] index and reads the whole collection (measured: 48 ms instead
// of 22 ms for 1000 of 2000 documents); RxDB documents the `index` field of a
// query for such cases.
const PK = ['pk'];

const docOf = (r) => ({ pk: pad(r[0]), ...toDoc(r) });
const plain = (docs) => docs.map((d) => d.toJSON());

export async function create(env) {
  const { createRxDatabase, removeRxDatabase, getRxStorageDexie } = await env.importPkg(null, 'vendor:rxdb');
  const storage = getRxStorageDexie();
  let db;
  let items;
  const exec = (query) => {
    items._queryCache._map.clear();
    return query.exec();
  };
  const openDb = async () => {
    db = await createRxDatabase({ name: NAME, storage, multiInstance: false, eventReduce: false });
    ({ items } = await db.addCollections({ items: { schema: SCHEMA } }));
  };
  const all = async () => plain(await exec(items.find()));
  const OPS = {
    point_lookup: async ([id]) => {
      const d = await exec(items.findOne(pad(id)));
      return d ? [toRow(d.toJSON())] : [];
    },
    index_lookup: async ([c]) => [[await exec(items.count({ selector: { category: c } }))]],
    range_scan: async ([a, b]) => sumPrice(plain(await exec(items.find({ selector: { pk: { $gte: pad(a), $lte: pad(b) } }, index: PK })))),
    full_scan: async () => [[(await exec(items.find({ selector: { price: { $gt: 50 }, qty: { $lt: 10 } } }))).length]],
    group_by: async () => groupBy(await all()),
    top_k: async () => (await exec(items.find({ sort: [{ price: 'desc' }, { id: 'asc' }], limit: 10 }))).map((d) => [d.id, d.price]),
    materialize_10k: async () => plain(await exec(items.find({ selector: { pk: { $lt: pad(10000) } }, index: PK }))).map(toRow),
  };
  return {
    async: true,
    async open({ fresh: clean } = {}) {
      if (clean) await removeRxDatabase(NAME, storage);
      await openDb();
    },
    async load(rows) {
      const batch = [];
      for (let i = 0; i < rows; i++) batch.push(docOf(makeRow(i)));
      const r = await items.bulkInsert(batch);
      if (r.error.length) throw new Error(`bulkInsert: ${r.error.length} errors`);
    },
    prepare(op) {
      return OPS[op.name];
    },
    query(fn, params) {
      return fn(params);
    },
    async write(op) {
      const list = await all();
      if (op.name === 'update_10pct') {
        await items.bulkUpsert(list.filter((d) => d.id % 10 === 0).map((d) => ({ ...d, qty: d.qty + 1 })));
      } else {
        await items.bulkRemove(list.filter((d) => d.id % 10 === 1).map((d) => d.pk));
      }
    },
    async verify() {
      return verifyDocs(await all());
    },
    async commit(row) {
      await items.insert(docOf(row));
    },
    async commitGroup(rows) {
      await Promise.all(rows.map((r) => items.insert(docOf(r))));
    },
    async reopen() {
      await db.close();
      await openDb();
    },
    async close({ destroy } = {}) {
      if (destroy) await db.remove();
      else await db.close();
    },
  };
}
