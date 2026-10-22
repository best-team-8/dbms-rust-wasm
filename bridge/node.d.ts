// Type declarations of the Node.js entry point (bridge/node.js).

import type { Store } from './index.js';

export * from './index.js';

/** File store compatible with the native CLI (`snapshot.fsql` + `wal.fsql`). */
export class FsStore implements Store {
  constructor(dir: string);
  readonly kind: 'fs';
  readonly dir: string;
  walSize: number;
  snapshotSize: number;
  load(): Promise<{ snapshot: Uint8Array | null; wal: Uint8Array | null; meta: Record<string, unknown> }>;
  appendWal(bytes: Uint8Array): Promise<void>;
  writeSnapshot(bytes: Uint8Array): Promise<void>;
  setMeta(key: string, value: unknown): Promise<void>;
  getMeta(key: string): Promise<unknown>;
  destroy(): Promise<void>;
}
