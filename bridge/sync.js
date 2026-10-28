// SyncClient – asynchronous replication of a FlatDB with a sync server.
//
// The application never waits for the network: writes go to the local
// database, land in the engine's outbox and are pushed in batches when the
// server is reachable. One sync round:
//
//   1. push: export outbox batch -> POST /sync/push -> ack on success
//   2. pull: GET /sync/pull?since=<cursor> -> apply changes (last-writer-wins)
//   3. flush the local WAL, then persist the new cursor
//
// Every step is idempotent, so a round interrupted by a lost connection is
// simply repeated later. While offline the client retries with exponential
// backoff (and immediately on the browser's `online` event).
//
// Clock: every response carries the server time. When the device clock
// differs from it by more than maxDriftMs(), or the server rejected pushed
// changes as stamped too far ahead, the client pauses: the batch is not
// acknowledged (it stays in the outbox), nothing is pulled, and every round
// only checks the clock again (GET /sync/time). Once the clock is right,
// changes stamped while it ran ahead get new stamps (FlatDB.rebaseClock)
// before they are pushed.

import { maxDriftMs } from '../pkg/flatsql.js';

const sleep = (ms) => new Promise((r) => setTimeout(r, ms));

/** The device clock differs too much from the server clock. */
export class ClockError extends Error {
  constructor(skewMs, rejected, limitMs) {
    const s = Math.round(Math.abs(skewMs ?? 0) / 1000);
    const where = (skewMs ?? 0) >= 0 ? 'ahead of' : 'behind';
    super(
      rejected > 0
        ? `the server rejected ${rejected} change(s) stamped more than ${limitMs / 1000} s ahead (device clock ${s} s ${where} the server)`
        : `device clock is ${s} s ${where} the server (limit ${limitMs / 1000} s)`,
    );
    this.name = 'ClockError';
    this.skewMs = skewMs;
    this.rejected = rejected;
  }
}

export class SyncClient extends EventTarget {
  #timer = null;
  #running = null;
  #started = false;
  #backoff;
  #simulatedOffline = false;
  #onChange = () => this.#schedule(this.debounceMs);
  #onOnline = () => this.#schedule(0);
  #onOffline = () => this.#setStatus('offline');

  /**
   * @param {import('./flatdb.js').FlatDB} db
   * @param {object} options
   * @param {string} options.url base URL of the sync server
   * @param {number} [options.intervalMs] periodic sync interval
   * @param {number} [options.debounceMs] delay between a local write and its push
   * @param {number} [options.batchSize] max changes per push request
   * @param {Function} [options.fetch] fetch implementation (tests)
   */
  constructor(db, { url, intervalMs = 5000, debounceMs = 250, batchSize = 1000, fetch } = {}) {
    super();
    if (!url) throw new Error('SyncClient needs the server url');
    this.db = db;
    db.enableSync();
    this.url = url.replace(/\/$/, '');
    this.intervalMs = intervalMs;
    this.debounceMs = debounceMs;
    this.batchSize = batchSize;
    this.fetchImpl = fetch ?? globalThis.fetch.bind(globalThis);
    this.#backoff = intervalMs;
    this.status = 'idle';
    this.maxDriftMs = maxDriftMs();
    /** Device clock minus server clock (ms) from the last response, null before the first. */
    this.clockSkewMs = null;
    this.clockPaused = false;
    this.cursor = 0;
    this.epoch = null;
    this.lastSync = null;
    this.lastError = null;
    this.stats = { rounds: 0, pushed: 0, pulled: 0, failures: 0 };
  }

  get clientId() {
    return this.db.nodeId.toString(16);
  }

  get pending() {
    return this.db.outboxLength();
  }

  /** Loads the persisted cursor and starts background synchronisation. */
  async start() {
    if (this.#started) return;
    this.#started = true;
    const saved = await this.db.store.getMeta('sync');
    if (saved) {
      this.cursor = saved.cursor ?? 0;
      this.epoch = saved.epoch ?? null;
    }
    this.db.addEventListener('change', this.#onChange);
    if (typeof window !== 'undefined') {
      window.addEventListener('online', this.#onOnline);
      window.addEventListener('offline', this.#onOffline);
    }
    this.#schedule(0);
  }

  stop() {
    this.#started = false;
    clearTimeout(this.#timer);
    this.#timer = null;
    this.db.removeEventListener('change', this.#onChange);
    if (typeof window !== 'undefined') {
      window.removeEventListener('online', this.#onOnline);
      window.removeEventListener('offline', this.#onOffline);
    }
  }

  /** Makes every request fail as if the network was down (demo / tests). */
  setSimulatedOffline(offline) {
    this.#simulatedOffline = offline;
    if (offline) this.#setStatus('offline');
    else this.#schedule(0);
  }

  get simulatedOffline() {
    return this.#simulatedOffline;
  }

  #schedule(ms) {
    if (!this.#started) return;
    clearTimeout(this.#timer);
    this.#timer = setTimeout(() => {
      this.#timer = null;
      this.syncNow().finally(() => {
        if (this.#started && this.#timer === null) this.#schedule(this.#backoff);
      });
    }, ms);
  }

  #setStatus(status, detail) {
    if (this.status === status && !detail) return;
    this.status = status;
    this.dispatchEvent(new CustomEvent('status', { detail: { status, ...detail } }));
  }

  async #fetch(path, init) {
    if (this.#simulatedOffline || (typeof navigator !== 'undefined' && navigator.onLine === false)) {
      throw new TypeError('network unavailable (offline)');
    }
    const res = await this.fetchImpl(`${this.url}${path}`, init);
    if (!res.ok) {
      const text = await res.text().catch(() => '');
      const err = new Error(`HTTP ${res.status} ${path}: ${text}`);
      err.httpStatus = res.status;
      throw err;
    }
    return res;
  }

  /** fetch that also measures the clock difference from the x-server-time header. */
  async #timed(path, init) {
    const t0 = Date.now();
    const res = await this.#fetch(path, init);
    const t1 = Date.now();
    const server = Number(res.headers.get('x-server-time'));
    if (server > 0) this.clockSkewMs = Math.round((t0 + t1) / 2 - server);
    return res;
  }

  /** Emits `conflict` for tables defined differently here and on the server. */
  #reportConflicts(conflicts = [], direction) {
    if (conflicts.length) this.dispatchEvent(new CustomEvent('conflict', { detail: { direction, conflicts } }));
    return conflicts;
  }

  /** Pauses synchronisation when the clocks differ too much. */
  #checkClock(rejected = 0) {
    if (rejected > 0 || (this.clockSkewMs !== null && Math.abs(this.clockSkewMs) > this.maxDriftMs)) {
      this.clockPaused = true;
      throw new ClockError(this.clockSkewMs, rejected, this.maxDriftMs);
    }
  }

  /** Runs one sync round now (concurrent calls share the same round). */
  syncNow() {
    if (!this.#running) {
      this.#running = this.#round().finally(() => {
        this.#running = null;
      });
    }
    return this.#running;
  }

  async #round() {
    if (this.db.inTransaction()) return { skipped: 'transaction in progress' };
    this.#setStatus('syncing');
    const t0 = performance.now();
    try {
      // Before the first push the clock difference is not known yet.
      if (this.clockPaused || this.clockSkewMs === null) {
        await this.#timed('/sync/time');
        this.#checkClock();
        this.clockPaused = false;
      }
      // Changes stamped while the clock ran ahead would be rejected again.
      let restamped = 0;
      if (this.db.clockAhead()) restamped = await this.db.rebaseClock();

      let pushed = 0;
      const conflicts = [];
      for (let guard = 0; this.db.outboxLength() > 0 && guard < 10_000; guard++) {
        const batch = this.db.exportOutbox(this.batchSize);
        if (batch.count === 0) break;
        const res = await this.#timed(`/sync/push?client=${this.clientId}`, {
          method: 'POST',
          headers: { 'content-type': 'application/octet-stream' },
          body: batch.bytes,
        });
        const reply = await res.json().catch(() => ({}));
        // Not acknowledged: changes rejected for the clock stay in the outbox.
        this.#checkClock(reply.clockDrift ?? 0);
        // Rows of a table the server defines differently cannot be applied by
        // sending them again; they leave the outbox and are reported.
        conflicts.push(...this.#reportConflicts(reply.conflicts, 'push'));
        this.db.ackOutbox(batch.lastSeq);
        pushed += batch.count;
        batch.free();
      }

      const epoch = this.epoch ?? '';
      const res = await this.#timed(`/sync/pull?since=${this.cursor}&epoch=${epoch}&client=${this.clientId}`);
      this.#checkClock();
      const body = new Uint8Array(await res.arrayBuffer());
      let report = { applied: 0, ignored: 0, rejected: 0, clockDrift: 0, errors: [] };
      if (body.length > 0) report = this.db.applyChanges(body);
      conflicts.push(...this.#reportConflicts(report.conflicts, 'pull'));

      // Pulled rows and acks must be durable before the cursor moves on.
      await this.db.flush();
      // Changes rejected as stamped ahead of this clock are pulled again
      // later (the clocks of two devices may together differ by more than
      // the limit, which resolves as time passes).
      if (!report.clockDrift) {
        this.cursor = Number(res.headers.get('x-sync-cursor') ?? this.cursor);
        this.epoch = res.headers.get('x-sync-epoch') ?? this.epoch;
        await this.db.store.setMeta('sync', { cursor: this.cursor, epoch: this.epoch });
      }

      this.#backoff = this.intervalMs;
      this.lastSync = new Date();
      this.lastError = null;
      this.stats.rounds++;
      this.stats.pushed += pushed;
      this.stats.pulled += report.applied;
      const result = {
        pushed,
        pulled: report.applied,
        ignored: report.ignored,
        rejected: report.rejected,
        clockDrift: report.clockDrift ?? 0,
        restamped,
        conflicts,
        ms: performance.now() - t0,
      };
      this.#setStatus('idle', { result });
      this.dispatchEvent(new CustomEvent('synced', { detail: result }));
      return result;
    } catch (error) {
      this.stats.failures++;
      this.lastError = error;
      if (error instanceof ClockError) {
        // Not a network problem: check again at the normal pace.
        this.#backoff = this.intervalMs;
        this.#setStatus('clock', { error: error.message, skewMs: error.skewMs });
        const detail = { skewMs: error.skewMs, rejected: error.rejected, maxDriftMs: this.maxDriftMs };
        this.dispatchEvent(new CustomEvent('clock', { detail }));
        return { error };
      }
      this.#backoff = Math.min(this.#backoff * 2, 30_000);
      // fetch() rejects with TypeError when the server cannot be reached.
      const offline = error instanceof TypeError;
      this.#setStatus(offline ? 'offline' : 'error', { error: String(error.message ?? error) });
      return { error };
    }
  }

  /** Waits until the outbox is empty and one round succeeded (tests, demos). */
  async drain({ timeoutMs = 10_000 } = {}) {
    const end = Date.now() + timeoutMs;
    for (;;) {
      const r = await this.syncNow();
      if (!r.error && !r.skipped && this.db.outboxLength() === 0) return r;
      if (Date.now() > end) throw new Error(`sync did not complete: ${r.error?.message ?? r.skipped}`);
      await sleep(50);
    }
  }
}
