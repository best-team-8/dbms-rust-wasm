// Web Worker hosting one engine adapter for the "worker" group. The main
// thread drives the workload; every adapter call is a postMessage round trip
// (structured clone of parameters and results), which is the cost an
// application pays for moving the database off the main thread.

import { createEngine } from '../engines/index.js';
import { READ_OPS, WRITE_OPS } from '../suite.js';
import { browserEnv } from './env.js';

const OPS = Object.fromEntries([...READ_OPS, ...WRITE_OPS].map((op) => [op.name, op]));
let engine = null;
const handles = new Map();
let nextHandle = 1;

const methods = {
  async create(id) {
    engine = await createEngine(id, browserEnv('worker'));
    return { durable: engine.durable, version: engine.version ?? null };
  },
  open: (o) => engine.open(o),
  load: (rows) => engine.load(rows),
  async prepare(name) {
    const h = nextHandle++;
    handles.set(h, { st: await engine.prepare(OPS[name]), op: OPS[name] });
    return h;
  },
  query(h, params) {
    const { st, op } = handles.get(h);
    return engine.query(st, params, op);
  },
  async finalize(h) {
    await engine.finalize?.(handles.get(h).st);
    handles.delete(h);
  },
  write: (name) => engine.write(OPS[name]),
  verify: () => engine.verify(),
  commit: (row) => engine.commit(row),
  commitGroup: (rows) => engine.commitGroup(rows),
  reopen: () => engine.reopen(),
  checkpoint: () => engine.checkpoint?.(),
  close: (o) => engine.close(o),
};

self.onmessage = async ({ data: { id, method, args } }) => {
  try {
    const result = await methods[method](...args);
    self.postMessage({ id, result });
  } catch (err) {
    self.postMessage({ id, error: String(err?.stack ?? err) });
  }
};
