// PouchDB: document database with CouchDB replication, IndexedDB adapter in
// the browser (no WebAssembly, no SQL). Document _id is the zero-padded id, so
// allDocs ranges follow the numeric order. Mapping of the workload:
//   point_lookup    get(_id)
//   index_lookup    persistent view by category with the _count reducer
//   range_scan      allDocs({ startkey, endkey, include_docs }) + sum in JS
//   full_scan       allDocs({ include_docs }) + filter in JS
//   group_by        allDocs({ include_docs }) + aggregation in JS
//   top_k           persistent view by price, descending, limit 10, then every
//                   row with the 10th price, ties ordered by id in JS
//   materialize_10k allDocs over the first 10 000 ids
//   update/delete   allDocs + bulkDocs (one request)
// The views are built when first queried; the build is measured separately
// as "index_build" right after loading the data.

import { makeRow } from '../suite.js';
import { countFiltered, groupBy, pad, sumPrice, toDoc, toRow, top10, verifyDocs } from './docs.js';

const NAME = 'bench-pouchdb';
const DESIGN = {
  _id: '_design/b',
  views: {
    by_category: { map: 'function (doc) { emit(doc.category); }', reduce: '_count' },
    by_price: { map: 'function (doc) { emit(doc.price); }' },
  },
};

const docOf = (r) => ({ _id: pad(r[0]), ...toDoc(r) });
const docs = (res) => res.rows.filter((r) => r.doc && !r.id.startsWith('_design/')).map((r) => r.doc);

export async function create(env) {
  const PouchDB = (await env.importPkg(null, 'vendor:pouchdb')).default;
  let db;
  const all = async () => docs(await db.allDocs({ include_docs: true }));
  const OPS = {
    point_lookup: async ([id]) => {
      try {
        return [toRow(await db.get(pad(id)))];
      } catch (e) {
        if (e.status === 404) return [];
        throw e;
      }
    },
    index_lookup: async ([c]) => [[(await db.query('b/by_category', { key: c, reduce: true })).rows[0]?.value ?? 0]],
    range_scan: async ([a, b]) => sumPrice(docs(await db.allDocs({ startkey: pad(a), endkey: pad(b), include_docs: true }))),
    full_scan: async () => countFiltered(await all()),
    group_by: async () => groupBy(await all()),
    top_k: async () => {
      const first = await db.query('b/by_price', { descending: true, limit: 10, reduce: false });
      const p10 = first.rows[first.rows.length - 1].key;
      const ties = await db.query('b/by_price', { descending: true, endkey: p10, reduce: false, include_docs: true });
      return top10(ties.rows.map((r) => r.doc));
    },
    materialize_10k: async () => docs(await db.allDocs({ startkey: pad(0), endkey: pad(9999), include_docs: true })).map(toRow),
  };
  return {
    async: true,
    version: PouchDB.version,
    async open({ fresh } = {}) {
      if (fresh) await new PouchDB(NAME).destroy();
      db = new PouchDB(NAME);
      await db.put(DESIGN).catch((e) => {
        if (e.status !== 409) throw e;
      });
    },
    async load(rows) {
      const batch = [];
      for (let i = 0; i < rows; i++) batch.push(docOf(makeRow(i)));
      await db.bulkDocs(batch);
    },
    /** Builds the persistent views (map/reduce indexes) after a load. */
    async buildIndexes() {
      await db.query('b/by_category', { limit: 0 });
      await db.query('b/by_price', { limit: 0 });
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
        await db.bulkDocs(list.filter((d) => d.id % 10 === 0).map((d) => ({ ...d, qty: d.qty + 1 })));
      } else {
        await db.bulkDocs(list.filter((d) => d.id % 10 === 1).map((d) => ({ ...d, _deleted: true })));
      }
    },
    async verify() {
      return verifyDocs(await all());
    },
    async commit(row) {
      await db.put(docOf(row));
    },
    async commitGroup(rows) {
      await Promise.all(rows.map((r) => db.put(docOf(r))));
    },
    async reopen() {
      await db.close();
      db = new PouchDB(NAME);
      await db.info();
    },
    async close({ destroy } = {}) {
      if (destroy) await db.destroy();
      else await db.close();
    },
  };
}
