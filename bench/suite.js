// Benchmark workload shared by the Node runner (bench/run.js) and the
// browser page (web/bench.html). Every engine runs exactly the same
// operations on the same data set; see docs/research/benchmark-scenario.md.

// INTEGER PRIMARY KEY makes the column an alias of SQLite's rowid (its fastest
// table layout), so SQLite is measured in its best configuration.
export const SCHEMA = 'CREATE TABLE items (id INTEGER PRIMARY KEY, category TEXT, price FLOAT, qty INT, name TEXT)';
export const INDEX = 'CREATE INDEX items_category ON items (category)';

export function makeRow(i) {
  return [i, `c${i % 100}`, ((i * 7919) % 10000) / 100, i % 50, `item-${i}`];
}

/** Deterministic random numbers (same sequence in every run). */
export class Rng {
  constructor(seed = 42) {
    this.s = BigInt(seed);
  }
  below(n) {
    this.s = (this.s * 6364136223846793005n + 1442695040888963407n) & 0xffffffffffffffffn;
    return Number((this.s >> 33n) % BigInt(n));
  }
}

/**
 * Read operations. `sql` is used by SQL engines, `js` by the hand written
 * JavaScript baseline (Map + secondary index Map, no SQL layer).
 */
export const READ_OPS = [
  {
    name: 'point_lookup',
    label: 'PK lookup',
    n: 10_000,
    sql: 'SELECT * FROM items WHERE id = ?',
    params: (rng, rows) => [rng.below(rows)],
    js: (s, [id]) => [s.byId.get(id)],
  },
  {
    name: 'index_lookup',
    label: 'Index lookup + COUNT',
    n: 500,
    sql: 'SELECT COUNT(*) FROM items WHERE category = ?',
    params: (rng) => [`c${rng.below(100)}`],
    js: (s, [c]) => [[s.byCat.get(c)?.size ?? 0]],
  },
  {
    name: 'range_scan',
    label: 'PK range (1000 rows) + SUM',
    n: 200,
    sql: 'SELECT SUM(price) FROM items WHERE id BETWEEN ? AND ?',
    params: (rng, rows) => {
      const a = rng.below(Math.max(rows - 1000, 1));
      return [a, a + 999];
    },
    js: (s, [a, b]) => {
      let sum = 0;
      for (let id = a; id <= b; id++) {
        const r = s.byId.get(id);
        if (r) sum += r[2];
      }
      return [[sum]];
    },
  },
  {
    name: 'full_scan',
    label: 'Full scan + filter',
    n: 20,
    sql: 'SELECT COUNT(*) FROM items WHERE price > 50 AND qty < 10',
    params: () => [],
    js: (s) => {
      let n = 0;
      for (const r of s.byId.values()) if (r[2] > 50 && r[3] < 10) n++;
      return [[n]];
    },
  },
  {
    name: 'group_by',
    label: 'GROUP BY (100 groups)',
    n: 20,
    sql: 'SELECT category, COUNT(*), AVG(price), MAX(qty) FROM items GROUP BY category',
    params: () => [],
    js: (s) => {
      const g = new Map();
      for (const r of s.byId.values()) {
        let a = g.get(r[1]);
        if (!a) g.set(r[1], (a = { n: 0, sum: 0, max: -Infinity }));
        a.n++;
        a.sum += r[2];
        if (r[3] > a.max) a.max = r[3];
      }
      return [...g].sort((x, y) => (x[0] < y[0] ? -1 : 1)).map(([k, a]) => [k, a.n, a.sum / a.n, a.max]);
    },
  },
  {
    name: 'top_k',
    label: 'ORDER BY price LIMIT 10',
    n: 20,
    sql: 'SELECT id, price FROM items ORDER BY price DESC LIMIT 10',
    params: () => [],
    js: (s) => {
      const top = [];
      for (const r of s.byId.values()) {
        if (top.length === 10 && r[2] <= top[9][1]) continue;
        top.push([r[0], r[2]]);
        top.sort((x, y) => y[1] - x[1] || x[0] - y[0]);
        if (top.length > 10) top.pop();
      }
      return top;
    },
  },
  {
    name: 'materialize_10k',
    label: 'Fetch 10k rows to JS',
    n: 20,
    sql: 'SELECT * FROM items WHERE id < 10000',
    params: () => [],
    js: (s) => {
      const out = [];
      for (let id = 0; id < 10000; id++) out.push(s.byId.get(id).slice());
      return out;
    },
  },
];

function summarize(samples, totalMs) {
  const sorted = Float64Array.from(samples).sort();
  const pick = (p) => sorted[Math.min(sorted.length - 1, Math.round((sorted.length - 1) * p))];
  return {
    n: samples.length,
    meanUs: (totalMs * 1000) / samples.length,
    p50Us: pick(0.5) * 1000,
    p95Us: pick(0.95) * 1000,
    p99Us: pick(0.99) * 1000,
    totalMs,
  };
}

const now = () => performance.now();
const yieldToEventLoop = () => new Promise((r) => setTimeout(r, 0));

/**
 * Runs the workload on one engine adapter (see bench/engines.js).
 * @returns {Promise<{engine: string, results: object, checks: object}>}
 */
export async function runEngine(engine, { rows = 100_000, onProgress = () => {} } = {}) {
  const results = {};
  const checks = {};
  const rng = new Rng(42);

  // Warm-up on a throw-away database: V8 first compiles WebAssembly with a
  // baseline compiler and optimises hot functions later, so the first
  // engine would otherwise pay for tier-up inside the measurements.
  onProgress(`${engine.name}: warm-up`);
  engine.open();
  const warmRows = Math.min(rows, 10_000);
  engine.load(warmRows);
  for (const op of READ_OPS) {
    const stmt = engine.prepare(op);
    for (let i = 0; i < Math.min(op.n, 200); i++) engine.query(stmt, op.params(rng, warmRows), op);
    engine.finalize?.(stmt);
  }
  engine.close();
  await yieldToEventLoop();

  engine.open();

  onProgress(`${engine.name}: insert ${rows} rows`);
  await yieldToEventLoop();
  let t = now();
  engine.load(rows);
  results.insert = { ...summarize([now() - t], now() - t), rowsPerSec: rows / ((now() - t) / 1000) };

  for (const op of READ_OPS) {
    onProgress(`${engine.name}: ${op.label}`);
    await yieldToEventLoop();
    const stmt = engine.prepare(op);
    for (let i = 0; i < Math.max(1, op.n / 10); i++) engine.query(stmt, op.params(rng, rows), op); // warm-up
    const samples = new Array(op.n);
    let last;
    const start = now();
    for (let i = 0; i < op.n; i++) {
      const params = op.params(rng, rows);
      const t0 = now();
      last = engine.query(stmt, params, op);
      samples[i] = now() - t0;
    }
    results[op.name] = summarize(samples, now() - start);
    checks[op.name] = Array.isArray(last) ? last.length : last;
    engine.finalize?.(stmt);
  }

  for (const [name, label, sql] of [
    ['update_10pct', 'UPDATE 10%', 'UPDATE items SET qty = qty + 1 WHERE id % 10 = 0'],
    ['delete_10pct', 'DELETE 10%', 'DELETE FROM items WHERE id % 10 = 1'],
  ]) {
    onProgress(`${engine.name}: ${label}`);
    await yieldToEventLoop();
    t = now();
    engine.write(name, sql);
    results[name] = summarize([now() - t], now() - t);
  }

  onProgress(`${engine.name}: snapshot / restore`);
  await yieldToEventLoop();
  t = now();
  const snap = engine.snapshot();
  results.snapshot = { ...summarize([now() - t], now() - t), bytes: snap.length };
  t = now();
  engine.restore(snap);
  results.restore = summarize([now() - t], now() - t);
  checks.rowsAfterRestore = engine.count();

  engine.close();
  return { engine: engine.name, results, checks };
}
