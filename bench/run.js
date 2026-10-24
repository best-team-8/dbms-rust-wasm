// Benchmark runner: repeats every measurement in fresh processes and stores
// the raw result of each process (docs/research/benchmark-scenario.md).
//
//   node bench/run.js [--targets node,native,browser] [--runs 10]
//                     [--kinds workload,cold,memory,network,fsync,checkpoint,worker]
//                     [--engines id,id,...] [--rows 100000]
//                     [--flatsql DIR] [--out DIR] [--label NAME]
//                     [--browser-channel msedge] [--browser-exe PATH] [--headed]
//
// --flatsql selects the measured version of Flat-SQL: a checkout with built
// pkg/ (wasm-pack), bridge/, server/ and target/release/flatsql[.exe], e.g. a
// worktree of a tag prepared by `node bench/build-version.js prototyp`. The
// benchmark code (this directory) always comes from the current checkout, so
// two versions of the engine are measured by the same tool.
//
// Output (--out, default bench/results/<date>-<label>/):
//   manifest.json                      environment, versions, options
//   raw/<target>/<kind>/<engine>/NN.json one file per measured process
//   summary.md                         quick overview (median [Q1–Q3])

import { spawn, spawnSync } from 'node:child_process';
import { existsSync, readFileSync } from 'node:fs';
import { mkdir, readdir, readFile, writeFile } from 'node:fs/promises';
import { cpus, totalmem, release, platform, arch, tmpdir } from 'node:os';
import { join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

import { ENGINES, enginesIn } from './engines/index.js';
import { summarize } from './stats.js';
import { DEFAULT_ROWS, READ_OPS, Rng, WORKLOAD } from './suite.js';

const ROOT = fileURLToPath(new URL('..', import.meta.url));
const NODE_KINDS = ['workload', 'cold', 'memory', 'network', 'fsync'];

function parseArgs(argv) {
  const o = {
    targets: ['node', 'native'],
    runs: 10,
    rows: DEFAULT_ROWS,
    kinds: null,
    engines: null,
    flatsql: ROOT,
    out: null,
    label: null,
    browserChannel: 'msedge',
    browserExe: null,
    headed: false,
  };
  for (let i = 0; i < argv.length; i++) {
    const a = argv[i];
    const next = () => argv[++i];
    if (a === '--targets') o.targets = next().split(',');
    else if (a === '--runs') o.runs = Number(next());
    else if (a === '--rows') o.rows = Number(next());
    else if (a === '--kinds') o.kinds = next().split(',');
    else if (a === '--engines') o.engines = next().split(',');
    else if (a === '--flatsql') o.flatsql = resolve(next());
    else if (a === '--out') o.out = resolve(next());
    else if (a === '--label') o.label = next();
    else if (a === '--browser-channel') o.browserChannel = next();
    else if (a === '--browser-exe') o.browserExe = next();
    else if (a === '--headed') o.headed = true;
    else throw new Error(`unknown option ${a}`);
  }
  return o;
}

const sh = (cmd, args, cwd) => {
  const r = spawnSync(cmd, args, { cwd, encoding: 'utf8', shell: false });
  return r.status === 0 ? r.stdout.trim() : null;
};

function packageVersions() {
  const names = ['sql.js', '@sqlite.org/sqlite-wasm', '@journeyapps/wa-sqlite', '@electric-sql/pglite', '@duckdb/duckdb-wasm', 'alasql', 'dexie', 'pouchdb-browser', 'rxdb', 'playwright-core', 'esbuild'];
  return Object.fromEntries(
    names.map((n) => {
      try {
        return [n, JSON.parse(readFileSync(join(ROOT, 'node_modules', n, 'package.json'), 'utf8')).version];
      } catch {
        return [n, null];
      }
    }),
  );
}

function nativeExe(root) {
  return join(root, 'target', 'release', process.platform === 'win32' ? 'flatsql.exe' : 'flatsql');
}

/** Fisher–Yates with the deterministic generator: engine order of run `r`. */
function shuffled(list, seed) {
  const rng = new Rng(seed);
  const a = [...list];
  for (let i = a.length - 1; i > 0; i--) {
    const j = rng.below(i + 1);
    [a[i], a[j]] = [a[j], a[i]];
  }
  return a;
}

function runProcess(cmd, args, { timeoutMs = 30 * 60_000 } = {}) {
  return new Promise((resolvePromise) => {
    const child = spawn(cmd, args, { stdio: ['ignore', 'pipe', 'pipe'] });
    let err = '';
    child.stderr.on('data', (d) => (err += d));
    child.stdout.on('data', () => {});
    const timer = setTimeout(() => child.kill(), timeoutMs);
    child.on('close', (code) => {
      clearTimeout(timer);
      resolvePromise({ code, err });
    });
  });
}

const pad = (n) => String(n).padStart(2, '0');
const log = (msg) => process.stderr.write(`${new Date().toISOString().slice(11, 19)} ${msg}\n`);

async function runNode(o, outDir) {
  const kinds = (o.kinds ?? NODE_KINDS).filter((k) => NODE_KINDS.includes(k));
  const engines = (o.engines ?? enginesIn('node')).filter((id) => ENGINES[id]?.where.includes('node'));
  const failures = [];
  for (let run = 1; run <= o.runs; run++) {
    for (const kind of kinds) {
      const list = kind === 'network' || kind === 'fsync' ? ['flatsql'] : shuffled(engines, run * 31 + kinds.indexOf(kind));
      for (const engine of list) {
        const dir = join(outDir, 'raw', 'node', kind, engine);
        await mkdir(dir, { recursive: true });
        const out = join(dir, `${pad(run)}.json`);
        if (existsSync(out)) continue; // resume an interrupted measurement
        const task = { kind, engine, rows: o.rows, flatsqlRoot: o.flatsql, out, run };
        const args = [...(kind === 'memory' ? ['--expose-gc'] : []), join(ROOT, 'bench', 'node', 'child.js'), JSON.stringify(task)];
        log(`node ${kind} ${engine} run ${run}/${o.runs}`);
        const r = await runProcess(process.execPath, args);
        if (r.code !== 0) {
          failures.push({ target: 'node', kind, engine, run, error: r.err.slice(-2000) });
          log(`  FAILED: ${r.err.split('\n').find((l) => /Error/.test(l)) ?? r.err.slice(-300)}`);
        }
      }
    }
  }
  return failures;
}

async function runNative(o, outDir) {
  const exe = nativeExe(o.flatsql);
  if (!existsSync(exe)) throw new Error(`native binary not found: ${exe} (cargo build --release -p flatsql-cli)`);
  const failures = [];
  const kinds = o.kinds ?? ['workload', 'fsync'];
  for (let run = 1; run <= o.runs; run++) {
    for (const kind of kinds.filter((k) => k === 'workload' || k === 'fsync')) {
      const dir = join(outDir, 'raw', 'native', kind, 'flatsql-native');
      await mkdir(dir, { recursive: true });
      const out = join(dir, `${pad(run)}.json`);
      if (existsSync(out)) continue;
      const args = ['bench', '--rows', String(o.rows), '--json', out];
      if (kind === 'fsync') args.push('--storage', join(tmpdir(), `flatsql-bench-${process.pid}-${run}`));
      log(`native ${kind} run ${run}/${o.runs}`);
      const r = await runProcess(exe, args);
      if (r.code !== 0) {
        failures.push({ target: 'native', kind, run, error: r.err.slice(-2000) });
        continue;
      }
      const result = JSON.parse(await readFile(out, 'utf8'));
      if (result.workload !== WORKLOAD) {
        throw new Error(`native benchmark reports workload ${result.workload}, this runner measures ${WORKLOAD}`);
      }
    }
  }
  return failures;
}

async function readRaw(outDir) {
  const base = join(outDir, 'raw');
  const out = [];
  if (!existsSync(base)) return out;
  for (const target of await readdir(base)) {
    for (const kind of await readdir(join(base, target))) {
      for (const engine of await readdir(join(base, target, kind))) {
        for (const f of await readdir(join(base, target, kind, engine))) {
          out.push({ target, kind, engine, data: JSON.parse(await readFile(join(base, target, kind, engine, f), 'utf8')) });
        }
      }
    }
  }
  return out;
}

const fmt = (us) => (us >= 1e6 ? `${(us / 1e6).toFixed(2)} s` : us >= 1000 ? `${(us / 1000).toFixed(2)} ms` : `${us.toFixed(us < 10 ? 2 : 1)} µs`);
const cell = (s) => (s ? `${fmt(s.median)} [${fmt(s.q1)}–${fmt(s.q3)}]` : '–');

/** Quick overview; the tables of the thesis come from docs/research/data/<benchmark>/. */
function overview(raw) {
  const lines = [];
  const workloads = raw.filter((r) => r.kind === 'workload');
  const byTarget = Map.groupBy(workloads, (r) => r.target);
  for (const [target, items] of byTarget) {
    const byGroup = Map.groupBy(items, (r) => ENGINES[r.engine]?.group ?? 'native');
    for (const [group, list] of byGroup) {
      const byEngine = Map.groupBy(list, (r) => r.engine);
      const engines = [...byEngine.keys()];
      const ops = ['insert', ...READ_OPS.map((op) => op.name), 'update_10pct', 'delete_10pct', 'snapshot', 'restore', 'commit_single', 'commit_group', 'reopen'];
      lines.push(`### ${target} – ${group} (n = runs)`, '', `| Operation | ${engines.join(' | ')} |`, `|---|${engines.map(() => '---:').join('|')}|`);
      for (const op of ops) {
        const cells = engines.map((e) => {
          const runs = byEngine.get(e).map((r) => r.data.result.ops?.[op]?.meanUs);
          return runs.some((v) => v !== undefined) ? `${cell(summarize(runs))} n=${runs.filter((v) => v !== undefined).length}` : '–';
        });
        if (cells.some((c) => c !== '–')) lines.push(`| ${op} | ${cells.join(' | ')} |`);
      }
      lines.push('');
      // Checksums: every engine of the group must return the same results.
      const ref = byEngine.get(engines[0])?.[0]?.data.result;
      for (const e of engines.slice(1)) {
        for (const r of byEngine.get(e)) {
          for (const [op, c] of Object.entries(r.data.result.checks ?? {})) {
            const want = ref?.checks?.[op];
            if (want && (c.head !== want.head || (c.n === want.n && c.full !== want.full))) lines.push(`- checksum mismatch: ${e} ${op} (run ${r.data.task?.run})`);
          }
        }
      }
    }
  }
  return lines.join('\n');
}

const o = parseArgs(process.argv.slice(2));
const stamp = new Date().toISOString().slice(0, 19).replace(/[:T]/g, '-');
const outDir = o.out ?? join(ROOT, 'bench', 'results', `${stamp}${o.label ? `-${o.label}` : ''}`);
await mkdir(outDir, { recursive: true });

const manifestPath = join(outDir, 'manifest.json');
const manifest = existsSync(manifestPath)
  ? JSON.parse(await readFile(manifestPath, 'utf8'))
  : {
      workload: WORKLOAD,
      date: new Date().toISOString(),
      options: { ...o, flatsql: o.flatsql },
      flatsql: {
        root: o.flatsql,
        describe: sh('git', ['describe', '--tags', '--always', '--dirty'], o.flatsql),
        commit: sh('git', ['rev-parse', 'HEAD'], o.flatsql),
      },
      tool: { commit: sh('git', ['rev-parse', 'HEAD'], ROOT), describe: sh('git', ['describe', '--tags', '--always', '--dirty'], ROOT) },
      host: {
        cpu: cpus()[0]?.model?.trim(),
        threads: cpus().length,
        memoryBytes: totalmem(),
        os: `${platform()} ${release()} ${arch()}`,
      },
      node: process.version,
      rustc: sh('rustc', ['--version'], ROOT),
      packages: packageVersions(),
      // Engine registry at measurement time: labels, groups and requirements
      // used by the report scripts.
      engines: ENGINES,
      targets: {},
    };

const failures = [];
for (const target of o.targets) {
  const started = new Date().toISOString();
  if (target === 'node') failures.push(...(await runNode(o, outDir)));
  else if (target === 'native') failures.push(...(await runNative(o, outDir)));
  else if (target === 'browser') {
    const { runBrowser } = await import('./browser/run.js');
    const r = await runBrowser({ ...o, outDir, log });
    manifest.browser = r.browser;
    failures.push(...r.failures);
  } else throw new Error(`unknown target ${target}`);
  manifest.targets[target] = { ...(manifest.targets[target] ?? {}), started, finished: new Date().toISOString(), runs: o.runs };
}
manifest.failures = [...(manifest.failures ?? []), ...failures];
await writeFile(manifestPath, JSON.stringify(manifest, null, 2));

const md = overview(await readRaw(outDir));
await writeFile(join(outDir, 'summary.md'), `${md}\n`);
console.log(md);
if (failures.length) console.error(`${failures.length} failed processes, see manifest.json`);
console.error(`results: ${outDir}`);
// A failed process or a different answer of an engine fails the run (CI smoke run).
if (failures.length || md.includes('- checksum mismatch:')) process.exitCode = 1;
