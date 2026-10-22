// Engine adapters used by the benchmark suite. Each adapter hides the API
// differences of one engine behind the same small interface.

import { INDEX, makeRow, SCHEMA } from './suite.js';

const INSERT = 'INSERT INTO items VALUES (?, ?, ?, ?, ?)';

/**
 * Flat-SQL compiled to WebAssembly.
 * `json: true` returns results as a JSON string parsed by JavaScript instead
 * of JS objects built from Rust – the two Rust↔JS interop strategies.
 */
export function flatsqlEngine(Database, { json = false } = {}) {
  let db;
  return {
    name: json ? 'flatsql-wasm (json)' : 'flatsql-wasm',
    open() {
      db = new Database(1);
      db.exec(SCHEMA);
      db.exec(INDEX);
    },
    load(rows) {
      const st = db.prepare(INSERT);
      db.exec('BEGIN');
      for (let i = 0; i < rows; i++) db.execute(st, makeRow(i));
      db.exec('COMMIT');
      st.free();
      // WAL bytes are produced exactly as in production; drop them here.
      db.takeWal();
    },
    prepare(op) {
      return db.prepare(op.sql);
    },
    query(st, params) {
      return json ? JSON.parse(db.runJson(st, params)).rows : db.run(st, params).rows;
    },
    finalize(st) {
      st.free();
    },
    write(_name, sql) {
      db.exec(sql);
      db.takeWal();
    },
    snapshot() {
      return db.snapshot();
    },
    restore(bytes) {
      db.free();
      db = Database.restore(bytes, undefined, 1);
    },
    count() {
      return db.exec('SELECT COUNT(*) FROM items').rows[0][0];
    },
    close() {
      db.free();
    },
  };
}

/** SQLite compiled to WebAssembly with Emscripten (sql.js), in-memory. */
export function sqljsEngine(SQL) {
  let db;
  return {
    name: 'sql.js (SQLite)',
    open() {
      db = new SQL.Database();
      db.exec(SCHEMA);
      db.exec(INDEX);
    },
    load(rows) {
      const st = db.prepare(INSERT);
      db.exec('BEGIN');
      for (let i = 0; i < rows; i++) st.run(makeRow(i));
      db.exec('COMMIT');
      st.free();
    },
    prepare(op) {
      return db.prepare(op.sql);
    },
    query(st, params) {
      if (params.length) st.bind(params);
      const rows = [];
      while (st.step()) rows.push(st.get());
      st.reset();
      return rows;
    },
    finalize(st) {
      st.free();
    },
    write(_name, sql) {
      db.exec(sql);
    },
    snapshot() {
      return db.export();
    },
    restore(bytes) {
      db.close();
      db = new SQL.Database(bytes);
    },
    count() {
      return db.exec('SELECT COUNT(*) FROM items')[0].values[0][0];
    },
    close() {
      db.close();
    },
  };
}

/** Hand written JavaScript on Map/Set – the "no database" baseline. */
export function jsEngine() {
  let s;
  const add = (r) => {
    s.byId.set(r[0], r);
    let set = s.byCat.get(r[1]);
    if (!set) s.byCat.set(r[1], (set = new Set()));
    set.add(r[0]);
  };
  return {
    name: 'plain JS (Map)',
    open() {
      s = { byId: new Map(), byCat: new Map() };
    },
    load(rows) {
      for (let i = 0; i < rows; i++) add(makeRow(i));
    },
    prepare(op) {
      return op;
    },
    query(op, params) {
      return op.js(s, params);
    },
    write(name) {
      if (name === 'update_10pct') {
        for (const r of s.byId.values()) if (r[0] % 10 === 0) r[3]++;
      } else {
        for (const [id, r] of s.byId) {
          if (id % 10 === 1) {
            s.byId.delete(id);
            s.byCat.get(r[1]).delete(id);
          }
        }
      }
    },
    snapshot() {
      return new TextEncoder().encode(JSON.stringify([...s.byId.values()]));
    },
    restore(bytes) {
      const rows = JSON.parse(new TextDecoder().decode(bytes));
      s = { byId: new Map(), byCat: new Map() };
      for (const r of rows) add(r);
    },
    count() {
      return s.byId.size;
    },
    close() {
      s = null;
    },
  };
}
