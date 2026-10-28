// Node.js store: same file layout as the native FileStorage
// (`snapshot.fsql` + `wal.fsql`), so a directory written by the sync server
// can be opened with the CLI and vice versa.
//
// Durability follows FileStorage: every WAL append is fsynced (unless
// `fsync: false`), and a checkpoint is synced step by step whatever the
// option: snapshot.fsql.tmp + fsync -> rename -> fsync of the directory ->
// truncate the WAL. Truncating the WAL before the rename is durable could
// leave the old snapshot with an empty WAL after a crash.
//
// Like FileStorage, the store keeps the WAL file open for appending between
// checkpoints instead of opening and closing it for every append; the
// handle is closed before the WAL is truncated, by close() and destroy().

import fsp from 'node:fs/promises';
import { dirname, join } from 'node:path';

// A directory is flushed through a handle. On Windows it has to be opened for
// writing (libuv adds FILE_FLAG_BACKUP_SEMANTICS); elsewhere opening a
// directory for writing fails with EISDIR.
const DIR_FLAGS = process.platform === 'win32' ? 'r+' : 'r';

async function syncDir(dir) {
  const fh = await fsp.open(dir, DIR_FLAGS);
  try {
    await fh.sync();
  } finally {
    await fh.close();
  }
}

async function readOrNull(path) {
  try {
    return new Uint8Array(await fsp.readFile(path));
  } catch (err) {
    if (err.code === 'ENOENT') return null;
    throw err;
  }
}

function parseMeta(bytes) {
  if (!bytes) return {};
  try {
    return JSON.parse(new TextDecoder().decode(bytes));
  } catch {
    // meta.json written without fsync and cut by a crash: it holds only the
    // node id (also in the snapshot) and the sync cursor (a lost cursor means
    // a full pull, which is safe).
    return {};
  }
}

export class FsStore {
  /**
   * @param {string} dir database directory
   * @param {object} [options]
   * @param {boolean} [options.fsync] fsync every WAL append (default true);
   *   checkpoints are synced either way
   */
  #walHandle = null;

  constructor(dir, { fsync = true } = {}) {
    this.dir = dir;
    this.fsync = fsync;
    this.walSize = 0;
    this.snapshotSize = 0;
    this.meta = {};
  }

  get kind() {
    return 'fs';
  }

  get #wal() {
    return join(this.dir, 'wal.fsql');
  }

  async load() {
    // New directory entries are durable only after their directory is synced.
    const created = await fsp.mkdir(this.dir, { recursive: true });
    if (created !== undefined) await syncDir(dirname(created));
    const snapshot = await readOrNull(join(this.dir, 'snapshot.fsql'));
    const wal = await readOrNull(this.#wal);
    if (wal === null) {
      await (await fsp.open(this.#wal, 'a')).close();
      await syncDir(this.dir);
    }
    this.meta = parseMeta(await readOrNull(join(this.dir, 'meta.json')));
    this.walSize = wal ? wal.length : 0;
    this.snapshotSize = snapshot ? snapshot.length : 0;
    return { snapshot, wal: wal && wal.length ? wal : null, meta: { ...this.meta } };
  }

  async appendWal(bytes) {
    this.#walHandle ??= await fsp.open(this.#wal, 'a');
    try {
      await this.#walHandle.writeFile(bytes);
      if (this.fsync) await this.#walHandle.datasync();
    } catch (err) {
      // A failed write may leave the handle unusable; the next append opens
      // the file again.
      await this.#closeWal().catch(() => {});
      throw err;
    }
    this.walSize += bytes.length;
  }

  async #closeWal() {
    const fh = this.#walHandle;
    this.#walHandle = null;
    await fh?.close();
  }

  /** Closes the WAL file (FlatDB.close calls it). */
  async close() {
    await this.#closeWal();
  }

  async writeSnapshot(bytes) {
    await this.#replace(join(this.dir, 'snapshot.fsql'), bytes, true);
    await this.#closeWal();
    const fh = await fsp.open(this.#wal, 'w');
    try {
      if (this.fsync) await fh.sync();
    } finally {
      await fh.close();
    }
    this.walSize = 0;
    this.snapshotSize = bytes.length;
  }

  async setMeta(key, value) {
    this.meta[key] = value;
    await this.#replace(join(this.dir, 'meta.json'), JSON.stringify(this.meta, null, 2), this.fsync);
  }

  async getMeta(key) {
    return this.meta[key];
  }

  async destroy() {
    await this.#closeWal();
    await fsp.rm(this.dir, { recursive: true, force: true });
    this.walSize = 0;
    this.snapshotSize = 0;
    this.meta = {};
  }

  /** Atomic replace: write a temporary file, then rename it over `path`. */
  async #replace(path, data, durable) {
    const tmp = `${path}.tmp`;
    const fh = await fsp.open(tmp, 'w');
    try {
      await fh.writeFile(data);
      if (durable) await fh.sync();
    } finally {
      await fh.close();
    }
    await fsp.rename(tmp, path);
    if (durable) await syncDir(this.dir);
  }
}
