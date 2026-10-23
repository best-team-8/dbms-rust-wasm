// In-memory IndexedDB for Node.js tests of the browser store: the subset of
// the API that IndexedDbStore uses. Every transaction is logged with its mode
// and durability hint, so tests can check what the store asked for.

function later(fn) {
  setImmediate(fn);
}

class Request {
  constructor() {
    this.result = undefined;
    this.error = null;
    this.onsuccess = null;
    this.onerror = null;
  }

  succeed(result) {
    this.result = result;
    this.onsuccess?.({ target: this });
  }
}

class ObjectStore {
  constructor(tx, store) {
    this.tx = tx;
    this.store = store;
  }

  #request(fn) {
    const req = new Request();
    this.tx.pending++;
    later(() => {
      req.succeed(fn());
      this.tx.pending--;
      later(() => this.tx.complete());
    });
    return req;
  }

  #keys() {
    return [...this.store.data.keys()].sort((a, b) => (a < b ? -1 : a > b ? 1 : 0));
  }

  get(key) {
    return this.#request(() => structuredClone(this.store.data.get(key)));
  }

  getAll() {
    return this.#request(() => this.#keys().map((k) => structuredClone(this.store.data.get(k))));
  }

  getAllKeys() {
    return this.#request(() => this.#keys());
  }

  put(value, key) {
    const copy = structuredClone(value);
    return this.#request(() => (this.store.data.set(key, copy), key));
  }

  add(value) {
    const copy = structuredClone(value);
    const key = this.store.nextKey++;
    return this.#request(() => (this.store.data.set(key, copy), key));
  }

  clear() {
    return this.#request(() => this.store.data.clear());
  }
}

class Transaction {
  constructor(db, names, mode, durability) {
    this.db = db;
    this.mode = mode;
    this.durability = durability;
    this.pending = 0;
    this.done = false;
    this.oncomplete = null;
    this.onerror = null;
    this.onabort = null;
    // Completes once no request is pending (also when none was made).
    later(() => this.complete());
  }

  objectStore(name) {
    return new ObjectStore(this, this.db.stores.get(name));
  }

  complete() {
    if (this.done || this.pending > 0) return;
    this.done = true;
    this.oncomplete?.({ target: this });
  }
}

class Connection {
  constructor(factory, db) {
    this.factory = factory;
    this.db = db;
    this.onversionchange = null;
  }

  createObjectStore(name, { autoIncrement = false } = {}) {
    this.db.stores.set(name, { autoIncrement, nextKey: 1, data: new Map() });
  }

  transaction(names, mode = 'readonly', options = {}) {
    const durability = options.durability ?? 'default';
    this.factory.transactions.push({ db: this.db.name, stores: [].concat(names), mode, durability });
    return new Transaction(this.db, names, mode, durability);
  }

  close() {}
}

export class FakeIndexedDB {
  constructor() {
    this.databases = new Map();
    this.transactions = [];
  }

  open(name, version = 1) {
    const req = new Request();
    req.onupgradeneeded = null;
    req.onblocked = null;
    later(() => {
      let db = this.databases.get(name);
      const upgrade = !db || db.version < version;
      if (!db) {
        db = { name, version, stores: new Map() };
        this.databases.set(name, db);
      }
      db.version = version;
      req.result = new Connection(this, db);
      if (upgrade) req.onupgradeneeded?.({ target: req });
      req.succeed(req.result);
    });
    return req;
  }

  deleteDatabase(name) {
    const req = new Request();
    later(() => {
      this.databases.delete(name);
      req.succeed(undefined);
    });
    return req;
  }
}

/** Puts a fresh FakeIndexedDB on globalThis.indexedDB; returns it and a cleanup function. */
export function installFakeIndexedDB() {
  const had = Object.prototype.hasOwnProperty.call(globalThis, 'indexedDB');
  const previous = globalThis.indexedDB;
  const fake = new FakeIndexedDB();
  globalThis.indexedDB = fake;
  fake.uninstall = () => {
    if (had) globalThis.indexedDB = previous;
    else delete globalThis.indexedDB;
  };
  return fake;
}
