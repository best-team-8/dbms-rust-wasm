// Simulated crashes for FsStore. Calls to node:fs/promises inside one
// directory are recorded as file system operations; the real files are still
// written, so the store works normally. A crash state is then built from the
// operations the file system could have persisted: only fsync of a file
// (its writes) and fsync of the directory (created and renamed files) force
// earlier operations to disk. Same model as crates/flatsql-core/tests/checkpoint.rs.

import fsp from 'node:fs/promises';
import { syncBuiltinESMExports } from 'node:module';
import { basename, join, resolve } from 'node:path';
import { mock } from 'node:test';

function bytesOf(data) {
  if (typeof data === 'string') return new TextEncoder().encode(data);
  return new Uint8Array(data.buffer, data.byteOffset, data.byteLength).slice();
}

export class Recorder {
  constructor(dir) {
    this.dir = resolve(dir);
    this.names = new Map();
    this.data = [];
    this.trace = [];
    this.recording = false;
  }

  /** Wraps the node:fs/promises functions used by FsStore. */
  install() {
    const real = { open: fsp.open, rename: fsp.rename, appendFile: fsp.appendFile, writeFile: fsp.writeFile };
    const rec = this;
    mock.method(fsp, 'open', async (path, flags = 'r', ...rest) => {
      const fh = await real.open.call(fsp, path, flags, ...rest);
      const p = resolve(String(path));
      if (!rec.recording) return fh;
      if (p === rec.dir) return rec.#dirHandle(fh);
      if (!rec.#inside(p)) return fh;
      let inode = rec.names.get(basename(p));
      if (inode === undefined) inode = rec.#link(basename(p));
      else if (flags.includes('w')) rec.#setLen(inode, 0);
      return rec.#fileHandle(fh, inode, flags.includes('a'));
    });
    mock.method(fsp, 'rename', async (from, to) => {
      await real.rename.call(fsp, from, to);
      if (!rec.recording || !rec.#inside(resolve(String(from)))) return;
      const [f, t] = [basename(String(from)), basename(String(to))];
      const inode = rec.names.get(f);
      rec.names.delete(f);
      rec.names.set(t, inode);
      rec.trace.push({ op: 'rename', from: f, to: t, inode });
    });
    mock.method(fsp, 'appendFile', async (path, data, ...rest) => {
      await real.appendFile.call(fsp, path, data, ...rest);
      const p = resolve(String(path));
      if (!rec.recording || !rec.#inside(p)) return;
      const inode = rec.names.get(basename(p)) ?? rec.#link(basename(p));
      rec.#write(inode, rec.data[inode].length, bytesOf(data));
    });
    mock.method(fsp, 'writeFile', async (path, data, ...rest) => {
      await real.writeFile.call(fsp, path, data, ...rest);
      const p = resolve(String(path));
      if (!rec.recording || !rec.#inside(p)) return;
      let inode = rec.names.get(basename(p));
      if (inode === undefined) inode = rec.#link(basename(p));
      else rec.#setLen(inode, 0);
      rec.#write(inode, 0, bytesOf(data));
    });
    // Named imports of built-in modules follow the exports object only after this.
    syncBuiltinESMExports();
  }

  uninstall() {
    mock.restoreAll();
    syncBuiltinESMExports();
  }

  #inside(p) {
    return resolve(p, '..') === this.dir;
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
    const out = new Uint8Array(Math.max(old.length, offset + bytes.length));
    out.set(old.subarray(0, offset));
    out.set(bytes, offset);
    this.data[inode] = out.subarray(0, offset + bytes.length);
    this.trace.push({ op: 'write', inode, offset, bytes });
  }

  #setLen(inode, len) {
    const old = this.data[inode];
    const out = new Uint8Array(len);
    out.set(old.subarray(0, len));
    this.data[inode] = out;
    this.trace.push({ op: 'setLen', inode, len });
  }

  #fileHandle(fh, inode, append) {
    const rec = this;
    let pos = 0;
    return new Proxy(fh, {
      get(target, prop) {
        if (prop === 'writeFile' || prop === 'write') {
          return async (data, ...args) => {
            const r = await target[prop](data, ...args);
            const bytes = bytesOf(data);
            const offset = append ? rec.data[inode].length : pos;
            rec.#write(inode, offset, bytes);
            pos = offset + bytes.length;
            return r;
          };
        }
        if (prop === 'truncate') {
          return async (len = 0) => {
            const r = await target.truncate(len);
            rec.#setLen(inode, len);
            return r;
          };
        }
        if (prop === 'sync' || prop === 'datasync') {
          return async () => {
            const r = await target[prop]();
            rec.trace.push({ op: 'sync', inode });
            return r;
          };
        }
        const v = Reflect.get(target, prop, target);
        return typeof v === 'function' ? v.bind(target) : v;
      },
    });
  }

  #dirHandle(fh) {
    const rec = this;
    return new Proxy(fh, {
      get(target, prop) {
        if (prop === 'sync' || prop === 'datasync') {
          return async () => {
            const r = await target[prop]();
            rec.trace.push({ op: 'syncDir' });
            return r;
          };
        }
        const v = Reflect.get(target, prop, target);
        return typeof v === 'function' ? v.bind(target) : v;
      },
    });
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
      const end = op.offset + p;
      const out = new Uint8Array(Math.max(d.length, end));
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

export async function materialize(files, dir) {
  await fsp.mkdir(dir, { recursive: true });
  for (const [name, bytes] of files) await fsp.writeFile(join(dir, name), bytes);
}
