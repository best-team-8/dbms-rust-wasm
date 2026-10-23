// Prepares another version of Flat-SQL for measurement: a git worktree of
// `ref` in bench/versions/<name>/ with the WebAssembly package (wasm-pack,
// release) and the native CLI (cargo, release) built. The benchmark code is
// not taken from the worktree: bench/run.js of the current checkout measures
// it with --flatsql bench/versions/<name>.
//
//   node bench/build-version.js <ref> [--name NAME]
//   node bench/run.js --flatsql bench/versions/<name> --label <name> …

import { spawnSync } from 'node:child_process';
import { existsSync } from 'node:fs';
import { join } from 'node:path';
import { fileURLToPath } from 'node:url';

const ROOT = fileURLToPath(new URL('..', import.meta.url));
const args = process.argv.slice(2);
const ref = args[0];
if (!ref) {
  console.error('usage: node bench/build-version.js <ref> [--name NAME]');
  process.exit(2);
}
const name = args.includes('--name') ? args[args.indexOf('--name') + 1] : ref.replace(/[^\w.-]/g, '_');
const dir = join(ROOT, 'bench', 'versions', name);

function run(cmd, cmdArgs, cwd) {
  console.error(`$ ${cmd} ${cmdArgs.join(' ')}  (${cwd})`);
  const r = spawnSync(cmd, cmdArgs, { cwd, stdio: 'inherit', shell: process.platform === 'win32' });
  if (r.status !== 0) process.exit(r.status ?? 1);
}

if (!existsSync(dir)) run('git', ['worktree', 'add', '--detach', dir, ref], ROOT);
else console.error(`${dir} exists, rebuilding`);
run('wasm-pack', ['build', 'crates/flatsql-wasm', '--release', '--target', 'web', '--out-dir', '../../pkg', '--out-name', 'flatsql'], dir);
run('cargo', ['build', '--release', '-p', 'flatsql-cli'], dir);
console.error(`ready: node bench/run.js --flatsql ${join('bench', 'versions', name)} --label ${name}`);
