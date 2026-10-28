import { readFileSync } from 'node:fs';
import { WASI } from 'node:wasi';
const wasi = new WASI({ version: 'preview1', args: ['layout'] });
const { instance } = await WebAssembly.instantiate(readFileSync(process.argv[2]), wasi.getImportObject());
wasi.start(instance);
