// Type declarations of the Node.js entry point (bridge/node.js).

import type { Store } from './index.js';

export * from './index.js';

/**
 * File store compatible with the native CLI (`snapshot.fsql` + `wal.fsql`).
 * Every WAL append is fsynced unless `fsync: false`; checkpoints (snapshot
 * file, rename, directory, WAL truncation) are synced either way.
 */
export class FsStore implements Store {
  constructor(dir: string, options?: { fsync?: boolean });
  readonly kind: 'fs';
  readonly dir: string;
  /** fsync after every WAL append and meta.json update (default true). */
  readonly fsync: boolean;
  walSize: number;
  snapshotSize: number;
  load(): Promise<{ snapshot: Uint8Array | null; wal: Uint8Array | null; meta: Record<string, unknown> }>;
  appendWal(bytes: Uint8Array): Promise<void>;
  writeSnapshot(bytes: Uint8Array): Promise<void>;
  setMeta(key: string, value: unknown): Promise<void>;
  getMeta(key: string): Promise<unknown>;
  destroy(): Promise<void>;
}
