// Node.js store: same file layout as the native FileStorage
// (`snapshot.fsql` + `wal.fsql`), so a directory written by the sync server
// can be opened with the CLI and vice versa.

import { appendFile, mkdir, readFile, rename, rm, writeFile } from 'node:fs/promises';
import { join } from 'node:path';

async function readOrNull(path) {
  try {
    return new Uint8Array(await readFile(path));
  } catch (err) {
    if (err.code === 'ENOENT') return null;
    throw err;
  }
}

async function writeAtomic(path, data) {
  const tmp = `${path}.tmp`;
  await writeFile(tmp, data);
  await rename(tmp, path);
}

export class FsStore {
  constructor(dir) {
    this.dir = dir;
    this.walSize = 0;
    this.snapshotSize = 0;
    this.meta = {};
  }

  get kind() {
    return 'fs';
  }

  async load() {
    await mkdir(this.dir, { recursive: true });
    const snapshot = await readOrNull(join(this.dir, 'snapshot.fsql'));
    const wal = await readOrNull(join(this.dir, 'wal.fsql'));
    const meta = await readOrNull(join(this.dir, 'meta.json'));
    this.meta = meta ? JSON.parse(new TextDecoder().decode(meta)) : {};
    this.walSize = wal ? wal.length : 0;
    this.snapshotSize = snapshot ? snapshot.length : 0;
    return { snapshot, wal: wal && wal.length ? wal : null, meta: { ...this.meta } };
  }

  async appendWal(bytes) {
    await appendFile(join(this.dir, 'wal.fsql'), bytes);
    this.walSize += bytes.length;
  }

  async writeSnapshot(bytes) {
    await writeAtomic(join(this.dir, 'snapshot.fsql'), bytes);
    await writeFile(join(this.dir, 'wal.fsql'), new Uint8Array(0));
    this.walSize = 0;
    this.snapshotSize = bytes.length;
  }

  async setMeta(key, value) {
    this.meta[key] = value;
    await writeAtomic(join(this.dir, 'meta.json'), JSON.stringify(this.meta, null, 2));
  }

  async getMeta(key) {
    return this.meta[key];
  }

  async destroy() {
    await rm(this.dir, { recursive: true, force: true });
    this.walSize = 0;
    this.snapshotSize = 0;
    this.meta = {};
  }
}
