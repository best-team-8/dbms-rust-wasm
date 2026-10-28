// Tables and figures of Benchmark 2 (prototype against the version after the
// optimisations) from the raw results in this directory.
//
//   node docs/research/data/benchmark-2/make-report.mjs [DIR]
//
// Reads DIR/manifest.json (bench/compare.js), DIR/before/raw and DIR/after/raw
// (prototype and final version, Windows: node, native, browser) and, if
// present, DIR/linux/before/raw and DIR/linux/after/raw (WSL 2: node and native
// file writes, reported as node-linux and native-linux). Writes
//   DIR/tables.md     – tables: median of runs, quartiles in brackets, change,
//                       and whether the 95% confidence intervals of the means
//                       are disjoint,
//   DIR/summary.json  – every aggregated value,
//   DIR/figures/*.svg – figures of the thesis.
// Only Node.js built-ins are used.

import { existsSync } from 'node:fs';
import { mkdir, readdir, readFile, writeFile } from 'node:fs/promises';
import { join } from 'node:path';
import { fileURLToPath } from 'node:url';

const DIR = process.argv[2] ?? fileURLToPath(new URL('.', import.meta.url));

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
const mibStat = (s) => (s ? `${dec(s.median / 1048576, 1)} MiB` : '–');
const ratio = (x) => (x >= 10 ? dec(x, 0) : x >= 2 ? dec(x, 1) : dec(x, 2)) + '×';
const pct = (x) => `${x >= 0 ? '+' : '−'}${dec(Math.abs(x) * 100, 1)}%`;

// ---- loading ----------------------------------------------------------------

async function loadRaw(base, side, rename = (t) => t) {
  const out = [];
  if (!existsSync(base)) return out;
  for (const target of await readdir(base)) {
    for (const kind of await readdir(join(base, target))) {
      for (const engine of await readdir(join(base, target, kind))) {
        for (const f of (await readdir(join(base, target, kind, engine))).filter((x) => x.endsWith('.json'))) {
          const data = JSON.parse(await readFile(join(base, target, kind, engine, f), 'utf8'));
          out.push({ side, target: rename(target), kind, engine, run: Number(f.slice(0, -5)), result: data.result, data });
        }
      }
    }
  }
  return out;
}

const manifest = JSON.parse(await readFile(join(DIR, 'manifest.json'), 'utf8'));
const linuxManifest = existsSync(join(DIR, 'linux', 'manifest.json')) ? JSON.parse(await readFile(join(DIR, 'linux', 'manifest.json'), 'utf8')) : null;
const raw = [
  ...(await loadRaw(join(DIR, 'before', 'raw'), 'before')),
  ...(await loadRaw(join(DIR, 'after', 'raw'), 'after')),
  ...(await loadRaw(join(DIR, 'linux', 'before', 'raw'), 'before', (t) => `${t}-linux`)),
  ...(await loadRaw(join(DIR, 'linux', 'after', 'raw'), 'after', (t) => `${t}-linux`)),
];
const pick = (side, target, kind, engine) => raw.filter((r) => r.side === side && r.target === target && r.kind === kind && (!engine || r.engine === engine));

const LABEL = {
  flatsql: 'Flat-SQL (obiekty)',
  'flatsql-json': 'Flat-SQL (JSON)',
  'flatsql-native': 'Flat-SQL natywnie',
  'flatsql-idb': 'Flat-SQL + IndexedDB',
  'flatsql-idb-strict': 'Flat-SQL + IndexedDB (strict)',
  'flatsql-worker': 'Flat-SQL w Workerze (pamięć)',
  'flatsql-worker-idb': 'Flat-SQL w Workerze + IndexedDB',
};
const TARGET = { node: 'Node.js', native: 'natywnie', browser: 'przeglądarka', 'node-linux': 'Node.js, Linux', 'native-linux': 'natywnie, Linux' };
const OPS = [
  ['insert', 'Wstawienie N wierszy (1 transakcja)'],
  ['insert_many', 'Wstawienie N wierszy jednym wywołaniem'],
  ['point_lookup', 'Odczyt po kluczu'],
  ['index_lookup', 'COUNT po indeksie'],
  ['range_scan', 'Zakres klucza (1000 wierszy) + SUM'],
  ['full_scan', 'Pełny skan z filtrem'],
  ['group_by', 'GROUP BY (100 grup)'],
  ['top_k', 'ORDER BY … LIMIT 10'],
  ['materialize_10k', 'Pobranie 10 000 wierszy'],
  ['update_10pct', 'UPDATE 10% wierszy'],
  ['delete_10pct', 'DELETE 10% wierszy'],
  ['snapshot', 'Snapshot'],
  ['restore', 'Odtworzenie'],
  ['commit_single', 'Trwałe zatwierdzenie 1 wiersza'],
  ['commit_group', '1000 zatwierdzeń, jedno oczekiwanie'],
  ['reopen', 'Ponowne otwarcie + 1. zapytanie'],
];
const OP_LABEL = Object.fromEntries(OPS);

// ---- metrics ------------------------------------------------------------------

/** name → value per run (µs for durations, bytes for sizes). */
function metrics(r) {
  const m = {};
  const res = r.result;
  if (r.kind === 'workload') {
    for (const [op, v] of Object.entries(res.ops ?? {})) if (Number.isFinite(v.meanUs)) m[op] = v.meanUs;
  } else if (r.kind === 'cold') {
    for (const k of ['initMs', 'openMs', 'firstQueryMs', 'totalMs']) if (Number.isFinite(res[k])) m[k] = res[k] * 1000;
    if (Number.isFinite(res.modules?.gzipBytes)) m.gzipBytes = res.modules.gzipBytes;
  } else if (r.kind === 'memory') {
    if (Number.isFinite(res.after?.wasm)) m.wasmBytes = res.after.wasm;
  } else if (r.kind === 'fsync') {
    for (const [mode, ops] of Object.entries(res.fsStore ?? {})) for (const [op, v] of Object.entries(ops)) if (Number.isFinite(v.meanUs)) m[`FsStore ${mode} ${op}`] = v.meanUs;
    for (const [mode, v] of Object.entries(res.syncPush ?? {})) if (Number.isFinite(v.meanUs)) m[`sync round ${mode}`] = v.meanUs;
    // The native CLI reports FileStorage under fsync / nofsync.
    const file = res.file ?? res.storage ?? (r.engine === 'flatsql-native' ? { fsync: res.fsync, nofsync: res.nofsync } : {});
    for (const [mode, ops] of Object.entries(file)) {
      if (ops && typeof ops === 'object') for (const [op, v] of Object.entries(ops)) if (Number.isFinite(v?.meanUs)) m[`FileStorage ${mode} ${op}`] = v.meanUs;
    }
  } else if (r.kind === 'network') {
    for (const mode of res.modes ?? []) if (Number.isFinite(mode.meanMs)) m[mode.mode] = mode.meanMs * 1000;
  } else if (r.kind === 'checkpoint') {
    if (Number.isFinite(res.idle?.maxGapMs)) m['pause idle'] = res.idle.maxGapMs * 1000;
    for (const s of res.sizes ?? []) {
      if (Number.isFinite(s.maxGapMs)) m[`pause ${s.rows}`] = s.maxGapMs * 1000;
      if (Number.isFinite(s.totalMs)) m[`checkpoint ${s.rows}`] = s.totalMs * 1000;
    }
  }
  return m;
}

/** Statistics of every metric of one group, both versions. */
function group(target, kind, engine) {
  const out = {};
  for (const side of ['before', 'after']) {
    const runs = pick(side, target, kind, engine);
    const names = new Set(runs.flatMap((r) => Object.keys(metrics(r))));
    for (const name of names) (out[name] ??= {})[side] = stat(runs.map((r) => metrics(r)[name]));
  }
  return out;
}

const shown = (b, a) => (b && a && b.n > 1 && a.n > 1 ? (a.ci95[1] < b.ci95[0] ? 'tak, mniej' : a.ci95[0] > b.ci95[1] ? 'tak, więcej' : 'nie') : '–');

const groups = [...new Set(raw.map((r) => `${r.target}|${r.kind}|${r.engine}`))].sort();
const summary = { benchmark: 'Benchmark 2', before: manifest.before, after: manifest.after, options: manifest.options, groups: {} };
for (const g of groups) {
  const [target, kind, engine] = g.split('|');
  summary.groups[g] = group(target, kind, engine);
}

// ---- answers: checksums and table state ----------------------------------------

function checks() {
  let compared = 0;
  const diff = [];
  for (const target of ['node', 'native', 'browser']) {
    for (const engine of new Set(raw.filter((r) => r.target === target && r.kind === 'workload').map((r) => r.engine))) {
      const all = raw.filter((r) => r.target === target && r.kind === 'workload' && r.engine === engine);
      const ref = all.find((r) => r.side === 'before')?.result;
      if (!ref) continue;
      for (const r of all) {
        for (const [op, c] of Object.entries(r.result.checks ?? {})) {
          const want = ref.checks?.[op];
          if (!want) continue;
          compared++;
          if (c.head !== want.head || (c.n === want.n && c.full !== want.full)) diff.push(`${target}/${engine}/${op}`);
        }
        for (const [k, v] of Object.entries(r.result.stateValues ?? {})) {
          const w = ref.stateValues?.[k];
          if (!w) continue;
          compared++;
          if (JSON.stringify(v) !== JSON.stringify(w)) diff.push(`${target}/${engine}/${k}`);
        }
      }
    }
  }
  return { compared, diff: [...new Set(diff)] };
}
const answers = checks();
summary.checks = answers;

// ---- SVG (as in Benchmark 1) -------------------------------------------------------

const C = { ink: '#0b0b0b', ink2: '#52514e', grid: '#e4e3df', surface: '#ffffff', s1: '#8b8a86', s2: '#2a78d6', s3: '#1baf7a', mute: '#8b8a86' };
const esc = (s) => String(s).replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;');
const FONT = 'font-family="Segoe UI, system-ui, sans-serif"';
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
const textWidth = (str, px = 12) => str.length * px * 0.56;

/** Horizontal dot plot on a log axis: median dot, whisker Q1–Q3. */
function dotPlot({ title, subtitle, rows, series, unit = 'us' }) {
  const left = Math.max(150, ...rows.map((r) => textWidth(r.label ?? '') + 24));
  const width = left + 560;
  const plotW = width - left - 70;
  const rowH = 14 + series.length * 12;
  const top = 64 + (series.length > 1 ? 22 : 0);
  const height = top + rows.length * rowH + 44;
  const values = rows.flatMap((r) => (r.points ?? []).flatMap((p) => (p.value ? [p.value.q1, p.value.q3, p.value.median] : []))).filter((x) => x > 0);
  const ticks = logTicks(Math.min(...values), Math.max(...values));
  const lo = Math.log10(ticks[0]);
  const hi = Math.log10(ticks[ticks.length - 1]);
  const x = (v) => left + ((Math.log10(v) - lo) / (hi - lo || 1)) * plotW;
  const fmt = unit === 'us' ? durPlain : (v) => `${num(v)}${unit}`;
  const tickFmt = unit === 'us' ? tickLabel : fmt;
  const out = [];
  out.push(`<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 ${width} ${height}" width="${width}" height="${height}" ${FONT}>`);
  out.push(`<rect width="${width}" height="${height}" fill="${C.surface}"/>`);
  out.push(`<text x="16" y="24" font-size="15" font-weight="600" fill="${C.ink}">${esc(title)}</text>`);
  if (subtitle) out.push(`<text x="16" y="42" font-size="12" fill="${C.ink2}">${esc(subtitle)}</text>`);
  if (series.length > 1) {
    let lx = left;
    for (const s of series) {
      out.push(`<circle cx="${lx + 5}" cy="${top - 16}" r="4.5" fill="${s.color}"/><text x="${lx + 14}" y="${top - 12}" font-size="12" fill="${C.ink}">${esc(s.name)}</text>`);
      lx += 24 + s.name.length * 6.6;
    }
  }
  const bottom = top + rows.length * rowH;
  for (const t of ticks) {
    out.push(`<line x1="${x(t)}" x2="${x(t)}" y1="${top - 4}" y2="${bottom}" stroke="${C.grid}" stroke-width="1"/>`);
    out.push(`<text x="${x(t)}" y="${bottom + 16}" font-size="11" fill="${C.ink2}" text-anchor="middle">${esc(tickFmt(t))}</text>`);
  }
  rows.forEach((row, i) => {
    const y0 = top + i * rowH;
    out.push(`<text x="${left - 10}" y="${y0 + rowH / 2 + 4}" font-size="12" fill="${C.ink}" text-anchor="end">${esc(row.label)}</text>`);
    row.points.forEach((p, j) => {
      if (!p.value) return;
      const s = series.find((q) => q.name === p.series) ?? series[0];
      const y = y0 + 9 + j * 12;
      out.push(`<g><title>${esc(`${row.label} – ${s.name}: mediana ${fmt(p.value.median)}, Q1–Q3 ${fmt(p.value.q1)}–${fmt(p.value.q3)}, n = ${p.value.n}`)}</title>`);
      out.push(`<line x1="${x(p.value.q1)}" x2="${x(p.value.q3)}" y1="${y}" y2="${y}" stroke="${s.color}" stroke-width="2" stroke-linecap="round"/>`);
      out.push(`<circle cx="${x(p.value.median)}" cy="${y}" r="4.5" fill="${s.color}" stroke="${C.surface}" stroke-width="2"/>`);
      if (j === row.points.length - 1 && row.note) out.push(`<text x="${Math.max(x(p.value.q3), x(row.points[0].value?.q3 ?? p.value.q3)) + 8}" y="${y0 + rowH / 2 + 4}" font-size="11" fill="${C.ink2}">${esc(row.note)}</text>`);
      out.push('</g>');
    });
  });
  out.push(`<text x="${width - 16}" y="${height - 8}" font-size="10.5" fill="${C.ink2}" text-anchor="end">skala logarytmiczna; punkt – mediana uruchomień, odcinek – Q1–Q3</text>`);
  out.push('</svg>');
  return out.join('\n');
}

/** Lines over a few x values with Q1–Q3 whiskers and an optional reference line. */
function lineChart({ title, subtitle, xs, xLabel, series, ref, yUnit = 'ms' }) {
  const left = 70;
  const width = 820;
  const height = 380;
  const top = 76;
  const labelW = Math.max(...series.map((q) => textWidth(q.name))) + 20;
  const plotW = width - left - labelW;
  const plotH = height - top - 56;
  const max = Math.max(ref?.value ?? 0, ...series.flatMap((s) => s.points.map((p) => p?.q3 ?? 0))) * 1.1;
  const step = [1, 2, 5, 10, 20, 25, 50, 100].find((s) => max / s <= 6) ?? 100;
  const yMax = Math.ceil(max / step) * step;
  const x = (i) => left + (i / (xs.length - 1)) * plotW;
  const y = (v) => top + plotH - (v / yMax) * plotH;
  const out = [];
  out.push(`<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 ${width} ${height}" width="${width}" height="${height}" ${FONT}>`);
  out.push(`<rect width="${width}" height="${height}" fill="${C.surface}"/>`);
  out.push(`<text x="16" y="24" font-size="15" font-weight="600" fill="${C.ink}">${esc(title)}</text>`);
  if (subtitle) out.push(`<text x="16" y="42" font-size="12" fill="${C.ink2}">${esc(subtitle)}</text>`);
  for (let v = 0; v <= yMax; v += step) {
    out.push(`<line x1="${left}" x2="${left + plotW}" y1="${y(v)}" y2="${y(v)}" stroke="${C.grid}"/>`);
    out.push(`<text x="${left - 8}" y="${y(v) + 4}" font-size="11" fill="${C.ink2}" text-anchor="end">${v} ${yUnit}</text>`);
  }
  xs.forEach((xv, i) => out.push(`<text x="${x(i)}" y="${top + plotH + 18}" font-size="11" fill="${C.ink2}" text-anchor="middle">${esc(xv)}</text>`));
  out.push(`<text x="${left + plotW / 2}" y="${height - 8}" font-size="11" fill="${C.ink2}" text-anchor="middle">${esc(xLabel)}</text>`);
  if (ref) {
    out.push(`<line x1="${left}" x2="${left + plotW}" y1="${y(ref.value)}" y2="${y(ref.value)}" stroke="${C.ink2}" stroke-width="1.5" stroke-dasharray="4 3"/>`);
    out.push(`<text x="${left + 4}" y="${y(ref.value) - 5}" font-size="11" fill="${C.ink2}">${esc(ref.label)}</text>`);
  }
  const endLabels = [];
  for (const s of series) {
    const pts = s.points.map((p, i) => (p ? [x(i), y(p.median)] : null)).filter(Boolean);
    out.push(`<polyline points="${pts.map((p) => p.join(',')).join(' ')}" fill="none" stroke="${s.color}" stroke-width="2"/>`);
    s.points.forEach((p, i) => {
      if (!p) return;
      out.push(`<g><title>${esc(`${s.name}, ${xs[i]}: mediana ${num(p.median)} ${yUnit}, Q1–Q3 ${num(p.q1)}–${num(p.q3)} ${yUnit}`)}</title>`);
      out.push(`<line x1="${x(i)}" x2="${x(i)}" y1="${y(p.q1)}" y2="${y(p.q3)}" stroke="${s.color}" stroke-width="2"/>`);
      out.push(`<circle cx="${x(i)}" cy="${y(p.median)}" r="4.5" fill="${s.color}" stroke="${C.surface}" stroke-width="2"/></g>`);
    });
    const last = pts[pts.length - 1];
    endLabels.push({ x: last[0] + 10, y: last[1] + 4, name: s.name });
  }
  endLabels.sort((a, b) => a.y - b.y);
  for (let i = 1; i < endLabels.length; i++) endLabels[i].y = Math.max(endLabels[i].y, endLabels[i - 1].y + 15);
  for (const l of endLabels) out.push(`<text x="${l.x}" y="${l.y}" font-size="12" fill="${C.ink}">${esc(l.name)}</text>`);
  out.push('</svg>');
  return out.join('\n');
}

// ---- tables -----------------------------------------------------------------------

const md = [];
const table = (head, rows, textCols = 1) => {
  md.push(`| ${head.join(' | ')} |`, `|${head.map((_, i) => (i < textCols ? '---' : '---:')).join('|')}|`);
  for (const r of rows) md.push(`| ${r.join(' | ')} |`);
  md.push('');
};
const before = manifest.before;
const after = manifest.after;
md.push('# Benchmark 2 – tabele', '');
md.push(
  `Wygenerowano skryptem \`make-report.mjs\` z surowych wyników w \`before/raw\` (prototyp, \`${before.describe}\`, ${before.commit?.slice(0, 10)}) i \`after/raw\` (wersja po optymalizacjach, \`${after.describe}\`, ${after.commit?.slice(0, 10)}). Obie wersje mierzone tym samym runnerem, uruchomienia naprzemienne, ${manifest.options.runs} na wersję. Wartość: mediana z uruchomień procesu, w nawiasie pierwszy i trzeci kwartyl. „Zmiana” to zmiana mediany, „krotność” – mediana prototypu przez medianę wersji końcowej, „rozłączne” – czy 95% przedziały ufności średnich są rozłączne.`,
  '',
);
const browserInfo = raw.find((r) => r.target === 'browser')?.data;
md.push('## Środowisko', '');
table(['Element', 'Wartość'], [
  ['Data pomiaru', `${manifest.date?.slice(0, 10)} – ${manifest.finished?.slice(0, 10) ?? '?'}`],
  ['Obciążenie procesora przed pomiarem', `${dec(manifest.cpuLoadBefore * 100, 1)}%`],
  ['Node.js', raw.find((r) => r.target === 'node')?.data.node ?? '–'],
  ['Przeglądarka', browserInfo?.env?.userAgent ?? browserInfo?.userAgent ?? '–'],
  ['Moduł WebAssembly (gzip -9)', `${before.module?.gzipBytes ?? '?'} B → ${after.module?.gzipBytes ?? '?'} B`],
  ...(linuxManifest ? [['Linux (pliki)', `${linuxManifest.note ?? ''}`]] : []),
], 2);
md.push(answers.diff.length ? `Różne wyniki: ${answers.diff.join(', ')} (${answers.compared} porównań).` : `Wyniki zgodne w obu wersjach: ${answers.compared} porównań sum kontrolnych zapytań i stanu tabeli.`, '');

function compareTable(target, kind, engine, names, nameLabel = (n) => n, fmt = dur) {
  const g = summary.groups[`${target}|${kind}|${engine}`];
  if (!g) return;
  const rows = [];
  for (const n of names.filter((x) => g[x])) {
    const b = g[n].before;
    const a = g[n].after;
    rows.push([nameLabel(n), fmt(b, n), fmt(a, n), b && a ? pct(a.median / b.median - 1) : '–', b && a ? ratio(b.median / a.median) : '–', shown(b, a)]);
  }
  if (rows.length) table(['Pomiar', 'Prototyp', 'Wersja końcowa', 'Zmiana', 'Krotność', 'Rozłączne'], rows);
}

const opNames = OPS.map(([op]) => op);
for (const [target, engines] of [
  ['node', ['flatsql', 'flatsql-json']],
  ['native', ['flatsql-native']],
  ['browser', ['flatsql', 'flatsql-json', 'flatsql-idb', 'flatsql-idb-strict', 'flatsql-worker', 'flatsql-worker-idb']],
]) {
  for (const engine of engines) {
    if (!summary.groups[`${target}|workload|${engine}`]) continue;
    md.push(`## Obciążenie: ${LABEL[engine] ?? engine}, ${TARGET[target]}`, '');
    compareTable(target, 'workload', engine, opNames, (n) => OP_LABEL[n] ?? n);
  }
}

md.push('## Narzut WebAssembly: krotność czasu natywnego', '');
{
  const nat = summary.groups['native|workload|flatsql-native'];
  const rows = [];
  for (const op of opNames) {
    if (!nat?.[op]) continue;
    const r = [OP_LABEL[op]];
    for (const [target, engine] of [['node', 'flatsql'], ['node', 'flatsql-json'], ['browser', 'flatsql'], ['browser', 'flatsql-json']]) {
      const w = summary.groups[`${target}|workload|${engine}`]?.[op];
      for (const side of ['before', 'after']) r.push(w?.[side] && nat[op][side] ? ratio(w[side].median / nat[op][side].median) : '–');
    }
    rows.push(r);
  }
  table(['Operacja', 'Node.js obiekty: prototyp', 'po', 'Node.js JSON: prototyp', 'po', 'przeglądarka obiekty: prototyp', 'po', 'przeglądarka JSON: prototyp', 'po'], rows);
}

md.push('## Zimny start', '');
for (const [target, engines] of [['node', ['flatsql', 'flatsql-json']], ['browser', ['flatsql', 'flatsql-idb', 'flatsql-worker']]]) {
  for (const engine of engines) {
    if (!summary.groups[`${target}|cold|${engine}`]) continue;
    md.push(`### ${LABEL[engine] ?? engine}, ${TARGET[target]}`, '');
    compareTable(target, 'cold', engine, ['initMs', 'openMs', 'firstQueryMs', 'totalMs', 'gzipBytes'], (n) => ({ initMs: 'Import i inicjalizacja modułu', openMs: 'Otwarcie bazy', firstQueryMs: 'Pierwsze zapytanie', totalMs: 'Razem', gzipBytes: 'Pobrane pliki (gzip)' })[n], (s, n) => (n === 'gzipBytes' ? (s ? `${dec(s.median / 1024, 0)} KiB` : '–') : dur(s)));
  }
}

md.push('## Pamięć liniowa WebAssembly po wczytaniu N wierszy', '');
{
  const rows = [];
  for (const [target, engines] of [['node', ['flatsql', 'flatsql-json']], ['browser', ['flatsql', 'flatsql-json', 'flatsql-worker']]]) {
    for (const engine of engines) {
      const g = summary.groups[`${target}|memory|${engine}`]?.wasmBytes;
      if (g) rows.push([`${LABEL[engine]}, ${TARGET[target]}`, mibStat(g.before), mibStat(g.after), g.before && g.after ? pct(g.after.median / g.before.median - 1) : '–']);
    }
  }
  if (rows.length) table(['Pomiar', 'Prototyp', 'Wersja końcowa', 'Zmiana'], rows);
}

md.push('## Pauza checkpointu (przeglądarka)', '');
for (const engine of ['flatsql-idb', 'flatsql-idb-strict']) {
  if (!summary.groups[`browser|checkpoint|${engine}`]) continue;
  md.push(`### ${LABEL[engine]}`, '');
  compareTable('browser', 'checkpoint', engine, ['pause idle', 'pause 10000', 'checkpoint 10000', 'pause 50000', 'checkpoint 50000', 'pause 100000', 'checkpoint 100000'], (n) => n.replace('pause idle', 'najdłuższa przerwa w bezczynności').replace(/^pause (\d+)/, (_, r) => `najdłuższa pauza, ${Number(r).toLocaleString('pl-PL')} wierszy`).replace(/^checkpoint (\d+)/, (_, r) => `cały checkpoint, ${Number(r).toLocaleString('pl-PL')} wierszy`));
}

md.push('## Local-First a Cloud-First', '');
for (const target of ['node', 'browser']) {
  const g = summary.groups[`${target}|network|flatsql`];
  if (!g) continue;
  md.push(`### ${TARGET[target]}`, '');
  compareTable(target, 'network', 'flatsql', Object.keys(g));
}

md.push('## Zapis plików (fsync)', '');
for (const [target, engine] of [['node', 'flatsql'], ['native', 'flatsql-native'], ['node-linux', 'flatsql'], ['native-linux', 'flatsql-native']]) {
  const g = summary.groups[`${target}|fsync|${engine}`];
  if (!g) continue;
  md.push(`### ${TARGET[target]}`, '');
  compareTable(target, 'fsync', engine, Object.keys(g).sort());
}

// ---- figures ------------------------------------------------------------------------

const SERIES = [
  { name: 'prototyp', color: C.s1 },
  { name: 'po optymalizacjach', color: C.s2 },
];
const figures = {};
function workloadFigure(target, engine, title) {
  const g = summary.groups[`${target}|workload|${engine}`];
  if (!g) return null;
  const rows = OPS.filter(([op]) => g[op]).map(([op, label]) => ({
    label,
    note: g[op].before && g[op].after ? ratio(g[op].before.median / g[op].after.median) : '',
    points: [
      { series: 'prototyp', value: g[op].before },
      { series: 'po optymalizacjach', value: g[op].after },
    ],
  }));
  return dotPlot({ title, subtitle: `N = ${(manifest.options.rows ?? 100000).toLocaleString('pl-PL')} wierszy; liczba po prawej: krotność przyspieszenia (mediana prototypu / mediana po zmianach)`, rows, series: SERIES });
}
figures['01-obciazenie-node.svg'] = workloadFigure('node', 'flatsql', 'Obciążenie w Node.js, wynik jako obiekty JS');
figures['02-obciazenie-przegladarka.svg'] = workloadFigure('browser', 'flatsql', 'Obciążenie w przeglądarce, wynik jako obiekty JS');
figures['03-obciazenie-natywnie.svg'] = workloadFigure('native', 'flatsql-native', 'Obciążenie natywnie (bez WebAssembly)');
figures['04-obciazenie-indexeddb.svg'] = workloadFigure('browser', 'flatsql-idb', 'Obciążenie w przeglądarce z IndexedDB');
{
  const nat = summary.groups['native|workload|flatsql-native'];
  const w = summary.groups['browser|workload|flatsql'];
  if (nat && w) {
    const point = (side, op) => {
      const r = w[op]?.[side] && nat[op]?.[side] ? w[op][side].median / nat[op][side].median : null;
      return r ? { n: 1, median: r, q1: r, q3: r } : null;
    };
    const rows = OPS.filter(([op]) => nat[op] && w[op]).map(([op, label]) => ({
      label,
      points: [
        { series: 'prototyp', value: point('before', op) },
        { series: 'po optymalizacjach', value: point('after', op) },
      ],
    }));
    figures['05-narzut-wasm.svg'] = dotPlot({ title: 'Narzut WebAssembly w przeglądarce: czas przez czas natywny', subtitle: 'ścieżka obiektowa wobec natywnego benchmarku tej samej wersji; 1× – tyle co natywnie', rows, series: SERIES, unit: '×' });
  }
}
{
  const g = summary.groups['browser|checkpoint|flatsql-idb'];
  if (g) {
    const sizes = [10000, 50000, 100000];
    const ms = (s) => (s ? { ...s, median: s.median / 1000, q1: s.q1 / 1000, q3: s.q3 / 1000 } : null);
    figures['06-pauza-checkpointu.svg'] = lineChart({
      title: 'Najdłuższa pauza wątku głównego podczas checkpointu',
      subtitle: 'Flat-SQL + IndexedDB w przeglądarce, sonda MessageChannel',
      xs: sizes.map((n) => `${n.toLocaleString('pl-PL')}`),
      xLabel: 'liczba wierszy',
      series: [
        { name: 'prototyp', color: C.s1, points: sizes.map((n) => ms(g[`pause ${n}`]?.before)) },
        { name: 'po optymalizacjach', color: C.s2, points: sizes.map((n) => ms(g[`pause ${n}`]?.after)) },
      ],
      ref: { value: 16.7, label: 'jedna klatka przy 60 Hz (16,7 ms)' },
    });
  }
}

await mkdir(join(DIR, 'figures'), { recursive: true });
for (const [name, svg] of Object.entries(figures)) if (svg) await writeFile(join(DIR, 'figures', name), `${svg}\n`);
md.push('## Wykresy', '');
for (const name of Object.keys(figures).filter((n) => figures[n])) md.push(`![${name}](figures/${name})`, '');
await writeFile(join(DIR, 'tables.md'), `${md.join('\n')}\n`);
await writeFile(join(DIR, 'summary.json'), `${JSON.stringify(summary, null, 1)}\n`);
console.log(`tables.md, summary.json, ${Object.values(figures).filter(Boolean).length} figures`);
