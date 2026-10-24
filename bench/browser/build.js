// Bundles the engines whose browser builds import other npm packages (no
// bundler is needed for the rest). Output: bench/browser/vendor/ (ignored by
// git, rebuilt by the runner). Minified, as an application would ship them.

import { build } from 'esbuild';
import { fileURLToPath } from 'node:url';

const here = (p) => fileURLToPath(new URL(p, import.meta.url));

export const BUNDLES = ['duckdb-blocking', 'pouchdb', 'rxdb'];

export async function buildVendor() {
  await build({
    entryPoints: Object.fromEntries(BUNDLES.map((b) => [b, here(`./entries/${b}.js`)])),
    outdir: here('./vendor'),
    bundle: true,
    format: 'esm',
    platform: 'browser',
    target: 'es2022',
    minify: true,
    define: { 'process.env.NODE_ENV': '"production"', global: 'globalThis' },
    logLevel: 'warning',
  });
}

if (process.argv[1] === fileURLToPath(import.meta.url)) await buildVendor();
