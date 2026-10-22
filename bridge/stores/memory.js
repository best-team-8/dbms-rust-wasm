// Volatile store: keeps snapshot and WAL in memory. Reusing the same instance
// for FlatDB.open() simulates closing and reopening the application.

function concat(chunks, size) {
  const out = new Uint8Array(size);
  let off = 0;
  for (const c of chunks) {
    out.set(c, off);
    off += c.length;
  }
  return out;
}

export class MemoryStore {
  constructor() {
    this.snapshot = null;
    this.walChunks = [];
    this.walSize = 0;
    this.snapshotSize = 0;
    this.meta = {};
  }

  get kind() {
    return 'memory';
  }

  async load() {
    return {
      snapshot: this.snapshot,
      wal: this.walSize ? concat(this.walChunks, this.walSize) : null,
      meta: { ...this.meta },
    };
  }

  async appendWal(bytes) {
    this.walChunks.push(bytes.slice());
    this.walSize += bytes.length;
  }

  async writeSnapshot(bytes) {
    this.snapshot = bytes.slice();
    this.snapshotSize = bytes.length;
    this.walChunks = [];
    this.walSize = 0;
  }

  async setMeta(key, value) {
    this.meta[key] = value;
  }

  async getMeta(key) {
    return this.meta[key];
  }

  async destroy() {
    this.snapshot = null;
    this.walChunks = [];
    this.walSize = 0;
    this.snapshotSize = 0;
    this.meta = {};
  }
}
