// RxDB: reactive document database with replication, here on its free Dexie
// (IndexedDB) storage; no WebAssembly, no SQL. Primary key: zero-padded id.
// Mapping of the workload onto RxDB queries (Mango selectors):
//   point_lookup    findOne(pk)
//   index_lookup    count({ selector: { category } })        (index on category)
//   range_scan      find({ pk: { $gte, $lte } }) + sum in JS
//   full_scan       find({ price: { $gt: 50 }, qty: { $lt: 10 } }), length
//   group_by        find() + aggregation in JS                (no GROUP BY)
//   top_k           find({ sort: [price desc, id asc], limit: 10 })
//   materialize_10k find({ pk: { $lt } })
//   update/delete   find() + bulkUpsert / bulkRemove
// RxDB returns the cached result of a repeated query while the collection is
// unchanged. Every read adds a condition that is always true and differs per
// call (id ≠ −n), so the query is executed again. A query that names the
// primary key is still answered from RxDB's in-memory document cache when the
// document was read before (rx-query.js, queryCollection); in this workload
// the state check after loading reads every document, so point_lookup
// measures that cache.

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

const docOf = (r) => ({ pk: pad(r[0]), ...toDoc(r) });
const plain = (docs) => docs.map((d) => d.toJSON());

export async function create(env) {
  const { createRxDatabase, removeRxDatabase, getRxStorageDexie } = await env.importPkg(null, 'vendor:rxdb');
  const storage = getRxStorageDexie();
  let db;
  let items;
  let n = 0;
  const fresh = () => ({ id: { $ne: -1 - n++ } });
  const openDb = async () => {
    db = await createRxDatabase({ name: NAME, storage, multiInstance: false, eventReduce: false });
    ({ items } = await db.addCollections({ items: { schema: SCHEMA } }));
  };
  const all = async () => plain(await items.find({ selector: fresh() }).exec());
  const OPS = {
    point_lookup: async ([id]) => {
      const d = await items.findOne({ selector: { pk: pad(id), ...fresh() } }).exec();
      return d ? [toRow(d.toJSON())] : [];
    },
    index_lookup: async ([c]) => [[await items.count({ selector: { category: c } }).exec()]],
    range_scan: async ([a, b]) => sumPrice(plain(await items.find({ selector: { pk: { $gte: pad(a), $lte: pad(b) }, ...fresh() } }).exec())),
    full_scan: async () => [[(await items.find({ selector: { price: { $gt: 50 }, qty: { $lt: 10 }, ...fresh() } }).exec()).length]],
    group_by: async () => groupBy(await all()),
    top_k: async () =>
      (await items.find({ selector: fresh(), sort: [{ price: 'desc' }, { id: 'asc' }], limit: 10 }).exec()).map((d) => [d.id, d.price]),
    materialize_10k: async () => plain(await items.find({ selector: { pk: { $lt: pad(10000) }, ...fresh() } }).exec()).map(toRow),
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
