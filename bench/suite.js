// Benchmark workload shared by every runner: Node.js (bench/node/), the
// browser harness (bench/browser/) and, re-implemented in Rust, the native
// benchmark (crates/flatsql-cli/src/bench.rs). Every engine runs exactly the
// same operations on the same data; see docs/research/benchmark-scenario.md.
//
// WORKLOAD names this version of the workload. Results measured with a
// different WORKLOAD are not comparable, and the runner refuses to mix them
// (the native benchmark reports its own WORKLOAD). COMPATIBLE_WORKLOADS are
// earlier versions whose operations this one keeps unchanged, so their
// results compare operation by operation: benchmark-2 adds `insert_many` at
// the end of benchmark-1, a native benchmark built from an older version
// reports benchmark-1.

export const WORKLOAD = 'benchmark-2';
export const COMPATIBLE_WORKLOADS = ['benchmark-1'];
export const DEFAULT_ROWS = 100_000;

// INTEGER PRIMARY KEY makes the column an alias of SQLite's rowid (its fastest
// table layout), so SQLite is measured in its best configuration.
export const SCHEMA = 'CREATE TABLE items (id INTEGER PRIMARY KEY, category TEXT, price FLOAT, qty INT, name TEXT)';
export const INDEX = 'CREATE INDEX items_category ON items (category)';
export const INSERT = 'INSERT INTO items VALUES (?, ?, ?, ?, ?)';

// `insert_many`: the rows of the insert phase once more, into a second table
// of the same definition, all of them in one transaction and where the
// engine has one, in one call (Flat-SQL: executeMany).
export const MANY_SCHEMA = SCHEMA.replace('items', 'items_many');
export const MANY_INDEX = INDEX.replace(/items/g, 'items_many');
export const MANY_INSERT = INSERT.replace('items', 'items_many');
export const MANY_VERIFY_SQL = 'SELECT COUNT(*), SUM(qty), SUM(price) FROM items_many';
export const COLUMNS = ['id', 'category', 'price', 'qty', 'name'];

export function makeRow(i) {
  return [i, `c${i % 100}`, ((i * 7919) % 10000) / 100, i % 50, `item-${i}`];
}

/** Deterministic random numbers (the same sequence in every run and engine). */
export class Rng {
  constructor(seed = 42) {
    this.s = BigInt(seed);
  }
  below(n) {
    this.s = (this.s * 6364136223846793005n + 1442695040888963407n) & 0xffffffffffffffffn;
    return Number((this.s >> 33n) % BigInt(n));
  }
}

/** Seed of the measured parameter sequence of read operation `index`. */
export const opSeed = (index) => 1000 + index;
const WARM_SEED = 7;

/**
 * Read operations. `sql` is the query for SQL engines; engines without SQL
 * map each operation by `name` onto their own API (see bench/engines/).
 * `ordered: false` – the query has no ORDER BY, so the checksum sorts rows.
 */
export const READ_OPS = [
  {
    name: 'point_lookup',
    label: 'PK lookup',
    n: 10_000,
    sql: 'SELECT * FROM items WHERE id = ?',
    params: (rng, rows) => [rng.below(rows)],
  },
  {
    name: 'index_lookup',
    label: 'Index lookup + COUNT',
    n: 500,
    sql: 'SELECT COUNT(*) FROM items WHERE category = ?',
    params: (rng) => [`c${rng.below(100)}`],
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
  },
  {
    name: 'full_scan',
    label: 'Full scan + filter',
    n: 20,
    sql: 'SELECT COUNT(*) FROM items WHERE price > 50 AND qty < 10',
    params: () => [],
  },
  {
    name: 'group_by',
    label: 'GROUP BY (100 groups)',
    n: 20,
    sql: 'SELECT category, COUNT(*), AVG(price), MAX(qty) FROM items GROUP BY category',
    params: () => [],
    ordered: false,
  },
  {
    // Every price occurs ten times, so `id` breaks the ties and the answer is
    // the same in every engine.
    name: 'top_k',
    label: 'ORDER BY price LIMIT 10',
    n: 20,
    sql: 'SELECT id, price FROM items ORDER BY price DESC, id ASC LIMIT 10',
    params: () => [],
  },
  {
    name: 'materialize_10k',
    label: 'Fetch 10k rows to JS',
    n: 20,
    sql: 'SELECT * FROM items WHERE id < 10000',
    params: () => [],
    ordered: false,
  },
];

export const WRITE_OPS = [
  { name: 'update_10pct', label: 'UPDATE 10%', sql: 'UPDATE items SET qty = qty + 1 WHERE id % 10 = 0' },
  { name: 'delete_10pct', label: 'DELETE 10%', sql: 'DELETE FROM items WHERE id % 10 = 1' },
];

/** Table state checked after every write phase: [COUNT(*), SUM(qty), SUM(price)]. */
export const VERIFY_SQL = 'SELECT COUNT(*), SUM(qty), SUM(price) FROM items';

/** Durable writes (engines with persistent storage). */
export const DURABLE = { single: 100, group: 1000, firstId: 10_000_000 };

// ---- checksum ----------------------------------------------------------

/**
 * Canonical text of a value: numbers as integers scaled by 10^6 (Int and
 * Float, BigInt and Decimal compare equal; summation order does not matter
 * at this precision), text prefixed with S, NULL as N. The Rust benchmark
 * produces the same text.
 */
export function canon(v) {
  if (v === null || v === undefined) return 'N';
  switch (typeof v) {
    case 'number':
      return String(Math.round(v * 1e6) + 0);
    case 'bigint':
      return String(v * 1000000n);
    case 'string':
      return `S${v}`;
    case 'boolean':
      return v ? '1000000' : '0';
    default:
      return canon(Number(v));
  }
}

/**
 * 64-bit checksum of a result sequence: two 32-bit FNV-1a lanes with
 * different primes over the canonical text. Rows end with '|', results
 * with ';'. Engine independent and cheap enough for 10 000-row results.
 */
export class Checksum {
  constructor() {
    this.h1 = 0x811c9dc5;
    this.h2 = 0x050c5d1f;
    this.results = 0;
  }
  text(s) {
    let h1 = this.h1;
    let h2 = this.h2;
    for (let i = 0; i < s.length; i++) {
      const c = s.charCodeAt(i);
      h1 = Math.imul(h1 ^ c, 0x01000193);
      h2 = Math.imul(h2 ^ c, 0x5bd1e995);
    }
    this.h1 = h1;
    this.h2 = h2;
  }
  /** Adds one query result (array of rows, each an array of values). */
  result(rows, ordered = true) {
    const lines = rows.map((row) => row.map(canon).join(','));
    if (!ordered) lines.sort();
    for (const line of lines) this.text(`${line}|`);
    this.text(';');
    this.results++;
  }
  hex() {
    return (this.h1 >>> 0).toString(16).padStart(8, '0') + (this.h2 >>> 0).toString(16).padStart(8, '0');
  }
}

/** Checksums of the first HEAD results are compared even when a slow engine stopped early. */
export const HEAD = 3;

// ---- runner --------------------------------------------------------------

const yieldToEventLoop = () => new Promise((r) => setTimeout(r, 0));

function sampleStats(samples, n, totalMs) {
  const sorted = Float64Array.from(samples.subarray(0, n)).sort();
  const pick = (p) => sorted[Math.min(n - 1, Math.round((n - 1) * p))] * 1000;
  return { n, meanUs: (totalMs * 1000) / n, p50Us: pick(0.5), p95Us: pick(0.95), p99Us: pick(0.99), maxUs: sorted[n - 1] * 1000 };
}

/**
 * Runs the workload once on one engine adapter (bench/engines/). Adapters of
 * engines with an asynchronous API set `async: true`; their calls are awaited,
 * calls of synchronous adapters are not.
 *
 * @param {object} engine adapter
 * @param {object} [o]
 * @param {number} [o.rows] table size
 * @param {number} [o.capMs] time limit of one read operation; a slower engine
 *   stops early (at least HEAD executions) and reports the executed count
 * @param {boolean} [o.durable] also run the durable-write phase
 */
export async function runWorkload(engine, { rows = DEFAULT_ROWS, capMs = 20_000, warmRows = 10_000, warmIters = 200, durable = false, onProgress = () => {}, now = () => performance.now() } = {}) {
  const isAsync = engine.async === true;
  const call = isAsync ? async (r) => await r : (r) => r;
  const ops = {};
  const checks = {};
  const state = {};

  // Warm-up on a throw-away database: V8 first compiles WebAssembly with a
  // baseline compiler and optimises hot functions later, and JavaScript
  // engines need the same; without it the first phases pay for tier-up.
  onProgress(`${engine.id}: warm-up`);
  await call(engine.open({ fresh: true }));
  await call(engine.load(Math.min(rows, warmRows)));
  const warmRng = new Rng(WARM_SEED);
  for (const op of READ_OPS) {
    const st = await call(engine.prepare(op));
    const t0 = now();
    for (let i = 0; i < warmIters && (i < HEAD || now() - t0 < capMs / 10); i++) {
      await call(engine.query(st, op.params(warmRng, Math.min(rows, warmRows)), op));
    }
    await call(engine.finalize?.(st));
  }
  await call(engine.close({ destroy: true }));
  await yieldToEventLoop();

  await call(engine.open({ fresh: true }));

  onProgress(`${engine.id}: insert ${rows} rows`);
  await yieldToEventLoop();
  let t = now();
  await call(engine.load(rows));
  let ms = now() - t;
  ops.insert = { n: 1, meanUs: ms * 1000, rowsPerSec: rows / (ms / 1000) };
  state.afterInsert = await verify(engine, call);
  if (engine.buildIndexes) {
    // Engines that build their indexes lazily (PouchDB views) pay for it at
    // the first query; measured separately so the reads are comparable.
    t = now();
    await call(engine.buildIndexes());
    ops.index_build = { n: 1, meanUs: (now() - t) * 1000 };
  }

  for (const [index, op] of READ_OPS.entries()) {
    onProgress(`${engine.id}: ${op.label}`);
    await yieldToEventLoop();
    const st = await call(engine.prepare(op));
    const rng = new Rng(opSeed(index));
    const params = Array.from({ length: op.n }, () => op.params(rng, rows));
    for (let i = 0; i < Math.max(1, op.n / 10); i++) await call(engine.query(st, params[i], op)); // warm-up
    const samples = new Float64Array(op.n);
    const results = new Array(op.n);
    let n = 0;
    const start = now();
    for (; n < op.n; n++) {
      const t0 = now();
      let r = engine.query(st, params[n], op);
      if (isAsync) r = await r;
      const t1 = now();
      samples[n] = t1 - t0;
      results[n] = r;
      if (n >= HEAD && t1 - start > capMs) {
        n++;
        break;
      }
    }
    const total = now() - start;
    await call(engine.finalize?.(st));
    ops[op.name] = sampleStats(samples, n, total);
    const head = new Checksum();
    const full = new Checksum();
    for (let i = 0; i < n; i++) {
      if (i < HEAD) head.result(results[i], op.ordered !== false);
      full.result(results[i], op.ordered !== false);
    }
    checks[op.name] = { head: head.hex(), full: full.hex(), n, rows: results[n - 1].length };
  }

  for (const op of WRITE_OPS) {
    onProgress(`${engine.id}: ${op.label}`);
    await yieldToEventLoop();
    t = now();
    await call(engine.write(op));
    ms = now() - t;
    ops[op.name] = { n: 1, meanUs: ms * 1000 };
    state[`after_${op.name}`] = await verify(engine, call);
  }

  if (engine.snapshot) {
    onProgress(`${engine.id}: snapshot / restore`);
    await yieldToEventLoop();
    t = now();
    const snap = await call(engine.snapshot());
    ms = now() - t;
    ops.snapshot = { n: 1, meanUs: ms * 1000, bytes: snap.byteLength ?? snap.length };
    t = now();
    await call(engine.restore(snap));
    ms = now() - t;
    ops.restore = { n: 1, meanUs: ms * 1000 };
    state.afterRestore = await verify(engine, call);
  }

  if (engine.loadMany) {
    onProgress(`${engine.id}: insert ${rows} rows in one call`);
    await yieldToEventLoop();
    t = now();
    const method = await call(engine.loadMany(rows));
    ms = now() - t;
    ops.insert_many = { n: 1, meanUs: ms * 1000, rowsPerSec: rows / (ms / 1000), method };
    state.afterInsertMany = (await call(engine.verifyMany())).map(canon);
  }

  if (durable) await runDurable(engine, { ops, state, call, now, onProgress });

  await call(engine.close({ destroy: true }));
  return { engine: engine.id, workload: WORKLOAD, rows, ops, checks, state: checksumState(state), stateValues: state };
}

async function verify(engine, call) {
  const rows = await call(engine.verify());
  return rows.map(canon);
}

function checksumState(state) {
  const c = new Checksum();
  for (const key of Object.keys(state).sort()) c.result([[key, ...state[key]]]);
  return c.hex();
}

/**
 * Durable writes: single commits (each waits until the write is durable),
 * a group of commits made durable together, and reopening the database
 * (storage → memory, first query).
 */
async function runDurable(engine, { ops, state, call, now, onProgress }) {
  onProgress(`${engine.id}: ${DURABLE.single} durable commits`);
  await yieldToEventLoop();
  const samples = new Float64Array(DURABLE.single);
  let start = now();
  for (let i = 0; i < DURABLE.single; i++) {
    const t0 = now();
    await call(engine.commit(makeRow(DURABLE.firstId + i)));
    samples[i] = now() - t0;
  }
  ops.commit_single = sampleStats(samples, DURABLE.single, now() - start);

  onProgress(`${engine.id}: ${DURABLE.group} commits, one wait`);
  await yieldToEventLoop();
  const group = [];
  for (let i = 0; i < DURABLE.group; i++) group.push(makeRow(DURABLE.firstId + DURABLE.single + i));
  start = now();
  await call(engine.commitGroup(group));
  const ms = now() - start;
  ops.commit_group = { n: 1, meanUs: ms * 1000, perCommitUs: (ms * 1000) / DURABLE.group };
  state.afterCommits = await verify(engine, call);

  onProgress(`${engine.id}: reopen`);
  await yieldToEventLoop();
  start = now();
  const info = await call(engine.reopen());
  const st = await call(engine.prepare(READ_OPS[0]));
  const probe = await call(engine.query(st, [42], READ_OPS[0]));
  ops.reopen = { n: 1, meanUs: (now() - start) * 1000, ...(info ?? {}) };
  await call(engine.finalize?.(st));
  state.afterReopen = [...(await verify(engine, call)), ...probe[0].map(canon)];
}
