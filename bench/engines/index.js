// Registry of benchmarked engine configurations.
//
// group – run conditions shared by every engine in the group; results are
//   only compared within a group (docs/research/benchmark-scenario.md):
//     memory  – data in memory, engine in the calling thread (Node.js process,
//               browser main thread);
//     durable – data in IndexedDB, browser main thread, a write is measured
//               until it is durable;
//     worker  – engine in a dedicated Web Worker, every call is a postMessage
//               round trip from the main thread.
// where – environments the configuration runs in (node, browser; worker
//   configurations run in the browser inside bench/browser/worker.js).
// api – sync / async API of the engine as used by the adapter.
// model – SQL, or an API without SQL (queries mapped by the adapter).
// needs – requirements of the environment besides WebAssembly.

export const ENGINES = {
  // ---- group: memory ----
  flatsql: { label: 'Flat-SQL', group: 'memory', where: ['node', 'browser'], module: 'flatsql', api: 'sync', model: 'SQL', needs: [] },
  'flatsql-json': { label: 'Flat-SQL (JSON)', group: 'memory', where: ['node', 'browser'], module: 'flatsql', options: { json: true }, api: 'sync', model: 'SQL', needs: [] },
  sqljs: { label: 'sql.js', group: 'memory', where: ['node', 'browser'], module: 'sqljs', api: 'sync', model: 'SQL', needs: [] },
  'sqlite-wasm': { label: 'SQLite WASM (:memory:)', group: 'memory', where: ['node', 'browser'], module: 'sqlite-wasm', api: 'sync', model: 'SQL', needs: [] },
  pglite: { label: 'PGlite (pamięć)', group: 'memory', where: ['node', 'browser'], module: 'pglite', api: 'async', model: 'SQL', needs: [] },
  duckdb: { label: 'DuckDB-Wasm', group: 'memory', where: ['node', 'browser'], module: 'duckdb', api: 'sync', model: 'SQL', needs: [] },
  alasql: { label: 'AlaSQL', group: 'memory', where: ['node', 'browser'], module: 'alasql', api: 'sync', model: 'SQL', needs: [], wasm: false },
  'js-map': { label: 'czysty JS (Map)', group: 'memory', where: ['node', 'browser'], module: 'js-map', api: 'sync', model: 'kod JS', needs: [], wasm: false },

  // ---- group: durable (browser main thread, IndexedDB) ----
  'flatsql-idb': { label: 'Flat-SQL + IndexedDB', group: 'durable', where: ['browser'], module: 'flatsql', options: { store: 'idb', durability: 'default' }, api: 'async', model: 'SQL', needs: ['IndexedDB'] },
  'flatsql-idb-strict': { label: 'Flat-SQL + IndexedDB (strict)', group: 'durable', where: ['browser'], module: 'flatsql', options: { store: 'idb', durability: 'strict' }, api: 'async', model: 'SQL', needs: ['IndexedDB'] },
  'sqljs-idb': { label: 'sql.js + eksport do IndexedDB', group: 'durable', where: ['browser'], module: 'sqljs', options: { store: 'idb' }, api: 'async', model: 'SQL', needs: ['IndexedDB'] },
  'wa-sqlite-idb': { label: 'wa-sqlite + IDBBatchAtomicVFS', group: 'durable', where: ['browser'], module: 'wa-sqlite', options: { synchronous: 'full' }, api: 'async', model: 'SQL', needs: ['IndexedDB'] },
  'wa-sqlite-idb-relaxed': { label: 'wa-sqlite + IDBBatchAtomicVFS (synchronous=normal)', group: 'durable', where: ['browser'], module: 'wa-sqlite', options: { synchronous: 'normal' }, api: 'async', model: 'SQL', needs: ['IndexedDB'] },
  'pglite-idb': { label: 'PGlite + IndexedDB', group: 'durable', where: ['browser'], module: 'pglite', options: { fs: 'idb' }, api: 'async', model: 'SQL', needs: ['IndexedDB'] },
  dexie: { label: 'Dexie.js', group: 'durable', where: ['browser'], module: 'dexie', options: { durability: 'default' }, api: 'async', model: 'API IndexedDB', needs: ['IndexedDB'], wasm: false },
  'dexie-strict': { label: 'Dexie.js (strict)', group: 'durable', where: ['browser'], module: 'dexie', options: { durability: 'strict' }, api: 'async', model: 'API IndexedDB', needs: ['IndexedDB'], wasm: false },
  pouchdb: { label: 'PouchDB', group: 'durable', where: ['browser'], module: 'pouchdb', api: 'async', model: 'API dokumentowe', needs: ['IndexedDB'], wasm: false },
  rxdb: { label: 'RxDB (Dexie storage)', group: 'durable', where: ['browser'], module: 'rxdb', api: 'async', model: 'API dokumentowe', needs: ['IndexedDB'], wasm: false },

  // ---- group: worker (browser, dedicated Web Worker) ----
  'flatsql-worker': { label: 'Flat-SQL w Workerze (pamięć)', group: 'worker', where: ['worker'], module: 'flatsql', api: 'sync', model: 'SQL', needs: ['Worker'] },
  'flatsql-worker-idb': { label: 'Flat-SQL w Workerze + IndexedDB', group: 'worker', where: ['worker'], module: 'flatsql', options: { store: 'idb', durability: 'default' }, api: 'async', model: 'SQL', needs: ['Worker', 'IndexedDB'] },
  'sqlite-opfs': { label: 'SQLite WASM + opfs-sahpool', group: 'worker', where: ['worker'], module: 'sqlite-wasm', options: { vfs: 'opfs-sahpool' }, api: 'sync', model: 'SQL', needs: ['Worker', 'OPFS'] },
  'pglite-opfs': { label: 'PGlite + OPFS (opfs-ahp)', group: 'worker', where: ['worker'], module: 'pglite', options: { fs: 'opfs' }, api: 'async', model: 'SQL', needs: ['Worker', 'OPFS'] },
  'duckdb-worker': { label: 'DuckDB-Wasm w Workerze (pamięć)', group: 'worker', where: ['worker'], module: 'duckdb', api: 'sync', model: 'SQL', needs: ['Worker'] },
};

/** Engine configurations of group `group` that run in environment `where`. */
export const enginesIn = (where, group) => Object.keys(ENGINES).filter((id) => ENGINES[id].where.includes(where) && (!group || ENGINES[id].group === group));

/** Creates adapter `id`: imports and instantiates the engine (measured as part of the cold start). */
export async function createEngine(id, env) {
  const spec = ENGINES[id];
  if (!spec) throw new Error(`unknown engine ${id}`);
  const mod = await import(`./${spec.module}.js`);
  const engine = await mod.create(env, spec.options ?? {});
  engine.id = id;
  engine.durable = typeof engine.commit === 'function';
  return engine;
}
