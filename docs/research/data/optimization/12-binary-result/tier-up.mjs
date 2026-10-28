// The JSON path of the Node.js workload with the version before the binary
// result format (tag po-2.13, built by bench/build-version.js) and after it,
// 5 processes each in alternating order, medians of range_scan, group_by and
// index_lookup. V8 flags given on the command line are passed to the measured
// processes, e.g. --no-liftoff (only the optimising compiler, no tier-up).
//
//   node docs/research/data/optimization/12-binary-result/tier-up.mjs [V8 flags]

import { spawnSync } from 'node:child_process';
import { readFileSync, rmSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const R = resolve(fileURLToPath(new URL('.', import.meta.url)), '../../../../..');
const versions = { before: `${R}/bench/versions/po-2.13`, after: R };
const flags = process.argv.slice(2);
const res = { before: [], after: [] };
for (let run = 1; run <= 5; run++) {
  for (const v of run % 2 ? ['before', 'after'] : ['after', 'before']) {
    const out = `${tmpdir()}/tier-${v}.json`;
    const task = { kind: 'workload', engine: 'flatsql-json', rows: 100000, flatsqlRoot: versions[v], out, run };
    const r = spawnSync(process.execPath, [...flags, `${R}/bench/node/child.js`, JSON.stringify(task)], { encoding: 'utf8' });
    if (r.status !== 0) throw new Error(r.stderr);
    const j = JSON.parse(readFileSync(out, 'utf8'));
    const ops = j.result.ops;
    res[v].push([ops.range_scan.meanUs, ops.group_by.meanUs / 1000, ops.index_lookup.meanUs]);
    rmSync(out);
  }
}
const med = (a) => [...a].sort((x, y) => x - y)[2];
for (const v of ['before', 'after']) console.log(flags.join(' ') || '(default)', v, 'range_scan µs', med(res[v].map((x) => x[0])).toFixed(1), 'group_by ms', med(res[v].map((x) => x[1])).toFixed(2), 'index_lookup µs', med(res[v].map((x) => x[2])).toFixed(2));
