// Runs `cost-split` or `allocations` against a chosen version of the engine
// core, so the same operations are measured before and after a change.
//
//   node docs/research/data/benchmark-1/analysis/cost-split/run.mjs [--flatsql DIR] [--bin allocations] [--out FILE]
//
// DIR is a checkout of Flat-SQL (default: this repository), e.g. a worktree
// made by bench/build-version.js. The sources of this package are copied to
// DIR/target/cost-split-src with the dependency pointing at
// DIR/crates/flatsql-core and built there; the program itself always comes
// from this directory. Parts of the program that need an internal API of a
// particular version are chosen by build.rs from the core's sources.

import { spawnSync } from 'node:child_process';
import { cpSync, mkdirSync, readFileSync, writeFileSync } from 'node:fs';
import { join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const HERE = fileURLToPath(new URL('.', import.meta.url));
const REPO = resolve(HERE, '../../../../../..');
const args = process.argv.slice(2);
const opt = (name, fallback) => (args.includes(name) ? args[args.indexOf(name) + 1] : fallback);
const root = resolve(opt('--flatsql', REPO));
const bin = opt('--bin', 'cost-split');
const out = opt('--out', null);

const src = join(root, 'target', 'cost-split-src');
mkdirSync(src, { recursive: true });
cpSync(join(HERE, 'src'), join(src, 'src'), { recursive: true });
cpSync(join(HERE, 'build.rs'), join(src, 'build.rs'));
const core = join(root, 'crates', 'flatsql-core').replaceAll('\\', '/');
const manifest = readFileSync(join(HERE, 'Cargo.toml'), 'utf8').replace(/flatsql-core = \{ path = "[^"]*" \}/, `flatsql-core = { path = "${core}" }`);
writeFileSync(join(src, 'Cargo.toml'), manifest);

const r = spawnSync(
  'cargo',
  ['run', '--quiet', '--release', '--manifest-path', join(src, 'Cargo.toml'), '--target-dir', join(root, 'target', 'analysis'), '--bin', bin],
  { encoding: 'utf8', stdio: ['ignore', 'pipe', 'inherit'], env: { ...process.env, FLATSQL_CORE_DIR: core } },
);
if (r.status !== 0) process.exit(r.status ?? 1);
process.stdout.write(r.stdout);
if (out) writeFileSync(out, r.stdout);
