# Analiza porównawcza silników baz danych działających w WebAssembly

Celem analizy jest umiejscowienie Flat-SQL wśród istniejących rozwiązań, które pozwalają wykonywać zapytania do bazy danych po stronie klienta (przeglądarka, węzeł edge), oraz uzasadnienie wyboru silników porównywanych w benchmarkach ([scenariusz](benchmark-scenario.md), [wyniki](benchmark-results.md)).

Stan na październik 2026. Rozmiary modułów oznaczone „zmierzono” pochodzą z tego repozytorium (`npm run bench`); pozostałe są deklaracjami autorów projektów i mają charakter orientacyjny.

## 1. Kryteria

| Kryterium | Dlaczego ma znaczenie w modelu Local-First |
|---|---|
| Model danych i język zapytań | relacyjny OLTP, analityczny (kolumnowy), dokumentowy – decyduje o klasie zastosowań |
| Implementacja i toolchain WASM | C + Emscripten vs Rust + `wasm-bindgen`; wpływa na rozmiar, integrację z JS i możliwość pracy natywnej |
| Rozmiar modułu | czas pobrania i kompilacji przy pierwszym uruchomieniu (cold start) |
| Trwałość w przeglądarce | brak (tylko pamięć), eksport całej bazy, VFS na IndexedDB, VFS na OPFS; koszt pojedynczego trwałego zatwierdzenia |
| Wymagania środowiska | Web Worker, nagłówki COOP/COEP (`SharedArrayBuffer`), JSPI/Asyncify |
| Synchronizacja | wbudowana, przez zewnętrzną warstwę, brak |
| Dojrzałość | stabilność, utrzymanie projektu |

## 2. Przegląd rozwiązań

### 2.1 SQLite (oficjalny build WASM)

Projekt SQLite publikuje własny build WebAssembly (`@sqlite.org/sqlite-wasm`, Emscripten) z obsługą trwałości w **Origin Private File System**. Dostępne są dwa VFS: `opfs` – wymaga nagłówków COOP/COEP, bo korzysta z `SharedArrayBuffer`, oraz `opfs-sahpool` – bez wymagań co do nagłówków, najszybszy, ale dopuszcza tylko jedno połączenie (druga karta nie otworzy tej samej bazy). Synchroniczne uchwyty OPFS istnieją wyłącznie w Web Workerach, więc baza na OPFS nie może działać w wątku głównym. Pełny SQL, dojrzały silnik; brak wbudowanej synchronizacji.

### 2.2 sql.js

SQLite skompilowany Emscriptenem z wygodnym API JavaScript; działa w przeglądarce i Node.js. Baza żyje w pamięci (MEMFS), a trwałość sprowadza się do `db.export()` – serializacji **całego** pliku bazy – i zapisania go przez aplikację (np. w IndexedDB). Zmierzono: 643 KiB (315 KiB gzip). Najpopularniejszy sposób uruchamiania SQL w przeglądarce, dlatego wybrany jako główny punkt odniesienia.

### 2.3 wa-sqlite i absurd-sql

`wa-sqlite` to build SQLite z obsługą asynchronicznych VFS (Asyncify, a nowsze wersje JSPI) i gotowymi VFS na IndexedDB i OPFS – zapisuje bazę stronami, więc trwałe zatwierdzenie nie wymaga eksportu całości. `absurd-sql` (2021) pokazał to samo podejście dla sql.js (strony bazy jako bloki w IndexedDB, praca w Workerze z `SharedArrayBuffer`); projekt nie jest już rozwijany, a jego rolę przejęły VFS OPFS.

### 2.4 PGlite

PostgreSQL skompilowany do WASM (ElectricSQL), poniżej 3 MB po kompresji gzip. Trwałość przez wymienny system plików: pamięć, IndexedDB, OPFS (`opfs-ahp`) lub system plików w Node/Bun. Pełny dialekt PostgreSQL z rozszerzeniami (np. pgvector), jedno połączenie. Synchronizacja przez zewnętrzne narzędzia (Electric). Klasa „pełny serwer relacyjny w przeglądarce” – największe możliwości kosztem rozmiaru i czasu startu. Dane w IndexedDB mogą zostać usunięte przez przeglądarkę przy braku miejsca (dotyczy to wszystkich rozwiązań bez trwałego przydziału `navigator.storage.persist()`).

### 2.5 DuckDB-WASM

Kolumnowy silnik analityczny (OLAP). Odczytuje Parquet, CSV, JSON i Arrow z plików lub HTTP; API asynchroniczne przez Web Worker. Moduł WASM to ok. 2,5 MB po kompresji brotli. Najlepszy wybór dla scenariusza „analiza pliku CSV 500 MB w przeglądarce” – przetwarzanie wektorowe i kolumnowe; przy prostych zapytaniach na małych zbiorach i krytycznym czasie startu autorzy sami wskazują lżejsze alternatywy.

### 2.6 GlueSQL

Silnik SQL w Rust (parser `sqlparser-rs`, funkcyjna warstwa wykonawcza, wymienne magazyny), z obsługą WebAssembly i wiązaniami JavaScript; magazyny m.in. w pamięci, w plikach, a w przeglądarce – localStorage, sessionStorage i IndexedDB. Architektonicznie najbliższy Flat-SQL (Rust → `wasm32`, magazyn jako abstrakcja), ale bez wbudowanej synchronizacji Local-First i z pełniejszym (relacyjnym) dialektem.

### 2.7 Turso (wcześniej Limbo)

Przepisanie SQLite od zera w Rust, kompatybilne z formatem pliku SQLite, z asynchronicznym I/O zaprojektowanym m.in. pod WebAssembly i środowiska edge, gdzie blokujące I/O nie jest możliwe. Status beta – obiecujący kierunek (Rust + async I/O + WASM), jeszcze nie w pełni dojrzały.

### 2.8 Bazy na IndexedDB i warstwy synchronizacji

- **Dexie.js, RxDB, PouchDB** – bazy dokumentowe / nakładki na IndexedDB w czystym JavaScript. Brak modułu WASM i języka SQL; zapytania przez API (indeksy, filtry). RxDB i PouchDB mają wbudowaną replikację (PouchDB – protokół CouchDB), co czyni je dojrzałymi rozwiązaniami Local-First dla danych dokumentowych.
- **cr-sqlite, ElectricSQL, PowerSync** – warstwy synchronizacji dla SQLite/PostgreSQL (CRDT na poziomie kolumn w cr-sqlite, replikacja strumieniowa w Electric i PowerSync). Uzupełniają silniki z punktów 2.1–2.4, które same synchronizacji nie mają.

## 3. Zestawienie

| Rozwiązanie | Model / język | Implementacja | Moduł (orientacyjnie) | Trwałość w przeglądarce | Wymagania | Synchronizacja |
|---|---|---|---|---|---|---|
| **Flat-SQL** | płaskie tabele z kluczem, Flat-SQL (bez JOIN) | Rust → `wasm32`, `wasm-bindgen` | **545 KiB / 206 KiB gzip** (zmierzono) | WAL + snapshot w IndexedDB (przyrostowo, group commit) | brak (wątek główny lub Worker) | **wbudowana**: HLC + LWW, outbox, serwer |
| SQLite WASM | relacyjny, SQL | C → Emscripten | ok. 1 MB | OPFS (`opfs`, `opfs-sahpool`), kvvfs | Worker; `opfs` wymaga COOP/COEP | brak (zewnętrzne, np. cr-sqlite) |
| sql.js | relacyjny, SQL | C → Emscripten | 643 KiB / 315 KiB gzip (zmierzono) | brak – eksport całej bazy | brak | brak |
| wa-sqlite | relacyjny, SQL | C → Emscripten (Asyncify/JSPI) | zbliżony do SQLite | VFS IndexedDB / OPFS (stronami) | zależnie od VFS | brak |
| PGlite | relacyjny, PostgreSQL | C → Emscripten | < 3 MB gzip | IndexedDB, OPFS | jedno połączenie | zewnętrzna (Electric) |
| DuckDB-WASM | analityczny (kolumnowy), SQL | C++ → Emscripten | ok. 2,5 MB brotli | pliki/OPFS, głównie odczyt danych | Web Worker | brak |
| GlueSQL | relacyjny, SQL | Rust → `wasm32` | – | localStorage / sessionStorage / IndexedDB | brak | brak |
| Turso | relacyjny, SQL (zgodny z SQLite) | Rust, async I/O | – (beta) | w rozwoju | – | w ekosystemie Turso |
| Dexie / RxDB / PouchDB | dokumentowy, API JS | JavaScript | – | IndexedDB | brak | RxDB, PouchDB: wbudowana |

## 4. Wnioski i miejsce Flat-SQL

1. **Rozwiązania oparte na SQLite/PostgreSQL** dają pełny SQL, ale trwałość w przeglądarce wymaga albo eksportu całej bazy (sql.js – koszt zatwierdzenia rośnie z rozmiarem bazy), albo VFS na OPFS (Worker, czasem COOP/COEP, ograniczenia wielu kart). Synchronizacja jest zawsze osobną warstwą.
2. **DuckDB-WASM** rozwiązuje inny problem (analityka na dużych plikach) i jest właściwym wyborem dla scenariusza „CSV 500 MB”; Flat-SQL celuje w dane operacyjne aplikacji (ekwipunek gry, katalog assetów, sesje, flagi funkcji).
3. **Flat-SQL** wybiera węższy dialekt w zamian za:
   - mały moduł (ok. 2/3 rozmiaru sql.js po kompresji) i brak wymagań środowiskowych,
   - rdzeń bez I/O – ten sam kod w przeglądarce, Node.js, na edge i natywnie (droga do panelu WinUI przez FFI),
   - **przyrostową trwałość** (koszt zatwierdzenia proporcjonalny do zmiany, nie do bazy),
   - **wbudowaną synchronizację** Local-First z deterministycznym rozwiązywaniem konfliktów, możliwą dzięki adresowaniu wierszy kluczem (brak relacji = konflikt zawsze dotyczy jednego klucza).
4. Ograniczenia Flat-SQL względem dojrzałych silników: brak zapytań relacyjnych, całość danych w pamięci, mniej optymalizacji (np. brak indeksów pokrywających – widoczne w benchmarku lookupów po indeksie).

## 5. Wybór silników do benchmarku

| Silnik | Rola w badaniu |
|---|---|
| Flat-SQL (WASM, dwie ścieżki interop) | badany silnik; porównanie budowania wyników w Rust vs JSON |
| Flat-SQL natywnie | koszt samego WebAssembly (ten sam kod, bez WASM) |
| sql.js | dojrzały silnik SQL w WASM, działa identycznie w Node.js i przeglądarce bez Workerów i nagłówków – uczciwe porównanie „SQL w pamięci w WASM”; także punkt odniesienia dla trwałości przez eksport |
| czysty JavaScript (`Map`) | dolna granica: koszt warstwy SQL i granicy WASM↔JS |
| serwer HTTP z tym samym silnikiem | model Cloud-First z symulowanym RTT |

PGlite i DuckDB-WASM nie zostały włączone do pomiarów, bo reprezentują inne klasy (pełny PostgreSQL, OLAP) i wymagają osobnej konfiguracji (Worker, trwałość na OPFS); interfejs adapterów w `bench/engines.js` pozwala dodać je w kolejnym etapie.

## Źródła

- SQLite WASM – trwałość i VFS OPFS: <https://sqlite.org/wasm/doc/tip/persistence.md>
- SQLite w przeglądarce z OPFS (omówienie `opfs` vs `opfs-sahpool`): <https://blog.openreplay.com/sqlite-browser-opfs/>
- sql.js: <https://github.com/sql-js/sql.js>
- wa-sqlite: <https://github.com/rhashimoto/wa-sqlite>
- absurd-sql: <https://github.com/jlongster/absurd-sql>
- PGlite: <https://github.com/electric-sql/pglite>, <https://pglite.dev>
- DuckDB-Wasm: Efficient Analytical SQL in the Browser: <https://duckdb.org/2021/10/29/duckdb-wasm>
- GlueSQL: <https://docs.rs/crate/gluesql/latest>, <https://github.com/gluesql/gluesql>
- Turso (Limbo): <https://turso.tech/blog/introducing-limbo-a-complete-rewrite-of-sqlite-in-rust>
- cr-sqlite: <https://github.com/vlcn-io/cr-sqlite>; ElectricSQL: <https://electric-sql.com>; PowerSync: <https://www.powersync.com>
- Dexie.js: <https://dexie.org>; RxDB: <https://rxdb.info>; PouchDB: <https://pouchdb.com>
- M. Kleppmann i in., „Local-first software: you own your data, in spite of the cloud”, Onward! 2019: <https://www.inkandswitch.com/local-first/>
- S. Kulkarni i in., „Logical Physical Clocks and Consistent Snapshots in Globally Distributed Databases” (HLC), 2014
