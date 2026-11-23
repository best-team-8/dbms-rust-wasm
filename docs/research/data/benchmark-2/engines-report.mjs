// Flat-SQL after the optimisations among the engines of Benchmark 1.
//
//   node docs/research/data/benchmark-2/engines-report.mjs [DIR] [BENCHMARK1_DIR]
//
// Benchmark 2 measured only the configurations of Flat-SQL: the prototype and
// the final version in alternating runs. The reference engines were measured
// in Benchmark 1 with the same tool, package versions, machine and options.
// This script places the final version (DIR/after/raw) among the reference
// engines (BENCHMARK1_DIR/raw) and, to show how far the two sessions differ,
// compares the prototype measured in both of them (DIR/before/raw against the
// Flat-SQL results in BENCHMARK1_DIR/raw). Writes
//   DIR/engines.md         – tables,
//   DIR/engines.json       – aggregated values, places and session differences,
//   DIR/figures/07–12-*.svg – figures of the thesis.
// Only Node.js built-ins are used.

import { existsSync } from 'node:fs';
import { mkdir, readdir, readFile, writeFile } from 'node:fs/promises';
import { join } from 'node:path';
import { fileURLToPath } from 'node:url';

const DIR = process.argv[2] ?? fileURLToPath(new URL('.', import.meta.url));
const B1 = process.argv[3] ?? join(DIR, '..', 'benchmark-1');

// ---- statistics (as in Benchmark 1) ---------------------------------------

const T975 = [12.706, 4.303, 3.182, 2.776, 2.571, 2.447, 2.365, 2.306, 2.262, 2.228, 2.201, 2.179, 2.16, 2.145, 2.131, 2.12, 2.11, 2.101, 2.093, 2.086];

function quantile(sorted, p) {
  const h = (sorted.length - 1) * p;
  const lo = Math.floor(h);
  const hi = Math.min(lo + 1, sorted.length - 1);
  return sorted[lo] + (h - lo) * (sorted[hi] - sorted[lo]);
}

function stat(values) {
  const v = values.filter((x) => Number.isFinite(x)).sort((a, b) => a - b);
  const n = v.length;
  if (!n) return null;
  const mean = v.reduce((a, b) => a + b, 0) / n;
  const sd = n > 1 ? Math.sqrt(v.reduce((a, b) => a + (b - mean) ** 2, 0) / (n - 1)) : 0;
  const half = n > 1 ? ((T975[n - 2] ?? 1.96) * sd) / Math.sqrt(n) : 0;
  return { n, median: quantile(v, 0.5), q1: quantile(v, 0.25), q3: quantile(v, 0.75), min: v[0], max: v[n - 1], mean, sd, ci95: [mean - half, mean + half], cv: mean ? sd / mean : 0 };
}

// ---- formatting (Polish: decimal comma) -----------------------------------

const dec = (x, d) => x.toFixed(d).replace('.', ',');
function num(x) {
  const a = Math.abs(x);
  return a >= 100 ? dec(x, 0) : a >= 10 ? dec(x, 1) : dec(x, 2);
}
function unitFor(us) {
  return us >= 1e6 ? ['s', 1e6] : us >= 1000 ? ['ms', 1000] : ['µs', 1];
}
function dur(s) {
  if (!s) return '–';
  const [u, k] = unitFor(s.median);
  const spread = s.n > 1 ? ` (${num(s.q1 / k)}–${num(s.q3 / k)})` : '';
  return `${num(s.median / k)}${spread} ${u}`;
}
const durPlain = (us) => {
  const [u, k] = unitFor(us);
  return `${num(us / k)} ${u}`;
};
const mib = (b) => `${dec(b / 1048576, b >= 100 * 1048576 ? 0 : 1)} MiB`;
const kib = (b) => `${dec(b / 1024, 0)} KiB`;
const ratio = (x) => (x >= 10 ? dec(x, 0) : x >= 2 ? dec(x, 1) : x >= 0.1 ? dec(x, 2) : x.toPrecision(2).replace('.', ',')) + '×';
const pct = (x) => `${x >= 0 ? '+' : '−'}${dec(Math.abs(x) * 100, 1)}%`;

// ---- loading ----------------------------------------------------------------

async function loadRaw(base, source) {
  const out = [];
  if (!existsSync(base)) return out;
  for (const target of (await readdir(base)).filter((t) => ['node', 'native', 'browser'].includes(t))) {
    for (const kind of await readdir(join(base, target))) {
      for (const engine of await readdir(join(base, target, kind))) {
        for (const f of (await readdir(join(base, target, kind, engine))).filter((x) => x.endsWith('.json'))) {
          const data = JSON.parse(await readFile(join(base, target, kind, engine, f), 'utf8'));
          out.push({ source, target, kind, engine, run: Number(f.slice(0, -5)), result: data.result, data });
        }
      }
    }
  }
  return out;
}

const b1Manifest = JSON.parse(await readFile(join(B1, 'manifest.json'), 'utf8'));
const manifest = JSON.parse(await readFile(join(DIR, 'manifest.json'), 'utf8'));
const raw = [
  ...(await loadRaw(join(B1, 'raw'), 'b1')),
  ...(await loadRaw(join(DIR, 'before', 'raw'), 'before')),
  ...(await loadRaw(join(DIR, 'after', 'raw'), 'after')),
];
const ENGINES = b1Manifest.engines;
const label = (id) => (id === 'flatsql-native' ? 'Flat-SQL natywnie' : ENGINES[id]?.label ?? id);
const ROWS = (b1Manifest.options?.rows ?? 100000).toLocaleString('pl-PL');

// Role of every configuration with respect to Flat-SQL, after the comparison
// of engines (docs/research/engines-and-use-cases.md). Related
// engines solve the same problem: SQL in WebAssembly for the operational data
// of an application (sql.js, SQLite WASM, wa-sqlite) or a local-first store
// with synchronisation competing for offline applications (PouchDB, RxDB).
// PGlite (PostgreSQL compatibility) and DuckDB-Wasm (analytics of large
// files) serve another use; AlaSQL (SQL without WebAssembly), Dexie.js
// (IndexedDB without SQL) and plain JavaScript (lower bound) are reference
// points.
const ROLE = {
  sqljs: 'related',
  'sqlite-wasm': 'related',
  'sqljs-idb': 'related',
  'wa-sqlite-idb': 'related',
  'wa-sqlite-idb-relaxed': 'related',
  'sqlite-opfs': 'related',
  pouchdb: 'related',
  rxdb: 'related',
  pglite: 'other',
  'pglite-idb': 'other',
  'pglite-opfs': 'other',
  duckdb: 'other',
  'duckdb-worker': 'other',
  alasql: 'reference',
  dexie: 'reference',
  'dexie-strict': 'reference',
  'js-map': 'reference',
};
const role = (e) => (e.startsWith('flatsql') ? 'flatsql' : ROLE[e] ?? 'other');
const ROLE_NAME = { flatsql: 'badany', related: 'pokrewny', other: 'inne zastosowanie', reference: 'punkt odniesienia' };

// Values declared by authors of engines that could not be measured, as in
// Benchmark 1. Zeta-Lite is the closest to Flat-SQL in architecture.
const DECLARED = [{ engine: 'Zeta-Lite', source: '[D5]', role: 'related', pointLookupUs: [3.2, 3.7] }];

// ---- metrics ------------------------------------------------------------------

/** name → value of one run (µs for durations, bytes for sizes). */
function metrics(r) {
  const m = {};
  const res = r.result;
  if (r.kind === 'workload') {
    for (const [op, v] of Object.entries(res.ops ?? {})) {
      if (Number.isFinite(v.meanUs)) m[op] = v.meanUs;
      if (Number.isFinite(v.n)) m[`${op}:n`] = v.n;
    }
  } else if (r.kind === 'cold') {
    for (const k of ['initMs', 'totalMs']) if (Number.isFinite(res[k])) m[k] = res[k] * 1000;
    if (Number.isFinite(res.modules?.gzipBytes)) m.gzipBytes = res.modules.gzipBytes;
  } else if (r.kind === 'memory') {
    if (res.after?.wasm) m.wasmBytes = res.after.wasm;
    if (Number.isFinite(res.delta?.heapUsed)) m.heapDelta = res.delta.heapUsed;
  } else if (r.kind === 'checkpoint') {
    if (Number.isFinite(res.idle?.maxGapMs)) m['pause idle'] = res.idle.maxGapMs * 1000;
    for (const s of res.sizes ?? []) {
      if (Number.isFinite(s.maxGapMs)) m[`pause ${s.rows}`] = s.maxGapMs * 1000;
      if (Number.isFinite(s.totalMs)) m[`checkpoint ${s.rows}`] = s.totalMs * 1000;
    }
  }
  return m;
}

const cache = new Map();
/** Statistics of every metric of one configuration in one source, or null. */
function stats(source, target, kind, engine) {
  const key = [source, target, kind, engine].join('|');
  if (!cache.has(key)) {
    const runs = raw.filter((r) => r.source === source && r.target === target && r.kind === kind && r.engine === engine);
    let out = null;
    if (runs.length) {
      out = {};
      const per = runs.map(metrics);
      for (const name of new Set(per.flatMap((p) => Object.keys(p)))) {
        const values = per.map((p) => p[name]).filter(Number.isFinite);
        out[name] = name.endsWith(':n') ? Math.min(...values) : stat(values);
      }
    }
    cache.set(key, out);
  }
  return cache.get(key);
}
// Flat-SQL: the prototype from Benchmark 1, the final version from Benchmark 2;
// every other engine from Benchmark 1.
const proto = (target, kind, engine) => stats('b1', target, kind, engine);
const final = (target, kind, engine) => stats(engine.startsWith('flatsql') ? 'after' : 'b1', target, kind, engine);

const OPS = [
  ['insert', 'Wstawienie N wierszy (1 transakcja)'],
  ['index_build', 'Budowa indeksów (widoki PouchDB)'],
  ['point_lookup', 'Odczyt po kluczu'],
  ['index_lookup', 'COUNT po indeksie'],
  ['range_scan', 'Zakres klucza (1000 wierszy) + SUM'],
  ['full_scan', 'Pełny skan z filtrem'],
  ['group_by', 'GROUP BY (100 grup)'],
  ['top_k', 'ORDER BY … LIMIT 10'],
  ['materialize_10k', 'Pobranie 10 000 wierszy'],
  ['update_10pct', 'UPDATE 10% wierszy'],
  ['delete_10pct', 'DELETE 10% wierszy'],
  ['snapshot', 'Snapshot / eksport'],
  ['restore', 'Odtworzenie / import'],
  ['commit_single', 'Trwałe zatwierdzenie 1 wiersza'],
  ['commit_group', '1000 zatwierdzeń, jedno oczekiwanie'],
  ['reopen', 'Ponowne otwarcie + 1. zapytanie'],
];
const OP_LABEL = Object.fromEntries(OPS);
const READ_N = { point_lookup: 10000, index_lookup: 500, range_scan: 200, full_scan: 20, group_by: 20, top_k: 20, materialize_10k: 20 };

// Groups of run conditions as in Benchmark 1. `flat` is the configuration of
// Flat-SQL whose place is given; `ranked` are the default configurations of
// the other engines it is ranked among (variants with another durability
// setting and plain JavaScript are shown, but not ranked).
const GROUPS = [
  {
    id: 'node-memory',
    target: 'node',
    title: 'Dane w pamięci, Node.js',
    flat: 'flatsql',
    flatExtra: ['flatsql-json'],
    others: ['sqljs', 'sqlite-wasm', 'pglite', 'duckdb', 'alasql', 'js-map'],
    ranked: ['sqljs', 'sqlite-wasm', 'pglite', 'duckdb', 'alasql'],
  },
  {
    id: 'browser-memory',
    target: 'browser',
    title: 'Dane w pamięci, przeglądarka',
    flat: 'flatsql',
    flatExtra: ['flatsql-json'],
    others: ['sqljs', 'sqlite-wasm', 'pglite', 'duckdb', 'alasql', 'js-map'],
    ranked: ['sqljs', 'sqlite-wasm', 'pglite', 'duckdb', 'alasql'],
  },
  {
    id: 'browser-durable',
    target: 'browser',
    title: 'Trwały zapis w IndexedDB, wątek główny przeglądarki',
    flat: 'flatsql-idb',
    flatExtra: ['flatsql-idb-strict'],
    others: ['sqljs-idb', 'wa-sqlite-idb', 'wa-sqlite-idb-relaxed', 'pglite-idb', 'dexie', 'dexie-strict', 'pouchdb', 'rxdb'],
    ranked: ['sqljs-idb', 'wa-sqlite-idb', 'pglite-idb', 'dexie', 'pouchdb', 'rxdb'],
  },
  {
    id: 'browser-worker',
    target: 'browser',
    title: 'Silnik w Web Workerze',
    flat: 'flatsql-worker-idb',
    flatExtra: ['flatsql-worker'],
    others: ['sqlite-opfs', 'pglite-opfs', 'duckdb-worker'],
    ranked: ['sqlite-opfs', 'pglite-opfs', 'duckdb-worker'],
  },
];

// Two sessions differ by up to about 10% for the same code (see `drift`), so
// a place of the final version is uncertain when another ranked engine is
// closer than that.
const CLOSE = 0.1;

/** Place of `value` among `values` (1 = fastest), the number of engines and
 * whether another engine is within CLOSE of it. */
function place(value, values) {
  if (!Number.isFinite(value)) return null;
  const vs = values.filter(Number.isFinite);
  return { place: 1 + vs.filter((x) => x < value).length, of: vs.length + 1, close: vs.some((x) => Math.abs(value / x - 1) < CLOSE) };
}

const result = { benchmark: 'Benchmark 2, Flat-SQL wśród silników Benchmarku 1', final: manifest.after, prototype: b1Manifest.flatsql, sessions: {}, groups: {}, figures: [] };

// ---- how far the two sessions differ: the prototype measured in both ----------

const drift = [];
for (const target of ['node', 'native', 'browser']) {
  for (const engine of [...new Set(raw.filter((r) => r.source === 'before' && r.target === target && r.kind === 'workload').map((r) => r.engine))]) {
    const a = stats('b1', target, 'workload', engine);
    const b = stats('before', target, 'workload', engine);
    if (!a || !b) continue;
    for (const [op] of OPS) if (a[op] && b[op]) drift.push({ target, engine, op, b1: a[op].median, b2: b[op].median, change: b[op].median / a[op].median - 1 });
  }
}
for (const [target, engine, kind, name] of [
  ['browser', 'flatsql', 'cold', 'totalMs'],
  ['browser', 'flatsql-idb', 'cold', 'totalMs'],
  ['node', 'flatsql', 'cold', 'totalMs'],
  ['browser', 'flatsql-idb', 'checkpoint', 'pause 100000'],
]) {
  const a = stats('b1', target, kind, engine)?.[name];
  const b = stats('before', target, kind, engine)?.[name];
  if (a && b) drift.push({ target, engine, op: `${kind} ${name}`, b1: a.median, b2: b.median, change: b.median / a.median - 1 });
}
const absChange = drift.map((d) => Math.abs(d.change)).sort((a, b) => a - b);
const signedChange = drift.map((d) => d.change).sort((a, b) => a - b);
result.sessions = {
  b1: { date: b1Manifest.date, browser: raw.find((r) => r.source === 'b1' && r.target === 'browser')?.data.browser },
  b2: { date: manifest.date, browser: raw.find((r) => r.source === 'after' && r.target === 'browser')?.data.browser },
  compared: drift.length,
  medianChange: quantile(signedChange, 0.5),
  faster: drift.filter((d) => d.change < 0).length,
  medianAbsChange: quantile(absChange, 0.5),
  p90AbsChange: quantile(absChange, 0.9),
  within5: drift.filter((d) => Math.abs(d.change) <= 0.05).length,
  within10: drift.filter((d) => Math.abs(d.change) <= 0.1).length,
  range: [signedChange[0], signedChange[signedChange.length - 1]],
  pairs: drift,
};

// ---- answers: the final version against Benchmark 1 ------------------------------

// The final version gives the same answers as the prototype (make-report.mjs)
// and the prototype the same as every reference engine (Benchmark 1); this
// compares the final version directly with the Flat-SQL results of Benchmark 1.
function checkAnswers() {
  let compared = 0;
  const diff = [];
  for (const r of raw.filter((x) => x.source === 'after' && x.kind === 'workload')) {
    const ref = raw.find((x) => x.source === 'b1' && x.kind === 'workload' && x.target === r.target && x.engine === (r.engine === 'flatsql-native' ? 'flatsql-native' : r.engine))?.result;
    if (!ref) continue;
    for (const [op, c] of Object.entries(r.result.checks ?? {})) {
      const want = ref.checks?.[op];
      if (!want) continue;
      compared++;
      if (c.head !== want.head || (c.n === want.n && c.full !== want.full)) diff.push(`${r.target}/${r.engine}/${op}`);
    }
  }
  return { compared, diff: [...new Set(diff)] };
}
result.checks = checkAnswers();

// ---- tables -----------------------------------------------------------------------

const md = [];
const table = (head, rows, textCols = 1) => {
  md.push(`| ${head.join(' | ')} |`, `|${head.map((_, i) => (i < textCols ? '---' : '---:')).join('|')}|`);
  for (const r of rows) md.push(`| ${r.join(' | ')} |`);
  md.push('');
};
const s = result.sessions;
md.push('# Benchmark 2 – Flat-SQL wśród silników Benchmarku 1', '');
md.push(
  `Wygenerowano skryptem \`engines-report.mjs\`. Flat-SQL po optymalizacjach (\`${manifest.after.describe}\`, ${manifest.after.commit.slice(0, 10)}) pochodzi z \`after/raw\` (pomiar ${manifest.date.slice(0, 10)}), prototyp Flat-SQL (\`${b1Manifest.flatsql.describe}\`) i pozostałe silniki z \`../benchmark-1/raw\` (pomiar ${b1Manifest.date.slice(0, 10)}). Wartość: mediana z ${b1Manifest.options.runs} uruchomień procesu, w nawiasie pierwszy i trzeci kwartyl; [n/N] – operacja przerwana limitem czasu po n z N powtórzeń. Miejsce liczone według mediany wśród domyślnych konfiguracji silników grupy (bez czystego JavaScriptu i bez wariantów z innym ustawieniem trwałości). Rola silnika według analizy porównawczej silników: pokrewny – SQL w WebAssembly dla danych operacyjnych aplikacji albo magazyn local-first z synchronizacją; inne zastosowanie – PGlite i DuckDB-Wasm; punkt odniesienia – AlaSQL, Dexie.js, czysty JavaScript.`,
  '',
);

md.push('## Różnica między sesjami pomiaru', '');
md.push(
  `Prototyp zmierzony w obu sesjach (Benchmark 1: ${s.b1.date.slice(0, 10)}, przeglądarka ${s.b1.browser?.name} ${s.b1.browser?.version}; Benchmark 2: ${s.b2.date.slice(0, 10)}, przeglądarka ${s.b2.browser?.name} ${s.b2.browser?.version}). Zmiana mediany prototypu z Benchmarku 2 wobec Benchmarku 1 w ${s.compared} parach konfiguracja–operacja: mediana ${pct(s.medianChange)}, w Benchmarku 2 szybciej w ${s.faster} parach; mediana wartości bezwzględnej ${dec(s.medianAbsChange * 100, 1)}%, 90. percentyl ${dec(s.p90AbsChange * 100, 1)}%; w granicach ±5%: ${s.within5}, ±10%: ${s.within10}. Poniżej pary o zmianie większej niż 5%. Miejsce wersji końcowej oznaczone gwiazdką (*) jest niepewne: inny silnik grupy różni się od niej o mniej niż ${dec(CLOSE * 100, 0)}%.`,
  '',
);
table(
  ['Środowisko', 'Konfiguracja', 'Pomiar', 'Benchmark 1', 'Benchmark 2', 'Zmiana'],
  drift.filter((d) => Math.abs(d.change) > 0.05).map((d) => [d.target, label(d.engine), OP_LABEL[d.op] ?? d.op, durPlain(d.b1), durPlain(d.b2), pct(d.change)]),
  3,
);
md.push(result.checks.diff.length ? `Wyniki wersji końcowej różne od Benchmarku 1: ${result.checks.diff.join(', ')} (${result.checks.compared} porównań).` : `Wyniki wersji końcowej zgodne z wynikami Flat-SQL z Benchmarku 1 (a więc ze wszystkimi silnikami): ${result.checks.compared} porównań sum kontrolnych.`, '');

function cell(st, op) {
  const v = st?.[op];
  if (!v) return '–';
  const n = st[`${op}:n`];
  const ref = READ_N[op];
  return dur(v) + (ref && n < ref ? ` [${n}/${ref}]` : '');
}

for (const g of GROUPS) {
  const out = (result.groups[g.id] = { title: g.title, flat: g.flat, ops: {} });
  const pFlat = proto(g.target, 'workload', g.flat);
  const fFlat = final(g.target, 'workload', g.flat);
  if (!pFlat || !fFlat) continue;
  const extras = g.flatExtra.filter((e) => final(g.target, 'workload', e));
  const others = g.others.filter((e) => final(g.target, 'workload', e));
  md.push(`## ${g.title}`, '');
  const head = ['Operacja', `${label(g.flat)}: prototyp`, `${label(g.flat)}: po optymalizacjach`, ...extras.map((e) => `${label(e)}: po optymalizacjach`), ...others.map(label), 'Miejsce Flat-SQL: prototyp → po'];
  const rows = [['Rola', ROLE_NAME.flatsql, ROLE_NAME.flatsql, ...extras.map(() => ROLE_NAME.flatsql), ...others.map((e) => ROLE_NAME[role(e)]), '']];
  for (const [op, name] of OPS) {
    const rankedP = g.ranked.map((e) => proto(g.target, 'workload', e)?.[op]?.median);
    const before = place(pFlat[op]?.median, rankedP);
    const after = place(fFlat[op]?.median, rankedP);
    const related = g.ranked.filter((e) => role(e) === 'related' && final(g.target, 'workload', e)?.[op]);
    const fastest = related.sort((a, b) => final(g.target, 'workload', a)[op].median - final(g.target, 'workload', b)[op].median)[0];
    const cells = [cell(pFlat, op), cell(fFlat, op), ...extras.map((e) => cell(final(g.target, 'workload', e), op)), ...others.map((e) => cell(final(g.target, 'workload', e), op))];
    if (!cells.some((c) => c !== '–')) continue;
    const bothPlaces = before && after;
    rows.push([name, ...cells, bothPlaces ? `${before.place} → ${after.place}${after.close ? '*' : ''} z ${after.of}` : '–']);
    out.ops[op] = {
      prototype: pFlat[op] ?? null,
      final: fFlat[op] ?? null,
      placeBefore: before,
      placeAfter: after,
      fastestRelated: fastest ?? null,
      vsRelatedBefore: fastest && pFlat[op] ? pFlat[op].median / final(g.target, 'workload', fastest)[op].median : null,
      vsRelatedAfter: fastest && fFlat[op] ? fFlat[op].median / final(g.target, 'workload', fastest)[op].median : null,
    };
  }
  table(head, rows);
}

md.push('## Flat-SQL wobec najszybszego pokrewnego silnika', '');
md.push('Czas Flat-SQL podzielony przez czas najszybszego pokrewnego silnika grupy (domyślne konfiguracje); poniżej 1× Flat-SQL jest szybszy.', '');
for (const g of GROUPS) {
  const ops = Object.entries(result.groups[g.id]?.ops ?? {}).filter(([, v]) => v.vsRelatedBefore && v.vsRelatedAfter);
  if (!ops.length) continue;
  md.push(`### ${g.title}`, '');
  table(
    ['Operacja', 'Najszybszy pokrewny', 'Prototyp', 'Po optymalizacjach'],
    ops.map(([op, v]) => [OP_LABEL[op], label(v.fastestRelated), ratio(v.vsRelatedBefore), ratio(v.vsRelatedAfter)]),
    2,
  );
}

// Module size of a package: the largest set of files loaded by any
// configuration of the same adapter, as in Benchmark 1.
function moduleGzip(e, source) {
  const ids = Object.keys(ENGINES).filter((x) => ENGINES[x]?.module === ENGINES[e]?.module);
  const sizes = ids.map((x) => stats(x.startsWith('flatsql') ? source : 'b1', 'browser', 'cold', x)?.gzipBytes?.median ?? 0);
  return Math.max(0, ...sizes) || null;
}
md.push('## Rozmiar modułu, zimny start i pamięć', '');
{
  const engines = ['flatsql', 'flatsql-idb', 'flatsql-worker-idb', 'sqljs', 'sqlite-wasm', 'pglite', 'duckdb', 'alasql', 'js-map', 'sqljs-idb', 'wa-sqlite-idb', 'pglite-idb', 'dexie', 'pouchdb', 'rxdb', 'sqlite-opfs', 'pglite-opfs', 'duckdb-worker'];
  const rows = [];
  for (const e of engines) {
    const sides = e.startsWith('flatsql') ? [['b1', 'prototyp'], ['after', 'po optymalizacjach']] : [['b1', '']];
    for (const [source, tag] of sides) {
      const cold = stats(source, 'browser', 'cold', e);
      const coldNode = stats(source, 'node', 'cold', e);
      const mem = stats(source, 'browser', 'memory', e) ?? stats(source, 'node', 'memory', e);
      const size = moduleGzip(e, source);
      rows.push([
        tag ? `${label(e)}, ${tag}` : label(e),
        ROLE_NAME[role(e)],
        size ? kib(size) : '–',
        dur(cold?.totalMs),
        dur(coldNode?.totalMs),
        mem?.wasmBytes ? mib(mem.wasmBytes.median) : ENGINES[e]?.wasm === false ? 'nie dotyczy' : '–',
      ]);
    }
  }
  table(['Silnik', 'Rola', 'Moduł (gzip)', 'Zimny start – przeglądarka', 'Zimny start – Node.js', 'Pamięć WASM po wczytaniu'], rows, 2);
}

md.push('## Pauza checkpointu', '');
{
  const sizes = [10000, 50000, 100000];
  const rows = [];
  for (const [e, source, tag] of [['flatsql-idb', 'b1', 'prototyp'], ['flatsql-idb', 'after', 'po optymalizacjach'], ['flatsql-idb-strict', 'b1', 'prototyp'], ['flatsql-idb-strict', 'after', 'po optymalizacjach'], ['sqljs-idb', 'b1', '']]) {
    const cp = stats(source, 'browser', 'checkpoint', e);
    if (!cp) continue;
    rows.push([tag ? `${label(e)}, ${tag}` : label(e), ...sizes.flatMap((n) => [dur(cp[`pause ${n}`]), dur(cp[`checkpoint ${n}`])])]);
  }
  table(['Silnik', ...sizes.flatMap((n) => [`${n / 1000} tys. – najdłuższa pauza`, `${n / 1000} tys. – cały checkpoint`])], rows);
}

// ---- SVG --------------------------------------------------------------------------

const C = { ink: '#0b0b0b', ink2: '#52514e', grid: '#e4e3df', surface: '#ffffff', blue: '#2a78d6', orange: '#eb6834', aqua: '#1baf7a', mute: '#8b8a86', paleBlue: '#9ec5f4' };
const esc = (t) => String(t).replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;');
const FONT = 'font-family="Segoe UI, system-ui, sans-serif"';
const textWidth = (t, px = 12) => t.length * px * 0.56;
function logTicks(lo, hi) {
  const ticks = [];
  for (let e = Math.floor(Math.log10(lo)); e <= Math.ceil(Math.log10(hi)); e++) ticks.push(10 ** e);
  return ticks;
}
const tickLabel = (us) => {
  const [u, k] = unitFor(us);
  const v = us / k;
  return `${v < 1 ? dec(v, 1) : Math.round(v)} ${u}`;
};

// Marks by role: Flat-SQL blue (the prototype as an empty circle), related
// engines orange diamonds, the rest grey circles; a value declared by an
// author and not measured is an empty dashed diamond.
const MARK = {
  final: { name: 'Flat-SQL po optymalizacjach', color: C.blue, shape: 'circle', fill: true },
  proto: { name: 'Flat-SQL, prototyp', color: C.blue, shape: 'circle', fill: false },
  related: { name: 'silnik pokrewny', color: C.orange, shape: 'diamond', fill: true },
  other: { name: 'inne zastosowanie lub punkt odniesienia', color: C.mute, shape: 'circle', fill: true },
  declared: { name: 'pokrewny, deklaracja autora (niezmierzone)', color: C.orange, shape: 'diamond', fill: false, dashed: true },
};
const markOf = (e) => (role(e) === 'related' ? MARK.related : MARK.other);
function marker(m, cx, cy) {
  const fill = m.fill ? m.color : C.surface;
  const stroke = m.fill ? C.surface : m.color;
  const dash = m.dashed ? ' stroke-dasharray="2 1.5"' : '';
  if (m.shape === 'diamond') return `<path d="M${cx} ${cy - 6}L${cx + 6} ${cy}L${cx} ${cy + 6}L${cx - 6} ${cy}Z" fill="${fill}" stroke="${stroke}" stroke-width="2"${dash}/>`;
  return `<circle cx="${cx}" cy="${cy}" r="${m.fill ? 5 : 4.5}" fill="${fill}" stroke="${stroke}" stroke-width="2"${dash}/>`;
}
function legend(out, marks, x0, y) {
  let lx = x0;
  for (const m of marks) {
    out.push(marker(m, lx + 6, y - 4), `<text x="${lx + 16}" y="${y}" font-size="12" fill="${C.ink}">${esc(m.name)}</text>`);
    lx += 34 + textWidth(m.name);
  }
}

/**
 * One row per configuration on a log axis; a Flat-SQL row shows the prototype
 * and the final version joined by a line, and each value is written next to
 * its row. Rows are sorted by time within each section unless `sort` is false;
 * `unit` '×' plots ratios; `valueColumn` writes the values in a column right
 * of the plot; a row's `note` goes to a column at the right edge.
 */
function rolePlot({ title, subtitle, sections, refs = [], unit = 'us', sort = true, valueColumn = false, footer }) {
  const rows = [];
  for (const sec of sections) {
    const list = sec.rows.filter((r) => r.points.some((p) => p.value));
    if (!list.length) continue;
    rows.push({ section: sec.name });
    rows.push(...(sort ? list.sort((a, b) => key(a) - key(b)) : list));
  }
  function key(r) {
    const p = r.points.find((q) => q.mark === MARK.final) ?? r.points[0];
    return p.value.median;
  }
  const fmt = unit === 'us' ? durPlain : ratio;
  const tickFmt = unit === 'us' ? tickLabel : (v) => `${v < 1 ? dec(v, v < 0.1 ? 2 : 1) : v}×`;
  const left = Math.round(Math.max(170, ...rows.map((r) => textWidth(r.label ?? '') + 24)));
  const noteW = Math.round(Math.max(0, ...rows.map((r) => (r.note ? textWidth(r.note, 11) + 24 : 0))));
  const plotW = 520;
  const used = [MARK.final, MARK.proto, MARK.related, MARK.other, MARK.declared].filter((m) => rows.some((r) => (r.points ?? []).some((p) => p.mark === m && p.value)));
  const legendW = Math.round(32 + used.reduce((a, m) => a + 34 + textWidth(m.name), 0));
  const width = Math.max(left + plotW + 170 + noteW, legendW);
  const rowH = 22;
  const top = 86;
  const height = top + rows.length * rowH + 48;
  const values = rows.flatMap((r) => (r.points ?? []).flatMap((p) => (p.value ? [p.value.q1, p.value.q3, p.value.median] : []))).filter((v) => v > 0);
  const ticks = logTicks(Math.min(...values, ...refs.map((r) => r.value)), Math.max(...values, ...refs.map((r) => r.value)));
  const lo = Math.log10(ticks[0]);
  const hi = Math.log10(ticks[ticks.length - 1]);
  const x = (v) => left + ((Math.log10(v) - lo) / (hi - lo || 1)) * plotW;
  const out = [];
  out.push(`<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 ${width} ${height}" width="${width}" height="${height}" ${FONT}>`);
  out.push(`<rect width="${width}" height="${height}" fill="${C.surface}"/>`);
  out.push(`<text x="16" y="24" font-size="15" font-weight="600" fill="${C.ink}">${esc(title)}</text>`);
  if (subtitle) out.push(`<text x="16" y="42" font-size="12" fill="${C.ink2}">${esc(subtitle)}</text>`);
  legend(out, used, 16, 66);
  const bottom = top + rows.length * rowH;
  for (const t of ticks) {
    out.push(`<line x1="${x(t)}" x2="${x(t)}" y1="${top - 4}" y2="${bottom}" stroke="${C.grid}" stroke-width="1"/>`);
    out.push(`<text x="${x(t)}" y="${bottom + 16}" font-size="11" fill="${C.ink2}" text-anchor="middle">${esc(tickFmt(t))}</text>`);
  }
  for (const r of refs) {
    out.push(`<line x1="${x(r.value)}" x2="${x(r.value)}" y1="${top - 4}" y2="${bottom}" stroke="${C.ink2}" stroke-width="1.5" stroke-dasharray="4 3"/>`);
    out.push(`<text x="${x(r.value) + 4}" y="${top + 8}" font-size="11" fill="${C.ink2}">${esc(r.label)}</text>`);
  }
  rows.forEach((row, i) => {
    const y0 = top + i * rowH;
    const y = y0 + rowH / 2;
    if (row.section) {
      if (i > 0) out.push(`<line x1="16" x2="${width - 16}" y1="${y0 + 2}" y2="${y0 + 2}" stroke="${C.grid}"/>`);
      out.push(`<text x="16" y="${y + 5}" font-size="12" font-weight="600" fill="${C.ink2}">${esc(row.section)}</text>`);
      return;
    }
    const pts = row.points.filter((p) => p.value);
    const tip = (p) =>
      p.mark === MARK.declared
        ? `${fmt(p.value.q1)}–${fmt(p.value.q3)} według autora, niepotwierdzone pomiarem`
        : `${p.mark === MARK.proto ? 'prototyp ' : p.mark === MARK.final ? 'po optymalizacjach ' : ''}${p.value.n > 1 ? `mediana ${fmt(p.value.median)}, Q1–Q3 ${fmt(p.value.q1)}–${fmt(p.value.q3)}, n = ${p.value.n}` : fmt(p.value.median)}`;
    out.push(`<g><title>${esc(`${row.label}: ${pts.map(tip).join('; ')}${row.note ? ` (${row.note})` : ''}`)}</title>`);
    out.push(`<text x="${left - 10}" y="${y + 4}" font-size="12" fill="${C.ink}" text-anchor="end">${esc(row.label)}</text>`);
    const pP = pts.find((p) => p.mark === MARK.proto);
    const pF = pts.find((p) => p.mark === MARK.final);
    if (pP && pF) out.push(`<line x1="${x(pP.value.median)}" x2="${x(pF.value.median)}" y1="${y}" y2="${y}" stroke="${C.paleBlue}" stroke-width="2"/>`);
    for (const p of pts) {
      out.push(`<line x1="${x(p.value.q1)}" x2="${x(p.value.q3)}" y1="${y}" y2="${y}" stroke="${p.mark.color}" stroke-width="2" stroke-linecap="round"${p.mark.dashed ? ' stroke-dasharray="3 3"' : ''}/>`);
    }
    for (const p of [...pts].sort((a, b) => (a.mark === MARK.final) - (b.mark === MARK.final))) out.push(marker(p.mark, x(p.value.median), y));
    const end = valueColumn ? left + plotW + 3 : Math.max(...pts.map((p) => x(p.value.q3)));
    const text = pP && pF ? `${fmt(pP.value.median)} → ${fmt(pF.value.median)}` : pts[0].mark === MARK.declared ? `${fmt(pts[0].value.q1)}–${fmt(pts[0].value.q3)}, deklaracja autora` : fmt(pts[0].value.median);
    out.push(`<text x="${end + 9}" y="${y + 4}" font-size="11" fill="${C.ink2}">${esc(text)}</text>`);
    if (row.note) out.push(`<text x="${width - 16}" y="${y + 4}" font-size="11" fill="${C.ink2}" text-anchor="end">${esc(row.note)}</text>`);
    out.push('</g>');
  });
  out.push(`<text x="${width - 16}" y="${height - 8}" font-size="10.5" fill="${C.ink2}" text-anchor="end">${esc(footer ?? 'skala logarytmiczna; punkt – mediana uruchomień, odcinek – Q1–Q3; w sekcji silniki od najszybszego')}</text>`);
  out.push('</svg>');
  return out.join('\n');
}

/** Horizontal dot plot on a log axis (durations), one line per series in a row. */
function dotPlot({ title, subtitle, rows, series }) {
  const left = Math.round(Math.max(170, ...rows.map((r) => textWidth(r.label ?? '') + 24)));
  const width = left + 640;
  const plotW = width - left - 150;
  const rowH = 14 + series.length * 12;
  const top = 86;
  const height = top + rows.length * rowH + 44;
  const values = rows.flatMap((r) => r.points.flatMap((p) => (p.value ? [p.value.q1, p.value.q3, p.value.median] : []))).filter((v) => v > 0);
  const ticks = logTicks(Math.min(...values), Math.max(...values));
  const lo = Math.log10(ticks[0]);
  const hi = Math.log10(ticks[ticks.length - 1]);
  const x = (v) => left + ((Math.log10(v) - lo) / (hi - lo || 1)) * plotW;
  const out = [];
  out.push(`<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 ${width} ${height}" width="${width}" height="${height}" ${FONT}>`);
  out.push(`<rect width="${width}" height="${height}" fill="${C.surface}"/>`);
  out.push(`<text x="16" y="24" font-size="15" font-weight="600" fill="${C.ink}">${esc(title)}</text>`);
  if (subtitle) out.push(`<text x="16" y="42" font-size="12" fill="${C.ink2}">${esc(subtitle)}</text>`);
  legend(out, series.map((q) => q.mark), 16, 66);
  const bottom = top + rows.length * rowH;
  for (const t of ticks) {
    out.push(`<line x1="${x(t)}" x2="${x(t)}" y1="${top - 4}" y2="${bottom}" stroke="${C.grid}" stroke-width="1"/>`);
    out.push(`<text x="${x(t)}" y="${bottom + 16}" font-size="11" fill="${C.ink2}" text-anchor="middle">${esc(tickLabel(t))}</text>`);
  }
  rows.forEach((row, i) => {
    const y0 = top + i * rowH;
    out.push(`<text x="${left - 10}" y="${y0 + rowH / 2 + 4}" font-size="12" fill="${C.ink}" text-anchor="end">${esc(row.label)}</text>`);
    row.points.forEach((p, j) => {
      if (!p.value) return;
      const sr = series.find((q) => q.name === p.series) ?? series[0];
      const y = y0 + 9 + j * 12;
      out.push(`<g><title>${esc(`${row.label} – ${sr.name}: mediana ${durPlain(p.value.median)}, Q1–Q3 ${durPlain(p.value.q1)}–${durPlain(p.value.q3)}, n = ${p.value.n}`)}</title>`);
      out.push(`<line x1="${x(p.value.q1)}" x2="${x(p.value.q3)}" y1="${y}" y2="${y}" stroke="${sr.mark.color}" stroke-width="2" stroke-linecap="round"/>`);
      out.push(marker(sr.mark, x(p.value.median), y));
      out.push('</g>');
    });
  });
  out.push(`<text x="${width - 16}" y="${height - 8}" font-size="10.5" fill="${C.ink2}" text-anchor="end">skala logarytmiczna; punkt – mediana uruchomień, odcinek – Q1–Q3</text>`);
  out.push('</svg>');
  return out.join('\n');
}

/** Lines over a few x values with Q1–Q3 whiskers and a reference line (ms). */
function lineChart({ title, subtitle, xs, xLabel, series, ref }) {
  const left = 70;
  const width = 860;
  const height = 400;
  const top = 92;
  const labelW = Math.max(...series.map((q) => textWidth(q.name))) + 20;
  const plotW = width - left - labelW;
  const plotH = height - top - 56;
  const max = Math.max(ref?.value ?? 0, ...series.flatMap((sr) => sr.points.map((p) => p?.q3 ?? 0))) * 1.1;
  const step = [1, 2, 5, 10, 20, 25, 50, 100].find((st) => max / st <= 6) ?? 100;
  const yMax = Math.ceil(max / step) * step;
  const x = (i) => left + (i / (xs.length - 1)) * plotW;
  const y = (v) => top + plotH - (v / yMax) * plotH;
  const out = [];
  out.push(`<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 ${width} ${height}" width="${width}" height="${height}" ${FONT}>`);
  out.push(`<rect width="${width}" height="${height}" fill="${C.surface}"/>`);
  out.push(`<text x="16" y="24" font-size="15" font-weight="600" fill="${C.ink}">${esc(title)}</text>`);
  if (subtitle) out.push(`<text x="16" y="42" font-size="12" fill="${C.ink2}">${esc(subtitle)}</text>`);
  legend(out, [...new Set(series.map((sr) => sr.mark))], 16, 66);
  for (let v = 0; v <= yMax; v += step) {
    out.push(`<line x1="${left}" x2="${left + plotW}" y1="${y(v)}" y2="${y(v)}" stroke="${C.grid}"/>`);
    out.push(`<text x="${left - 8}" y="${y(v) + 4}" font-size="11" fill="${C.ink2}" text-anchor="end">${v} ms</text>`);
  }
  xs.forEach((xv, i) => out.push(`<text x="${x(i)}" y="${top + plotH + 18}" font-size="11" fill="${C.ink2}" text-anchor="middle">${esc(xv)}</text>`));
  out.push(`<text x="${left + plotW / 2}" y="${height - 8}" font-size="11" fill="${C.ink2}" text-anchor="middle">${esc(xLabel)}</text>`);
  if (ref) {
    out.push(`<line x1="${left}" x2="${left + plotW}" y1="${y(ref.value)}" y2="${y(ref.value)}" stroke="${C.ink2}" stroke-width="1.5" stroke-dasharray="4 3"/>`);
    out.push(`<text x="${left + 4}" y="${y(ref.value) - 5}" font-size="11" fill="${C.ink2}">${esc(ref.label)}</text>`);
  }
  const endLabels = [];
  for (const sr of series) {
    const pts = sr.points.map((p, i) => (p ? [x(i), y(p.median)] : null)).filter(Boolean);
    out.push(`<polyline points="${pts.map((p) => p.join(',')).join(' ')}" fill="none" stroke="${sr.mark.color}" stroke-width="2"${sr.mark.fill ? '' : ' stroke-dasharray="5 3"'}/>`);
    sr.points.forEach((p, i) => {
      if (!p) return;
      out.push(`<g><title>${esc(`${sr.name}, ${xs[i]}: mediana ${num(p.median)} ms, Q1–Q3 ${num(p.q1)}–${num(p.q3)} ms`)}</title>`);
      out.push(`<line x1="${x(i)}" x2="${x(i)}" y1="${y(p.q1)}" y2="${y(p.q3)}" stroke="${sr.mark.color}" stroke-width="2"/>`);
      out.push(`${marker(sr.mark, x(i), y(p.median))}</g>`);
    });
    const last = pts[pts.length - 1];
    endLabels.push({ x: last[0] + 12, y: last[1] + 4, name: sr.name });
  }
  endLabels.sort((a, b) => a.y - b.y);
  for (let i = 1; i < endLabels.length; i++) endLabels[i].y = Math.max(endLabels[i].y, endLabels[i - 1].y + 15);
  for (const l of endLabels) out.push(`<text x="${l.x}" y="${l.y}" font-size="12" fill="${C.ink}">${esc(l.name)}</text>`);
  out.push('</svg>');
  return out.join('\n');
}

// ---- figures ------------------------------------------------------------------------

const figures = {};
const SECTION = { 'browser-memory': 'dane w pamięci', 'browser-durable': 'trwały zapis w IndexedDB', 'browser-worker': 'Web Worker' };
/** Rows of a role plot for one metric over the browser groups. */
function browserSections(kind, name, include = () => true) {
  const sections = [];
  for (const g of GROUPS.filter((x) => x.target === 'browser')) {
    const rows = [];
    for (const e of [g.flat, ...g.flatExtra]) {
      const p = proto('browser', kind, e)?.[name];
      const f = final('browser', kind, e)?.[name];
      if (p || f) rows.push({ label: label(e), points: [{ mark: MARK.proto, value: p }, { mark: MARK.final, value: f }] });
    }
    for (const e of g.others.filter(include)) {
      const v = final('browser', kind, e)?.[name];
      if (v) rows.push({ label: label(e), points: [{ mark: markOf(e), value: v }] });
    }
    sections.push({ name: `grupa „${SECTION[g.id]}”`, rows });
  }
  return sections;
}

figures['07-flatsql-wobec-pokrewnych.svg'] = (() => {
  const point = (v) => (v ? { n: 1, median: v, q1: v, q3: v } : null);
  const sections = ['browser-memory', 'browser-durable'].map((id) => ({
    name: `grupa „${SECTION[id]}”`,
    rows: Object.entries(result.groups[id]?.ops ?? {})
      .filter(([, v]) => v.vsRelatedBefore && v.vsRelatedAfter)
      .map(([op, v]) => ({ label: OP_LABEL[op], note: label(v.fastestRelated), points: [{ mark: MARK.proto, value: point(v.vsRelatedBefore) }, { mark: MARK.final, value: point(v.vsRelatedAfter) }] })),
  }));
  return rolePlot({
    title: 'Flat-SQL wobec najszybszego pokrewnego silnika – przeglądarka',
    subtitle: 'czas Flat-SQL przez czas najszybszego pokrewnego silnika grupy (podany z prawej); poniżej 1× Flat-SQL jest szybszy',
    sections,
    refs: [{ value: 1, label: 'tyle samo' }],
    unit: '×',
    sort: false,
    valueColumn: true,
    footer: 'skala logarytmiczna; punkt – iloraz median uruchomień',
  });
})();

figures['08-odczyt-po-kluczu.svg'] = rolePlot({
  title: 'Odczyt po kluczu – przeglądarka, Flat-SQL po optymalizacjach wśród silników',
  subtitle: `SELECT * FROM items WHERE id = ?, ${ROWS} wierszy; średnia z 10 000 wykonań w każdym uruchomieniu`,
  sections: [
    ...browserSections('workload', 'point_lookup'),
    { name: 'niemierzone', rows: DECLARED.map((d) => ({ label: `${d.engine} ${d.source}`, points: [{ mark: MARK.declared, value: { n: 0, q1: d.pointLookupUs[0], q3: d.pointLookupUs[1], median: (d.pointLookupUs[0] + d.pointLookupUs[1]) / 2 } }] })) },
  ],
});

figures['09-flatsql-a-sqlite.svg'] = (() => {
  const engines = [
    ['Flat-SQL, prototyp', MARK.proto, (op) => proto('browser', 'workload', 'flatsql')?.[op]],
    ['Flat-SQL po optymalizacjach', MARK.final, (op) => final('browser', 'workload', 'flatsql')?.[op]],
    ['sql.js', { name: 'sql.js (pokrewny)', color: C.orange, shape: 'diamond', fill: true }, (op) => final('browser', 'workload', 'sqljs')?.[op]],
    ['SQLite WASM', { name: 'SQLite WASM (:memory:, pokrewny)', color: C.aqua, shape: 'diamond', fill: true }, (op) => final('browser', 'workload', 'sqlite-wasm')?.[op]],
  ];
  const series = engines.map(([name, mark]) => ({ name, mark: { ...mark, name: mark.name.startsWith('Flat') ? name : mark.name } }));
  const rows = OPS.filter(([op]) => engines.every(([, , get]) => get(op))).map(([op, name]) => ({ label: name, points: engines.map(([sname, , get]) => ({ series: sname, value: get(op) })) }));
  return dotPlot({ title: 'Flat-SQL a pokrewne silniki SQLite – przeglądarka, dane w pamięci', subtitle: `${ROWS} wierszy; czas jednej operacji`, series, rows });
})();

figures['10-zimny-start.svg'] = rolePlot({
  title: 'Zimny start w przeglądarce – Flat-SQL po optymalizacjach wśród silników',
  subtitle: 'nowy proces przeglądarki z pustym profilem; od importu silnika do wyniku pierwszego zapytania',
  sections: browserSections('cold', 'totalMs'),
});

figures['11-trwale-zatwierdzenie.svg'] = rolePlot({
  title: 'Trwałe zatwierdzenie jednego wiersza – przeglądarka',
  subtitle: 'zapis jednego wiersza i oczekiwanie na trwałość; 100 zatwierdzeń w każdym uruchomieniu',
  sections: browserSections('workload', 'commit_single').filter((sec) => sec.rows.length),
});

figures['12-pauza-checkpointu.svg'] = (() => {
  const sizes = [10000, 50000, 100000];
  const ms = (v) => (v ? { ...v, median: v.median / 1000, q1: v.q1 / 1000, q3: v.q3 / 1000 } : null);
  const get = (source, e) => sizes.map((n) => ms(stats(source, 'browser', 'checkpoint', e)?.[`pause ${n}`]));
  return lineChart({
    title: 'Najdłuższa pauza wątku głównego podczas checkpointu',
    subtitle: 'przeglądarka, IndexedDB; sql.js – eksport całej bazy i zapis pliku',
    xs: sizes.map((n) => `${n / 1000} tys.`),
    xLabel: 'liczba wierszy w bazie',
    ref: { value: 1000 / 60, label: 'jedna klatka przy 60 Hz (16,7 ms)' },
    series: [
      { name: 'Flat-SQL + IndexedDB, prototyp', mark: MARK.proto, points: get('b1', 'flatsql-idb') },
      { name: 'Flat-SQL + IndexedDB po optymalizacjach', mark: MARK.final, points: get('after', 'flatsql-idb') },
      { name: 'sql.js + eksport do IndexedDB', mark: MARK.related, points: get('b1', 'sqljs-idb') },
    ],
  });
})();

await mkdir(join(DIR, 'figures'), { recursive: true });
for (const [name, svg] of Object.entries(figures)) {
  await writeFile(join(DIR, 'figures', name), `${svg}\n`);
  result.figures.push(name);
}
md.push('## Wykresy', '');
for (const name of Object.keys(figures)) md.push(`![${name}](figures/${name})`, '');
await writeFile(join(DIR, 'engines.md'), `${md.join('\n')}\n`);
await writeFile(join(DIR, 'engines.json'), `${JSON.stringify(result, null, 1)}\n`);
console.log(`engines.md, engines.json, ${Object.keys(figures).length} figures; sessions: ${s.compared} pairs, median |change| ${dec(s.medianAbsChange * 100, 1)}%, p90 ${dec(s.p90AbsChange * 100, 1)}%`);
