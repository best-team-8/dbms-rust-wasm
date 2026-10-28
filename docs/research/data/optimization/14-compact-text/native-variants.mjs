// Native workload (`flatsql bench`) of several builds of the CLI in
// alternating order, medians of chosen operations. Used to find the cause of
// native changes of operations that do not handle text.
//
//   node docs/research/data/optimization/14-compact-text/native-variants.mjs RUNS name=EXE …

import { spawnSync } from 'node:child_process';
import { readFileSync, rmSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';

const [runs, ...specs] = process.argv.slice(2);
const variants = specs.map((s) => s.split('='));
const OPS = ['insert', 'insert_many', 'point_lookup', 'range_scan', 'full_scan', 'group_by', 'top_k', 'materialize_10k', 'restore'];
const res = Object.fromEntries(variants.map(([name]) => [name, Object.fromEntries(OPS.map((o) => [o, []]))]));
for (let run = 0; run < Number(runs); run++) {
  const order = run % 2 ? [...variants].reverse() : variants;
  for (const [name, exe] of order) {
    const out = join(tmpdir(), `flatsql-variant-${name}.json`);
    const r = spawnSync(exe, ['bench', '--rows', '100000', '--json', out], { encoding: 'utf8' });
    if (r.status !== 0) throw new Error(`${name}: ${r.stderr}`);
    const ops = JSON.parse(readFileSync(out, 'utf8')).result.ops;
    for (const o of OPS) if (ops[o]) res[name][o].push(ops[o].meanUs);
    rmSync(out);
  }
}
const median = (v) => [...v].sort((a, b) => a - b)[Math.floor(v.length / 2)];
const fmt = (us) => (us >= 1000 ? `${(us / 1000).toFixed(2)} ms` : `${us.toFixed(2)} µs`);
console.log(`median of ${runs} runs per build, alternating order`);
console.log(`operation | ${variants.map(([n]) => n).join(' | ')}`);
for (const o of OPS) console.log(`${o} | ${variants.map(([n]) => (res[n][o].length ? fmt(median(res[n][o])) : '–')).join(' | ')}`);
