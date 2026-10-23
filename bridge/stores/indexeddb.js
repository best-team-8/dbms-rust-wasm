// Browser store on top of IndexedDB.
//
// Object stores:
//   snapshot – a single record ("main") with the latest checkpoint
//   wal      – WAL chunks with auto-increment keys (append order)
//   meta     – small key/value records (node id, sync cursor, ...)
//
// A checkpoint replaces the snapshot and clears the WAL in ONE readwrite
// transaction, so the two can never get out of step.

function request(req) {
  return new Promise((resolve, reject) => {
    req.onsuccess = () => resolve(req.result);
    req.onerror = () => reject(req.error);
  });
}

function done(tx) {
  return new Promise((resolve, reject) => {
    tx.oncomplete = () => resolve();
    tx.onerror = () => reject(tx.error);
    tx.onabort = () => reject(tx.error ?? new Error('IndexedDB transaction aborted'));
  });
}

function concat(chunks) {
  let size = 0;
  for (const c of chunks) size += c.byteLength;
  const out = new Uint8Array(size);
  let off = 0;
  for (const c of chunks) {
    out.set(new Uint8Array(c.buffer ?? c, c.byteOffset ?? 0, c.byteLength), off);
    off += c.byteLength;
  }
  return out;
}

export class IndexedDbStore {
  #db = null;

  /**
   * @param {string} name database name (one IndexedDB database per FlatDB)
   * @param {object} [options]
   * @param {'default'|'strict'|'relaxed'} [options.durability] IndexedDB
   *   transaction durability hint used for WAL appends and checkpoints:
   *   'strict' completes after the data reached the disk, 'relaxed' after it
   *   was handed to the operating system, 'default' lets the browser choose
   *   (relaxed in Chrome 121+).
   */
  constructor(name = 'default', { durability = 'default' } = {}) {
    this.name = name;
    this.dbName = `flatsql:${name}`;
    this.durability = durability;
    this.walSize = 0;
    this.snapshotSize = 0;
  }

  get kind() {
    return 'indexeddb';
  }

  #open() {
    if (!this.#db) {
      this.#db = new Promise((resolve, reject) => {
        const req = indexedDB.open(this.dbName, 1);
        req.onupgradeneeded = () => {
          const db = req.result;
          db.createObjectStore('snapshot');
          db.createObjectStore('wal', { autoIncrement: true });
          db.createObjectStore('meta');
        };
        req.onsuccess = () => {
          const db = req.result;
          // Let other connections delete or upgrade the database instead of
          // blocking them forever.
          db.onversionchange = () => {
            db.close();
            this.#db = null;
          };
          resolve(db);
        };
        req.onerror = () => reject(req.error);
        req.onblocked = () => reject(new Error(`IndexedDB "${this.dbName}" is blocked by another tab`));
      });
    }
    return this.#db;
  }

  /** Closes the connection (it is reopened on the next operation). */
  async close() {
    if (this.#db) {
      const db = await this.#db;
      this.#db = null;
      db.close();
    }
  }

  #tx(db, stores, mode) {
    return db.transaction(stores, mode, { durability: this.durability });
  }

  async load() {
    const db = await this.#open();
    const tx = this.#tx(db, ['snapshot', 'wal', 'meta'], 'readonly');
    const [snapshot, wal, metaKeys, metaValues] = await Promise.all([
      request(tx.objectStore('snapshot').get('main')),
      request(tx.objectStore('wal').getAll()),
      request(tx.objectStore('meta').getAllKeys()),
      request(tx.objectStore('meta').getAll()),
      done(tx),
    ]);
    const meta = {};
    metaKeys.forEach((k, i) => (meta[k] = metaValues[i]));
    const walBytes = wal.length ? concat(wal) : null;
    this.walSize = walBytes ? walBytes.length : 0;
    this.snapshotSize = snapshot ? snapshot.byteLength : 0;
    return { snapshot: snapshot ? new Uint8Array(snapshot) : null, wal: walBytes, meta };
  }

  async appendWal(bytes) {
    const db = await this.#open();
    const tx = this.#tx(db, ['wal'], 'readwrite');
    tx.objectStore('wal').add(bytes);
    await done(tx);
    this.walSize += bytes.length;
  }

  async writeSnapshot(bytes) {
    const db = await this.#open();
    const tx = this.#tx(db, ['snapshot', 'wal'], 'readwrite');
    tx.objectStore('snapshot').put(bytes, 'main');
    tx.objectStore('wal').clear();
    await done(tx);
    this.walSize = 0;
    this.snapshotSize = bytes.length;
  }

  async setMeta(key, value) {
    const db = await this.#open();
    const tx = this.#tx(db, ['meta'], 'readwrite');
    tx.objectStore('meta').put(value, key);
    await done(tx);
  }

  async getMeta(key) {
    const db = await this.#open();
    const tx = this.#tx(db, ['meta'], 'readonly');
    const value = await request(tx.objectStore('meta').get(key));
    await done(tx);
    return value;
  }

  /** Deletes the whole IndexedDB database. */
  async destroy() {
    await this.close();
    await request(indexedDB.deleteDatabase(this.dbName));
    this.walSize = 0;
    this.snapshotSize = 0;
  }
}
