// One measured browser process: the runner (bench/browser/run.js) launches a
// fresh browser for every task and reads window.__bench when it is done.
//
// ?task={"kind": …, "engine": …, "rows": …} with kind
//   workload   – the whole workload once (bench/suite.js); engines of the
//                "worker" group run in bench/browser/worker.js;
//   cold       – cold start: download + compile + instantiate, open, first query;
//   memory     – WebAssembly memory and page memory after loading `rows` rows;
//   checkpoint – longest main-thread pause during a checkpoint (task.sizes);
//   network    – Local-First vs Cloud-First against task.syncUrl.

import { installWasmMemoryProbe } from '../wasm-memory.js';

const pageStart = performance.now(); // ms since navigation start
const task = JSON.parse(new URLSearchParams(location.search).get('task') ?? '{}');
const wasmMemory = task.kind === 'memory' ? installWasmMemoryProbe() : null;

const { ENGINES, createEngine } = await import('../engines/index.js');
const { READ_OPS, runWorkload } = await import('../suite.js');
const { browserEnv } = await import('./env.js');

const now = () => performance.now();
const logEl = document.getElementById('log');
const log = (m) => {
  logEl.textContent += `${m}\n`;
};

/** Adapter proxy: the engine lives in a Worker, every call is a round trip. */
async function workerEngine(id) {
  const w = new Worker(new URL('./worker.js', import.meta.url), { type: 'module' });
  let seq = 0;
  const pending = new Map();
  w.onmessage = ({ data }) => {
    const p = pending.get(data.id);
    pending.delete(data.id);
    if (data.error) p.reject(new Error(data.error));
    else p.resolve(data.result);
  };
  w.onerror = (e) => {
    for (const p of pending.values()) p.reject(new Error(`worker: ${e.message}`));
  };
  const rpc = (method, ...args) =>
    new Promise((resolve, reject) => {
      const n = ++seq;
      pending.set(n, { resolve, reject });
      w.postMessage({ id: n, method, args });
    });
  const info = await rpc('create', id);
  const engine = {
    id,
    async: true,
    version: info.version,
    durable: info.durable,
    open: (o) => rpc('open', o ?? {}),
    load: (rows) => rpc('load', rows),
    prepare: (op) => rpc('prepare', op.name),
    query: (h, params) => rpc('query', h, params),
    finalize: (h) => rpc('finalize', h),
    write: (op) => rpc('write', op.name),
    verify: () => rpc('verify'),
    async close(o) {
      await rpc('close', o ?? {});
    },
    terminate: () => w.terminate(),
    resources: () => rpc('resources'),
  };
  if (info.durable) {
    Object.assign(engine, {
      commit: (row) => rpc('commit', row),
      commitGroup: (rows) => rpc('commitGroup', rows),
      reopen: () => rpc('reopen'),
      checkpoint: () => rpc('checkpoint'),
    });
  }
  return engine;
}

const open = (id) => (ENGINES[id].group === 'worker' ? workerEngine(id) : createEngine(id, browserEnv('browser')));

/** Longest gap between event-loop turns: a MessageChannel ping-pong. */
function heartbeat() {
  const ch = new MessageChannel();
  let last = now();
  let max = 0;
  let over = 0;
  let running = true;
  const tick = () => {
    const t = now();
    const gap = t - last;
    if (gap > max) max = gap;
    if (gap > 1000 / 60) over++;
    last = t;
  };
  ch.port1.onmessage = () => {
    tick();
    if (running) ch.port2.postMessage(0);
  };
  ch.port2.postMessage(0);
  return {
    stop() {
      running = false;
      tick();
      ch.port1.close();
      return { maxGapMs: max, gapsOverFrame: over };
    },
  };
}

const call = async (r) => await r;

async function cold() {
  const t0 = now();
  const engine = await open(task.engine);
  const t1 = now();
  await call(engine.open({ fresh: true }));
  const t2 = now();
  await call(engine.load(1));
  const st = await call(engine.prepare(READ_OPS[0]));
  const first = await call(engine.query(st, [0], READ_OPS[0]));
  const t3 = now();
  const resources = performance.getEntriesByType('resource').map((r) => ({
    name: new URL(r.name).pathname,
    transferSize: r.transferSize,
    encodedBodySize: r.encodedBodySize,
    decodedBodySize: r.decodedBodySize,
    startMs: r.startTime,
    durationMs: r.duration,
  }));
  if (engine.resources) resources.push(...(await engine.resources()));
  await call(engine.finalize?.(st));
  await call(engine.close({ destroy: true }));
  return { pageStartMs: pageStart, initMs: t1 - t0, openMs: t2 - t1, firstQueryMs: t3 - t2, totalMs: t3 - t0, sinceNavigationMs: t3, firstRow: first[0]?.length ?? 0, resources };
}

async function pageMemory() {
  if (!crossOriginIsolated || !performance.measureUserAgentSpecificMemory) return null;
  const m = await performance.measureUserAgentSpecificMemory();
  return m.bytes;
}

async function memory() {
  const engine = await open(task.engine);
  await call(engine.open({ fresh: true }));
  const before = { wasm: wasmMemory().wasmBytes, page: await pageMemory() };
  await call(engine.load(task.rows));
  const after = { wasm: wasmMemory().wasmBytes, page: await pageMemory() };
  const result = { rows: task.rows, before, after, delta: { wasm: after.wasm - before.wasm, page: after.page - before.page }, memories: wasmMemory().memories };
  await call(engine.close({ destroy: true }));
  return result;
}

async function checkpoint() {
  const engine = await open(task.engine);
  const idle = heartbeat();
  await new Promise((r) => setTimeout(r, 200));
  const out = { idle: idle.stop(), sizes: [] };
  for (const rows of task.sizes) {
    await call(engine.open({ fresh: true }));
    await call(engine.load(rows));
    await new Promise((r) => setTimeout(r, 50));
    const hb = heartbeat();
    const t0 = now();
    const info = await engine.checkpoint();
    const totalMs = now() - t0;
    out.sizes.push({ rows, totalMs, ...hb.stop(), ...(info ?? {}) });
    await call(engine.close({ destroy: true }));
  }
  return out;
}

async function network() {
  const flat = await browserEnv().flatsql();
  const { runNetwork } = await import('../network.js');
  return { modes: await runNetwork({ baseUrl: task.syncUrl, Database: flat.Database, onProgress: log }) };
}

async function main() {
  const env = {
    userAgent: navigator.userAgent,
    crossOriginIsolated: globalThis.crossOriginIsolated,
    hardwareConcurrency: navigator.hardwareConcurrency,
    deviceMemory: navigator.deviceMemory ?? null,
  };
  let result;
  if (task.kind === 'workload') {
    const engine = await open(task.engine);
    result = await runWorkload(engine, { rows: task.rows, durable: engine.durable, onProgress: log });
    result.version = engine.version ?? null;
    engine.terminate?.();
  } else if (task.kind === 'cold') result = await cold();
  else if (task.kind === 'memory') result = await memory();
  else if (task.kind === 'checkpoint') result = await checkpoint();
  else if (task.kind === 'network') result = await network();
  else throw new Error(`unknown task kind ${task.kind}`);
  const flat = await browserEnv().flatsql();
  return { task, env, flatsqlVersion: flat.version(), result };
}

/** Manual mode (bench/browser/manual.html): show the result and offer it as a file. */
function offerDownload(r) {
  const blob = new Blob([JSON.stringify(r)], { type: 'application/json' });
  const a = Object.assign(document.createElement('a'), {
    href: URL.createObjectURL(blob),
    download: `${task.kind}-${task.engine}-${String(task.run ?? 1).padStart(2, '0')}.json`,
    textContent: 'Zapisz wynik (JSON)',
  });
  document.body.append(a, Object.assign(document.createElement('p'), { innerHTML: '<a href="manual.html' + location.search.replace(/[?&]task=[^&]*/, '') + '">← następne zadanie</a>' }));
  log(JSON.stringify(r.result, (k, v) => (k === 'resources' ? undefined : v), 1).slice(0, 4000));
}

main().then(
  (r) => {
    window.__bench = { ok: true, ...r };
    if (new URLSearchParams(location.search).has('manual')) offerDownload({ ok: true, ...r, started: new Date().toISOString() });
  },
  (err) => {
    console.error(err);
    window.__bench = { ok: false, error: String(err?.stack ?? err) };
  },
);
