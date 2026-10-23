// Type declarations of the WASM-Bridge (bridge/index.js, bridge/node.js).
// The low-level engine classes are declared by wasm-bindgen in pkg/flatsql.d.ts.

import type { Database, PreparedStatement, OutboxBatch } from '../pkg/flatsql.js';

export { Database, PreparedStatement, OutboxBatch, describeChanges, version } from '../pkg/flatsql.js';

/** A SQL value as seen from JavaScript (INT and FLOAT are numbers). */
export type SqlValue = null | boolean | number | string;

export interface QueryResult {
  columns: string[];
  rows: SqlValue[][];
  rowsAffected: number;
  rowsScanned: number;
  /** Command tag, e.g. "SELECT 3", "INSERT 2", "CREATE TABLE". */
  message: string;
}

/** Errors thrown by the engine carry a category. */
export interface FlatSqlError extends Error {
  kind: 'lex' | 'parse' | 'schema' | 'constraint' | 'type' | 'eval' | 'transaction' | 'storage' | 'unsupported';
}

export interface RecoveryReport {
  snapshotBytes: number;
  snapshotLsn: number;
  walBytes: number;
  framesReplayed: number;
  framesSkipped: number;
  tornBytes: number;
  opsFailed: number;
  stopReason: string | null;
  /** Time spent restoring, in milliseconds. */
  ms: number;
}

export interface ApplyReport {
  applied: number;
  ignored: number;
  rejected: number;
  errors: string[];
  /** Batch with only the changes that took effect. */
  accepted: Uint8Array;
}

/** Durable home of a database: one snapshot plus an append-only WAL. */
export interface Store {
  readonly kind: string;
  walSize: number;
  snapshotSize: number;
  load(): Promise<{ snapshot: Uint8Array | null; wal: Uint8Array | null; meta: Record<string, unknown> }>;
  appendWal(bytes: Uint8Array): Promise<void>;
  /** Atomically replaces the snapshot and clears the WAL. */
  writeSnapshot(bytes: Uint8Array): Promise<void>;
  setMeta(key: string, value: unknown): Promise<void>;
  getMeta(key: string): Promise<unknown>;
  close?(): Promise<void>;
  destroy(): Promise<void>;
}

export class MemoryStore implements Store {
  readonly kind: 'memory';
  walSize: number;
  snapshotSize: number;
  load(): Promise<{ snapshot: Uint8Array | null; wal: Uint8Array | null; meta: Record<string, unknown> }>;
  appendWal(bytes: Uint8Array): Promise<void>;
  writeSnapshot(bytes: Uint8Array): Promise<void>;
  setMeta(key: string, value: unknown): Promise<void>;
  getMeta(key: string): Promise<unknown>;
  destroy(): Promise<void>;
}

/**
 * Durability of IndexedDB transactions: 'strict' – `complete` (and so
 * `flush()`) after the data reached the disk, survives a system crash or
 * power loss; 'relaxed' – after the data was handed to the operating system,
 * survives closing the tab or a browser crash; 'default' – the browser
 * decides (relaxed in Chrome 121+).
 */
export type Durability = 'default' | 'strict' | 'relaxed';

export class IndexedDbStore implements Store {
  constructor(name?: string, options?: { durability?: Durability });
  readonly kind: 'indexeddb';
  readonly name: string;
  durability: Durability;
  walSize: number;
  snapshotSize: number;
  load(): Promise<{ snapshot: Uint8Array | null; wal: Uint8Array | null; meta: Record<string, unknown> }>;
  appendWal(bytes: Uint8Array): Promise<void>;
  writeSnapshot(bytes: Uint8Array): Promise<void>;
  setMeta(key: string, value: unknown): Promise<void>;
  getMeta(key: string): Promise<unknown>;
  close(): Promise<void>;
  destroy(): Promise<void>;
}

export interface FlatDBOptions {
  /**
   * Where the database lives. Default in the browser: `new IndexedDbStore(name)`.
   * Outside the browser there is no default: pass `MemoryStore` or `FsStore`
   * (bridge/node.js), otherwise `open` rejects.
   */
  store?: Store;
  /** Name of the default IndexedDB store (default 'default'); not allowed with `store`. */
  name?: string;
  /** Replica id used in sync stamps (generated and remembered by default). */
  nodeId?: number;
  /** WAL size that triggers a checkpoint (default 1 MiB). */
  checkpointBytes?: number;
  /** 'auto' (default) flushes the WAL on the next microtask after a write. */
  flush?: 'auto' | 'manual';
  /** Durability of IndexedDB writes (only for IndexedDbStore; default: 'default'). */
  durability?: Durability;
}

export interface DbStats {
  nodeId: number;
  tables: number;
  rows: number;
  indexes: number;
  tombstones: number;
  lsn: number;
  walPending: number;
  outbox: number;
  ackedSeq: number;
  inTransaction: boolean;
  store: string;
  /** Durability of IndexedDB writes, null for other stores. */
  durability: Durability | null;
  walBytes: number;
  snapshotBytes: number;
}

/**
 * The engine bound to a store. Queries run synchronously; durability is
 * asynchronous (group commit) – `await flush()` makes every committed
 * transaction durable.
 *
 * Events: `change` (local write committed), `remote-change` (sync applied
 * changes, detail: ApplyReport), `checkpoint` (detail: { bytes }),
 * `error` (detail: Error from the store).
 */
export class FlatDB extends EventTarget {
  static open(options?: FlatDBOptions): Promise<FlatDB>;
  readonly store: Store;
  readonly nodeId: number;
  recovery: RecoveryReport;
  checkpointBytes: number;
  autoFlush: boolean;

  exec(sql: string): QueryResult;
  execAll(sql: string): QueryResult[];
  execJson(sql: string): string;
  query<T = Record<string, SqlValue>>(sql: string, params?: SqlValue[]): T[];
  prepare(sql: string): PreparedStatement;
  run(statement: PreparedStatement, params?: SqlValue[]): QueryResult;
  runJson(statement: PreparedStatement, params?: SqlValue[]): string;
  /** Executes a prepared write and returns the number of affected rows. */
  execute(statement: PreparedStatement, params?: SqlValue[]): number;
  inTransaction(): boolean;
  tableNames(): string[];
  stats(): DbStats;

  flush(): Promise<void>;
  checkpoint(): Promise<void>;
  close(): Promise<void>;

  outboxLength(): number;
  exportOutbox(max?: number): OutboxBatch;
  ackOutbox(lastSeq: number): void;
  applyChanges(batch: Uint8Array): ApplyReport;
  exportState(): Uint8Array;
}

export interface SyncOptions {
  /** Base URL of the sync server. */
  url: string;
  intervalMs?: number;
  debounceMs?: number;
  batchSize?: number;
  fetch?: typeof fetch;
}

export interface SyncResult {
  pushed: number;
  pulled: number;
  ignored: number;
  rejected: number;
  ms: number;
}

/**
 * Background replication with the sync server. Events: `status`
 * (detail: { status, result?, error? }) and `synced` (detail: SyncResult).
 */
export class SyncClient extends EventTarget {
  constructor(db: FlatDB, options: SyncOptions);
  readonly clientId: string;
  readonly pending: number;
  status: 'idle' | 'syncing' | 'offline' | 'error';
  cursor: number;
  epoch: string | null;
  lastSync: Date | null;
  lastError: Error | null;
  url: string;
  stats: { rounds: number; pushed: number; pulled: number; failures: number };
  readonly simulatedOffline: boolean;

  start(): Promise<void>;
  stop(): void;
  syncNow(): Promise<SyncResult | { error: Error } | { skipped: string }>;
  setSimulatedOffline(offline: boolean): void;
  drain(options?: { timeoutMs?: number }): Promise<SyncResult>;
}

export function initEngine(source?: RequestInfo | URL | Response | BufferSource | WebAssembly.Module): Promise<unknown>;
export function engineReady(): Promise<unknown> | null;
