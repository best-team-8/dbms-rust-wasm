// Static server of the browser benchmark. Sends COOP/COEP headers, so the
// page is cross-origin isolated: performance.now() then has 5 µs resolution
// instead of 100 µs, and performance.measureUserAgentSpecificMemory() is
// available. No benchmarked configuration uses SharedArrayBuffer or threads
// (DuckDB runs its single-threaded build, SQLite the opfs-sahpool VFS), so the
// headers change only the measuring instruments.
//
//   /bench/…          → bench/ of this checkout (harness, workload, adapters)
//   /node_modules/…   → node_modules/ of this checkout (engines)
//   /vendor/…         → bench/browser/vendor/ (bundled engines)
//   /flatsql/…        → the measured Flat-SQL checkout (bridge/, pkg/)

import { createServer } from 'node:http';
import { readFile, stat } from 'node:fs/promises';
import { extname, join, normalize, resolve, sep } from 'node:path';
import { fileURLToPath } from 'node:url';

const ROOT = fileURLToPath(new URL('../..', import.meta.url));

const MIME = {
  '.html': 'text/html; charset=utf-8',
  '.js': 'text/javascript; charset=utf-8',
  '.mjs': 'text/javascript; charset=utf-8',
  '.cjs': 'text/javascript; charset=utf-8',
  '.json': 'application/json',
  '.wasm': 'application/wasm',
  '.data': 'application/octet-stream',
  '.css': 'text/css',
  '.svg': 'image/svg+xml',
  '.gz': 'application/gzip',
};

export async function startBenchServer({ flatsqlRoot, port = 0, isolated = true } = {}) {
  const dirs = [
    ['/bench/', join(ROOT, 'bench')],
    ['/node_modules/', join(ROOT, 'node_modules')],
    ['/vendor/', join(ROOT, 'bench', 'browser', 'vendor')],
    ['/flatsql/', resolve(flatsqlRoot)],
  ];
  const server = createServer(async (req, res) => {
    const url = new URL(req.url, 'http://localhost');
    let path = decodeURIComponent(url.pathname);
    const entry = dirs.find(([p]) => path.startsWith(p));
    const headers = {
      'cache-control': 'no-store',
      ...(isolated ? { 'cross-origin-opener-policy': 'same-origin', 'cross-origin-embedder-policy': 'require-corp' } : {}),
    };
    if (!entry) {
      res.writeHead(404, headers).end('not found');
      return;
    }
    const [prefix, base] = entry;
    const file = normalize(join(base, path.slice(prefix.length)));
    if (!file.startsWith(normalize(base) + sep)) {
      res.writeHead(403, headers).end('forbidden');
      return;
    }
    try {
      if (!(await stat(file)).isFile()) throw new Error('not a file');
      const data = await readFile(file);
      res.writeHead(200, { ...headers, 'content-type': MIME[extname(file)] ?? 'application/octet-stream', 'content-length': data.length });
      res.end(data);
    } catch {
      res.writeHead(404, headers).end('not found');
    }
  });
  await new Promise((r) => server.listen(port, '127.0.0.1', r));
  const url = `http://127.0.0.1:${server.address().port}`;
  return { url, close: () => new Promise((r) => server.close(r)), filePath: (pathname) => {
    const entry = dirs.find(([p]) => pathname.startsWith(p));
    return entry ? normalize(join(entry[1], pathname.slice(entry[0].length))) : null;
  } };
}
