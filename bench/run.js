// Node.js benchmark runner.
//
//   node bench/run.js [--rows 100000] [--engines flatsql,json,sqljs,js]
//                     [--native] [--no-network] [--out file.json] [--md file.md]
//
// --native also runs `flatsql bench` (build it first with
// `cargo build --release -p flatsql-cli`) to show WebAssembly overhead.

import { existsSync } from 'node:fs';
import http from 'node:http';
import { mkdir, readFile, rm, writeFile } from 'node:fs/promises';
import { spawnSync } from 'node:child_process';
import { gzipSync } from 'node:zlib';
import { tmpdir, cpus } from 'node:os';
import { join } from 'node:path';
import { fileURLToPath } from 'node:url';
import initSqlJs from 'sql.js';

import { Database, version } from '../bridge/node.js';
import { createServer } from '../server/server.js';
import { flatsqlEngine, jsEngine, sqljsEngine } from './engines.js';
import { runNetwork } from './network.js';
import { READ_OPS, runEngine } from './suite.js';

const ROOT = fileURLToPath(new URL('..', import.meta.url));

function parseArgs(argv) {
  const o = { rows: 100_000, engines: ['flatsql', 'json', 'sqljs', 'js'], native: false, network: true, out: null, md: null };
  for (let i = 0; i < argv.length; i++) {
    const a = argv[i];
    if (a === '--rows') o.rows = Number(argv[++i]);
    else if (a === '--engines') o.engines = argv[++i].split(',');
    else if (a === '--native') o.native = true;
    else if (a === '--no-network') o.network = false;
    else if (a === '--out') o.out = argv[++i];
    else if (a === '--md') o.md = argv[++i];
    else throw new Error(`unknown option ${a}`);
  }
  return o;
}

async function wasmInfo(name, path) {
  const bytes = await readFile(path);
  const times = [];
  for (let i = 0; i < 5; i++) {
    const t0 = performance.now();
    await WebAssembly.compile(bytes);
    times.push(performance.now() - t0);
  }
  times.sort((a, b) => a - b);
  return { name, bytes: bytes.length, gzipBytes: gzipSync(bytes, { level: 9 }).length, compileMs: times[2] };
}

function runNative(rows) {
  const exe = join(ROOT, 'target', 'release', process.platform === 'win32' ? 'flatsql.exe' : 'flatsql');
  if (!existsSync(exe)) {
    console.warn(`native binary not found (${exe}); run: cargo build --release -p flatsql-cli`);
    return null;
  }
  const out = join(tmpdir(), `flatsql-native-${process.pid}.json`);
  const r = spawnSync(exe, ['bench', '--rows', String(rows), '--json', out], { encoding: 'utf8' });
  if (r.status !== 0) {
    console.warn('native benchmark failed:', r.stderr);
    return null;
  }
  return out;
}

// Minimal fetch() over node:http with keep-alive. Node's built-in fetch
// (undici) adds ~15 ms per request on Windows (timer granularity), which
// would distort the cloud API numbers; browsers are not affected.
const agent = new http.Agent({ keepAlive: true });
function nodeFetch(url, { method = 'GET', body } = {}) {
  return new Promise((resolve, reject) => {
    const req = http.request(url, { method, agent }, (res) => {
      const chunks = [];
      res.on('data', (c) => chunks.push(c));
      res.on('end', () => {
        const text = Buffer.concat(chunks).toString('utf8');
        resolve({ ok: res.statusCode < 400, status: res.statusCode, json: async () => JSON.parse(text), text: async () => text });
      });
    });
    req.on('error', reject);
    req.end(body);
  });
}

const fmtUs = (us) => (us >= 1000 ? `${(us / 1000).toFixed(2)} ms` : `${us.toFixed(us < 10 ? 2 : 1)} µs`);

function markdown(report) {
  const names = report.engines.map((e) => e.engine);
  const ops = [
    ['insert', 'Insert (1 txn, prepared)'],
    ...READ_OPS.map((o) => [o.name, `${o.label}`]),
    ['update_10pct', 'UPDATE 10% rows'],
    ['delete_10pct', 'DELETE 10% rows'],
    ['snapshot', 'Snapshot / export'],
    ['restore', 'Restore / import'],
  ];
  const lines = [];
  lines.push(`Rows: ${report.rows.toLocaleString('en-US')} · ${report.env} · ${report.date}`, '');
  lines.push(`| Operation | ${names.join(' | ')} |`, `|---|${names.map(() => '---:').join('|')}|`);
  for (const [key, label] of ops) {
    const cells = report.engines.map((e) => {
      const r = e.results[key];
      if (!r) return '–';
      if (key === 'insert') return `${fmtUs(r.meanUs)} (${Math.round(r.rowsPerSec ?? report.rows / (r.totalMs / 1000)).toLocaleString('en-US')} rows/s)`;
      return r.n > 1 ? `${fmtUs(r.meanUs)} (p95 ${fmtUs(r.p95Us)})` : fmtUs(r.meanUs);
    });
    lines.push(`| ${label} | ${cells.join(' | ')} |`);
  }
  const snap = report.engines.map((e) => (e.results.snapshot?.bytes ? `${(e.results.snapshot.bytes / 1e6).toFixed(2)} MB` : '–'));
  lines.push(`| Snapshot size | ${snap.join(' | ')} |`, '');
  if (report.wasm?.length) {
    lines.push('| WebAssembly module | size | gzip | compile |', '|---|---:|---:|---:|');
    for (const w of report.wasm) {
      lines.push(`| ${w.name} | ${(w.bytes / 1024).toFixed(0)} KiB | ${(w.gzipBytes / 1024).toFixed(0)} KiB | ${w.compileMs.toFixed(1)} ms |`);
    }
    lines.push('');
  }
  if (report.network?.length) {
    lines.push('| Point lookup | mean | p50 | p95 |', '|---|---:|---:|---:|');
    for (const n of report.network) {
      lines.push(`| ${n.mode} | ${n.meanMs.toFixed(3)} ms | ${n.p50Ms.toFixed(3)} ms | ${n.p95Ms.toFixed(3)} ms |`);
    }
    lines.push('');
  }
  return lines.join('\n');
}

const opts = parseArgs(process.argv.slice(2));
const report = {
  env: `Node ${process.version} (${process.platform}/${process.arch}, ${cpus()[0]?.model?.trim() ?? 'cpu'})`,
  date: new Date().toISOString(),
  flatsqlVersion: version(),
  rows: opts.rows,
  engines: [],
  wasm: [],
  network: null,
};

const log = (msg) => process.stderr.write(`  ${msg}\n`);
console.error(`Flat-SQL benchmark – ${opts.rows} rows`);

report.wasm.push(await wasmInfo('flatsql (Rust → wasm32)', join(ROOT, 'pkg', 'flatsql_bg.wasm')));
report.wasm.push(await wasmInfo('sql.js (SQLite → Emscripten)', join(ROOT, 'node_modules', 'sql.js', 'dist', 'sql-wasm.wasm')));

const SQL = await initSqlJs();
const factories = {
  flatsql: () => flatsqlEngine(Database),
  json: () => flatsqlEngine(Database, { json: true }),
  sqljs: () => sqljsEngine(SQL),
  js: () => jsEngine(),
};
for (const key of opts.engines) {
  if (!factories[key]) throw new Error(`unknown engine ${key}`);
  const result = await runEngine(factories[key](), { rows: opts.rows, onProgress: log });
  report.engines.push(result);
}

if (opts.native) {
  log('native: flatsql bench');
  const file = runNative(opts.rows);
  if (file) {
    const native = JSON.parse(await readFile(file, 'utf8'));
    native.results.insert.rowsPerSec = opts.rows / (native.results.insert.totalMs / 1000);
    report.engines.push({ engine: 'flatsql-native', results: native.results, checks: {} });
    await rm(file, { force: true });
  }
}

if (opts.network) {
  const srv = await createServer({ port: 0, quiet: true });
  try {
    report.network = await runNetwork({ baseUrl: srv.url, Database, fetch: nodeFetch, onProgress: log });
  } finally {
    agent.destroy();
    await srv.close();
  }
}

// Sanity check: every engine must have produced the same answers.
const reference = report.engines[0]?.checks;
for (const e of report.engines.slice(1)) {
  for (const [k, v] of Object.entries(e.checks)) {
    if (reference && reference[k] !== undefined && reference[k] !== v) {
      console.error(`WARNING: ${e.engine} returned ${v} for ${k}, ${report.engines[0].engine} returned ${reference[k]}`);
    }
  }
}

const md = markdown(report);
console.log(md);
await mkdir(join(ROOT, 'bench', 'results'), { recursive: true });
const out = opts.out ?? join(ROOT, 'bench', 'results', `${report.date.slice(0, 19).replace(/[:T]/g, '-')}-node.json`);
await writeFile(out, JSON.stringify(report, null, 2));
if (opts.md) await writeFile(opts.md, md + '\n');
console.error(`results: ${out}`);
