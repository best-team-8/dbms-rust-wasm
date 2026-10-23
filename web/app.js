// Flat-SQL demo application: SQL console on a local WebAssembly database
// persisted in IndexedDB and replicated through the sync server.

import { FlatDB, initEngine, MemoryStore, SyncClient, version } from './bridge/index.js';

const $ = (id) => document.getElementById(id);
const app = { db: null, sync: null, store: null, persistent: true };

// ---------------------------------------------------------------------------
// formatting helpers

function fmtDuration(ms) {
  if (ms < 1) return `${(ms * 1000).toFixed(0)} µs`;
  if (ms < 100) return `${ms.toFixed(2)} ms`;
  return `${ms.toFixed(0)} ms`;
}

function fmtBytes(n) {
  if (n < 1024) return `${n} B`;
  if (n < 1024 * 1024) return `${(n / 1024).toFixed(1)} KiB`;
  return `${(n / 1024 / 1024).toFixed(2)} MiB`;
}

function fmtTime(date) {
  return date ? date.toLocaleTimeString('pl-PL') : '–';
}

function el(tag, attrs = {}, ...children) {
  const node = document.createElement(tag);
  for (const [k, v] of Object.entries(attrs)) {
    if (k === 'class') node.className = v;
    else if (k.startsWith('on')) node.addEventListener(k.slice(2), v);
    else node.setAttribute(k, v);
  }
  for (const c of children) node.append(c instanceof Node ? c : document.createTextNode(String(c)));
  return node;
}

function log(message, kind = 'info') {
  const list = $('log');
  list.prepend(el('li', { class: `log-${kind}` }, el('time', {}, new Date().toLocaleTimeString('pl-PL')), message));
  while (list.children.length > 200) list.lastChild.remove();
}

function setChip(id, text, state) {
  const chip = $(id);
  chip.textContent = text;
  chip.dataset.state = state ?? '';
}

// ---------------------------------------------------------------------------
// sample data and queries (use cases from the project description)

const SAMPLES = [
  {
    label: 'Gra: ekwipunek gracza',
    sql: "SELECT item, kind, rarity, qty, power\nFROM inventory\nWHERE player = 'ayla'\nORDER BY power DESC\nLIMIT 20;",
  },
  {
    label: 'Gra: statystyki wg rzadkości',
    sql: 'SELECT rarity, COUNT(*) AS items, ROUND(AVG(power), 1) AS avg_power, MAX(power) AS best\nFROM inventory\nGROUP BY rarity\nORDER BY best DESC;',
  },
  {
    label: 'Gra: transakcja (wymiana przedmiotu)',
    sql: "BEGIN;\nUPDATE inventory SET player = 'borin' WHERE id = 1;\nUPDATE inventory SET qty = qty + 10 WHERE player = 'borin' AND kind = 'potion';\nCOMMIT;\nSELECT id, player, item, qty FROM inventory WHERE player = 'borin' ORDER BY id LIMIT 10;",
  },
  {
    label: 'Plan zapytania (indeks)',
    sql: "EXPLAIN SELECT * FROM inventory WHERE player = 'cyra' AND power > 50;",
  },
  {
    label: 'Assety: filtrowanie katalogu',
    sql: "SELECT name, kind, width, height, size_kb\nFROM assets\nWHERE kind = 'image' AND width >= 1920 AND name LIKE '%forest%'\nORDER BY size_kb DESC\nLIMIT 25;",
  },
  {
    label: 'Assety: rozmiar wg typu',
    sql: 'SELECT kind, COUNT(*) AS files, ROUND(SUM(size_kb) / 1024.0, 1) AS total_mb\nFROM assets\nGROUP BY kind\nORDER BY total_mb DESC;',
  },
  {
    label: 'Flat-SQL: brak JOIN (celowo)',
    sql: 'SELECT * FROM inventory JOIN assets ON inventory.item = assets.name;',
  },
  { label: 'Pokaż tabele', sql: 'SHOW TABLES;' },
];

const PLAYERS = ['ayla', 'borin', 'cyra', 'dax', 'eryn'];
const ITEMS = [
  ['sword', 'weapon'], ['axe', 'weapon'], ['bow', 'weapon'], ['dagger', 'weapon'], ['staff', 'weapon'],
  ['shield', 'armor'], ['helmet', 'armor'], ['boots', 'armor'], ['cloak', 'armor'],
  ['health potion', 'potion'], ['mana potion', 'potion'], ['elixir', 'potion'],
  ['ruby', 'gem'], ['emerald', 'gem'], ['scroll of fire', 'scroll'], ['scroll of ice', 'scroll'],
];
const RARITY = ['common', 'common', 'common', 'uncommon', 'uncommon', 'rare', 'epic', 'legendary'];

function seedGame(db) {
  db.exec(`CREATE TABLE IF NOT EXISTS inventory (
    id INT PRIMARY KEY, player TEXT NOT NULL, item TEXT NOT NULL, kind TEXT,
    rarity TEXT, qty INT, power FLOAT);
  CREATE INDEX IF NOT EXISTS inventory_player ON inventory (player);`);
  const start = db.query('SELECT COALESCE(MAX(id), 0) AS m FROM inventory')[0].m + 1;
  const st = db.prepare('INSERT INTO inventory VALUES (?, ?, ?, ?, ?, ?, ?)');
  db.exec('BEGIN');
  for (let i = 0; i < 2000; i++) {
    const [item, kind] = ITEMS[(i * 7) % ITEMS.length];
    const rarity = RARITY[(i * 13) % RARITY.length];
    const qty = kind === 'potion' || kind === 'gem' ? 1 + (i % 9) : 1;
    const power = Math.round((((i * 7919) % 1000) / 10 + RARITY.indexOf(rarity) * 5) * 10) / 10;
    db.execute(st, [start + i, PLAYERS[i % PLAYERS.length], item, kind, rarity, qty, power]);
  }
  db.exec('COMMIT');
  st.free();
}

const ASSET_WORDS = ['forest', 'castle', 'ocean', 'city', 'desert', 'night', 'portrait', 'logo', 'sans', 'serif', 'mono', 'intro', 'loop'];
function seedAssets(db) {
  db.exec(`CREATE TABLE IF NOT EXISTS assets (
    id INT PRIMARY KEY, name TEXT NOT NULL, kind TEXT, width INT, height INT, size_kb INT, tags TEXT);
  CREATE INDEX IF NOT EXISTS assets_kind ON assets (kind);`);
  const start = db.query('SELECT COALESCE(MAX(id), 0) AS m FROM assets')[0].m + 1;
  const st = db.prepare('INSERT INTO assets VALUES (?, ?, ?, ?, ?, ?, ?)');
  const kinds = ['image', 'image', 'image', 'font', 'video', 'audio'];
  const sizes = [[640, 480], [1280, 720], [1920, 1080], [3840, 2160]];
  db.exec('BEGIN');
  for (let i = 0; i < 10_000; i++) {
    const kind = kinds[i % kinds.length];
    const word = ASSET_WORDS[(i * 7) % ASSET_WORDS.length];
    const [w, h] = kind === 'image' || kind === 'video' ? sizes[(i * 3) % sizes.length] : [null, null];
    const ext = { image: 'png', font: 'ttf', video: 'mp4', audio: 'ogg' }[kind];
    const size = kind === 'video' ? 5000 + (i % 50000) : kind === 'image' ? 50 + ((i * 37) % 4000) : 20 + (i % 400);
    db.execute(st, [start + i, `${word}-${start + i}.${ext}`, kind, w, h, size, `${word},${kind}`]);
  }
  db.exec('COMMIT');
  st.free();
}

// ---------------------------------------------------------------------------
// rendering

function renderResult(result, ms, statements) {
  const meta = [];
  if (statements > 1) meta.push(`${statements} instrukcji`);
  meta.push(result.message);
  if (result.columns.length) meta.push(`${result.rows.length} wierszy`);
  if (result.rowsAffected) meta.push(`zmienione: ${result.rowsAffected}`);
  if (result.rowsScanned) meta.push(`przejrzane: ${result.rowsScanned}`);
  meta.push(`czas: ${fmtDuration(ms)} (lokalnie, bez sieci)`);
  $('result-meta').textContent = meta.join(' · ');

  const box = $('result');
  box.replaceChildren();
  if (!result.columns.length) return;
  const limit = 500;
  const table = el('table');
  table.append(el('thead', {}, el('tr', {}, ...result.columns.map((c) => el('th', {}, c)))));
  const body = el('tbody');
  for (const row of result.rows.slice(0, limit)) {
    body.append(
      el('tr', {}, ...row.map((v) => el('td', { class: v === null ? 'null' : typeof v === 'number' ? 'num' : '' }, v === null ? 'NULL' : String(v)))),
    );
  }
  table.append(body);
  box.append(table);
  if (result.rows.length > limit) box.append(el('p', { class: 'muted' }, `Pokazano ${limit} z ${result.rows.length} wierszy.`));
}

function renderTables() {
  const list = $('tables');
  const rows = app.db.exec('SHOW TABLES').rows;
  list.replaceChildren();
  if (!rows.length) {
    list.append(el('li', { class: 'muted' }, 'brak tabel – wczytaj dane przykładowe'));
    return;
  }
  for (const [name, count, , indexes] of rows) {
    list.append(
      el(
        'li',
        {},
        el('button', { class: 'link', onclick: () => runSql(`SELECT * FROM "${name}" LIMIT 100;`, true) }, name),
        el('span', { class: 'muted' }, ` ${count.toLocaleString('pl-PL')} wierszy · ${indexes} indeks(y)`),
      ),
    );
  }
}

const STATUS_LABEL = {
  idle: 'zsynchronizowano',
  syncing: 'synchronizacja…',
  offline: 'offline – zmiany czekają',
  error: 'błąd serwera',
};

function renderSync() {
  const { sync, db } = app;
  if (!sync) return;
  const pending = db.outboxLength();
  const status = sync.status;
  const label = status === 'idle' && pending > 0 ? 'oczekuje' : STATUS_LABEL[status] ?? status;
  $('sync-status').textContent = label + (sync.lastError && status !== 'idle' ? ` (${sync.lastError.message})` : '');
  $('sync-pending').textContent = pending;
  $('sync-last').textContent = fmtTime(sync.lastSync);
  $('sync-cursor').textContent = sync.epoch ? `${sync.cursor} @ ${sync.epoch}` : '–';
  const state = status === 'idle' ? (pending ? 'warn' : 'ok') : status === 'syncing' ? 'busy' : 'bad';
  setChip('chip-sync', `Sync: ${pending ? `${pending} oczekuje` : label}`, state);
}

function renderNetwork() {
  const online = navigator.onLine && !app.sync?.simulatedOffline;
  setChip('chip-network', online ? 'Sieć: online' : 'Sieć: offline', online ? 'ok' : 'bad');
}

const DURABILITY_LABEL = {
  default: 'domyślna przeglądarki (Chrome 121+: relaxed)',
  relaxed: 'relaxed – po przekazaniu do systemu',
  strict: 'strict – po zapisie na dysk',
};

function persistenceLabel(persisted) {
  if (persisted === true) return 'tak – przeglądarka nie usunie danych';
  if (persisted === false) return 'nie – przeglądarka może usunąć dane, także niewysłane zmiany';
  return app.persistent ? 'nieznany (brak navigator.storage)' : 'nie dotyczy (tylko pamięć)';
}

function renderStorage() {
  const s = app.db.stats();
  $('st-persisted').textContent = persistenceLabel(s.persisted);
  $('btn-persist').classList.toggle('hidden', s.persisted !== false);
  $('st-durability').textContent = s.durability ? DURABILITY_LABEL[s.durability] : 'brak (tylko pamięć)';
  $('st-node').textContent = s.nodeId.toString(16).padStart(8, '0');
  $('st-rows').textContent = `${s.rows.toLocaleString('pl-PL')} / ${s.tables}`;
  $('st-lsn').textContent = s.lsn;
  $('st-wal').textContent = app.persistent ? fmtBytes(s.walBytes) : `${fmtBytes(s.walBytes)} (tylko pamięć)`;
  $('st-snapshot').textContent = s.snapshotBytes ? fmtBytes(s.snapshotBytes) : '–';
}

// ---------------------------------------------------------------------------
// actions

function runSql(sql, fromTable = false) {
  if (fromTable) $('editor').value = sql;
  $('error').classList.add('hidden');
  const t0 = performance.now();
  try {
    const results = app.db.execAll(sql);
    const ms = performance.now() - t0;
    const last = results[results.length - 1] ?? { columns: [], rows: [], message: 'OK' };
    renderResult(last, ms, results.length);
    log(`${results.map((r) => r.message).join(', ')} – ${fmtDuration(ms)}`);
  } catch (err) {
    $('error').textContent = `${err.kind ? `[${err.kind}] ` : ''}${err.message}`;
    $('error').classList.remove('hidden');
    log(err.message, 'error');
  }
  renderTables();
  renderSync();
}

function timed(label, fn) {
  const t0 = performance.now();
  fn();
  const ms = performance.now() - t0;
  log(`${label}: ${fmtDuration(ms)}`, 'ok');
  renderTables();
  renderSync();
}

async function acquireTabLock() {
  if (!navigator.locks) return true;
  return new Promise((resolve) => {
    navigator.locks.request('flatsql-demo-db', { ifAvailable: true }, (lock) => {
      resolve(Boolean(lock));
      // Hold the lock for the lifetime of the tab.
      return lock ? new Promise(() => {}) : undefined;
    });
  });
}

function wireUi() {
  const samples = $('samples');
  SAMPLES.forEach((s, i) => samples.append(el('option', { value: String(i) }, s.label)));
  samples.addEventListener('change', () => {
    if (samples.value === '') return;
    $('editor').value = SAMPLES[Number(samples.value)].sql;
    samples.value = '';
    $('editor').focus();
  });
  $('btn-run').addEventListener('click', () => runSql($('editor').value));
  $('btn-explain').addEventListener('click', () => {
    const sql = $('editor').value.trim().replace(/;$/, '');
    runSql(/^explain/i.test(sql) ? sql : `EXPLAIN ${sql}`);
  });
  $('editor').addEventListener('keydown', (e) => {
    if (e.key === 'Enter' && (e.ctrlKey || e.metaKey)) {
      e.preventDefault();
      runSql($('editor').value);
    }
  });
  $('btn-seed-game').addEventListener('click', () => timed('Wczytano 2000 przedmiotów (1 transakcja)', () => seedGame(app.db)));
  $('btn-seed-assets').addEventListener('click', () => timed('Wczytano 10 000 assetów (1 transakcja)', () => seedAssets(app.db)));
  $('btn-sync').addEventListener('click', () => app.sync.syncNow());
  $('toggle-offline').addEventListener('change', (e) => {
    app.sync.setSimulatedOffline(e.target.checked);
    log(e.target.checked ? 'Symulacja braku sieci: zapisy trafiają tylko do lokalnej bazy' : 'Sieć przywrócona – wysyłam zaległe zmiany', 'warn');
    renderNetwork();
  });
  $('btn-checkpoint').addEventListener('click', async () => {
    const t0 = performance.now();
    await app.db.checkpoint();
    log(`Checkpoint (snapshot + obcięcie WAL): ${fmtDuration(performance.now() - t0)}`, 'ok');
    renderStorage();
  });
  $('btn-persist').addEventListener('click', async () => {
    const granted = await app.db.requestPersistence();
    log(granted ? 'Przeglądarka zgodziła się na trwały magazyn' : 'Przeglądarka odmówiła trwałego magazynu', granted ? 'ok' : 'warn');
    renderStorage();
  });
  $('btn-reset').addEventListener('click', async () => {
    if (!confirm('Usunąć lokalną bazę z tej przeglądarki? Dane zsynchronizowane z serwerem zostaną pobrane ponownie.')) return;
    app.sync.stop();
    await app.db.close();
    await app.store.destroy();
    location.reload();
  });
  window.addEventListener('online', renderNetwork);
  window.addEventListener('offline', renderNetwork);
}

async function main() {
  wireUi();
  if ('serviceWorker' in navigator) {
    navigator.serviceWorker.register('sw.js').catch((err) => log(`Service Worker: ${err.message}`, 'warn'));
  }

  const t0 = performance.now();
  await initEngine();
  const loadMs = performance.now() - t0;
  setChip('chip-engine', `Silnik: WASM ${version()}`, 'ok');
  log(`Moduł WebAssembly załadowany w ${fmtDuration(loadMs)}`);

  app.persistent = await acquireTabLock();
  if (!app.persistent) {
    const banner = $('banner');
    banner.textContent = 'Baza jest już otwarta w innej karcie – ta karta działa tylko w pamięci (bez zapisu do IndexedDB).';
    banner.classList.remove('hidden');
  }
  setChip('chip-store', `Magazyn: ${app.persistent ? 'IndexedDB' : 'pamięć'}`, app.persistent ? 'ok' : 'warn');

  // Without a store FlatDB keeps the database in IndexedDB;
  // ?durability=strict|relaxed|default selects the durability mode.
  const durability = new URLSearchParams(location.search).get('durability') ?? undefined;
  app.db = await FlatDB.open(app.persistent ? { name: 'demo', durability } : { store: new MemoryStore() });
  app.store = app.db.store;
  const r = app.db.recovery;
  $('st-recovery').textContent =
    `${fmtDuration(r.ms)} · snapshot ${fmtBytes(r.snapshotBytes)} + ${r.framesReplayed} ramek WAL` + (r.tornBytes ? ` · odrzucono ${r.tornBytes} B` : '');
  log(`Baza odtworzona z ${app.persistent ? 'IndexedDB' : 'pamięci'} w ${fmtDuration(r.ms)} (${r.framesReplayed} ramek WAL)`);
  app.db.addEventListener('checkpoint', (e) => log(`Automatyczny checkpoint: snapshot ${fmtBytes(e.detail.bytes)}`));
  app.db.persistence.then((persisted) => {
    if (persisted !== null) log(`Trwały magazyn: ${persistenceLabel(persisted)}`, persisted ? 'ok' : 'warn');
    renderStorage();
  });
  app.db.addEventListener('error', (e) => log(`Zapis do magazynu nie powiódł się: ${e.detail.message}`, 'error'));

  app.sync = new SyncClient(app.db, { url: location.origin, intervalMs: 4000 });
  app.sync.addEventListener('status', (e) => {
    renderSync();
    renderNetwork();
    if (e.detail.status === 'offline') log('Serwer niedostępny – praca offline, zmiany czekają w outboksie', 'warn');
  });
  app.sync.addEventListener('synced', (e) => {
    const { pushed, pulled, ms } = e.detail;
    if (pushed || pulled) log(`Synchronizacja: wysłano ${pushed}, odebrano ${pulled} zmian (${fmtDuration(ms)})`, 'ok');
    renderSync();
  });
  app.db.addEventListener('remote-change', () => renderTables());
  app.db.addEventListener('change', () => renderSync());
  await app.sync.start();

  renderTables();
  renderSync();
  renderNetwork();
  renderStorage();
  setInterval(renderStorage, 1000);
}

main().catch((err) => {
  console.error(err);
  setChip('chip-engine', 'Silnik: błąd', 'bad');
  log(`Błąd startu: ${err.message}`, 'error');
});
