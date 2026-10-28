// Identifies a version of the engine code, also before it is committed:
// SHA-256 of the lines "<blob> <path>" of every file of crates/, bridge/ and
// server/, sorted by path. Without a ref the files of the working tree are
// hashed as git would store them (tracked and new files); with a ref, the
// files of that commit. A change measured before its commit and the commit
// itself give the same value.
//
//   node bench/code-tree.js [REF] [--root DIR]

import { spawnSync } from 'node:child_process';
import { createHash } from 'node:crypto';
import { fileURLToPath } from 'node:url';

const DIRS = ['crates', 'bridge', 'server'];

export function codeTree(root, ref = null) {
  const git = (args, input) => spawnSync('git', args, { cwd: root, encoding: 'utf8', input });
  let entries;
  if (ref) {
    const r = git(['ls-tree', '-r', '--format=%(objectname) %(path)', ref, '--', ...DIRS]);
    if (r.status !== 0) return null;
    entries = r.stdout.split('\n').filter(Boolean).map((l) => [l.slice(l.indexOf(' ') + 1), l.slice(0, l.indexOf(' '))]);
  } else {
    const list = git(['ls-files', '-co', '--exclude-standard', '--', ...DIRS]);
    if (list.status !== 0) return null;
    const paths = list.stdout.split('\n').filter(Boolean);
    const blobs = git(['hash-object', '--stdin-paths'], `${paths.join('\n')}\n`);
    if (blobs.status !== 0) return null;
    entries = blobs.stdout.split('\n').filter(Boolean).map((blob, i) => [paths[i], blob]);
  }
  entries.sort((a, b) => (a[0] < b[0] ? -1 : a[0] > b[0] ? 1 : 0));
  return createHash('sha256').update(entries.map(([path, blob]) => `${blob} ${path}\n`).join('')).digest('hex');
}

if (process.argv[1] && fileURLToPath(import.meta.url) === process.argv[1]) {
  const args = process.argv.slice(2);
  const root = args.includes('--root') ? args[args.indexOf('--root') + 1] : process.cwd();
  const ref = args.find((a, i) => !a.startsWith('--') && args[i - 1] !== '--root') ?? null;
  console.log(codeTree(root, ref));
}
