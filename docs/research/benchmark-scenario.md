# Scenariusz badawczy i metodyka

Dokument definiuje pytania badawcze, hipotezy, obciążenie testowe, metryki i procedurę pomiarów wydajności oraz walidacji architektury Local-First w warunkach offline. Wyniki: [benchmark-results.md](benchmark-results.md).

## 1. Pytania badawcze i hipotezy

| | Pytanie | Hipoteza |
|---|---|---|
| **H1** | O ile lokalne wykonanie zapytania (WASM w procesie aplikacji) jest szybsze od modelu Cloud-First? | Lokalne zapytanie po kluczu trwa mikrosekundy; zapytanie przez sieć jest ograniczone od dołu czasem RTT – różnica to 2–4 rzędy wielkości. |
| **H2** | Jaki jest koszt uruchamiania silnika w WebAssembly zamiast natywnie? | Ten sam kod Rust w WASM jest 1,2–3× wolniejszy od natywnego, zależnie od rodzaju operacji. |
| **H3** | Ile kosztuje granica Rust↔JavaScript i jak ją ograniczyć? | Dla małych wyników dominuje koszt przekazywania danych; serializacja do JSON w Rust jest szybsza niż budowanie obiektów JS przez `js-sys`. |
| **H4** | Jak Flat-SQL wypada na tle dojrzałego silnika SQL w WASM (SQLite/sql.js)? | Flat-SQL jest konkurencyjny w zapytaniach punktowych, agregacji i sortowaniu, słabszy w operacjach, w których SQLite korzysta z formatu stronicowego (snapshot/odtwarzanie) i indeksów pokrywających. |
| **H5** | Jaki jest koszt trwałego zatwierdzenia w przeglądarce? | Przyrostowy WAL w IndexedDB daje koszt zależny od rozmiaru zmiany, a nie bazy; podejście z eksportem całej bazy (sql.js) rośnie liniowo z rozmiarem bazy. |
| **H6** | Czy aplikacja zachowuje pełną funkcjonalność offline i czy repliki są zbieżne po odzyskaniu połączenia? | Tak: odczyty i zapisy działają bez serwera, niewysłane zmiany przetrwają restart aplikacji, a po synchronizacji wszystkie repliki mają identyczny stan (LWW). |

## 2. Środowisko

| Element | Wartość użyta w wynikach |
|---|---|
| Sprzęt | AMD Ryzen 5 4600H (6 rdzeni / 12 wątków), 16 GB RAM |
| System | Windows 11 Home (10.0.26200) |
| Node.js | 22.23.2 (V8) |
| Przeglądarka | Chromium 152 (silnik V8) |
| Rust | 1.99.0, profil `release` (`opt-level = 3`, LTO, `codegen-units = 1`), `wasm-opt -O3` |
| sql.js | 1.14.2 |

Pomiary wykonano na laptopie w typowej konfiguracji, bez strojenia systemu; przed pomiarem należy zamknąć inne obciążające aplikacje.

## 3. Obciążenie (workload)

Tabela wspólna dla wszystkich silników:

```sql
CREATE TABLE items (id INTEGER PRIMARY KEY, category TEXT, price FLOAT, qty INT, name TEXT);
CREATE INDEX items_category ON items (category);
```

Wiersz `i` (deterministycznie): `(i, 'c' || (i % 100), ((i * 7919) % 10000) / 100.0, i % 50, 'item-' || i)` – 100 kategorii po 1000 wierszy, ceny rozłożone pseudolosowo. Domyślnie N = 100 000 wierszy. `INTEGER PRIMARY KEY` czyni kolumnę aliasem `rowid` w SQLite – najszybszy układ tabeli SQLite, więc punkt odniesienia jest mierzony w najlepszej konfiguracji.

| Operacja | Zapytanie | Powtórzeń |
|---|---|---:|
| Insert | N × prepared `INSERT INTO items VALUES (?, ?, ?, ?, ?)` w jednej transakcji | 1 |
| PK lookup | `SELECT * FROM items WHERE id = ?` (losowe id) | 10 000 |
| Lookup po indeksie | `SELECT COUNT(*) FROM items WHERE category = ?` | 500 |
| Zakres PK | `SELECT SUM(price) FROM items WHERE id BETWEEN ? AND ? + 999` | 200 |
| Pełny skan z filtrem | `SELECT COUNT(*) FROM items WHERE price > 50 AND qty < 10` | 20 |
| Grupowanie | `SELECT category, COUNT(*), AVG(price), MAX(qty) FROM items GROUP BY category` | 20 |
| Top-k | `SELECT id, price FROM items ORDER BY price DESC LIMIT 10` | 20 |
| Pobranie wyników | `SELECT * FROM items WHERE id < 10000` (10 000 wierszy do JS) | 20 |
| UPDATE 10% | `UPDATE items SET qty = qty + 1 WHERE id % 10 = 0` | 1 |
| DELETE 10% | `DELETE FROM items WHERE id % 10 = 1` | 1 |
| Snapshot | Flat-SQL `snapshot()`, sql.js `db.export()`, JS `JSON.stringify` | 1 |
| Odtworzenie | Flat-SQL `Database.restore()`, sql.js `new SQL.Database(bytes)`, JS `JSON.parse` + odbudowa | 1 |

Wszystkie zapytania odczytu są przygotowywane raz (prepared statement) i wykonywane z parametrami; wyniki są zawsze w pełni przenoszone do JavaScript (tablice wartości), co odpowiada rzeczywistemu użyciu w aplikacji.

**Uczestnicy**: Flat-SQL WASM (wyniki jako obiekty JS budowane w Rust), Flat-SQL WASM (wyniki jako JSON), sql.js, czysty JavaScript na `Map`/`Set` (ręcznie napisany odpowiednik każdego zapytania – dolna granica bez warstwy SQL), Flat-SQL natywnie (`flatsql bench`, ten sam workload w Rust bez WASM i bez granicy JS).

### Local-First vs Cloud-First (H1)

Ten sam silnik, ta sama tabela (10 000 wierszy) po obu stronach. Zapytanie `SELECT * FROM bench_remote WHERE id = <losowe>`:

- **lokalnie** – `db.exec(sql)` w procesie (parsowanie + wykonanie + przekazanie wyniku), 2000 powtórzeń (pierwsze 200 odrzucone),
- **przez API** – `POST /api/query` do serwera Node.js z tym samym silnikiem; opóźnienie sieci symulowane po stronie serwera: 0 (localhost), 20, 50, 100 ms; 40 żądań (20 dla ≥ 50 ms), pierwsze 3 odrzucone (rozgrzanie połączenia keep-alive).

Symulacja zakłada opóźnienie równe RTT, bez strat pakietów i bez czasu ustanawiania połączenia/TLS – wynik jest więc **dolnym ograniczeniem** kosztu modelu Cloud-First.

### Trwałość w przeglądarce (H5)

W prawdziwej przeglądarce, magazyn IndexedDB:

1. N wierszy (50 000) w jednej transakcji + `await flush()` – koszt wykonania i zapisania WAL,
2. 100 × (jeden `INSERT` + `await flush()`) – koszt trwałego zatwierdzenia pojedynczej transakcji,
3. 1000 × `INSERT` + jeden `flush()` – group commit,
4. checkpoint (snapshot do IndexedDB) i ponowne otwarcie bazy (IndexedDB → pamięć),
5. sql.js: 20 × (jeden `INSERT` + `db.export()` + zapis pliku do IndexedDB) – jedyna dostępna w sql.js droga do trwałości.

## 4. Metodyka pomiaru

- **Rozgrzewka**: przed pomiarem każdy silnik wykonuje skrócony przebieg (10 000 wierszy, do 200 wykonań każdego zapytania) na osobnej bazie. V8 kompiluje WASM najpierw kompilatorem bazowym (Liftoff), a gorące funkcje optymalizuje później (TurboFan); bez rozgrzewki pierwszy mierzony silnik byłby systematycznie gorszy (zaobserwowano do 2× dla insertów).
- **Pomiar czasu**: `performance.now()` wokół każdego wykonania; dla operacji powtarzanych raportowana jest średnia (całkowity czas / liczba wykonań) oraz percentyle p50/p95/p99 z pojedynczych próbek. Native: `std::time::Instant`.
- **Rozdzielczość zegara**: w Node.js submikrosekundowa; w przeglądarce bez izolacji cross-origin `performance.now()` jest zaokrąglane (w testowanym Chromium do 100 µs) – w przeglądarce wiarygodne są średnie, a percentyle operacji krótszych niż 100 µs są skwantowane.
- **Kontrola poprawności**: po każdym zapytaniu zapisywana jest liczba zwróconych wierszy i porównywana między silnikami (oraz liczba wierszy po odtworzeniu snapshotu); rozbieżność jest raportowana jako ostrzeżenie.
- **Klient HTTP w Node.js**: wbudowany `fetch` (undici) na Windows dodaje ok. 15 ms do każdego żądania (granulacja timera), dlatego runner używa `node:http` z keep-alive; serwer czeka na symulowane opóźnienie precyzyjnie (timer + dopełnienie przez `setImmediate`), bo `setTimeout` na Windows ma kwant ~15,6 ms (20 ms → 31 ms). W przeglądarce używany jest natywny `fetch`.
- **Powtarzalność**: dane i sekwencja losowych kluczy są deterministyczne (stałe ziarno); wynik każdego uruchomienia zapisywany jest w JSON (`bench/results/`).

## 5. Procedura

```bash
npm install
npm run build
cargo build --release -p flatsql-cli
node bench/run.js --rows 100000 --native --md wyniki.md
```

Przeglądarka: `npm start`, otwarcie `http://127.0.0.1:8080/bench.html`, wybór 100 000 wierszy, wszystkie silniki, „Trwałość” i „Sieć”, **Start**; wynik można pobrać jako JSON.

## 6. Walidacja Local-First w warunkach offline (H6)

Walidacja ma trzy poziomy – od deterministycznych testów jednostkowych po ręczny scenariusz w przeglądarce.

### 6.1 Testy silnika (Rust, `crates/flatsql-core/tests/`)

| Test | Weryfikuje |
|---|---|
| `every_wal_prefix_recovers_to_a_committed_state` | ucięcie WAL w każdym bajcie → baza równa stanowi po ostatniej pełnej transakcji |
| `torn_wal_tail_is_discarded`, `corrupted_snapshot_is_rejected` | urwany zapis i uszkodzony snapshot |
| `frames_covered_by_snapshot_are_skipped` | awaria między snapshotem a obcięciem WAL |
| `outbox_survives_restart_and_ack_is_durable` | niewysłane zmiany przetrwają restart; potwierdzone nie są wysyłane ponownie |
| `concurrent_offline_edits_converge_with_last_writer_wins`, `delete_wins_over_older_update` | konflikty edycji i usunięć |
| `randomized_offline_sessions_converge` | 25 ziaren × 300 losowych operacji na 3 replikach z przerywaną łącznością → identyczny stan wszystkich replik i serwera |

### 6.2 Testy end-to-end (Node.js, `test/offline-sync.test.js`)

Prawdziwy serwer HTTP, klienci `FlatDB` + `SyncClient`:

1. dwie repliki synchronizują dane przez serwer,
2. **serwer zostaje wyłączony** → 50 zapisów na replice A i edycja na replice B kończą się lokalnie (bez czekania na sieć), odczyty działają, runda synchronizacji kończy się statusem `offline`, outbox = 51,
3. aplikacja A zostaje **zamknięta i otwarta ponownie** offline → outbox nadal = 51, dane na miejscu,
4. **serwer startuje ponownie** (nowy proces, ten sam katalog danych, nowa epoka) → po synchronizacji outboxy są puste, a A, B i serwer mają identyczną zawartość tabeli,
5. konflikt: obie repliki offline edytują ten sam wiersz – na wszystkich wygrywa późniejsza edycja, niezależnie od kolejności ponownego połączenia,
6. zapisy wykonane przez cloud API docierają do replik lokalnych.

### 6.3 Scenariusz ręczny w przeglądarce (demo)

1. Wczytanie danych, zapytania – status „zsynchronizowano”.
2. „Symuluj brak sieci” → zapisy lokalne, licznik oczekujących zmian rośnie.
3. Zatrzymanie serwera → zapis → **odświeżenie strony**: aplikacja startuje z cache Service Workera, dane z IndexedDB, liczba oczekujących zmian zachowana.
4. Ponowne uruchomienie serwera → zmiany zostają wysłane, serwer zwraca je w zapytaniu przez API.

Kryterium zaliczenia (H6): wszystkie kroki zakończone bez utraty zatwierdzonych (spłukanych) danych i ze zbieżnym stanem replik.

## 7. Zagrożenia dla trafności

- Jedna maszyna i jeden system (Windows) – wyniki bezwzględne nie przenoszą się na inne urządzenia (zwłaszcza mobilne); istotne są proporcje między silnikami.
- Szum pomiarowy (JIT, GC, procesy w tle): dwie konfiguracje Flat-SQL, które dla insertów wykonują identyczny kod, różnią się w pojedynczym przebiegu o kilkadziesiąt procent – różnice poniżej ~30% przy pojedynczych pomiarach nie powinny być interpretowane.
- Symulowana sieć pomija zmienność opóźnień, utratę pakietów, TLS i ustanawianie połączeń.
- sql.js to jeden z możliwych sposobów użycia SQLite; build z VFS OPFS miałby inną charakterystykę trwałości (bez eksportu całej bazy).
- Obciążenie syntetyczne (jedna tabela, równomierne dane) nie oddaje wszystkich wzorców rzeczywistych aplikacji.
