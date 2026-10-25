// Flat-SQL sync / edge server.
//
// The server runs the very same WebAssembly engine as the browser (edge
// deployment mode) and plays three roles:
//
//   * sync hub   – POST /sync/push, GET /sync/pull (binary change batches)
//   * cloud API  – POST /api/query executes SQL remotely; used as the
//                  "Cloud-First" baseline in benchmarks (optional ?latency=ms)
//   * web server – serves the demo app (web/), the bridge and the wasm pkg
//
// Usage: node server/server.js [--port 8080] [--host 127.0.0.1]
//                              [--data server/data] [--latency 0]

import { createServer as createHttpServer } from 'node:http';
import { readFile, stat } from 'node:fs/promises';
import { extname, join, normalize, resolve, sep } from 'node:path';
import { fileURLToPath } from 'node:url';
import { randomBytes } from 'node:crypto';

import { FlatDB, FsStore, MemoryStore } from '../bridge/node.js';

const ROOT = resolve(fileURLToPath(new URL('..', import.meta.url)));
const MIME = {
  '.html': 'text/html; charset=utf-8',
  '.js': 'text/javascript; charset=utf-8',
  '.mjs': 'text/javascript; charset=utf-8',
  '.css': 'text/css; charset=utf-8',
  '.json': 'application/json; charset=utf-8',
  '.webmanifest': 'application/manifest+json',
  '.wasm': 'application/wasm',
  '.svg': 'image/svg+xml',
  '.ts': 'text/plain; charset=utf-8',
};
const MAX_BODY = 64 << 20;
const COMPACT_AFTER = 2000;

const sleep = (ms) => new Promise((r) => setTimeout(r, ms));

// Simulated network latency. setTimeout alone is too coarse on Windows
// (~15.6 ms ticks turn 20 ms into 31 ms), so the last part is awaited by
// yielding to the event loop until the deadline.
async function simulateLatency(ms) {
  const deadline = performance.now() + ms;
  if (ms > 20) await sleep(ms - 17);
  while (performance.now() < deadline) await new Promise((r) => setImmediate(r));
}

function concat(chunks) {
  const size = chunks.reduce((n, c) => n + c.length, 0);
  const out = new Uint8Array(size);
  let off = 0;
  for (const c of chunks) {
    out.set(c, off);
    off += c.length;
  }
  return out;
}

async function readBody(req) {
  const chunks = [];
  let size = 0;
  for await (const chunk of req) {
    size += chunk.length;
    if (size > MAX_BODY) throw Object.assign(new Error('request body too large'), { status: 413 });
    chunks.push(chunk);
  }
  return concat(chunks);
}

function send(res, status, body, headers = {}) {
  res.writeHead(status, {
    'access-control-allow-origin': '*',
    'access-control-expose-headers': 'x-sync-cursor, x-sync-epoch, x-server-ms, x-server-time',
    ...headers,
  });
  res.end(body);
}

function json(res, status, value, headers = {}) {
  send(res, status, JSON.stringify(value), { 'content-type': MIME['.json'], ...headers });
}

export async function createServer(options = {}) {
  const { port = 8080, host = '127.0.0.1', dataDir = null, latency = 0, quiet = false } = options;
  const store = dataDir ? new FsStore(dataDir) : new MemoryStore();
  const db = await FlatDB.open({ store, nodeId: 1, checkpointBytes: 8 << 20 });
  // Writes made through /api/query are published through the outbox.
  db.enableSync();

  // Changes accepted by the server, in arrival order. Each entry is an
  // already encoded batch, so a pull is a plain concatenation.
  const epoch = randomBytes(6).toString('hex');
  let seq = 0;
  let log = [];
  const stats = { pushes: 0, pulls: 0, queries: 0, applied: 0 };

  function compact() {
    seq += 1;
    log = [{ seq, origin: 'state', bytes: db.exportState() }];
  }
  if (db.tableNames().length > 0) compact();

  // Writes made through /api/query enter the server's own outbox; move them
  // into the log so clients receive them.
  function publishLocalWrites() {
    if (db.outboxLength() === 0) return;
    const batch = db.exportOutbox();
    seq += 1;
    log.push({ seq, origin: 'server', bytes: batch.bytes });
    db.ackOutbox(batch.lastSeq);
    batch.free();
  }

  const log_ = (...args) => {
    if (!quiet) console.log(new Date().toISOString().slice(11, 23), ...args);
  };

  async function handleApi(req, res, url) {
    const delay = Math.min(Number(url.searchParams.get('latency') ?? latency) || 0, 5000);
    if (delay > 0) await simulateLatency(delay);
    const t0 = performance.now();

    if (url.pathname === '/api/status' && req.method === 'GET') {
      return json(res, 200, { epoch, seq, logEntries: log.length, stats, db: db.stats() });
    }

    if (url.pathname === '/api/query' && req.method === 'POST') {
      const sql = new TextDecoder().decode(await readBody(req));
      stats.queries++;
      try {
        const result = db.execJson(sql);
        if (url.searchParams.get('publish') === '0') {
          // Benchmark data: keep it out of the sync log.
          const batch = db.exportOutbox();
          db.ackOutbox(batch.lastSeq);
          batch.free();
        } else {
          publishLocalWrites();
        }
        await db.flush();
        return send(res, 200, result, {
          'content-type': MIME['.json'],
          'x-server-ms': (performance.now() - t0).toFixed(3),
        });
      } catch (err) {
        return json(res, 400, { error: err.message, kind: err.kind ?? 'error' });
      }
    }

    // The server clock is the reference: a client whose clock differs by
    // more than maxDriftMs() pauses synchronisation (see bridge/sync.js).
    if (url.pathname === '/sync/time' && req.method === 'GET') {
      return json(res, 200, { serverTime: Date.now() }, { 'x-server-time': String(Date.now()) });
    }

    if (url.pathname === '/sync/push' && req.method === 'POST') {
      const client = url.searchParams.get('client') ?? 'unknown';
      const body = await readBody(req);
      stats.pushes++;
      let report;
      try {
        report = db.applyChanges(body);
      } catch (err) {
        return json(res, 400, { error: err.message, kind: err.kind ?? 'error' });
      }
      if (report.applied > 0) {
        seq += 1;
        log.push({ seq, origin: client, bytes: report.accepted });
        stats.applied += report.applied;
        if (log.length > COMPACT_AFTER) compact();
      }
      await db.flush();
      log_(`push  client=${client} applied=${report.applied} ignored=${report.ignored} rejected=${report.rejected}`);
      for (const c of report.conflicts) log_(`schema conflict client=${client} table=${c.table} local=${c.local} remote=${c.remote}`);
      return json(
        res,
        200,
        {
          applied: report.applied,
          ignored: report.ignored,
          rejected: report.rejected,
          clockDrift: report.clockDrift,
          conflicts: report.conflicts,
          errors: report.errors,
          seq,
          serverTime: Date.now(),
        },
        { 'x-server-time': String(Date.now()) },
      );
    }

    if (url.pathname === '/sync/pull' && req.method === 'GET') {
      const client = url.searchParams.get('client') ?? '';
      const since = Number(url.searchParams.get('since') ?? 0);
      const clientEpoch = url.searchParams.get('epoch') ?? '';
      stats.pulls++;
      let body;
      const firstSeq = log.length ? log[0].seq : seq + 1;
      if (clientEpoch !== epoch || since === 0 || since < firstSeq - 1 || since > seq) {
        // Unknown cursor (new client, server restarted, log compacted): full state.
        body = db.exportState();
      } else {
        body = concat(log.filter((e) => e.seq > since && e.origin !== client).map((e) => e.bytes));
      }
      return send(res, 200, body, {
        'content-type': 'application/octet-stream',
        'x-sync-cursor': String(seq),
        'x-sync-epoch': epoch,
        'x-server-time': String(Date.now()),
      });
    }

    return json(res, 404, { error: 'not found' });
  }

  // URL prefix -> directory. Everything else is served from web/.
  const STATIC_DIRS = [
    ['/pkg/', join(ROOT, 'pkg')],
    ['/bridge/', join(ROOT, 'bridge')],
  ];

  async function handleStatic(req, res, url) {
    let path = decodeURIComponent(url.pathname);
    if (path === '/') path = '/index.html';
    const [prefix, base] = STATIC_DIRS.find(([p]) => path.startsWith(p)) ?? ['/', join(ROOT, 'web')];
    const file = normalize(join(base, path.slice(prefix.length)));
    // The resolved file must stay inside the directory mapped to the prefix.
    if (!file.startsWith(base + sep)) return send(res, 403, 'forbidden');
    try {
      const info = await stat(file);
      if (!info.isFile()) return send(res, 404, 'not found');
      const data = await readFile(file);
      return send(res, 200, req.method === 'HEAD' ? '' : data, {
        'content-type': MIME[extname(file)] ?? 'application/octet-stream',
        'cache-control': 'no-cache',
      });
    } catch {
      return send(res, 404, 'not found');
    }
  }

  const server = createHttpServer(async (req, res) => {
    const url = new URL(req.url, 'http://localhost');
    try {
      if (req.method === 'OPTIONS') {
        return send(res, 204, '', {
          'access-control-allow-methods': 'GET, POST, OPTIONS',
          'access-control-allow-headers': 'content-type',
        });
      }
      if (url.pathname.startsWith('/api/') || url.pathname.startsWith('/sync/')) {
        return await handleApi(req, res, url);
      }
      if (req.method === 'GET' || req.method === 'HEAD') return await handleStatic(req, res, url);
      return send(res, 405, 'method not allowed');
    } catch (err) {
      log_('error', err);
      if (!res.headersSent) json(res, err.status ?? 500, { error: String(err.message ?? err) });
    }
  });

  await new Promise((resolveListen, reject) => {
    server.once('error', reject);
    server.listen(port, host, resolveListen);
  });
  const address = server.address();
  const url = `http://${host === '0.0.0.0' ? 'localhost' : host}:${address.port}`;

  return {
    url,
    server,
    db,
    epoch,
    get seq() {
      return seq;
    },
    async close() {
      await new Promise((r) => server.close(r));
      server.closeAllConnections?.();
      await db.close();
    },
  };
}

function parseArgs(argv) {
  const out = {};
  for (let i = 0; i < argv.length; i++) {
    const [key, inline] = argv[i].replace(/^--/, '').split('=');
    const value = inline ?? argv[++i];
    if (key === 'port') out.port = Number(value);
    else if (key === 'host') out.host = value;
    else if (key === 'data') out.dataDir = value;
    else if (key === 'latency') out.latency = Number(value);
    else throw new Error(`unknown option --${key}`);
  }
  return out;
}

if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  const options = parseArgs(process.argv.slice(2));
  const srv = await createServer(options);
  console.log(`Flat-SQL server listening on ${srv.url}`);
  console.log(`  demo app     ${srv.url}/`);
  console.log(`  sync         ${srv.url}/sync/push | /sync/pull`);
  console.log(`  cloud API    ${srv.url}/api/query${options.latency ? ` (+${options.latency} ms latency)` : ''}`);
  console.log(`  storage      ${options.dataDir ? options.dataDir : 'memory (use --data <dir> to persist)'}`);
  const shutdown = async () => {
    await srv.close();
    process.exit(0);
  };
  process.on('SIGINT', shutdown);
  process.on('SIGTERM', shutdown);
}
