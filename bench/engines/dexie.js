// Dexie.js: JavaScript wrapper of IndexedDB (no WebAssembly, no SQL). Every
// write is an IndexedDB transaction and is durable when its promise resolves.
// Mapping of the workload onto the Dexie API:
//   point_lookup    get(id)
//   index_lookup    where('category').equals(c).count()   (index, no rows read)
//   range_scan      where('id').between(a, b).toArray() + sum in JS
//   full_scan       toCollection().filter(…).count()       (filter in JS)
//   group_by        toArray() + aggregation in JS           (no GROUP BY)
//   top_k           orderBy('price').reverse().limit(10), then every row
//                   with the 10th price, ties ordered by id in JS
//   materialize_10k where('id').below(10000).toArray()
//   update/delete   filter(…).modify(…) / filter(…).delete() in one transaction

import { makeRow } from '../suite.js';
import { groupBy, sumPrice, toDoc, toRow, top10, verifyDocs } from './docs.js';

const NAME = 'bench-dexie';

export async function create(env, { durability = 'default' } = {}) {
  const Dexie = (await env.importPkg('dexie', 'dexie/dist/dexie.mjs')).default;
  let db;
  const openDb = async () => {
    db = new Dexie(NAME, { chromeTransactionDurability: durability });
    db.version(1).stores({ items: 'id, category, price' });
    await db.open();
  };
  const OPS = {
    point_lookup: async ([id]) => {
      const d = await db.items.get(id);
      return d ? [toRow(d)] : [];
    },
    index_lookup: async ([c]) => [[await db.items.where('category').equals(c).count()]],
    range_scan: async ([a, b]) => sumPrice(await db.items.where('id').between(a, b, true, true).toArray()),
    full_scan: async () => [[await db.items.filter((d) => d.price > 50 && d.qty < 10).count()]],
    group_by: async () => groupBy(await db.items.toArray()),
    top_k: async () => {
      const first = await db.items.orderBy('price').reverse().limit(10).toArray();
      const p10 = first[first.length - 1].price;
      return top10(await db.items.where('price').aboveOrEqual(p10).toArray());
    },
    materialize_10k: async () => (await db.items.where('id').below(10000).toArray()).map(toRow),
  };
  return {
    async: true,
    version: Dexie.semVer,
    async open({ fresh } = {}) {
      if (fresh) await Dexie.delete(NAME);
      await openDb();
    },
    async load(rows) {
      const docs = [];
      for (let i = 0; i < rows; i++) docs.push(toDoc(makeRow(i)));
      await db.items.bulkAdd(docs);
    },
    prepare(op) {
      return OPS[op.name];
    },
    query(fn, params) {
      return fn(params);
    },
    async write(op) {
      if (op.name === 'update_10pct') {
        await db.items.filter((d) => d.id % 10 === 0).modify((d) => {
          d.qty += 1;
        });
      } else {
        await db.items.filter((d) => d.id % 10 === 1).delete();
      }
    },
    async verify() {
      return verifyDocs(await db.items.toArray());
    },
    async commit(row) {
      await db.items.add(toDoc(row));
    },
    // Separate transactions, all in flight at once.
    async commitGroup(rows) {
      await Promise.all(rows.map((r) => db.items.add(toDoc(r))));
    },
    async reopen() {
      db.close();
      await openDb();
    },
    async close({ destroy } = {}) {
      db.close();
      if (destroy) await Dexie.delete(NAME);
    },
  };
}
