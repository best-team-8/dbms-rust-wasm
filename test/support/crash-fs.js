// Simulated file system for FsStore crash tests, the same model as
// crates/flatsql-core/tests/checkpoint.rs. Calls to node:fs/promises for a
// simulated directory (and the files in it) are served from memory and
// recorded as operations; every other path goes to the real file system.
// A crash state is built from the operations the file system could have
// persisted: only fsync of a file (its writes) and fsync of the directory
// (created and renamed files) force earlier operations to disk.

import fsp from 'node:fs/promises';
import { syncBuiltinESMExports } from 'node:module';
import { basename, dirname, resolve } from 'node:path';
import { mock } from 'node:test';

function bytesOf(data) {
  if (typeof data === 'string') return new TextEncoder().encode(data);
  return new Uint8Array(data.buffer, data.byteOffset, data.byteLength).slice();
}

function enoent(path) {
  return Object.assign(new Error(`ENOENT: no such file or directory, '${path}'`), { code: 'ENOENT' });
}

/** Simulated directories by resolved path. */
const active = new Map();
let installed = false;

/** [sim, file name] for a path inside a simulated directory, [sim, null] for the directory itself. */
function lookup(path) {
  const p = resolve(String(path));
  if (active.has(p)) return [active.get(p), null];
  if (active.has(dirname(p))) return [active.get(dirname(p)), basename(p)];
  return [null, null];
}

// The functions are replaced once and stay replaced: paths outside simulated
// directories go to the real file system anyway, and restoring node:test
// mocks gets slower with every mock ever created.
function installMocks() {
  if (installed) return;
  installed = true;
  const real = Object.fromEntries(['mkdir', 'readFile', 'open', 'rename', 'rm', 'appendFile', 'writeFile'].map((k) => [k, fsp[k]]));
  const route = (name, simulated) =>
    mock.method(fsp, name, async (path, ...args) => {
      const [sim, file] = lookup(path);
      return sim ? simulated(sim, file, path, ...args) : real[name].call(fsp, path, ...args);
    });
  route('mkdir', (sim) => sim.mkdir());
  route('readFile', (sim, file, path) => sim.readFile(file, path));
  route('open', (sim, file, path, flags = 'r') => sim.open(file, path, flags));
  route('rename', (sim, file, path, to) => sim.rename(file, basename(String(to)), path));
  route('rm', (sim) => sim.rm());
  route('appendFile', (sim, file, path, data) => sim.appendFile(file, data));
  route('writeFile', (sim, file, path, data) => sim.writeFile(file, data));
  // Named imports of built-in modules follow the exports object only after this.
  syncBuiltinESMExports();
}

export class SimFs {
  /** A simulated directory; `files` (Map name -> bytes) makes it exist with that content. */
  constructor(dir, files = null) {
    this.dir = resolve(dir);
    this.exists = files !== null;
    this.names = new Map();
    this.data = [];
    this.trace = [];
    for (const [name, bytes] of files ?? []) {
      this.names.set(name, this.data.length);
      this.data.push(bytes.slice());
    }
  }

  install() {
    active.set(this.dir, this);
    installMocks();
    return this;
  }

  uninstall() {
    active.delete(this.dir);
  }

  /** Current contents (what a clean reopen would see). */
  files() {
    return new Map([...this.names].map(([name, inode]) => [name, this.data[inode].slice()]));
  }

  // ---- node:fs/promises operations --------------------------------------

  // Creating the directory itself is not modelled: it is reported as already
  // existing, so the store does not sync its (real) parent.
  mkdir() {
    this.exists = true;
    return undefined;
  }

  readFile(file, path) {
    const inode = this.names.get(file);
    if (inode === undefined) throw enoent(path);
    return Buffer.from(this.data[inode]);
  }

  open(file, path, flags) {
    if (!this.exists) throw enoent(path);
    if (file === null) {
      const sync = async () => void this.trace.push({ op: 'syncDir' });
      return { sync, datasync: sync, close: async () => {} };
    }
    let inode = this.names.get(file);
    if (inode === undefined) {
      if (!/[wa]/.test(flags)) throw enoent(path);
      inode = this.#link(file);
    } else if (flags.includes('w')) {
      this.#setLen(inode, 0);
    }
    const append = flags.includes('a');
    let pos = 0;
    const write = async (data) => {
      const bytes = bytesOf(data);
      const offset = append ? this.data[inode].length : pos;
      this.#write(inode, offset, bytes);
      pos = offset + bytes.length;
      return { bytesWritten: bytes.length };
    };
    const sync = async () => void this.trace.push({ op: 'sync', inode });
    return {
      write,
      writeFile: write,
      truncate: async (len = 0) => this.#setLen(inode, len),
      sync,
      datasync: sync,
      close: async () => {},
    };
  }

  rename(from, to, path) {
    const inode = this.names.get(from);
    if (inode === undefined) throw enoent(path);
    this.names.delete(from);
    this.names.set(to, inode);
    this.trace.push({ op: 'rename', from, to, inode });
  }

  rm() {
    this.exists = false;
    this.names.clear();
  }

  appendFile(file, data) {
    const inode = this.names.get(file) ?? this.#link(file);
    this.#write(inode, this.data[inode].length, bytesOf(data));
  }

  writeFile(file, data) {
    let inode = this.names.get(file);
    if (inode === undefined) inode = this.#link(file);
    else this.#setLen(inode, 0);
    this.#write(inode, 0, bytesOf(data));
  }

  #link(name) {
    const inode = this.data.length;
    this.data.push(new Uint8Array(0));
    this.names.set(name, inode);
    this.trace.push({ op: 'link', name, inode });
    return inode;
  }

  #write(inode, offset, bytes) {
    if (bytes.length === 0) return;
    const old = this.data[inode];
    const out = new Uint8Array(offset + bytes.length);
    out.set(old.subarray(0, offset));
    out.set(bytes, offset);
    this.data[inode] = out;
    this.trace.push({ op: 'write', inode, offset, bytes });
  }

  #setLen(inode, len) {
    const out = new Uint8Array(len);
    out.set(this.data[inode].subarray(0, len));
    this.data[inode] = out;
    this.trace.push({ op: 'setLen', inode, len });
  }
}

const isDirOp = (op) => op.op === 'link' || op.op === 'rename';
const contentInode = (op) => (op.op === 'write' || op.op === 'setLen' ? op.inode : undefined);

/** Operations of trace[0..crash) that a completed fsync forced to disk. */
export function forced(trace, crash) {
  const out = new Array(crash).fill(false);
  for (let j = 0; j < crash; j++) {
    const op = trace[j];
    for (let i = 0; i < j; i++) {
      if (op.op === 'sync' && contentInode(trace[i]) === op.inode) out[i] = true;
      if (op.op === 'syncDir' && isDirOp(trace[i])) out[i] = true;
    }
  }
  return out;
}

/**
 * Directory contents after a crash: persisted[i] is how many bytes of
 * operation i reached the disk (null: none; any number for other operations).
 */
export function crashState(trace, persisted) {
  const names = new Map();
  const data = new Map();
  trace.forEach((op, i) => {
    const p = persisted[i];
    if (p === null || p === undefined) return;
    if (op.op === 'link') {
      names.set(op.name, op.inode);
      if (!data.has(op.inode)) data.set(op.inode, new Uint8Array(0));
    } else if (op.op === 'rename') {
      if (names.get(op.from) === op.inode) names.delete(op.from);
      names.set(op.to, op.inode);
    } else if (op.op === 'write') {
      const d = data.get(op.inode) ?? new Uint8Array(0);
      const out = new Uint8Array(Math.max(d.length, op.offset + p));
      out.set(d);
      out.set(op.bytes.subarray(0, p), op.offset);
      data.set(op.inode, out);
    } else if (op.op === 'setLen') {
      const d = data.get(op.inode) ?? new Uint8Array(0);
      const out = new Uint8Array(op.len);
      out.set(d.subarray(0, op.len));
      data.set(op.inode, out);
    }
  });
  return new Map([...names].map(([name, inode]) => [name, data.get(inode) ?? new Uint8Array(0)]));
}

/** The worst case: only operations forced by fsync reached the disk. */
export function minimalState(trace, crash) {
  const f = forced(trace, crash);
  return crashState(
    trace.slice(0, crash),
    f.map((x, i) => (x ? (trace[i].bytes?.length ?? 0) : null)),
  );
}

/** mulberry32: deterministic randomness with a fixed seed. */
export function rng(seed) {
  let a = seed >>> 0;
  return (n) => {
    a = (a + 0x6d2b79f5) >>> 0;
    let t = a;
    t = Math.imul(t ^ (t >>> 15), t | 1);
    t ^= t + Math.imul(t ^ (t >>> 7), t | 61);
    return ((t ^ (t >>> 14)) >>> 0) % n;
  };
}

// How much of a not forced operation may be on disk: nothing or all of it;
// a write also half of its bytes (torn).
function choices(op) {
  if (op.op !== 'write') return [null, 0];
  return op.bytes.length > 1 ? [null, op.bytes.length >> 1, op.bytes.length] : [null, op.bytes.length];
}

/**
 * Calls f(persisted) for every crash state the model permits after `crash`
 * operations; above `max` states the combinations are drawn with `random`.
 * Returns { total, sampled }.
 */
export async function forEachCrashState(trace, crash, { max, random }, f) {
  const isForced = forced(trace, crash);
  const base = trace.slice(0, crash).map((op, i) => (isForced[i] ? (op.bytes?.length ?? 0) : null));
  // Barriers change nothing on disk, so only the other operations vary.
  const free = [];
  for (let i = 0; i < crash; i++) {
    if (!isForced[i] && trace[i].op !== 'sync' && trace[i].op !== 'syncDir') free.push([i, choices(trace[i])]);
  }
  const total = free.reduce((n, [, c]) => Math.min(n * c.length, Number.MAX_SAFE_INTEGER), 1);
  const build = (digits) => {
    const p = base.slice();
    free.forEach(([i, c], k) => (p[i] = c[digits[k]]));
    return p;
  };
  if (total <= max) {
    const digits = new Array(free.length).fill(0);
    for (;;) {
      await f(build(digits));
      const pos = digits.findIndex((d, k) => d + 1 < free[k][1].length);
      if (pos < 0) break;
      digits[pos]++;
      digits.fill(0, 0, pos);
    }
    return { total, sampled: false };
  }
  for (let s = 0; s < max; s++) await f(build(free.map(([, c]) => random(c.length))));
  return { total, sampled: true };
}

/** A key identifying directory contents. */
export function contentKey(files) {
  return [...files].map(([name, bytes]) => `${name}:${Buffer.from(bytes).toString('base64')}`).join('|');
}
