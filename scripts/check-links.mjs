// Checks relative links in the Markdown files of the repository: the target
// must be a file or directory in the repository (tracked or not ignored by
// git) and an anchor must match a heading of the target (GitHub heading ids).
// Literature PDFs and other theses are kept only locally, so links into their
// directories are not checked. External links are not fetched.
//
//   node scripts/check-links.mjs

import { execFileSync } from 'node:child_process';
import { existsSync, readFileSync } from 'node:fs';
import { dirname, join, relative, resolve, sep } from 'node:path';
import { fileURLToPath } from 'node:url';

const ROOT = fileURLToPath(new URL('..', import.meta.url));
const LOCAL_ONLY = ['docs/references/papers/'];

const repoFiles = new Set(
  execFileSync('git', ['ls-files', '--cached', '--others', '--exclude-standard'], { cwd: ROOT, encoding: 'utf8' })
    .split('\n')
    .filter((p) => p && existsSync(join(ROOT, p))),
);
const repoDirs = new Set();
for (const p of repoFiles) {
  const parts = p.split('/');
  for (let i = 1; i < parts.length; i++) repoDirs.add(parts.slice(0, i).join('/'));
}
const files = [...repoFiles].filter((p) => p.endsWith('.md'));

function stripCode(text) {
  return text.replace(/^(```|~~~)[\s\S]*?^\1/gm, '').replace(/`[^`\n]*`/g, (m) => m.replace(/[[\]()]/g, ' '));
}

function slug(heading) {
  return heading
    .replace(/\[([^\]]*)\]\([^)]*\)/g, '$1')
    .trim()
    .toLowerCase()
    .replace(/[^\p{L}\p{N}\s_-]/gu, '')
    .replace(/\s/g, '-');
}

const anchorCache = new Map();
function anchors(path) {
  if (!anchorCache.has(path)) {
    const ids = new Set();
    const seen = new Map();
    const text = readFileSync(join(ROOT, path), 'utf8').replace(/^(```|~~~)[\s\S]*?^\1/gm, '');
    for (const m of text.matchAll(/^#{1,6}\s+(.+?)\s*#*\s*$/gm)) {
      const base = slug(m[1]);
      const n = seen.get(base) ?? 0;
      seen.set(base, n + 1);
      ids.add(n ? `${base}-${n}` : base);
    }
    anchorCache.set(path, ids);
  }
  return anchorCache.get(path);
}

const problems = [];
for (const file of files) {
  const text = stripCode(readFileSync(join(ROOT, file), 'utf8'));
  for (const m of text.matchAll(/\]\(([^)\s]+)(?:\s+"[^"]*")?\)/g)) {
    const link = m[1].replace(/^<|>$/g, '');
    if (/^[a-z][a-z0-9+.-]*:/i.test(link)) continue;
    const [rawPath, rawAnchor] = link.split('#');
    const target = rawPath
      ? relative(ROOT, resolve(ROOT, dirname(file), decodeURIComponent(rawPath))).split(sep).join('/')
      : file;
    if (LOCAL_ONLY.some((dir) => `${target}/`.startsWith(dir))) continue;
    if (target !== '' && !repoFiles.has(target) && !repoDirs.has(target)) {
      problems.push(`${file}: missing ${rawPath}`);
      continue;
    }
    if (rawAnchor && repoFiles.has(target) && target.endsWith('.md')) {
      if (!anchors(target).has(decodeURIComponent(rawAnchor).toLowerCase())) {
        problems.push(`${file}: no heading #${rawAnchor} in ${rawPath || file}`);
      }
    }
  }
}

for (const p of problems) console.log(p);
console.log(`${files.length} files, ${problems.length} broken links`);
process.exit(problems.length ? 1 : 0);
