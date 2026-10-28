// One table of the build variants measured against tag po-2.6 (each in its
// own directory made by bench/compare.js): module size and the change of the
// median of every operation of the Node.js workload (objects path), with *
// where the 95% confidence intervals do not overlap.
//
//   node docs/research/data/optimization/16-build-options/summary.mjs [--target node|browser] > summary.md

import { readdirSync, readFileSync, existsSync } from 'node:fs';
import { join } from 'node:path';
import { fileURLToPath } from 'node:url';

const HERE = fileURLToPath(new URL('.', import.meta.url));
const target = process.argv.includes('--target') ? process.argv[process.argv.indexOf('--target') + 1] : 'node';
const OPS = ['insert', 'insert_many', 'point_lookup', 'index_lookup', 'range_scan', 'full_scan', 'group_by', 'top_k', 'materialize_10k', 'update_10pct', 'delete_10pct', 'snapshot', 'restore', 'wasm_after_load (rozmiar)'];
const rows = [];
for (const dir of readdirSync(HERE, { withFileTypes: true }).filter((d) => d.isDirectory()).map((d) => d.name).sort()) {
  const base = target === 'node' ? join(HERE, dir) : join(HERE, dir, target);
  const md = join(base, 'compare.md');
  if (!existsSync(md)) continue;
  const text = readFileSync(md, 'utf8');
  const manifest = JSON.parse(readFileSync(join(target === 'node' ? base : join(HERE, dir), 'manifest.json'), 'utf8'));
  const size = manifest.after.module;
  // The objects path of the workload and the memory section.
  const sections = text.split('\n### ').filter((s) => s.startsWith(`${target} – workload – flatsql\n`) || s.startsWith(`${target} – memory – flatsql\n`));
  const cells = Object.fromEntries(OPS.map((op) => [op, '']));
  for (const s of sections) {
    for (const line of s.split('\n')) {
      const m = line.match(/^\| ([^|]+) \| [^|]+ \| [^|]+ \| ([^|]+) \| ([^|]+) \|$/);
      if (m && m[1].trim() in cells) cells[m[1].trim()] = `${m[2].trim()}${m[3].startsWith('tak') ? '*' : ''}`;
    }
  }
  rows.push({ dir, size, base: manifest.before.module, cells });
}
console.log(`| Wariant | Moduł (B) | gzip (B) | ${OPS.join(' | ')} |`);
console.log(`|---|---:|---:|${OPS.map(() => '---:').join('|')}|`);
if (rows.length) console.log(`| po-2.6 | ${rows[0].base.bytes} | ${rows[0].base.gzipBytes} | ${OPS.map(() => '').join(' | ')} |`);
for (const r of rows) console.log(`| ${r.dir} | ${r.size.bytes} | ${r.size.gzipBytes} | ${OPS.map((o) => r.cells[o]).join(' | ')} |`);
