// Browser benchmark page: the same suite as `npm run bench`, plus durable
// writes to IndexedDB, executed in a real browser engine.

import { Database, FlatDB, IndexedDbStore, initEngine, version } from './bridge/index.js';
import { flatsqlEngine, jsEngine, sqljsEngine } from './bench/engines.js';
import { runNetwork } from './bench/network.js';
import { makeRow, READ_OPS, runEngine } from './bench/suite.js';

const $ = (id) => document.getElementById(id);
const now = () => performance.now();
let lastReport = null;

const fmtUs = (us) => (us >= 1000 ? `${(us / 1000).toFixed(2)} ms` : `${us.toFixed(us < 10 ? 2 : 1)} µs`);
const fmtMs = (ms) => (ms < 1 ? `${(ms * 1000).toFixed(0)} µs` : `${ms.toFixed(ms < 10 ? 2 : 1)} ms`);

function table(headers, rows) {
  const t = document.createElement('table');
  const head = t.createTHead().insertRow();
  for (const h of headers) head.append(Object.assign(document.createElement('th'), { textContent: h }));
  const body = t.createTBody();
  for (const r of rows) {
    const tr = body.insertRow();
    r.forEach((v, i) => {
      const td = tr.insertCell();
      td.textContent = v;
      if (i > 0) td.className = 'num';
    });
  }
  return t;
}

function progress(msg) {
  $('progress').textContent = msg;
}

async function gzipSize(bytes) {
  if (typeof CompressionStream === 'undefined') return null;
  const stream = new Blob([bytes]).stream().pipeThrough(new CompressionStream('gzip'));
  return (await new Response(stream).arrayBuffer()).byteLength;
}

async function wasmInfo(name, url) {
  const t0 = now();
  const bytes = new Uint8Array(await (await fetch(url)).arrayBuffer());
  const fetchMs = now() - t0;
  const t1 = now();
  await WebAssembly.compile(bytes);
  return { name, bytes: bytes.length, gzipBytes: await gzipSize(bytes), fetchMs, compileMs: now() - t1 };
}

// Minimal IndexedDB helpers for the sql.js persistence baseline.
function idbOpen(name) {
  return new Promise((resolve, reject) => {
    const req = indexedDB.open(name, 1);
    req.onupgradeneeded = () => req.result.createObjectStore('kv');
    req.onsuccess = () => resolve(req.result);
    req.onerror = () => reject(req.error);
  });
}
function idbPut(db, key, value) {
  return new Promise((resolve, reject) => {
    const tx = db.transaction('kv', 'readwrite');
    tx.objectStore('kv').put(value, key);
    tx.oncomplete = resolve;
    tx.onerror = () => reject(tx.error);
  });
}
function idbDelete(name) {
  return new Promise((resolve) => {
    const req = indexedDB.deleteDatabase(name);
    req.onsuccess = req.onerror = req.onblocked = () => resolve();
  });
}

function stats(samples) {
  const s = [...samples].sort((a, b) => a - b);
  const mean = s.reduce((a, b) => a + b, 0) / s.length;
  return { n: s.length, meanMs: mean, p50Ms: s[s.length >> 1], p95Ms: s[Math.min(s.length - 1, Math.round(s.length * 0.95))] };
}

/** Durable writes: Flat-SQL WAL in IndexedDB vs exporting the whole sql.js file. */
async function runDurability(SQL, rows) {
  const out = {};
  const store = new IndexedDbStore('bench-durability');
  await store.destroy();
  const db = await FlatDB.open({ store, flush: 'manual', checkpointBytes: 1 << 30 });
  db.exec('CREATE TABLE items (id INTEGER PRIMARY KEY, category TEXT, price FLOAT, qty INT, name TEXT)');
  const st = db.prepare('INSERT INTO items VALUES (?, ?, ?, ?, ?)');
  progress(`trwałość: ${rows} wierszy w jednej transakcji + flush`);
  let t0 = now();
  db.exec('BEGIN');
  for (let i = 0; i < rows; i++) db.execute(st, makeRow(i));
  db.exec('COMMIT');
  const execMs = now() - t0;
  await db.flush();
  out.bulk = { rows, execMs, durableMs: now() - t0, walBytes: store.walSize };

  progress('trwałość: 100 pojedynczych zatwierdzeń z await flush()');
  const commits = [];
  for (let i = 0; i < 100; i++) {
    const t = now();
    db.execute(st, makeRow(rows + i));
    await db.flush();
    commits.push(now() - t);
  }
  out.flatsqlCommit = stats(commits);

  progress('trwałość: 1000 zatwierdzeń, jeden flush (group commit)');
  t0 = now();
  for (let i = 0; i < 1000; i++) db.execute(st, makeRow(rows + 100 + i));
  await db.flush();
  out.groupCommit = { commits: 1000, totalMs: now() - t0 };

  t0 = now();
  await db.checkpoint();
  out.checkpoint = { ms: now() - t0, snapshotBytes: store.snapshotSize };
  db.exec('INSERT INTO items VALUES (-1, \'x\', 0, 0, \'after checkpoint\')');
  await db.close();

  progress('trwałość: ponowne otwarcie bazy z IndexedDB');
  t0 = now();
  const reopened = await FlatDB.open({ store: new IndexedDbStore('bench-durability') });
  out.reopen = { ms: now() - t0, restoreMs: reopened.recovery.ms, rows: reopened.query('SELECT COUNT(*) AS n FROM items')[0].n };
  await reopened.close();
  await new IndexedDbStore('bench-durability').destroy();

  // sql.js has no incremental persistence: durability means exporting the
  // whole database file and storing it.
  progress('trwałość: sql.js – export całej bazy do IndexedDB po każdym zatwierdzeniu');
  await idbDelete('bench-sqljs');
  const idb = await idbOpen('bench-sqljs');
  const sdb = new SQL.Database();
  sdb.exec('CREATE TABLE items (id INTEGER PRIMARY KEY, category TEXT, price FLOAT, qty INT, name TEXT)');
  const ins = sdb.prepare('INSERT INTO items VALUES (?, ?, ?, ?, ?)');
  sdb.exec('BEGIN');
  for (let i = 0; i < rows; i++) ins.run(makeRow(i));
  sdb.exec('COMMIT');
  ins.free();
  const sqljsCommits = [];
  for (let i = 0; i < 20; i++) {
    const t = now();
    sdb.run('INSERT INTO items VALUES (?, ?, ?, ?, ?)', makeRow(rows + i));
    await idbPut(idb, 'db', sdb.export());
    sqljsCommits.push(now() - t);
  }
  out.sqljsCommit = { ...stats(sqljsCommits), fileBytes: sdb.export().length };
  sdb.close();
  idb.close();
  await idbDelete('bench-sqljs');
  return out;
}

function renderResults(report) {
  const engines = report.engines.map((e) => e.engine);
  const ops = [
    ['insert', 'Insert (1 txn, prepared)'],
    ...READ_OPS.map((o) => [o.name, o.label]),
    ['update_10pct', 'UPDATE 10% wierszy'],
    ['delete_10pct', 'DELETE 10% wierszy'],
    ['snapshot', 'Snapshot / export'],
    ['restore', 'Restore / import'],
  ];
  const rows = ops.map(([key, label]) => [
    label,
    ...report.engines.map((e) => {
      const r = e.results[key];
      if (!r) return '–';
      if (key === 'insert') return `${fmtUs(r.meanUs)} (${Math.round(r.rowsPerSec).toLocaleString('pl-PL')}/s)`;
      return r.n > 1 ? `${fmtUs(r.meanUs)} (p95 ${fmtUs(r.p95Us)})` : fmtUs(r.meanUs);
    }),
  ]);
  $('results').replaceChildren(table(['Operacja', ...engines], rows));
}

function renderDurability(d) {
  $('durability').replaceChildren(
    table(
      ['Pomiar', 'Wynik'],
      [
        [`Flat-SQL: ${d.bulk.rows.toLocaleString('pl-PL')} wierszy, 1 transakcja (wykonanie)`, fmtMs(d.bulk.execMs)],
        ['Flat-SQL: … do zapisania WAL w IndexedDB', `${fmtMs(d.bulk.durableMs)} (WAL ${(d.bulk.walBytes / 1e6).toFixed(2)} MB)`],
        ['Flat-SQL: pojedyncze zatwierdzenie + await flush()', `${fmtMs(d.flatsqlCommit.meanMs)} (p95 ${fmtMs(d.flatsqlCommit.p95Ms)})`],
        ['Flat-SQL: 1000 zatwierdzeń + 1 flush (group commit)', `${fmtMs(d.groupCommit.totalMs)} (${fmtUs((d.groupCommit.totalMs * 1000) / 1000)} / zatwierdzenie)`],
        ['Flat-SQL: checkpoint (snapshot do IndexedDB)', `${fmtMs(d.checkpoint.ms)} (${(d.checkpoint.snapshotBytes / 1e6).toFixed(2)} MB)`],
        ['Flat-SQL: ponowne otwarcie (IndexedDB → pamięć)', `${fmtMs(d.reopen.ms)} (w tym odtworzenie ${fmtMs(d.reopen.restoreMs)})`],
        ['sql.js: zatwierdzenie = export całej bazy + zapis do IndexedDB', `${fmtMs(d.sqljsCommit.meanMs)} (plik ${(d.sqljsCommit.fileBytes / 1e6).toFixed(2)} MB)`],
      ],
    ),
  );
}

function renderNetwork(net) {
  $('network').replaceChildren(
    table(
      ['Tryb', 'średnio', 'p50', 'p95'],
      net.map((n) => [n.mode, fmtMs(n.meanMs), fmtMs(n.p50Ms), fmtMs(n.p95Ms)]),
    ),
  );
}

function renderWasm(list) {
  $('wasm').replaceChildren(
    table(
      ['Moduł', 'rozmiar', 'gzip', 'pobranie', 'kompilacja'],
      list.map((w) => [w.name, `${(w.bytes / 1024).toFixed(0)} KiB`, w.gzipBytes ? `${(w.gzipBytes / 1024).toFixed(0)} KiB` : '–', fmtMs(w.fetchMs), fmtMs(w.compileMs)]),
    ),
  );
}

async function start() {
  $('btn-start').disabled = true;
  $('btn-download').disabled = true;
  try {
    const rows = Number($('rows').value);
    const selected = [...document.querySelectorAll('input[name=engine]:checked')].map((i) => i.value);
    progress('ładowanie silników…');
    await initEngine();
    const SQL = await globalThis.initSqlJs({ locateFile: (f) => `vendor/sql.js/${f}` });
    const report = {
      env: navigator.userAgent,
      date: new Date().toISOString(),
      flatsqlVersion: version(),
      rows,
      engines: [],
      wasm: [
        await wasmInfo('flatsql (Rust → wasm32)', 'pkg/flatsql_bg.wasm'),
        await wasmInfo('sql.js (SQLite → Emscripten)', 'vendor/sql.js/sql-wasm-browser.wasm'),
      ],
    };
    renderWasm(report.wasm);
    const factories = {
      flatsql: () => flatsqlEngine(Database),
      json: () => flatsqlEngine(Database, { json: true }),
      sqljs: () => sqljsEngine(SQL),
      js: () => jsEngine(),
    };
    for (const key of selected) {
      report.engines.push(await runEngine(factories[key](), { rows, onProgress: progress }));
      renderResults(report);
    }
    if ($('opt-durability').checked) {
      report.durability = await runDurability(SQL, Math.min(rows, 50_000));
      renderDurability(report.durability);
    }
    if ($('opt-network').checked) {
      report.network = await runNetwork({ baseUrl: location.origin, Database, onProgress: progress });
      renderNetwork(report.network);
    }
    lastReport = report;
    progress(`Zakończono – ${new Date().toLocaleTimeString('pl-PL')}`);
    $('btn-download').disabled = false;
  } catch (err) {
    console.error(err);
    progress(`Błąd: ${err.message}`);
  } finally {
    $('btn-start').disabled = false;
  }
}

$('btn-start').addEventListener('click', start);
$('btn-download').addEventListener('click', () => {
  const blob = new Blob([JSON.stringify(lastReport, null, 2)], { type: 'application/json' });
  const a = Object.assign(document.createElement('a'), {
    href: URL.createObjectURL(blob),
    download: `flatsql-bench-${lastReport.date.slice(0, 19).replace(/[:T]/g, '-')}.json`,
  });
  a.click();
  URL.revokeObjectURL(a.href);
});
