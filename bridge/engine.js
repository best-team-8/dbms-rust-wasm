// Loads the WebAssembly module exactly once.

import init from '../pkg/flatsql.js';

let ready = null;

/**
 * Instantiates the WebAssembly module. `source` may be a URL, a Response, raw
 * bytes or a compiled WebAssembly.Module; by default the browser fetches
 * `pkg/flatsql_bg.wasm` next to the generated glue code.
 */
export function initEngine(source) {
  if (!ready) {
    ready = init(source === undefined ? undefined : { module_or_path: source }).catch((err) => {
      ready = null;
      throw err;
    });
  }
  return ready;
}

export function engineReady() {
  return ready;
}
