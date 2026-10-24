// One measured process (Node.js). The runner (bench/run.js) starts a fresh
// process for every repetition, so every repetition pays for module loading,
// JIT warm-up and garbage collection anew – the unit of repetition is the
// process, not the loop inside it [B1].
//
//   node bench/node/child.js '<json task>'
//
// task: { kind, engine, rows, flatsqlRoot, out }
//   kind = workload  – the whole workload (bench/suite.js) once;
//          cold      – cold start: import + instantiate, open, first query;
//          memory    – memory after loading `rows` rows (needs --expose-gc);
//          network   – Local-First vs Cloud-First (bench/network.js);
//          fsync     – cost of durable commits in FsStore (bench/node/fsync.js).

import { writeFile } from 'node:fs/promises';
import { createRequire } from 'node:module';
import { join } from 'node:path';
import { pathToFileURL } from 'node:url';

import { createEngine } from '../engines/index.js';
import { READ_OPS, WORKLOAD, runWorkload } from '../suite.js';
import { installWasmMemoryProbe } from '../wasm-memory.js';

const task = JSON.parse(process.argv[2]);
const require = createRequire(import.meta.url);
const now = () => performance.now();

const env = {
  kind: 'node',
  flatsql: () => import(pathToFileURL(join(task.flatsqlRoot, 'bridge', 'node.js')).href),
  importPkg: (spec) => import(spec),
  path: (spec) => require.resolve(spec),
};

function memoryNow(wasm) {
  globalThis.gc();
  globalThis.gc();
  const m = process.memoryUsage();
  return { rss: m.rss, heapUsed: m.heapUsed, external: m.external, arrayBuffers: m.arrayBuffers, wasm: wasm().wasmBytes };
}

async function cold() {
  const call = async (r) => await r;
  const processMs = now(); // time since the process started (Node.js boot)
  const t0 = now();
  const engine = await createEngine(task.engine, env);
  const t1 = now();
  await call(engine.open({ fresh: true }));
  const t2 = now();
  await call(engine.load(1));
  const st = await call(engine.prepare(READ_OPS[0]));
  const first = await call(engine.query(st, [0], READ_OPS[0]));
  const t3 = now();
  const result = {
    processBootMs: processMs,
    initMs: t1 - t0,
    openMs: t2 - t1,
    firstQueryMs: t3 - t2,
    totalMs: t3 - t0,
    firstRow: first[0]?.length ?? 0,
  };
  await call(engine.finalize?.(st));
  await call(engine.close({ destroy: true }));
  return result;
}

// Memory of a database with `rows` rows, after full garbage collection, in a
// process of its own (nothing left over from other instances).
async function memory() {
  const call = async (r) => await r;
  const wasm = installWasmMemoryProbe();
  const engine = await createEngine(task.engine, env);
  await call(engine.open({ fresh: true }));
  const before = memoryNow(wasm);
  await call(engine.load(task.rows));
  const after = memoryNow(wasm);
  const delta = Object.fromEntries(Object.keys(after).map((k) => [k, after[k] - before[k]]));
  const result = { rows: task.rows, before, after, delta, memories: wasm().memories };
  await call(engine.close({ destroy: true }));
  return result;
}

async function network() {
  const flat = await env.flatsql();
  const { createServer } = await import(pathToFileURL(join(task.flatsqlRoot, 'server', 'server.js')).href);
  const { runNetwork, nodeFetchFactory } = await import('../network.js');
  const fetch = await nodeFetchFactory();
  const srv = await createServer({ port: 0, quiet: true });
  try {
    return { modes: await runNetwork({ baseUrl: srv.url, Database: flat.Database, fetch }) };
  } finally {
    fetch.close();
    await srv.close();
  }
}

async function main() {
  const started = new Date().toISOString();
  let result;
  if (task.kind === 'workload') {
    const engine = await createEngine(task.engine, env);
    result = await runWorkload(engine, { rows: task.rows, now });
    result.version = engine.version ?? null;
  } else if (task.kind === 'cold') {
    result = await cold();
  } else if (task.kind === 'memory') {
    result = await memory();
  } else if (task.kind === 'network') {
    result = await network();
  } else if (task.kind === 'fsync') {
    const { runFsync } = await import('./fsync.js');
    result = await runFsync({ env, rows: task.rows, root: task.flatsqlRoot });
  } else {
    throw new Error(`unknown task kind ${task.kind}`);
  }
  const flat = await env.flatsql();
  const out = { task, workload: WORKLOAD, started, flatsqlVersion: flat.version(), node: process.version, result };
  await writeFile(task.out, JSON.stringify(out));
}

main().catch((err) => {
  console.error(err?.stack ?? err);
  process.exit(1);
});
