// Tables and figures of Benchmark 1 from the raw results in this directory.
//
//   node docs/research/data/benchmark-1/make-report.mjs [DIR]
//
// Reads DIR/manifest.json and DIR/raw/<target>/<kind>/<engine>/NN.json (one
// file per measured process, written by bench/run.js) and writes
//   DIR/tables.md     – tables of the thesis (median and quartiles of runs),
//   DIR/summary.json  – every aggregated value (with mean and 95% CI),
//   DIR/figures/*.svg – figures of the thesis.
// Only Node.js built-ins are used, so the script works without the rest of
// the repository.

import { existsSync } from 'node:fs';
import { mkdir, readdir, readFile, writeFile } from 'node:fs/promises';
import { join } from 'node:path';
import { fileURLToPath } from 'node:url';

const DIR = process.argv[2] ?? fileURLToPath(new URL('.', import.meta.url));

// ---- statistics -----------------------------------------------------------

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
/** Duration given in µs, unit chosen by the median. */
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
const ratio = (x) => (x >= 10 ? dec(x, 0) : x >= 2 ? dec(x, 1) : dec(x, 2)) + '×';

// ---- loading ----------------------------------------------------------------

async function loadRaw() {
  const base = join(DIR, 'raw');
  const out = [];
  for (const target of await readdir(base)) {
    for (const kind of await readdir(join(base, target))) {
      for (const engine of await readdir(join(base, target, kind))) {
        for (const f of (await readdir(join(base, target, kind, engine))).filter((x) => x.endsWith('.json'))) {
          const data = JSON.parse(await readFile(join(base, target, kind, engine, f), 'utf8'));
          out.push({ target, kind, engine, run: Number(f.slice(0, -5)), result: data.result, data });
        }
      }
    }
  }
  return out;
}

const manifest = JSON.parse(await readFile(join(DIR, 'manifest.json'), 'utf8'));
const ENGINES = manifest.engines;
const ORDER = Object.keys(ENGINES);
const raw = await loadRaw();
const pick = (target, kind, engine) => raw.filter((r) => r.target === target && r.kind === kind && (!engine || r.engine === engine));
const ROWS = `${(manifest.options?.rows ?? 0).toLocaleString('pl-PL')}`;
const label = (id) => (id === 'flatsql-native' ? 'Flat-SQL natywnie' : ENGINES[id]?.label ?? id);
const sortEngines = (ids) => [...new Set(ids)].sort((a, b) => (ORDER.indexOf(a) + 1 || 999) - (ORDER.indexOf(b) + 1 || 999));

// Values declared by authors of engines that could not be measured (closed
// code, no module to download). Shown separately, marked as unverified.
const DECLARED = [
  {
    engine: 'Zeta-Lite',
    source: '[D5]',
    note: 'preprint, wrzesień 2026; warunki pomiaru autora, nie powtórzone w tym benchmarku',
    pointLookupUs: [3.2, 3.7], // 268–315 tys. odczytów punktowych/s
    moduleGzipBytes: 2.87e6,
  },
];

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

const summary = { benchmark: 'Benchmark 1', manifest: { date: manifest.date, flatsql: manifest.flatsql, host: manifest.host }, workload: {}, cold: {}, memory: {}, checkpoint: {}, network: {}, fsync: {}, checks: {} };

/** Aggregated workload of one target: engine → op → stat of per-run means. */
function workloadStats(target) {
  const out = {};
  for (const r of pick(target, 'workload')) {
    const ops = r.result.ops ?? {};
    for (const [op, v] of Object.entries(ops)) {
      ((out[r.engine] ??= {})[op] ??= []).push(v.meanUs);
      if (v.p95Us !== undefined) ((out[r.engine] ??= {})[`${op}:p95`] ??= []).push(v.p95Us);
      if (v.n !== undefined) ((out[r.engine] ??= {})[`${op}:n`] ??= []).push(v.n);
    }
  }
  for (const e of Object.keys(out)) for (const op of Object.keys(out[e])) out[e][op] = op.endsWith(':n') ? Math.min(...out[e][op]) : stat(out[e][op]);
  return out;
}

const W = {};
for (const t of ['node', 'browser', 'native']) if (existsSync(join(DIR, 'raw', t, 'workload'))) W[t] = workloadStats(t);
summary.workload = W;

// ---- checksums ----------------------------------------------------------------

/** Compares every run of every engine with the first run of the reference engine. */
function checkGroup(target, engines, ref) {
  const runs = pick(target, 'workload').filter((r) => engines.includes(r.engine));
  const refRun = runs.find((r) => r.engine === ref) ?? runs[0];
  if (!refRun) return [];
  const rows = [];
  for (const e of engines) {
    const mine = runs.filter((r) => r.engine === e);
    if (!mine.length) continue;
    let ok = 0;
    let headOnly = 0;
    let bad = [];
    let total = 0;
    for (const r of mine) {
      for (const [op, c] of Object.entries(refRun.result.checks)) {
        const m = r.result.checks?.[op];
        if (!m) continue;
        total++;
        if (m.head !== c.head) bad.push(`${op} (run ${r.run})`);
        else if (m.n === c.n && m.full !== c.full) bad.push(`${op} (run ${r.run}, pełna)`);
        else if (m.n !== c.n) headOnly++;
        else ok++;
      }
      for (const [k, v] of Object.entries(refRun.result.stateValues ?? {})) {
        const mv = r.result.stateValues?.[k];
        if (!mv) continue;
        total++;
        if (JSON.stringify(mv) === JSON.stringify(v)) ok++;
        else bad.push(`stan ${k} (run ${r.run})`);
      }
    }
    rows.push({ engine: e, runs: mine.length, total, ok, headOnly, bad });
  }
  return rows;
}

// ---- SVG ------------------------------------------------------------------------

const C = { ink: '#0b0b0b', ink2: '#52514e', grid: '#e4e3df', surface: '#ffffff', s1: '#2a78d6', s2: '#eb6834', s3: '#1baf7a', mute: '#8b8a86' };
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

/**
 * Horizontal dot plot on a log axis: median dot, whisker Q1–Q3.
 * rows: [{ label, points: [{ series, value: stat }] }]; series: [{ name, color }].
 */
function dotPlot({ title, subtitle, rows, series, unit = 'us', refs = [] }) {
  const left = Math.max(150, ...rows.map((r) => textWidth(r.label ?? "") + 24));
  const width = left + 560;
  const plotW = width - left - 40;
  const rowH = 14 + series.length * 12;
  const top = 64 + (series.length > 1 ? 22 : 0);
  const height = top + rows.length * rowH + 44;
  const values = rows.flatMap((r) => (r.points ?? []).flatMap((p) => (p.value ? [p.value.q1, p.value.q3, p.value.median] : []))).filter((x) => x > 0);
  const ticks = logTicks(Math.min(...values, ...refs.map((r) => r.value)), Math.max(...values, ...refs.map((r) => r.value)));
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
  for (const r of refs) {
    out.push(`<line x1="${x(r.value)}" x2="${x(r.value)}" y1="${top - 4}" y2="${bottom}" stroke="${C.ink2}" stroke-width="1.5" stroke-dasharray="4 3"/>`);
    out.push(`<text x="${x(r.value) + 4}" y="${top + 6}" font-size="11" fill="${C.ink2}">${esc(r.label)}</text>`);
  }
  rows.forEach((row, i) => {
    const y0 = top + i * rowH;
    if (row.section) {
      if (i > 0) out.push(`<line x1="16" x2="${width - 16}" y1="${y0 + 2}" y2="${y0 + 2}" stroke="${C.grid}"/>`);
      out.push(`<text x="16" y="${y0 + rowH / 2 + 6}" font-size="12" font-weight="600" fill="${C.ink2}">${esc(row.section)}</text>`);
      return;
    }
    out.push(`<text x="${left - 10}" y="${y0 + rowH / 2 + 4}" font-size="12" fill="${C.ink}" text-anchor="end">${esc(row.label)}</text>`);
    row.points.forEach((p, j) => {
      if (!p.value) return;
      const s = series.find((q) => q.name === p.series) ?? series[0];
      const y = y0 + 9 + j * 12 + (series.length === 1 ? rowH / 2 - 9 : 0);
      const tip = p.declared
        ? `${row.label}: ${fmt(p.value.q1)}–${fmt(p.value.q3)} według autora, niepotwierdzone pomiarem`
        : `${row.label}${series.length > 1 ? ` – ${s.name}` : ''}: mediana ${fmt(p.value.median)}, Q1–Q3 ${fmt(p.value.q1)}–${fmt(p.value.q3)}, n = ${p.value.n}`;
      out.push(`<g><title>${esc(tip)}</title>`);
      out.push(`<line x1="${x(p.value.q1)}" x2="${x(p.value.q3)}" y1="${y}" y2="${y}" stroke="${p.declared ? C.mute : s.color}" stroke-width="2" stroke-linecap="round"${p.declared ? ' stroke-dasharray="3 3"' : ''}/>`);
      out.push(
        p.declared
          ? `<circle cx="${x(p.value.median)}" cy="${y}" r="4.5" fill="${C.surface}" stroke="${C.mute}" stroke-width="2"/>`
          : `<circle cx="${x(p.value.median)}" cy="${y}" r="4.5" fill="${s.color}" stroke="${C.surface}" stroke-width="2"/>`,
      );
      if (series.length === 1 || p.labelled) {
        const text = p.declared ? `${fmt(p.value.q1)}–${fmt(p.value.q3)}, deklaracja autora, niepotwierdzona` : fmt(p.value.median);
        out.push(`<text x="${x(p.value.q3) + 8}" y="${y + 4}" font-size="11" fill="${C.ink2}">${esc(text)}</text>`);
      }
      out.push('</g>');
    });
  });
  const declared = rows.some((r) => (r.points ?? []).some((p) => p.declared)) ? '; pusty punkt – wartość deklarowana przez autora, niezmierzona' : '';
  out.push(`<text x="${width - 16}" y="${height - 8}" font-size="10.5" fill="${C.ink2}" text-anchor="end">skala logarytmiczna; punkt – mediana uruchomień, odcinek – Q1–Q3${declared}</text>`);
  out.push('</svg>');
  return out.join('\n');
}

/** Horizontal stacked bars (linear axis), parts in fixed series order. */
function stackedBars({ title, subtitle, rows, series, unitLabel = 'ms' }) {
  const left = 230;
  const width = 760;
  const plotW = width - left - 70;
  const rowH = 24;
  const top = 76;
  const height = top + rows.length * rowH + 40;
  const max = Math.max(...rows.map((r) => r.parts.reduce((a, p) => a + p, 0)));
  const step = [1, 2, 5, 10, 20, 50, 100, 200, 250, 500, 1000, 2000, 5000, 10000].find((s) => max / s <= 6) ?? 10000;
  const scale = plotW / (Math.ceil(max / step) * step);
  const out = [];
  out.push(`<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 ${width} ${height}" width="${width}" height="${height}" ${FONT}>`);
  out.push(`<rect width="${width}" height="${height}" fill="${C.surface}"/>`);
  out.push(`<text x="16" y="24" font-size="15" font-weight="600" fill="${C.ink}">${esc(title)}</text>`);
  if (subtitle) out.push(`<text x="16" y="42" font-size="12" fill="${C.ink2}">${esc(subtitle)}</text>`);
  let lx = left;
  series.forEach((s) => {
    out.push(`<rect x="${lx}" y="${top - 22}" width="10" height="10" rx="2" fill="${s.color}"/><text x="${lx + 15}" y="${top - 13}" font-size="12" fill="${C.ink}">${esc(s.name)}</text>`);
    lx += 30 + s.name.length * 6.6;
  });
  const bottom = top + rows.length * rowH;
  for (let v = 0; v * scale <= plotW + 0.5; v += step) {
    out.push(`<line x1="${left + v * scale}" x2="${left + v * scale}" y1="${top - 4}" y2="${bottom}" stroke="${C.grid}"/>`);
    out.push(`<text x="${left + v * scale}" y="${bottom + 16}" font-size="11" fill="${C.ink2}" text-anchor="middle">${num(v).replace(',00', '')} ${unitLabel}</text>`);
  }
  rows.forEach((row, i) => {
    const y = top + i * rowH + 5;
    out.push(`<text x="${left - 10}" y="${y + 10}" font-size="12" fill="${C.ink}" text-anchor="end">${esc(row.label)}</text>`);
    let x0 = left;
    row.parts.forEach((v, j) => {
      const w = Math.max(0, v * scale - (j < row.parts.length - 1 ? 2 : 0));
      out.push(`<rect x="${x0}" y="${y}" width="${w}" height="14" rx="${j === row.parts.length - 1 ? 3 : 0}" fill="${series[j].color}"><title>${esc(`${row.label} – ${series[j].name}: ${num(v)} ${unitLabel}`)}</title></rect>`);
      x0 += v * scale;
    });
    out.push(`<text x="${x0 + 6}" y="${y + 11}" font-size="11" fill="${C.ink2}">${num(row.parts.reduce((a, b) => a + b, 0))} ${unitLabel}</text>`);
  });
  out.push(`<text x="${width - 16}" y="${height - 8}" font-size="10.5" fill="${C.ink2}" text-anchor="end">mediany uruchomień</text>`);
  out.push('</svg>');
  return out.join('\n');
}

/** Lines over a few x values with Q1–Q3 whiskers and an optional reference line. */
function lineChart({ title, subtitle, xs, xLabel, series, ref, yUnit = 'ms' }) {
  const left = 70;
  const width = 760;
  const height = 380;
  const top = 76;
  const plotW = width - left - 150;
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
    out.push(`<text x="${last[0] + 10}" y="${last[1] + 4}" font-size="12" fill="${C.ink}">${esc(s.name)}</text>`);
  }
  out.push('</svg>');
  return out.join('\n');
}

// ---- tables -----------------------------------------------------------------

const md = [];
/** Markdown table; the first `textCols` columns are left-aligned, the rest (numbers) right-aligned. */
const table = (head, rows, textCols = 1) => {
  md.push(`| ${head.join(' | ')} |`, `|${head.map((_, i) => (i < textCols ? '---' : '---:')).join('|')}|`);
  for (const r of rows) md.push(`| ${r.join(' | ')} |`);
  md.push('');
};

md.push('# Benchmark 1 – tabele', '');
md.push(`Wygenerowano skryptem \`make-report.mjs\` z surowych wyników w \`raw/\`. Wartość: mediana z uruchomień procesu, w nawiasie pierwszy i trzeci kwartyl. Wersja Flat-SQL: \`${manifest.flatsql?.describe}\` (${manifest.flatsql?.commit?.slice(0, 10)}), obciążenie \`${manifest.workload}\`, N = ${manifest.options?.rows?.toLocaleString('pl-PL')} wierszy.`, '');

md.push('## Środowisko', '');
const browserInfo = raw.find((r) => r.target === 'browser')?.data;
table(
  ['Element', 'Wartość'],
  [
    ['Procesor', `${manifest.host.cpu} (${manifest.host.threads} wątków)`],
    ['Pamięć', `${dec(manifest.host.memoryBytes / 2 ** 30, 1)} GiB`],
    ['System', manifest.host.os],
    ['Node.js', manifest.node],
    ['Przeglądarka', browserInfo ? `${browserInfo.browser?.name} ${browserInfo.browser?.version} (${browserInfo.env?.userAgent?.match(/Edg\/[\d.]+|Chrome\/[\d.]+/)?.[0] ?? ''}), crossOriginIsolated = ${browserInfo.env?.crossOriginIsolated}` : '–'],
    ['Rust', manifest.rustc ?? '–'],
    ['Uruchomienia', Object.entries(manifest.targets ?? {}).map(([t, v]) => `${t}: ${v.runs}`).join(', ')],
    ...Object.entries(manifest.packages ?? {}).map(([k, v]) => [k, v ?? '–']),
  ],
  2,
);

const GROUP_NAME = { memory: 'pamięć', durable: 'trwały zapis', worker: 'Worker' };
const GROUP_TITLE = { memory: 'dane w pamięci, ten sam wątek', durable: 'trwały zapis w IndexedDB, wątek główny', worker: 'silnik w Web Workerze, wywołanie = komunikat w obie strony' };
const TARGET_TITLE = { node: 'Node.js', browser: 'przeglądarka', native: 'natywnie' };

function workloadTable(target, group, extra = []) {
  const ws = W[target];
  if (!ws) return;
  const engines = sortEngines(Object.keys(ws).filter((e) => (ENGINES[e]?.group ?? 'memory') === group)).concat(extra.filter((e) => ws[e] || W.native?.[e]));
  if (!engines.length) return;
  md.push(`## Obciążenie – ${TARGET_TITLE[target]}, grupa „${GROUP_NAME[group]}”: ${GROUP_TITLE[group]}`, '');
  const cols = engines.map((e) => label(e));
  const rows = [];
  for (const [op, name] of OPS) {
    const cells = engines.map((e) => {
      const src = e === 'flatsql-native' ? W.native?.[e] : ws[e];
      const s = src?.[op];
      if (!s) return '–';
      const n = src[`${op}:n`];
      const ref = READ_N[op];
      return dur(s) + (ref && n < ref ? ` [${n}/${ref}]` : '');
    });
    if (cells.some((c) => c !== '–')) rows.push([name, ...cells]);
  }
  table(['Operacja', ...cols], rows);
  const checks = checkGroup(target, engines.filter((e) => e !== 'flatsql-native'), group === 'worker' ? 'flatsql-worker' : engines[0]);
  summary.checks[`${target}/${group}`] = checks;
  const bad = checks.filter((c) => c.bad.length);
  md.push(`Zgodność wyników (sumy kontrolne wszystkich zapytań i stanu tabeli po każdej fazie, wszystkie uruchomienia): ${bad.length ? bad.map((c) => `${label(c.engine)} – rozbieżne: ${c.bad.slice(0, 5).join(', ')}`).join('; ') : `zgodne we wszystkich ${checks.reduce((a, c) => a + c.total, 0)} porównaniach`}${checks.some((c) => c.headOnly) ? '; przy operacjach przerwanych limitem czasu porównano pierwsze 3 wyniki' : ''}.`, '');
}
const READ_N = { point_lookup: 10000, index_lookup: 500, range_scan: 200, full_scan: 20, group_by: 20, top_k: 20, materialize_10k: 20 };

workloadTable('node', 'memory', ['flatsql-native']);
workloadTable('browser', 'memory');
workloadTable('browser', 'durable');
workloadTable('browser', 'worker');

// ---- H2: WebAssembly vs native ----------------------------------------------

function ratioTable(title, pairs, ops) {
  const rows = [];
  for (const [op, name] of OPS.filter(([o]) => ops.includes(o))) {
    const cells = pairs.map(([a, b]) => {
      const sa = a()?.[op];
      const sb = b()?.[op];
      return sa && sb ? ratio(sa.median / sb.median) : '–';
    });
    if (cells.some((c) => c !== '–')) rows.push([name, ...cells]);
  }
  if (rows.length) {
    md.push(`## ${title}`, '');
    table(['Operacja', ...pairs.map((p) => p[2])], rows);
  }
}
const READS_WRITES = ['insert', 'point_lookup', 'index_lookup', 'range_scan', 'full_scan', 'group_by', 'top_k', 'materialize_10k', 'update_10pct', 'delete_10pct', 'snapshot', 'restore'];
ratioTable(
  'Narzut WebAssembly (H2): czas WASM / czas natywny, mediany',
  [
    [() => W.node?.flatsql, () => W.native?.['flatsql-native'], 'Node.js, obiekty JS'],
    [() => W.node?.['flatsql-json'], () => W.native?.['flatsql-native'], 'Node.js, JSON'],
    [() => W.browser?.flatsql, () => W.native?.['flatsql-native'], 'przeglądarka, obiekty JS'],
    [() => W.browser?.['flatsql-json'], () => W.native?.['flatsql-native'], 'przeglądarka, JSON'],
  ],
  READS_WRITES,
);
ratioTable(
  'Flat-SQL na tle SQLite (H4): czas silnika / czas Flat-SQL (JSON), mediany; > 1 – Flat-SQL szybszy',
  [
    [() => W.node?.sqljs, () => W.node?.['flatsql-json'], 'sql.js, Node.js'],
    [() => W.node?.['sqlite-wasm'], () => W.node?.['flatsql-json'], 'SQLite WASM, Node.js'],
    [() => W.browser?.sqljs, () => W.browser?.['flatsql-json'], 'sql.js, przeglądarka'],
    [() => W.browser?.['sqlite-wasm'], () => W.browser?.['flatsql-json'], 'SQLite WASM, przeglądarka'],
  ],
  READS_WRITES,
);
ratioTable(
  'Koszt Workera (4.12): czas w Workerze / czas w wątku głównym, przeglądarka',
  [[() => W.browser?.['flatsql-worker'], () => W.browser?.flatsql, 'Flat-SQL, obiekty JS']],
  READS_WRITES.filter((o) => o !== 'snapshot' && o !== 'restore'),
);
{
  const a = W.browser?.['flatsql-worker'];
  const b = W.browser?.flatsql;
  if (a && b) {
    summary.workerCost = Object.fromEntries(['point_lookup', 'materialize_10k'].map((op) => [op, { workerUs: a[op].median, mainUs: b[op].median, differenceUs: a[op].median - b[op].median }]));
    md.push(`Różnica median (koszt komunikatu w obie strony): odczyt po kluczu ${durPlain(summary.workerCost.point_lookup.differenceUs)}, pobranie 10 000 wierszy ${durPlain(summary.workerCost.materialize_10k.differenceUs)}.`, '');
  }
}

// ---- cold start, module size, memory, requirements --------------------------------

function coldStats(target) {
  const out = {};
  for (const r of pick(target, 'cold')) {
    const o = (out[r.engine] ??= { initMs: [], openMs: [], firstQueryMs: [], totalMs: [], gzipBytes: [], bytes: [] });
    for (const k of ['initMs', 'openMs', 'firstQueryMs', 'totalMs']) o[k].push(r.result[k]);
    if (r.result.modules) {
      o.gzipBytes.push(r.result.modules.gzipBytes);
      o.bytes.push(r.result.modules.bytes);
    }
  }
  for (const e of Object.keys(out)) for (const k of Object.keys(out[e])) out[e][k] = stat(out[e][k]);
  return out;
}
function memoryStats(target) {
  const out = {};
  for (const r of pick(target, 'memory')) {
    const o = (out[r.engine] ??= { wasmAfter: [], wasmBefore: [], heapDelta: [], rssDelta: [], pageDelta: [] });
    o.wasmAfter.push(r.result.after.wasm);
    o.wasmBefore.push(r.result.before.wasm);
    if (r.result.delta.heapUsed !== undefined) o.heapDelta.push(r.result.delta.heapUsed);
    if (r.result.delta.rss !== undefined) o.rssDelta.push(r.result.delta.rss);
    if (r.result.delta.page !== undefined && Number.isFinite(r.result.delta.page)) o.pageDelta.push(r.result.delta.page);
  }
  for (const e of Object.keys(out)) for (const k of Object.keys(out[e])) out[e][k] = stat(out[e][k]);
  return out;
}
const msStat = (s) => (s ? dur({ ...s, median: s.median * 1000, q1: s.q1 * 1000, q3: s.q3 * 1000 }) : '–');
for (const t of ['browser', 'node']) {
  summary.cold[t] = coldStats(t);
  summary.memory[t] = memoryStats(t);
}
{
  const cold = summary.cold.browser;
  const coldNode = summary.cold.node;
  const mem = summary.memory.browser;
  const memNode = summary.memory.node;
  const engines = sortEngines([...Object.keys(cold), ...Object.keys(coldNode)]);
  if (engines.length) {
    md.push('## Rozmiar, zimny start, pamięć i wymagania (4.1, 4.3, 4.4)', '');
    md.push(`Zimny start: nowy proces przeglądarki z pustym profilem (bez pamięci podręcznej HTTP i skompilowanego kodu), od rozpoczęcia importu silnika do wyniku pierwszego zapytania: import i kompilacja modułu, otwarcie bazy ze schematem, pierwszy zapis i odczyt. Rozmiar: wszystkie pliki silnika pobrane przez stronę (bez kodu pomiaru), gzip -9. Pamięć: po wczytaniu ${manifest.options?.rows?.toLocaleString('pl-PL')} wierszy, pamięć liniowa WebAssembly (nie maleje) i przyrost sterty JS w Node.js po pełnym odśmiecaniu.`, '');
    table(
      ['Silnik', 'Grupa', 'Wymagania', 'Moduł (gzip)', 'Zimny start – przeglądarka', 'w tym import i kompilacja', 'Zimny start – Node.js', 'Pamięć WASM po wczytaniu', 'Przyrost sterty JS (Node.js)'],
      engines.map((e) => [
        label(e),
        GROUP_NAME[ENGINES[e]?.group] ?? '–',
        (ENGINES[e]?.needs ?? []).join(', ') || 'brak',
        cold[e]?.gzipBytes ? kib(cold[e].gzipBytes.median) : '–',
        msStat(cold[e]?.totalMs),
        msStat(cold[e]?.initMs),
        msStat(coldNode[e]?.totalMs),
        mem[e]?.wasmAfter?.median ? mib(mem[e].wasmAfter.median) : memNode[e]?.wasmAfter?.median ? mib(memNode[e].wasmAfter.median) : ENGINES[e]?.wasm === false ? 'nie dotyczy' : '–',
        memNode[e]?.heapDelta ? mib(Math.max(0, memNode[e].heapDelta.median)) : '–',
      ]),
      3,
    );
  }
}

md.push('### Silniki niemierzone – dane deklarowane przez autorów', '');
md.push('Wartości podane przez autorów, niepotwierdzone pomiarem w tym benchmarku; warunki ich pomiaru są inne niż w tabelach powyżej i nie można ich porównywać wprost.', '');
table(
  ['Silnik', 'Źródło', 'Odczyt po kluczu', 'Moduł (gzip)', 'Uwagi'],
  DECLARED.map((d) => [d.engine, d.source, `${num(d.pointLookupUs[0])}–${num(d.pointLookupUs[1])} µs`, `${dec(d.moduleGzipBytes / 1e6, 2)} MB`, d.note]),
  2,
);

// ---- durable writes, strict, checkpoint, fsync ---------------------------------

{
  const ws = W.browser;
  if (ws) {
    const engines = sortEngines(Object.keys(ws).filter((e) => ws[e].commit_single));
    if (engines.length) {
      md.push('## Trwały zapis (H5, 4.9)', '');
      table(
        ['Silnik', 'Grupa', 'Wstawienie N wierszy do trwałości', 'Zatwierdzenie 1 wiersza', 'p95 zatwierdzenia', '1000 zatwierdzeń – na jedno', 'Ponowne otwarcie + 1. zapytanie'],
        engines.map((e) => {
          const s = ws[e];
          const per = s.commit_group ? { ...s.commit_group, median: s.commit_group.median / 1000, q1: s.commit_group.q1 / 1000, q3: s.commit_group.q3 / 1000 } : null;
          return [label(e), GROUP_NAME[ENGINES[e].group], dur(s.insert), dur(s.commit_single), dur(s['commit_single:p95']), dur(per), dur(s.reopen)];
        }),
        2,
      );
    }
  }
}
{
  const runs = pick('browser', 'checkpoint');
  if (runs.length) {
    const out = {};
    for (const r of runs) {
      for (const s of r.result.sizes) {
        const o = ((out[r.engine] ??= {})[s.rows] ??= { maxGapMs: [], totalMs: [], bytes: [] });
        o.maxGapMs.push(s.maxGapMs);
        o.totalMs.push(s.totalMs);
        if (s.bytes) o.bytes.push(s.bytes);
      }
      ((out[r.engine] ??= {}).idle ??= []).push(r.result.idle.maxGapMs);
    }
    for (const e of Object.keys(out)) for (const k of Object.keys(out[e])) out[e][k] = k === 'idle' ? stat(out[e][k]) : Object.fromEntries(Object.entries(out[e][k]).map(([m, v]) => [m, stat(v)]));
    summary.checkpoint = out;
    md.push('## Pauza checkpointu w wątku głównym (4.10)', '');
    md.push('Najdłuższa przerwa między kolejnymi obrotami pętli zdarzeń (sonda `MessageChannel`) podczas `checkpoint()`; dla sql.js – eksport całej bazy i zapis do IndexedDB. Budżet jednej klatki przy 60 Hz: 16,7 ms.', '');
    const sizes = [...new Set(runs.flatMap((r) => r.result.sizes.map((s) => s.rows)))].sort((a, b) => a - b);
    table(
      ['Silnik', ...sizes.flatMap((n) => [`${n / 1000} tys. – najdłuższa pauza`, `${n / 1000} tys. – cały checkpoint`]), 'Snapshot przy 100 tys.', 'Bezczynność – najdłuższa przerwa'],
      sortEngines(Object.keys(out)).map((e) => [
        label(e),
        ...sizes.flatMap((n) => [msStat(out[e][n]?.maxGapMs), msStat(out[e][n]?.totalMs)]),
        out[e][sizes[sizes.length - 1]]?.bytes ? mib(out[e][sizes[sizes.length - 1]].bytes.median) : '–',
        msStat(out[e].idle),
      ]),
    );
  }
}
{
  const nat = pick('native', 'fsync');
  const node = pick('node', 'fsync');
  if (nat.length || node.length) {
    md.push('## Koszt utrwalania plików (4.13)', '');
    const agg = (list, get) => {
      const out = {};
      for (const r of list) for (const [mode, ops] of Object.entries(get(r))) for (const [op, v] of Object.entries(ops)) ((out[mode] ??= {})[op] ??= []).push(v.meanUs);
      for (const m of Object.keys(out)) for (const op of Object.keys(out[m])) out[m][op] = stat(out[m][op]);
      return out;
    };
    const n = agg(nat, (r) => ({ 'FileStorage (CLI), fsync': r.result.fsync, 'FileStorage (CLI), bez fsync': r.result.nofsync }));
    const j = agg(node, (r) => ({ 'FsStore (Node.js), fsync': r.result.fsStore.fsync, 'FsStore (Node.js), bez fsync': r.result.fsStore.nofsync }));
    const all = { ...n, ...j };
    summary.fsync = all;
    const ops = ['insert', 'commit_single', 'commit_group', 'checkpoint', 'reopen'];
    const names = { ...OP_LABEL, checkpoint: 'Checkpoint (snapshot + fsync katalogu)' };
    table(['Magazyn', ...ops.map((o) => names[o])], Object.entries(all).map(([m, s]) => [m, ...ops.map((o) => dur(s[o]))]));
    const push = { durableServer: [], memoryServer: [] };
    for (const r of node) for (const k of Object.keys(push)) push[k].push(r.result.syncPush[k].meanUs);
    if (node.length) {
      summary.fsync.syncPush = { durableServer: stat(push.durableServer), memoryServer: stat(push.memoryServer) };
      md.push(`Runda synchronizacji z jedną zmianą (\`/sync/push\` + \`/sync/pull\`, localhost): serwer z katalogiem danych (FsStore, fsync) ${dur(summary.fsync.syncPush.durableServer)}, serwer w pamięci ${dur(summary.fsync.syncPush.memoryServer)}.`, '');
    }
  }
}

// ---- H1: network ------------------------------------------------------------------

{
  const out = {};
  for (const t of ['node', 'browser']) {
    for (const r of pick(t, 'network')) for (const m of r.result.modes) ((out[t] ??= {})[m.mode] ??= []).push(m.meanMs * 1000);
    if (out[t]) for (const k of Object.keys(out[t])) out[t][k] = stat(out[t][k]);
  }
  summary.network = out;
  const modes = [...new Set(Object.values(out).flatMap((o) => Object.keys(o)))];
  if (modes.length) {
    md.push('## Local-First a Cloud-First (H1)', '');
    md.push('Odczyt po kluczu z tabeli 10 000 wierszy: lokalnie (`db.exec`, z parsowaniem SQL) i przez `POST /api/query` do serwera z tym samym silnikiem; opóźnienie sieci symulowane po stronie serwera.', '');
    const MODE = { 'local (WASM, in-process)': 'lokalnie (WASM w procesie)', 'cloud API (localhost HTTP)': 'serwer, localhost', 'cloud API (+20 ms RTT)': 'serwer, RTT 20 ms', 'cloud API (+50 ms RTT)': 'serwer, RTT 50 ms', 'cloud API (+100 ms RTT)': 'serwer, RTT 100 ms' };
    const localNode = out.node?.[modes[0]]?.median;
    const localBrowser = out.browser?.[modes[0]]?.median;
    table(
      ['Tryb', 'Node.js', 'krotność lokalnego', 'Przeglądarka', 'krotność lokalnego'],
      modes.map((m) => [MODE[m] ?? m, dur(out.node?.[m]), out.node?.[m] && localNode ? ratio(out.node[m].median / localNode) : '–', dur(out.browser?.[m]), out.browser?.[m] && localBrowser ? ratio(out.browser[m].median / localBrowser) : '–']),
    );
  }
}

// ---- spread of the measurement ----------------------------------------------------

{
  const rows = [];
  for (const t of Object.keys(W)) {
    for (const [e, ops] of Object.entries(W[t])) {
      for (const [op, s] of Object.entries(ops)) {
        if (op.includes(':') || !s?.n || s.n < 3) continue;
        rows.push({ t, e, op, cv: s.cv, iqrRel: s.iqr === undefined ? (s.q3 - s.q1) / s.median : 0 });
      }
    }
  }
  if (rows.length) {
    const cvs = rows.map((r) => (r.cv)).sort((a, b) => a - b);
    md.push('## Rozrzut między uruchomieniami', '');
    md.push(`Współczynnik zmienności (odchylenie standardowe / średnia) średnich z ${rows.length} par silnik–operacja: mediana ${dec(quantile(cvs, 0.5) * 100, 1)}%, 90. percentyl ${dec(quantile(cvs, 0.9) * 100, 1)}%, maksimum ${dec(cvs[cvs.length - 1] * 100, 1)}%. Największy rozrzut:`, '');
    table(
      ['Środowisko', 'Silnik', 'Operacja', 'CV'],
      rows.sort((a, b) => b.cv - a.cv).slice(0, 8).map((r) => [TARGET_TITLE[r.t], label(r.e), OP_LABEL[r.op] ?? r.op, `${dec(r.cv * 100, 1)}%`]),
      3,
    );
  }
}

// ---- figures ------------------------------------------------------------------------

const figures = {};
/** Rows of a figure, one section per group of run conditions. */
function grouped(ids, row) {
  const out = [];
  for (const g of ['memory', 'durable', 'worker']) {
    const list = sortEngines(ids.filter((e) => ENGINES[e]?.group === g));
    if (!list.length) continue;
    out.push({ section: `grupa „${GROUP_NAME[g]}”` });
    out.push(...list.map(row));
  }
  return out;
}
const memBrowser = W.browser ? sortEngines(Object.keys(W.browser).filter((e) => ENGINES[e]?.group === 'memory')) : [];
if (memBrowser.length) {
  figures['01-odczyt-po-kluczu'] = dotPlot({
    title: 'Odczyt po kluczu – przeglądarka',
    subtitle: `SELECT * FROM items WHERE id = ?, ${ROWS} wierszy; średnia z 10 000 wykonań w każdym uruchomieniu`,
    series: [{ name: 'czas', color: C.s1 }],
    rows: [
      ...grouped(Object.keys(W.browser).filter((e) => W.browser[e].point_lookup), (e) => ({ label: label(e), points: [{ series: 'czas', value: W.browser[e].point_lookup }] })),
      { section: 'niemierzone' },
      ...DECLARED.map((d) => ({
        label: `${d.engine} ${d.source}`,
        points: [{ series: 'czas', declared: true, value: { q1: d.pointLookupUs[0], q3: d.pointLookupUs[1], median: (d.pointLookupUs[0] + d.pointLookupUs[1]) / 2 } }],
      })),
    ],
  });
  const trio = ['flatsql-json', 'sqljs', 'sqlite-wasm'].filter((e) => W.browser[e]);
  figures['02-flatsql-a-sqlite'] = dotPlot({
    title: 'Flat-SQL a SQLite w WebAssembly – przeglądarka, dane w pamięci',
    subtitle: `${ROWS} wierszy; czas jednej operacji`,
    series: trio.map((e, i) => ({ name: label(e), color: [C.s1, C.s2, C.s3][i] })),
    rows: READS_WRITES.filter((op) => trio.some((e) => W.browser[e][op])).map((op) => ({ label: OP_LABEL[op], points: trio.map((e) => ({ series: label(e), value: W.browser[e][op] })) })),
  });
}
if (W.native && W.node) {
  const pairs = READS_WRITES.filter((op) => W.node.flatsql?.[op] && W.native['flatsql-native']?.[op]);
  figures['03-narzut-wasm'] = dotPlot({
    title: 'Ten sam silnik: WebAssembly w Node.js a kod natywny',
    subtitle: `${ROWS} wierszy; czas jednej operacji`,
    series: [
      { name: 'natywnie', color: C.s3 },
      { name: 'WASM, JSON', color: C.s1 },
      { name: 'WASM, obiekty JS', color: C.s2 },
    ],
    rows: pairs.map((op) => ({ label: OP_LABEL[op], points: [{ series: 'natywnie', value: W.native['flatsql-native'][op] }, { series: 'WASM, JSON', value: W.node['flatsql-json']?.[op] }, { series: 'WASM, obiekty JS', value: W.node.flatsql[op] }] })),
  });
}
if (Object.keys(summary.cold.browser ?? {}).length) {
  const cold = summary.cold.browser;
  figures['04-zimny-start'] = stackedBars({
    title: 'Zimny start w przeglądarce',
    subtitle: 'nowy proces przeglądarki z pustym profilem; od importu silnika do wyniku pierwszego zapytania',
    series: [
      { name: 'import i kompilacja', color: C.s1 },
      { name: 'otwarcie bazy', color: C.s2 },
      { name: 'pierwszy zapis i odczyt', color: C.s3 },
    ],
    rows: sortEngines(Object.keys(cold)).map((e) => ({ label: label(e), parts: [cold[e].initMs.median, cold[e].openMs.median, cold[e].firstQueryMs.median] })),
  });
}
if (Object.keys(summary.checkpoint).length) {
  const cp = summary.checkpoint;
  const engines = sortEngines(Object.keys(cp));
  const sizes = Object.keys(cp[engines[0]]).filter((k) => k !== 'idle').map(Number).sort((a, b) => a - b);
  figures['05-pauza-checkpointu'] = lineChart({
    title: 'Najdłuższa pauza wątku głównego podczas checkpointu',
    subtitle: 'przeglądarka, IndexedDB; sql.js – eksport całej bazy i zapis pliku',
    xs: sizes.map((n) => `${n / 1000} tys.`),
    xLabel: 'liczba wierszy w bazie',
    ref: { value: 1000 / 60, label: 'jedna klatka przy 60 Hz (16,7 ms)' },
    series: engines.map((e, i) => ({ name: label(e), color: [C.s1, C.s2, C.s3][i], points: sizes.map((n) => cp[e][n]?.maxGapMs ?? null) })),
  });
}
if (W.browser) {
  const engines = sortEngines(Object.keys(W.browser).filter((e) => W.browser[e].commit_single));
  if (engines.length) {
    figures['06-trwale-zatwierdzenie'] = dotPlot({
      title: 'Trwałe zatwierdzenie jednego wiersza – przeglądarka',
      subtitle: 'zapis jednego wiersza i oczekiwanie na trwałość; 100 zatwierdzeń w każdym uruchomieniu',
      series: [{ name: 'czas', color: C.s1 }],
      rows: grouped(engines, (e) => ({ label: label(e), points: [{ series: 'czas', value: W.browser[e].commit_single }] })),
    });
  }
}
if (Object.keys(summary.network).length) {
  const net = summary.network;
  const modes = Object.keys(net.node ?? net.browser);
  const NAMES = { 'local (WASM, in-process)': 'lokalnie', 'cloud API (localhost HTTP)': 'serwer, localhost', 'cloud API (+20 ms RTT)': 'serwer, RTT 20 ms', 'cloud API (+50 ms RTT)': 'serwer, RTT 50 ms', 'cloud API (+100 ms RTT)': 'serwer, RTT 100 ms' };
  figures['07-local-first-a-cloud-first'] = dotPlot({
    title: 'Odczyt po kluczu: lokalnie a przez serwer',
    subtitle: 'ten sam silnik po obu stronach; opóźnienie sieci symulowane',
    series: [
      { name: 'Node.js', color: C.s1 },
      { name: 'przeglądarka', color: C.s2 },
    ],
    rows: modes.map((m) => ({ label: NAMES[m] ?? m, points: [{ series: 'Node.js', value: net.node?.[m], labelled: true }, { series: 'przeglądarka', value: net.browser?.[m], labelled: true }] })),
  });
}

await mkdir(join(DIR, 'figures'), { recursive: true });
for (const [name, svg] of Object.entries(figures)) await writeFile(join(DIR, 'figures', `${name}.svg`), `${svg}\n`);
md.push('## Wykresy', '', ...Object.keys(figures).map((f) => `- [figures/${f}.svg](figures/${f}.svg)`), '');
await writeFile(join(DIR, 'tables.md'), md.join('\n'));
await writeFile(join(DIR, 'summary.json'), JSON.stringify(summary, null, 1));
console.error(`tables.md, summary.json and ${Object.keys(figures).length} figures written to ${DIR}`);
