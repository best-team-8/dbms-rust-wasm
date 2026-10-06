// Node.js entry point of the bridge: loads the .wasm binary from disk.

import { readFile } from 'node:fs/promises';
import { initEngine } from './engine.js';

export * from './index.js';
export { FsStore } from './stores/fs.js';

const wasmUrl = new URL('../pkg/flatsql_bg.wasm', import.meta.url);
await initEngine(await readFile(wasmUrl));
