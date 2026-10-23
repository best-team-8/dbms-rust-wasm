// Size of WebAssembly linear memory, for any engine. Engines do not expose
// their memories in a uniform way (Emscripten keeps them in a closure), so the
// probe wraps the WebAssembly API before any engine is loaded and remembers
// every memory that is created or exported. Imports are not inspected:
// Emscripten's dynamic linker (PGlite) resolves symbols lazily through a proxy
// on the import object, and enumerating it breaks loading; an imported memory
// is created with the Memory constructor anyway. Linear memory only grows
// and is never returned to the system [W11], so its size after loading the
// data is the memory the engine holds from then on. Memories are held
// weakly: a temporary instance (PGlite runs initdb in one) is not counted once
// it has been garbage collected.

export function installWasmMemoryProbe(WA = globalThis.WebAssembly) {
  const refs = [];
  const seen = new WeakSet();
  const add = (m) => {
    if (seen.has(m)) return;
    seen.add(m);
    refs.push(new WeakRef(m));
  };
  const collect = (instance) => {
    for (const v of Object.values(instance?.exports ?? {})) if (v instanceof WA.Memory) add(v);
  };
  const OrigMemory = WA.Memory;
  WA.Memory = new Proxy(OrigMemory, {
    construct(target, args) {
      const m = Reflect.construct(target, args);
      add(m);
      return m;
    },
  });
  const OrigInstance = WA.Instance;
  WA.Instance = new Proxy(OrigInstance, {
    construct(target, args) {
      const inst = Reflect.construct(target, args);
      collect(inst);
      return inst;
    },
  });
  const instantiate = WA.instantiate;
  WA.instantiate = async function (source, imports) {
    const r = await instantiate.call(this, source, imports);
    collect(r.instance ?? r);
    return r;
  };
  if (WA.instantiateStreaming) {
    const streaming = WA.instantiateStreaming;
    WA.instantiateStreaming = async function (source, imports) {
      const r = await streaming.call(this, source, imports);
      collect(r.instance);
      return r;
    };
  }
  return () => {
    let bytes = 0;
    let alive = 0;
    for (const ref of refs) {
      const m = ref.deref();
      if (!m) continue;
      bytes += m.buffer.byteLength;
      alive++;
    }
    return { wasmBytes: bytes, memories: alive };
  };
}
