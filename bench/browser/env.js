// Environment of the adapters in the browser (main thread and Worker).
// Paths are served by bench/browser/server.js.

export function browserEnv(kind = 'browser') {
  const base = globalThis.location.origin;
  return {
    kind,
    async flatsql() {
      const m = await import(`${base}/flatsql/bridge/index.js`);
      await m.initEngine(`${base}/flatsql/pkg/flatsql_bg.wasm`);
      return m;
    },
    importPkg(_nodeSpec, path) {
      return path.startsWith('vendor:') ? import(`${base}/vendor/${path.slice(7)}.js`) : import(`${base}/node_modules/${path}`);
    },
    script(path) {
      return new Promise((resolve, reject) => {
        const s = document.createElement('script');
        s.src = `${base}/node_modules/${path}`;
        s.onload = resolve;
        s.onerror = () => reject(new Error(`cannot load ${path}`));
        document.head.append(s);
      });
    },
    url: (path) => `${base}/node_modules/${path}`,
  };
}
