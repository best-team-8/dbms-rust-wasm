// WASM-Bridge: JavaScript API of the Flat-SQL engine.
//
// Works unchanged in the browser and in Node.js (>= 18). In Node, import
// `bridge/node.js` instead – it loads the .wasm file from disk first.

export { Database, describeChanges, maxDriftMs, version } from '../pkg/flatsql.js';
export { initEngine, engineReady } from './engine.js';
export { FlatDB } from './flatdb.js';
export { MemoryStore } from './stores/memory.js';
export { IndexedDbStore } from './stores/indexeddb.js';
export { ClockError, SyncClient } from './sync.js';
