// Hand written JavaScript on Map/Set – the "no database" baseline: the same
// answers as the SQL queries, without a query language, WebAssembly or a
// boundary between languages.

import { makeRow } from '../suite.js';

const OPS = {
  point_lookup: (s, [id]) => {
    const r = s.byId.get(id);
    return r ? [r] : [];
  },
  index_lookup: (s, [c]) => [[s.byCat.get(c)?.size ?? 0]],
  range_scan: (s, [a, b]) => {
    let sum = 0;
    for (let id = a; id <= b; id++) {
      const r = s.byId.get(id);
      if (r) sum += r[2];
    }
    return [[sum]];
  },
  full_scan: (s) => {
    let n = 0;
    for (const r of s.byId.values()) if (r[2] > 50 && r[3] < 10) n++;
    return [[n]];
  },
  group_by: (s) => {
    const g = new Map();
    for (const r of s.byId.values()) {
      let a = g.get(r[1]);
      if (!a) g.set(r[1], (a = { n: 0, sum: 0, max: -Infinity }));
      a.n++;
      a.sum += r[2];
      if (r[3] > a.max) a.max = r[3];
    }
    return [...g].map(([k, a]) => [k, a.n, a.sum / a.n, a.max]);
  },
  top_k: (s) => {
    const better = (x, y) => x[1] > y[1] || (x[1] === y[1] && x[0] < y[0]);
    const top = [];
    for (const r of s.byId.values()) {
      if (top.length === 10 && !better([r[0], r[2]], top[9])) continue;
      top.push([r[0], r[2]]);
      top.sort((x, y) => y[1] - x[1] || x[0] - y[0]);
      if (top.length > 10) top.pop();
    }
    return top;
  },
  materialize_10k: (s) => {
    const out = [];
    for (let id = 0; id < 10000; id++) {
      const r = s.byId.get(id);
      if (r) out.push(r.slice());
    }
    return out;
  },
};

export async function create() {
  let s;
  const add = (r) => {
    s.byId.set(r[0], r);
    let set = s.byCat.get(r[1]);
    if (!set) s.byCat.set(r[1], (set = new Set()));
    set.add(r[0]);
  };
  const empty = () => ({ byId: new Map(), byCat: new Map() });
  return {
    async: false,
    open() {
      s = empty();
    },
    load(rows) {
      for (let i = 0; i < rows; i++) add(makeRow(i));
    },
    prepare(op) {
      return OPS[op.name];
    },
    query(fn, params) {
      return fn(s, params);
    },
    write(op) {
      if (op.name === 'update_10pct') {
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
    verify() {
      let qty = 0;
      let price = 0;
      for (const r of s.byId.values()) {
        qty += r[3];
        price += r[2];
      }
      return [s.byId.size, qty, price];
    },
    snapshot() {
      return new TextEncoder().encode(JSON.stringify([...s.byId.values()]));
    },
    restore(bytes) {
      const rows = JSON.parse(new TextDecoder().decode(bytes));
      s = empty();
      for (const r of rows) add(r);
    },
    close() {
      s = null;
    },
  };
}
