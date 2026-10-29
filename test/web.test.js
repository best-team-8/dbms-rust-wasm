// Demo app (web/): the Service Worker must cache every file the page needs
// to start without the server. Checked in Node.js by running web/sw.js with
// stand-ins for the cache and the network that read files the way the server
// serves them.

import { test } from 'node:test';
import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import { join } from 'node:path';
import { fileURLToPath } from 'node:url';
import vm from 'node:vm';

const ROOT = fileURLToPath(new URL('..', import.meta.url));
const ORIGIN = 'http://demo.test';

/** File behind a URL path, mapped as in server/server.js. */
function fileOf(path) {
  if (path === '/') path = '/index.html';
  for (const prefix of ['/pkg/', '/bridge/']) {
    if (path.startsWith(prefix)) return join(ROOT, prefix, path.slice(prefix.length));
  }
  return join(ROOT, 'web', path);
}

async function serve(url) {
  try {
    return new Response(await readFile(fileOf(new URL(url).pathname)));
  } catch {
    return new Response('not found', { status: 404 });
  }
}

/** Runs web/sw.js and its install step; returns the cached URL paths. */
async function installServiceWorker() {
  const handlers = {};
  const cached = new Set();
  const resolve = (url) => new URL(url, `${ORIGIN}/sw.js`).href;
  const cache = {
    async addAll(urls) {
      for (const url of urls) {
        const response = await serve(resolve(url));
        if (!response.ok) throw new TypeError(`addAll: ${url} gives ${response.status}`);
        cached.add(new URL(resolve(url)).pathname);
      }
    },
    async put(request) {
      cached.add(new URL(resolve(request.url ?? request)).pathname);
    },
  };
  const self = {
    location: new URL(`${ORIGIN}/sw.js`),
    addEventListener: (type, handler) => (handlers[type] = handler),
    skipWaiting: async () => {},
    clients: { claim: async () => {} },
  };
  const context = {
    self,
    caches: { open: async () => cache, keys: async () => [], delete: async () => true },
    fetch: (url) => serve(resolve(url)),
    URL,
    Response,
  };
  vm.runInNewContext(await readFile(join(ROOT, 'web', 'sw.js'), 'utf8'), context);
  let installed;
  handlers.install({ waitUntil: (promise) => (installed = promise) });
  await installed;
  return cached;
}

/** URL paths the page loads: assets of index.html and the static import graph from app.js. */
async function pageFiles() {
  const files = new Set(['/']);
  const html = await readFile(fileOf('/index.html'), 'utf8');
  const queue = [];
  for (const [, ref] of html.matchAll(/(?:src|href)="([^"]+)"/g)) {
    const path = new URL(ref, `${ORIGIN}/`).pathname;
    files.add(path);
    if (path.endsWith('.js')) queue.push(path);
  }
  while (queue.length > 0) {
    const path = queue.shift();
    const source = await readFile(fileOf(path), 'utf8');
    const refs = [
      ...source.matchAll(/^\s*(?:import|export)\b[^;]*?\bfrom\s*["']([^"']+)["']/gm),
      ...source.matchAll(/^\s*import\s*["']([^"']+)["']/gm),
      ...source.matchAll(/new URL\(\s*["']([^"']+)["']\s*,\s*import\.meta\.url\s*\)/g),
    ];
    for (const [, ref] of refs) {
      const target = new URL(ref, `${ORIGIN}${path}`).pathname;
      if (files.has(target)) continue;
      files.add(target);
      if (target.endsWith('.js')) queue.push(target);
    }
  }
  return files;
}

test('the Service Worker caches at install every file the page loads, including wasm-bindgen snippets', async () => {
  const cached = await installServiceWorker();
  const needed = await pageFiles();
  assert.ok([...needed].some((p) => p.startsWith('/pkg/snippets/')), 'the module imports snippets');
  assert.ok(needed.has('/pkg/flatsql_bg.wasm'));
  const missing = [...needed].filter((p) => !cached.has(p));
  assert.deepEqual(missing, [], 'files the page loads but the Service Worker does not cache');
});
