// Browser target of bench/run.js: every measured process is a fresh browser
// (new profile: empty HTTP cache, no compiled-code cache, empty IndexedDB and
// OPFS), started through playwright-core with an installed browser (Edge by
// default, any Chromium with --browser-exe). The page is bench/browser/page.js.

import { existsSync } from 'node:fs';
import { mkdir, mkdtemp, readFile, rm, writeFile } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { gzipSync } from 'node:zlib';
import { pathToFileURL } from 'node:url';

import { ENGINES } from '../engines/index.js';
import { Rng } from '../suite.js';
import { buildVendor } from './build.js';
import { startBenchServer } from './server.js';

export const BROWSER_KINDS = ['workload', 'cold', 'memory', 'checkpoint', 'network'];
const CHECKPOINT_ENGINES = ['flatsql-idb', 'flatsql-idb-strict', 'sqljs-idb'];
const CHECKPOINT_SIZES = [10_000, 50_000, 100_000];

function shuffled(list, seed) {
  const rng = new Rng(seed);
  const a = [...list];
  for (let i = a.length - 1; i > 0; i--) {
    const j = rng.below(i + 1);
    [a[i], a[j]] = [a[j], a[i]];
  }
  return a;
}

const pad = (n) => String(n).padStart(2, '0');

function enginesFor(kind, selected) {
  const all = Object.keys(ENGINES).filter((id) => ENGINES[id].where.includes('browser') || ENGINES[id].where.includes('worker'));
  const list = selected ? all.filter((id) => selected.includes(id)) : all;
  if (kind === 'checkpoint') return list.filter((id) => CHECKPOINT_ENGINES.includes(id));
  if (kind === 'network') return ['flatsql'];
  if (kind === 'memory') return list.filter((id) => ENGINES[id].group === 'memory');
  return list;
}

/** Raw and gzip size of every file the page loaded for one engine (cold start). */
async function moduleSizes(resources, server) {
  const files = [];
  for (const r of resources) {
    if (r.name.startsWith('/bench/')) continue; // the harness, not the engine
    const path = server.filePath(r.name);
    if (!path || !existsSync(path)) continue;
    const bytes = await readFile(path);
    files.push({ name: r.name, bytes: bytes.length, gzipBytes: gzipSync(bytes, { level: 9 }).length });
  }
  return { files, bytes: files.reduce((a, f) => a + f.bytes, 0), gzipBytes: files.reduce((a, f) => a + f.gzipBytes, 0) };
}

export async function runBrowser(o) {
  const { chromium } = await import('playwright-core');
  await buildVendor();
  const server = await startBenchServer({ flatsqlRoot: o.flatsql });
  const { createServer } = await import(pathToFileURL(join(o.flatsql, 'server', 'server.js')).href);
  const sync = await createServer({ port: 0, quiet: true });
  const launchOptions = { headless: !o.headed, ...(o.browserExe ? { executablePath: o.browserExe } : { channel: o.browserChannel }) };
  const failures = [];
  let browserInfo = null;
  const kinds = (o.kinds ?? BROWSER_KINDS).filter((k) => BROWSER_KINDS.includes(k));
  try {
    for (let run = 1; run <= o.runs; run++) {
      for (const kind of kinds) {
        for (const engine of shuffled(enginesFor(kind, o.engines), run * 31 + kinds.indexOf(kind))) {
          const dir = join(o.outDir, 'raw', 'browser', kind, engine);
          await mkdir(dir, { recursive: true });
          const out = join(dir, `${pad(run)}.json`);
          if (existsSync(out)) continue;
          const task = { kind, engine, rows: o.rows, run, sizes: CHECKPOINT_SIZES, syncUrl: sync.url };
          o.log(`browser ${kind} ${engine} run ${run}/${o.runs}`);
          const profile = await mkdtemp(join(tmpdir(), 'flatsql-browser-'));
          let context;
          try {
            context = await chromium.launchPersistentContext(profile, launchOptions);
            browserInfo ??= { name: o.browserExe ?? o.browserChannel, version: context.browser()?.version() ?? null };
            const page = context.pages()[0] ?? (await context.newPage());
            const errors = [];
            page.on('pageerror', (e) => errors.push(String(e)));
            await page.goto(`${server.url}/bench/browser/index.html?task=${encodeURIComponent(JSON.stringify(task))}`);
            await page.waitForFunction(() => window.__bench !== undefined, null, { timeout: 45 * 60_000, polling: 500 });
            const r = await page.evaluate(() => window.__bench);
            if (!browserInfo.version) browserInfo.version = await page.evaluate(() => navigator.userAgent);
            if (!r.ok) throw new Error(r.error);
            if (kind === 'cold') r.result.modules = await moduleSizes(r.result.resources, server);
            await writeFile(out, JSON.stringify({ ...r, started: new Date().toISOString(), browser: browserInfo, errors }));
          } catch (err) {
            failures.push({ target: 'browser', kind, engine, run, error: String(err?.stack ?? err).slice(0, 4000) });
            o.log(`  FAILED: ${String(err?.message ?? err).split('\n')[0]}`);
          } finally {
            await context?.close().catch(() => {});
            await rm(profile, { recursive: true, force: true }).catch(() => {});
          }
        }
      }
    }
  } finally {
    await sync.close();
    await server.close();
  }
  return { failures, browser: browserInfo };
}

