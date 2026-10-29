// Service Worker: keeps the application shell (HTML, JS, the .wasm engine)
// available offline. Data lives in IndexedDB, so with this cache the whole
// app starts and works without any network connection.
//
// Strategy: network first (always fresh while the server is reachable),
// cache as fallback. Sync and API requests are never cached.

const CACHE = 'flatsql-shell-v1';
const SHELL = [
  './',
  'index.html',
  'app.js',
  'styles.css',
  'icon.svg',
  'bridge/index.js',
  'bridge/engine.js',
  'bridge/flatdb.js',
  'bridge/sync.js',
  'bridge/stores/memory.js',
  'bridge/stores/indexeddb.js',
  'pkg/flatsql.js',
  'pkg/flatsql_bg.wasm',
];

// wasm-bindgen writes the JavaScript of the bindings to pkg/snippets/ under a
// directory named by a hash, imported by pkg/flatsql.js, so the files are
// found by reading its imports. They must be cached at install: the first
// visit is not controlled by this worker, so the fetch handler never sees
// them, and without them the engine does not load offline.
async function snippets() {
  const glue = await (await fetch('pkg/flatsql.js')).text();
  const paths = [...glue.matchAll(/["']\.\/(snippets\/[^"']+)["']/g)].map((m) => `pkg/${m[1]}`);
  return [...new Set(paths)];
}

self.addEventListener('install', (event) => {
  event.waitUntil(
    caches
      .open(CACHE)
      .then(async (cache) => cache.addAll([...SHELL, ...(await snippets())]))
      .then(() => self.skipWaiting()),
  );
});

self.addEventListener('activate', (event) => {
  event.waitUntil(
    caches
      .keys()
      .then((keys) => Promise.all(keys.filter((k) => k !== CACHE).map((k) => caches.delete(k))))
      .then(() => self.clients.claim()),
  );
});

self.addEventListener('fetch', (event) => {
  const url = new URL(event.request.url);
  if (event.request.method !== 'GET' || url.origin !== self.location.origin) return;
  if (url.pathname.startsWith('/api/') || url.pathname.startsWith('/sync/')) return;

  event.respondWith(
    fetch(event.request)
      .then((response) => {
        if (response.ok) {
          const copy = response.clone();
          caches.open(CACHE).then((cache) => cache.put(event.request, copy));
        }
        return response;
      })
      .catch(async () => {
        const cached = await caches.match(event.request, { ignoreSearch: true });
        if (cached) return cached;
        if (event.request.mode === 'navigate') return caches.match('index.html');
        return Response.error();
      }),
  );
});
