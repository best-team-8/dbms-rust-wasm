// Manual mode of the browser benchmark: starts the benchmark server and a
// sync server and prints the address of bench/browser/manual.html, where a
// task is chosen by hand (another browser, a phone).
//
//   node bench/browser/serve.js [--flatsql DIR] [--port 8090] [--sync-port 8091]
//
// Cross-origin isolation, OPFS and the precise timer need a secure context:
// http://127.0.0.1 or http://localhost. On an Android phone connected by USB:
//   adb reverse tcp:8090 tcp:8090 && adb reverse tcp:8091 tcp:8091
// then open http://localhost:8090/bench/browser/manual.html in Chrome.

import { resolve, join } from 'node:path';
import { fileURLToPath, pathToFileURL } from 'node:url';

import { buildVendor } from './build.js';
import { startBenchServer } from './server.js';

const ROOT = fileURLToPath(new URL('../..', import.meta.url));
const args = process.argv.slice(2);
const opt = (name, def) => (args.includes(name) ? args[args.indexOf(name) + 1] : def);
const flatsqlRoot = resolve(opt('--flatsql', ROOT));

await buildVendor();
const server = await startBenchServer({ flatsqlRoot, port: Number(opt('--port', 8090)) });
const { createServer } = await import(pathToFileURL(join(flatsqlRoot, 'server', 'server.js')).href);
const sync = await createServer({ port: Number(opt('--sync-port', 8091)), quiet: true });
console.log(`benchmark: ${server.url}/bench/browser/manual.html?sync=${encodeURIComponent(sync.url)}`);
console.log(`Flat-SQL:  ${flatsqlRoot}`);
