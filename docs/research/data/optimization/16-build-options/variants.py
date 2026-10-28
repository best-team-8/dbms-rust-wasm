"""Build variants of tag po-2.6 that differ only in the build of the
WebAssembly module: each is a git worktree in bench/versions/v25-<name> with
one change of the build configuration, built by bench/build-version.js.

    python docs/research/data/optimization/16-build-options/variants.py [name ...]
"""

import os
import subprocess
import sys

ROOT = os.path.abspath(os.path.join(os.path.dirname(__file__), '..', '..', '..', '..', '..'))


def edit(path, old, new):
    with open(path, encoding='utf8', newline='') as f:
        s = f.read()
    nl = '\r\n' if '\r\n' in s else '\n'
    s = s.replace('\r\n', '\n')
    assert s.count(old) == 1, (path, old)
    s = s.replace(old, new)
    with open(path, 'w', encoding='utf8', newline='') as f:
        f.write(s.replace('\n', nl))


def profile(d, opt):
    edit(os.path.join(d, 'Cargo.toml'), 'opt-level = 3', f'opt-level = {opt}')


def wasm_opt(d, flags):
    edit(os.path.join(d, 'crates', 'flatsql-wasm', 'Cargo.toml'), 'wasm-opt = ["-O3"]', f'wasm-opt = {flags}')


def talc(d):
    edit(os.path.join(d, 'crates', 'flatsql-wasm', 'Cargo.toml'), 'console_error_panic_hook = "0.1"\n',
         'console_error_panic_hook = "0.1"\n\n[target.\'cfg(target_family = "wasm")\'.dependencies]\ntalc = "5.1"\n')
    edit(os.path.join(d, 'crates', 'flatsql-wasm', 'src', 'lib.rs'), '#[wasm_bindgen(start)]',
         '#[cfg(all(not(target_feature = "atomics"), target_family = "wasm"))]\n#[global_allocator]\n'
         'static TALC: talc::wasm::WasmDynamicTalc = talc::wasm::new_wasm_dynamic_allocator();\n\n#[wasm_bindgen(start)]')


def simd(d):
    os.makedirs(os.path.join(d, '.cargo'), exist_ok=True)
    with open(os.path.join(d, '.cargo', 'config.toml'), 'w', encoding='utf8') as f:
        f.write('[target.wasm32-unknown-unknown]\nrustflags = ["-C", "target-feature=+simd128"]\n')


VARIANTS = {
    'o2': lambda d: profile(d, '2'),
    'os': lambda d: (profile(d, '"s"'), wasm_opt(d, '["-Os"]')),
    'oz': lambda d: (profile(d, '"z"'), wasm_opt(d, '["-Oz"]')),
    'nowasmopt': lambda d: wasm_opt(d, 'false'),
    'wasmopt-o4': lambda d: wasm_opt(d, '["-O4"]'),
    'talc': talc,
    'simd': simd,
}

for name in sys.argv[1:] or VARIANTS:
    d = os.path.join(ROOT, 'bench', 'versions', f'v25-{name}')
    if not os.path.exists(d):
        subprocess.run(['git', 'worktree', 'add', '--detach', d, 'po-2.6'], cwd=ROOT, check=True)
        VARIANTS[name](d)
    subprocess.run(['node', 'bench/build-version.js', 'po-2.6', '--name', f'v25-{name}'], cwd=ROOT, check=True)
    print(f'built v25-{name}', flush=True)
