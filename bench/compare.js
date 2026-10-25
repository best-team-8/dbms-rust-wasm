// Before/after measurement of one optimisation: two versions of Flat-SQL are
// measured by the same runner (bench/run.js) in alternating runs, so a trend
// in the state of the machine (temperature, background work) affects both
// versions alike, and the result is compared with the criterion of the
// research scenario (§5.4): a change is shown when the 95% confidence
// intervals of the means do not overlap.
//
//   node bench/compare.js --before DIR --after DIR --out DIR
//                         [--runs 10] [--targets node,native] [--kinds workload]
//                         [--engines flatsql,flatsql-json] [--browser-channel msedge]
//
// --before / --after are checkouts with built pkg/ and target/release (e.g.
// worktrees made by bench/build-version.js, or the current checkout). Output:
//   DIR/before/, DIR/after/   raw results of bench/run.js
//   DIR/manifest.json         both versions, CPU load before the measurement,
//                             size of the WebAssembly module
//   DIR/compare.md            table: median (Q1–Q3) before and after, change,
//                             whether the change is shown; equal results

import { spawn, spawnSync } from 'node:child_process';
import { existsSync, readFileSync } from 'node:fs';
import { mkdir, readdir, readFile, writeFile } from 'node:fs/promises';
import { cpus } from 'node:os';
import { join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import { gzipSync } from 'node:zlib';

import { codeTree } from './code-tree.js';
import { summarize } from './stats.js';

const ROOT = fileURLToPath(new URL('..', import.meta.url));

function parseArgs(argv) {
  const o = { runs: 10, targets: 'node,native', kinds: 'workload', engines: 'flatsql,flatsql-json', browserChannel: 'msedge', rows: null };
  for (let i = 0; i < argv.length; i++) {
    const a = argv[i];
    const next = () => argv[++i];
    if (a === '--before') o.before = resolve(next());
    else if (a === '--after') o.after = resolve(next());
    else if (a === '--out') o.out = resolve(next());
    else if (a === '--runs') o.runs = Number(next());
    else if (a === '--targets') o.targets = next();
    else if (a === '--kinds') o.kinds = next();
    else if (a === '--engines') o.engines = next();
    else if (a === '--rows') o.rows = next();
    else if (a === '--browser-channel') o.browserChannel = next();
    else throw new Error(`unknown option ${a}`);
  }
  if (!o.before || !o.after || !o.out) throw new Error('usage: node bench/compare.js --before DIR --after DIR --out DIR');
  return o;
}

const log = (msg) => process.stderr.write(`${new Date().toISOString().slice(11, 19)} ${msg}\n`);

function run(args) {
  return new Promise((done, fail) => {
    const child = spawn(process.execPath, [join(ROOT, 'bench', 'run.js'), ...args], { stdio: ['ignore', 'ignore', 'inherit'] });
    child.on('close', (code) => (code === 0 ? done() : fail(new Error(`bench/run.js exited with ${code}`))));
  });
}

/** Share of busy CPU time over `ms` milliseconds, all logical processors. */
async function cpuLoad(ms = 3000) {
  const total = () => cpus().reduce((acc, c) => {
    const t = c.times;
    acc.busy += t.user + t.nice + t.sys + t.irq;
    acc.all += t.user + t.nice + t.sys + t.irq + t.idle;
    return acc;
  }, { busy: 0, all: 0 });
  const a = total();
  await new Promise((r) => setTimeout(r, ms));
  const b = total();
  return (b.busy - a.busy) / (b.all - a.all);
}

function moduleSize(root) {
  const file = join(root, 'pkg', 'flatsql_bg.wasm');
  if (!existsSync(file)) return null;
  const bytes = readFileSync(file);
  return { bytes: bytes.length, gzipBytes: gzipSync(bytes, { level: 9 }).length };
}

async function readRaw(dir) {
  const base = join(dir, 'raw');
  const out = [];
  if (!existsSync(base)) return out;
  for (const target of await readdir(base)) {
    for (const kind of await readdir(join(base, target))) {
      for (const engine of await readdir(join(base, target, kind))) {
        for (const f of (await readdir(join(base, target, kind, engine))).sort()) {
          out.push({ target, kind, engine, data: JSON.parse(await readFile(join(base, target, kind, engine, f), 'utf8')) });
        }
      }
    }
  }
  return out;
}

/** Metrics of one raw result: name → value (µs for durations, bytes for sizes). */
function metrics(r) {
  const m = {};
  const res = r.data.result;
  if (r.kind === 'workload') {
    for (const [op, v] of Object.entries(res.ops ?? {})) {
      if (Number.isFinite(v.meanUs)) m[op] = v.meanUs;
      if (Number.isFinite(v.bytes)) m[`${op}:bytes`] = v.bytes;
    }
  } else if (r.kind === 'memory') {
    if (Number.isFinite(res.after?.wasm)) m['wasm_after_load:bytes'] = res.after.wasm;
    if (Number.isFinite(res.delta?.heapUsed)) m['js_heap_delta:bytes'] = res.delta.heapUsed;
  } else if (r.kind === 'cold') {
    for (const k of ['initMs', 'openMs', 'firstQueryMs', 'totalMs']) if (Number.isFinite(res[k])) m[k.replace('Ms', '')] = res[k] * 1000;
  } else if (r.kind === 'fsync') {
    for (const [mode, ops] of Object.entries(res.fsStore ?? {})) {
      for (const [op, v] of Object.entries(ops)) if (Number.isFinite(v.meanUs)) m[`FsStore ${mode} ${op}`] = v.meanUs;
    }
    for (const [mode, v] of Object.entries(res.syncPush ?? {})) if (Number.isFinite(v.meanUs)) m[`sync round ${mode}`] = v.meanUs;
    for (const [mode, ops] of Object.entries(res.file ?? res.storage ?? {})) {
      if (ops && typeof ops === 'object') for (const [op, v] of Object.entries(ops)) if (Number.isFinite(v?.meanUs)) m[`FileStorage ${mode} ${op}`] = v.meanUs;
    }
  } else if (r.kind === 'network') {
    for (const mode of res.modes ?? []) if (Number.isFinite(mode.meanMs)) m[mode.mode] = mode.meanMs * 1000;
  } else if (r.kind === 'checkpoint') {
    for (const s of res.sizes ?? []) {
      if (Number.isFinite(s.maxPauseMs)) m[`pause ${s.rows}`] = s.maxPauseMs * 1000;
      if (Number.isFinite(s.checkpointMs)) m[`checkpoint ${s.rows}`] = s.checkpointMs * 1000;
    }
  }
  return m;
}

const fmtNum = (x) => {
  const a = Math.abs(x);
  return (a >= 100 ? x.toFixed(0) : a >= 10 ? x.toFixed(1) : x.toFixed(2)).replace('.', ',');
};
function fmt(value, metric) {
  if (metric.endsWith(':bytes')) return value >= 1 << 20 ? `${fmtNum(value / (1 << 20))} MiB` : value >= 1024 ? `${fmtNum(value / 1024)} KiB` : `${value} B`;
  const [u, k] = value >= 1e6 ? ['s', 1e6] : value >= 1000 ? ['ms', 1000] : ['µs', 1];
  return `${fmtNum(value / k)} ${u}`;
}
function cell(s, metric) {
  if (!s) return '–';
  const [u, k] = metric.endsWith(':bytes') ? ['', 1] : s.median >= 1e6 ? ['s', 1e6] : s.median >= 1000 ? ['ms', 1000] : ['µs', 1];
  if (metric.endsWith(':bytes')) return s.q1 === s.q3 ? fmt(s.median, metric) : `${fmt(s.median, metric)} (${fmt(s.q1, metric)}–${fmt(s.q3, metric)})`;
  return `${fmtNum(s.median / k)} (${fmtNum(s.q1 / k)}–${fmtNum(s.q3 / k)}) ${u}`;
}

function report(before, after, manifest) {
  const lines = [];
  const key = (r) => `${r.target}|${r.kind}|${r.engine}`;
  const groups = new Map();
  for (const [side, list] of [['before', before], ['after', after]]) {
    for (const r of list) {
      const k = key(r);
      if (!groups.has(k)) groups.set(k, { before: [], after: [] });
      groups.get(k)[side].push(r);
    }
  }
  lines.push(`Przed: ${manifest.before.describe} (${manifest.before.subject}); po: ${manifest.after.describe} (${manifest.after.subject}).`);
  lines.push(`Obciążenie procesora przed pomiarem: ${(manifest.cpuLoadBefore * 100).toFixed(1).replace('.', ',')}%. Uruchomienia naprzemienne, ${manifest.options.runs} na wersję.`);
  if (manifest.before.module && manifest.after.module) {
    const b = manifest.before.module;
    const a = manifest.after.module;
    lines.push(`Moduł WebAssembly: ${b.bytes} → ${a.bytes} B, gzip -9: ${b.gzipBytes} → ${a.gzipBytes} B (${a.gzipBytes >= b.gzipBytes ? '+' : ''}${(a.gzipBytes - b.gzipBytes)} B).`);
  }
  lines.push('');
  for (const [k, g] of [...groups].sort()) {
    const [target, kind, engine] = k.split('|');
    lines.push(`### ${target} – ${kind} – ${engine}`, '');
    lines.push('| Metryka | Przed | Po | Zmiana mediany | 95% CI średnich rozłączne |', '|---|---:|---:|---:|---|');
    const names = new Set([...g.before, ...g.after].flatMap((r) => Object.keys(metrics(r))));
    for (const name of names) {
      const sb = summarize(g.before.map((r) => metrics(r)[name]));
      const sa = summarize(g.after.map((r) => metrics(r)[name]));
      let change = '–';
      let shown = '–';
      if (sb && sa) {
        change = `${sa.median >= sb.median ? '+' : ''}${(((sa.median - sb.median) / sb.median) * 100).toFixed(1).replace('.', ',')}%`;
        if (sb.n > 1 && sa.n > 1) shown = sa.ci95[1] < sb.ci95[0] ? 'tak, szybciej/mniej' : sa.ci95[0] > sb.ci95[1] ? 'tak, wolniej/więcej' : 'nie (w rozrzucie)';
      }
      lines.push(`| ${name.replace(':bytes', ' (rozmiar)')} | ${cell(sb, name)} | ${cell(sa, name)} | ${change} | ${shown} |`);
    }
    // The optimisation must not change any answer: checksums of every query
    // and the table state after every phase are compared between versions.
    if (kind === 'workload') {
      const ref = g.before[0]?.data.result;
      let compared = 0;
      const diff = [];
      for (const r of [...g.before, ...g.after]) {
        const res = r.data.result;
        for (const [op, c] of Object.entries(res.checks ?? {})) {
          const want = ref?.checks?.[op];
          if (!want) continue;
          compared++;
          if (c.head !== want.head || (c.n === want.n && c.full !== want.full)) diff.push(op);
        }
        if (ref?.state && res.state) {
          compared++;
          if (res.state !== ref.state) diff.push('stan tabeli');
        }
      }
      lines.push('', diff.length ? `Różne wyniki: ${[...new Set(diff)].join(', ')} (${diff.length} z ${compared} porównań).` : `Wyniki zgodne w obu wersjach (${compared} porównań sum kontrolnych i stanu tabeli).`);
    }
    lines.push('');
  }
  return lines.join('\n');
}

function gitInfo(root) {
  const sh = (args) => {
    const r = spawnSync('git', args, { cwd: root, encoding: 'utf8' });
    return r.status === 0 ? r.stdout.trim() : null;
  };
  const info = { root, describe: sh(['describe', '--tags', '--always', '--dirty']), commit: sh(['rev-parse', 'HEAD']), subject: sh(['log', '-1', '--format=%s']) };
  info.codeTreeSha256 = codeTree(root);
  return info;
}

const o = parseArgs(process.argv.slice(2));
await mkdir(o.out, { recursive: true });
const manifestPath = join(o.out, 'manifest.json');
const manifest = existsSync(manifestPath)
  ? JSON.parse(await readFile(manifestPath, 'utf8'))
  : {
      date: new Date().toISOString(),
      options: o,
      before: { ...gitInfo(o.before), module: moduleSize(o.before) },
      after: { ...gitInfo(o.after), module: moduleSize(o.after) },
      cpuLoadBefore: await cpuLoad(),
    };
await writeFile(manifestPath, JSON.stringify(manifest, null, 2));

const common = ['--targets', o.targets, '--kinds', o.kinds, '--engines', o.engines, '--browser-channel', o.browserChannel, ...(o.rows ? ['--rows', o.rows] : [])];
for (let r = 1; r <= o.runs; r++) {
  // Alternate which version goes first, so neither always runs on a machine
  // that has just been busy.
  const order = r % 2 ? ['before', 'after'] : ['after', 'before'];
  for (const side of order) {
    log(`run ${r}/${o.runs}: ${side}`);
    await run([...common, '--runs', String(r), '--flatsql', o[side], '--out', join(o.out, side), '--label', side]);
  }
}
manifest.finished = new Date().toISOString();
await writeFile(manifestPath, JSON.stringify(manifest, null, 2));
const md = report(await readRaw(join(o.out, 'before')), await readRaw(join(o.out, 'after')), manifest);
await writeFile(join(o.out, 'compare.md'), `${md}\n`);
console.log(md);
