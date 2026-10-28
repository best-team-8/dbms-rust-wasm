// Main-thread time of writing a snapshot of the benchmark size (4.5 MB) to
// IndexedDB in Edge: the synchronous part of put() for a Uint8Array and for
// a Blob made from it (the time of new Blob() included), and the longest gap
// of a MessageChannel heartbeat while the transaction runs. Median of 9.
//
//   node docs/research/data/optimization/15-checkpoint-steps/idb-put.mjs

import { createServer } from 'node:http';
import { chromium } from 'playwright-core';

const page = `<!doctype html><script type="module">
const bytes = new Uint8Array(4_570_857).map((_, i) => (i * 2654435761) >>> 24);
const open = () => new Promise((ok, fail) => {
  const r = indexedDB.open('put-test', 1);
  r.onupgradeneeded = () => r.result.createObjectStore('s');
  r.onsuccess = () => ok(r.result);
  r.onerror = () => fail(r.error);
});
const db = await open();
function heartbeat() {
  const ch = new MessageChannel();
  let last = performance.now(), max = 0, run = true;
  ch.port1.onmessage = () => { const t = performance.now(); max = Math.max(max, t - last); last = t; if (run) ch.port2.postMessage(0); };
  ch.port2.postMessage(0);
  return () => { run = false; return max; };
}
async function once(kind) {
  await new Promise((r) => setTimeout(r, 30));
  const stop = heartbeat();
  await new Promise((r) => setTimeout(r, 5));
  const tx = db.transaction(['s'], 'readwrite');
  const t0 = performance.now();
  tx.objectStore('s').put(kind === 'blob' ? new Blob([bytes]) : bytes, 'main');
  const sync = performance.now() - t0;
  await new Promise((ok) => (tx.oncomplete = ok));
  const total = performance.now() - t0;
  return { sync, gap: stop(), total };
}
const out = {};
for (const kind of ['uint8array', 'blob']) {
  const runs = [];
  for (let i = 0; i < 9; i++) runs.push(await once(kind));
  const med = (k) => runs.map((r) => r[k]).sort((a, b) => a - b)[4];
  out[kind] = { syncMs: med('sync'), maxGapMs: med('gap'), totalMs: med('total') };
}
window.result = out;
</script>`;

const server = createServer((req, res) => res.writeHead(200, { 'content-type': 'text/html' }).end(page)).listen(0);
const browser = await chromium.launch({ channel: 'msedge' });
const tab = await browser.newPage();
await tab.goto(`http://127.0.0.1:${server.address().port}/`);
const result = await tab.waitForFunction(() => window.result, null, { timeout: 120_000 }).then((h) => h.jsonValue());
console.log(`Edge ${browser.version()}, snapshot of 4 570 857 bytes, median of 9`);
for (const [kind, r] of Object.entries(result)) console.log(`${kind.padEnd(10)} put() ${r.syncMs.toFixed(2)} ms, longest heartbeat gap ${r.maxGapMs.toFixed(2)} ms, until complete ${r.totalMs.toFixed(1)} ms`);
await browser.close();
server.close();
