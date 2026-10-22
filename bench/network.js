// Local-First vs Cloud-First: the same point lookup executed by the local
// WebAssembly engine and by the server through HTTP (/api/query), with an
// artificial delay emulating WAN round-trip times.

import { makeRow, Rng } from './suite.js';

const now = () => performance.now();

function stats(samples) {
  const s = Float64Array.from(samples).sort();
  const pick = (p) => s[Math.min(s.length - 1, Math.round((s.length - 1) * p))];
  const mean = samples.reduce((a, b) => a + b, 0) / samples.length;
  return { n: samples.length, meanMs: mean, p50Ms: pick(0.5), p95Ms: pick(0.95), p99Ms: pick(0.99) };
}

async function remote(baseUrl, sql, latency, fetchImpl) {
  const res = await fetchImpl(`${baseUrl}/api/query?latency=${latency}&publish=0`, { method: 'POST', body: sql });
  const body = await res.json();
  if (!res.ok) throw new Error(`server: ${body.error}`);
  return body;
}

/**
 * @param {object} o
 * @param {string} o.baseUrl     server URL
 * @param {Function} o.Database  wasm Database class (local engine)
 * @param {number[]} [o.latencies] simulated extra round-trip times in ms
 */
export async function runNetwork({ baseUrl, Database, latencies = [0, 20, 50, 100], rows = 10_000, requests = 40, fetch: fetchImpl = globalThis.fetch.bind(globalThis), onProgress = () => {} }) {
  const table = 'bench_remote';
  onProgress(`network: seeding ${rows} rows on the server and locally`);
  await remote(baseUrl, `DROP TABLE IF EXISTS ${table}`, 0, fetchImpl);
  const ddl = `CREATE TABLE ${table} (id INT PRIMARY KEY, category TEXT, price FLOAT, qty INT, name TEXT)`;
  await remote(baseUrl, ddl, 0, fetchImpl);
  const local = new Database(2);
  local.exec(ddl);
  for (let start = 0; start < rows; start += 1000) {
    const values = [];
    for (let i = start; i < Math.min(rows, start + 1000); i++) {
      const r = makeRow(i);
      values.push(`(${r[0]}, '${r[1]}', ${r[2]}, ${r[3]}, '${r[4]}')`);
    }
    const sql = `INSERT INTO ${table} VALUES ${values.join(',')}`;
    await remote(baseUrl, sql, 0, fetchImpl);
    local.exec(sql);
  }

  const rng = new Rng(7);
  const results = [];

  // Local engine, same query text (parse + execute + result conversion).
  const localSamples = [];
  for (let i = 0; i < 2000; i++) {
    const sql = `SELECT * FROM ${table} WHERE id = ${rng.below(rows)}`;
    const t0 = now();
    local.exec(sql);
    localSamples.push(now() - t0);
  }
  results.push({ mode: 'local (WASM, in-process)', latency: 0, ...stats(localSamples.slice(200)) });

  for (const latency of latencies) {
    onProgress(`network: cloud API with +${latency} ms simulated latency`);
    const n = latency >= 50 ? Math.ceil(requests / 2) : requests;
    const samples = [];
    for (let i = 0; i < n + 3; i++) {
      const sql = `SELECT * FROM ${table} WHERE id = ${rng.below(rows)}`;
      const t0 = now();
      await remote(baseUrl, sql, latency, fetchImpl);
      if (i >= 3) samples.push(now() - t0); // first requests warm up the connection
    }
    results.push({ mode: latency === 0 ? 'cloud API (localhost HTTP)' : `cloud API (+${latency} ms RTT)`, latency, ...stats(samples) });
  }

  await remote(baseUrl, `DROP TABLE IF EXISTS ${table}`, 0, fetchImpl);
  local.free();
  return results;
}
