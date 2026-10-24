# Analiza porównawcza silników baz danych działających w WebAssembly

Celem analizy jest umiejscowienie Flat-SQL wśród rozwiązań, które pozwalają wykonywać zapytania do bazy danych po stronie klienta (przeglądarka, węzeł edge), oraz uzasadnienie wyboru silników porównywanych w benchmarkach ([scenariusz](benchmark-scenario.md), [wyniki](benchmark-results.md)). To, kto i do czego faktycznie używa tych silników, opisuje [real-world-usage.md](real-world-usage.md); literaturę – [references/README.md](../references/README.md) (identyfikatory w nawiasach, np. [D1]).

Stan na październik 2026 (rewizja 24.10). Rozmiary oznaczone „zmierzono” pochodzą z Benchmarku 1 ([benchmark-results.md](benchmark-results.md)): wszystkie pliki pakietu pobrane przy zimnym starcie w przeglądarce (moduł WebAssembly razem z kodem JavaScript), bez kompresji i po kompresji gzip; pozostałe są deklaracjami autorów i mają charakter orientacyjny.

## 1. Kryteria

| Kryterium | Dlaczego ma znaczenie w modelu Local-First |
|---|---|
| Model danych i język zapytań | relacyjny OLTP, analityczny (kolumnowy), dokumentowy – decyduje o klasie zastosowań |
| Implementacja i toolchain WASM | C + Emscripten vs Rust + `wasm-bindgen`; wpływa na rozmiar, integrację z JS i możliwość pracy natywnej |
| Rozmiar modułu | czas pobrania i kompilacji przy pierwszym uruchomieniu; Notion ładuje SQLite asynchronicznie, bo kilkaset KB opóźniało start strony |
| Trwałość w przeglądarce | brak, eksport całej bazy, VFS na IndexedDB, VFS na OPFS, snapshot na żądanie; koszt pojedynczego trwałego zatwierdzenia; gwarancja po zatwierdzeniu (`relaxed`/`strict` w IndexedDB – [Q1]) |
| Wymagania środowiska | Web Worker, nagłówki COOP/COEP (`SharedArrayBuffer`), JSPI/Asyncify |
| Wiele kart | czy dwie karty mogą bezpiecznie używać tej samej bazy (w praktyce obowiązkowe – [real-world-usage.md §3](real-world-usage.md#3-wnioski-projektowe-z-cudzych-wdrożeń)) |
| Synchronizacja | wbudowana, przez zewnętrzną warstwę, brak |
| Dojrzałość | stabilność, utrzymanie projektu |

## 2. Przegląd rozwiązań

### 2.1 SQLite (oficjalny build WASM)

Projekt SQLite publikuje własny build WebAssembly (`@sqlite.org/sqlite-wasm`, Emscripten; ok. 0,8–1 MB bez kompresji) z trwałością w **Origin Private File System** ([Q7]). Dwa VFS: `opfs` – wymaga nagłówków COOP/COEP (`SharedArrayBuffer`), oraz `opfs-sahpool` – bez wymagań co do nagłówków, najszybszy, ale tylko z jednym połączeniem. Synchroniczne uchwyty OPFS istnieją wyłącznie w Web Workerach, więc baza na OPFS nie działa w wątku głównym. Pełny SQL, dojrzały silnik ([D2]); brak wbudowanej synchronizacji. Wdrożenie w Notion wymagało `SharedWorker` i Web Locks, żeby kilka kart nie uszkodziło bazy ([Q12]).

### 2.2 sql.js

SQLite skompilowany Emscriptenem z wygodnym API JavaScript; działa w przeglądarce i Node.js. Baza żyje w pamięci, a trwałość sprowadza się do `db.export()` – serializacji **całego** pliku bazy – i zapisania go przez aplikację. Zmierzono: 643 KiB (315 KiB gzip). Najpopularniejszy sposób uruchamiania SQL w przeglądarce (piaskownice, nauka SQL), dlatego wybrany jako główny punkt odniesienia.

### 2.3 wa-sqlite i absurd-sql

`wa-sqlite` to build SQLite z asynchronicznymi VFS (Asyncify, nowsze wersje JSPI) i gotowymi VFS na IndexedDB i OPFS – zapisuje bazę stronami, więc trwałe zatwierdzenie nie wymaga eksportu całości. `absurd-sql` (2021) pokazał to samo podejście dla sql.js (strony bazy jako bloki w IndexedDB, Worker z `SharedArrayBuffer`); używał go Actual Budget, projekt nie jest już rozwijany, a jego rolę przejęły VFS OPFS.

### 2.4 PGlite

PostgreSQL skompilowany do WASM (ElectricSQL), poniżej 3 MB gzip. Trwałość przez wymienny system plików: pamięć, IndexedDB, OPFS lub system plików w Node/Bun. Pełny dialekt PostgreSQL z rozszerzeniami (np. pgvector), jedno połączenie, synchronizacja przez zewnętrzne narzędzia (Electric). Używany głównie tam, gdzie liczy się zgodność z PostgreSQL: piaskownice, interaktywna dokumentacja, testy bez Dockera, wyszukiwanie semantyczne.

### 2.5 DuckDB-Wasm

Kolumnowy silnik analityczny [D1]: odczytuje Parquet, CSV, JSON i Arrow z plików lub HTTP fragmentami; API asynchroniczne przez Web Worker; wyniki przekazywane jako Arrow IPC czytane przez JS wprost ze sterty WASM. Moduł ok. 2,5–2,8 MB po kompresji. Właściwy wybór dla scenariusza „analiza pliku CSV 500 MB w przeglądarce” (pulpity BI, podgląd Parquet); przy prostych zapytaniach na małych zbiorach i krytycznym czasie startu autorzy wskazują lżejsze alternatywy.

### 2.6 Zeta-Lite

Przeglądarkowa wersja serwerowego silnika Zeta [D5] (preprint, wrzesień 2026): ten sam parser, planer i wykonawca skompilowany do `wasm32-unknown-unknown` z `wasm-bindgen` (2,87 MB gzip). MVCC w jednym wątku, rozgałęzianie bazy, dialekt PostgreSQL, wektory; 268–315 tys. odczytów punktowych/s. Trwałość tylko przez snapshot do OPFS na żądanie – zmiany po ostatnim snapshocie giną przy zamknięciu karty. Kod zamknięty. Architektonicznie najbliższy Flat-SQL („jeden rdzeń dla serwera i przeglądarki”, ten sam toolchain), z przeciwnymi wyborami: pełny SQL i duży moduł zamiast przyrostowej trwałości i synchronizacji.

### 2.7 GlueSQL

Silnik SQL w Rust (parser `sqlparser-rs`, funkcyjna warstwa wykonawcza, wymienne magazyny) z obsługą WebAssembly; w przeglądarce magazyny localStorage, sessionStorage i IndexedDB. Podobny do Flat-SQL pod względem toolchainu i abstrakcji magazynu, ale bez wbudowanej synchronizacji i z pełniejszym (relacyjnym) dialektem.

### 2.8 Turso

Przepisanie SQLite w Rust (wcześniej Limbo), zgodne z formatem pliku SQLite, z asynchronicznym I/O i współbieżnymi zapisami MVCC, z pakietem przeglądarkowym (WASM + OPFS, API asynchroniczne). W 2026 r. nadal w fazie beta. Osobno Turso rozwija repliki osadzone libSQL – lokalną kopię bazy głównej z synchronizacją, używaną m.in. w aplikacjach mobilnych, kasach i zbieraniu danych w terenie.

### 2.9 AlaSQL

Silnik SQL napisany w czystym JavaScripcie, bez WebAssembly: tabele są tablicami obiektów JS w pamięci, zapytania są kompilowane do funkcji JavaScript (`alasql.compile`). Obsługuje szeroki dialekt (także złączenia i zapytania na tablicach obiektów, plikach CSV i XLSX), działa w przeglądarce i Node.js; trwałość zapewniają opcjonalne adaptery magazynów (localStorage, IndexedDB), których benchmark nie mierzy. Jest punktem odniesienia „SQL bez WASM”: pokazuje, ile daje kompilacja silnika do WebAssembly w porównaniu z interpretacją zapytań w samym JavaScripcie.

### 2.10 Bazy na IndexedDB i warstwy synchronizacji

- **Dexie.js, RxDB, PouchDB** – bazy dokumentowe / nakładki na IndexedDB w JavaScript, bez modułu WASM i bez SQL. RxDB i PouchDB mają wbudowaną replikację (PouchDB – protokół CouchDB, np. w Community Health Toolkit).
- **cr-sqlite, ElectricSQL, PowerSync** – warstwy synchronizacji dla SQLite/PostgreSQL (CRDT na poziomie kolumn w cr-sqlite, replikacja strumieniowa w Electric i PowerSync). Uzupełniają silniki z punktów 2.1–2.4, które same synchronizacji nie mają. W literaturze odpowiadają im CRR [L24] (rejestr LWW per atrybut nad bazą relacyjną) i Simba [L27] (usługa synchronizacji tabel z atomowością per wiersz).

## 3. Zestawienie

| Rozwiązanie | Model / język | Implementacja | Moduł | Trwałość w przeglądarce | Wymagania | Synchronizacja |
|---|---|---|---|---|---|---|
| **Flat-SQL** | płaskie tabele z kluczem, Flat-SQL (bez JOIN) | Rust → `wasm32`, `wasm-bindgen` | **622 KiB / 230 KiB gzip** (zmierzono) | WAL + snapshot w IndexedDB (przyrostowo, group commit) | brak (wątek główny lub Worker) | **wbudowana**: HLC + LWW, outbox, serwer |
| SQLite WASM | relacyjny, SQL | C → Emscripten | 1532 KiB / 583 KiB gzip (zmierzono, z VFS OPFS) | OPFS (`opfs`, `opfs-sahpool`), kvvfs | Worker; `opfs` wymaga COOP/COEP | brak (zewnętrzne, np. cr-sqlite) |
| sql.js | relacyjny, SQL | C → Emscripten | 687 KiB / 331 KiB gzip (zmierzono) | brak – eksport całej bazy | brak | brak |
| AlaSQL | relacyjny, SQL | JavaScript (bez WASM) | 508 KiB / 111 KiB gzip (zmierzono) | opcjonalne adaptery (localStorage, IndexedDB) | brak | brak |
| wa-sqlite | relacyjny, SQL | C → Emscripten (Asyncify/JSPI) | 2440 KiB / 821 KiB gzip (zmierzono, build Asyncify z `IDBBatchAtomicVFS`) | VFS IndexedDB / OPFS (stronami) | zależnie od VFS | brak |
| PGlite | relacyjny, PostgreSQL | C → Emscripten | 17 MiB / 5,6 MiB gzip (zmierzono, z obrazem systemu plików `pglite.data`) | IndexedDB, OPFS | jedno połączenie | zewnętrzna (Electric) |
| DuckDB-Wasm [D1] | analityczny (kolumnowy), SQL | C++ → Emscripten | 34 MiB / 7,6 MiB gzip (zmierzono, build `eh` z Apache Arrow) | pliki/OPFS, głównie odczyt danych | Web Worker | brak |
| Zeta-Lite [D5] | relacyjny, PostgreSQL, MVCC | Rust → `wasm32`, `wasm-bindgen` | 2,87 MB gzip (deklaracja autora, niezmierzone) | snapshot do OPFS na żądanie | – | brak |
| GlueSQL | relacyjny, SQL | Rust → `wasm32` | – | localStorage / sessionStorage / IndexedDB | brak | brak |
| Turso | relacyjny, SQL (zgodny z SQLite) | Rust, async I/O | – (beta) | OPFS | API asynchroniczne | w ekosystemie Turso |
| Dexie / RxDB / PouchDB | dokumentowy, API JS | JavaScript | Dexie 49 KiB, PouchDB 42 KiB, RxDB z magazynem Dexie 80 KiB gzip (zmierzono, po minifikacji) | IndexedDB | brak | RxDB, PouchDB: wbudowana |

## 4. Wnioski i miejsce Flat-SQL

1. **Rozwiązania oparte na SQLite/PostgreSQL** dają pełny SQL, ale trwałość w przeglądarce wymaga eksportu całej bazy (sql.js – koszt zatwierdzenia rośnie z rozmiarem bazy), VFS na OPFS (Worker, czasem COOP/COEP, ograniczenia wielu kart) albo snapshotu na żądanie (Zeta-Lite – utrata zmian po ostatnim snapshocie). Synchronizacja jest zawsze osobną warstwą.
2. **DuckDB-Wasm** rozwiązuje inny problem (analityka dużych plików) i jest właściwym wyborem dla scenariusza „CSV 500 MB”; **PGlite** – tam, gdzie liczy się zgodność z PostgreSQL. Flat-SQL nie powinien z nimi konkurować.
3. **Flat-SQL** celuje w dane operacyjne aplikacji – zapis stanu gier, dane zbierane w terenie, kopie danych serwera, konfigurację – i wybiera węższy dialekt w zamian za:
   - mały moduł (230 KiB po kompresji: ok. 2/3 sql.js, 2,5 raza mniej niż SQLite WASM, 12–34 razy mniej niż Zeta-Lite, PGlite i DuckDB-Wasm) i brak wymagań środowiskowych,
   - rdzeń bez I/O – ten sam kod w przeglądarce, Node.js, na edge i natywnie (droga do panelu WinUI przez interfejs C),
   - **przyrostową trwałość** (koszt zatwierdzenia proporcjonalny do zmiany, nie do bazy) – w przeciwieństwie do sql.js i Zeta-Lite,
   - **wbudowaną synchronizację** Local-First z deterministycznym rozwiązywaniem konfliktów, prostą dzięki adresowaniu wierszy kluczem.
4. Ograniczenia Flat-SQL względem dojrzałych silników: brak zapytań relacyjnych, całość danych w pamięci, mniej optymalizacji (np. brak indeksów pokrywających), synchroniczny checkpoint całej bazy ([S10]), brak obsługi wielu kart w bibliotece (dziś tylko blokada w demo).
5. Analiza wdrożeń ([real-world-usage.md](real-world-usage.md)) potwierdza tę niszę: najbliższym odpowiednikiem architektury jest Actual Budget (SQLite w przeglądarce, HLC, własny serwer synchronizacji), a najmocniejsze dowody potrzeby dają aplikacje terenowe offline (PouchDB, PowerSync).

## 5. Wybór silników do benchmarku

Benchmark porównuje silniki w grupach o tych samych warunkach uruchomienia ([benchmark-scenario.md §4](benchmark-scenario.md#4-silniki-i-grupy-warunków-uruchomienia)). Wersje są dokładnie przypięte w `package.json`.

| Silnik (pakiet, wersja) | Grupy | Rola w badaniu |
|---|---|---|
| Flat-SQL (WASM, dwie ścieżki przekazywania wyników) | pamięć, trwały zapis, Worker | badany silnik; wyniki jako obiekty JS budowane w Rust i jako JSON (H3) |
| Flat-SQL natywnie (`flatsql bench`) | natywnie | koszt samego WebAssembly: ten sam kod bez WASM i bez granicy JS (H2) |
| sql.js 1.14.2 | pamięć, trwały zapis | SQLite w pamięci WASM; trwałość tylko przez eksport całej bazy (H5) |
| SQLite WASM 3.54.1 (`@sqlite.org/sqlite-wasm`) | pamięć (`:memory:`), Worker (`opfs-sahpool`) | oficjalny build SQLite; z OPFS uczciwy punkt odniesienia dla trwałości, bez którego teza dotyczyłaby tylko sql.js [B3], [Q7] |
| wa-sqlite 2.0.8 (`@journeyapps/wa-sqlite`, utrzymywana gałąź projektu wa-sqlite) | trwały zapis | SQLite zapisujący strony bazy w IndexedDB (`IDBBatchAtomicVFS`), w wątku głównym; ten sam magazyn co Flat-SQL |
| PGlite 0.5.11 | pamięć, trwały zapis (IndexedDB), Worker (OPFS) | PostgreSQL w WASM; rozmiar i zimny start dużego silnika |
| DuckDB-Wasm 1.32.0 | pamięć, Worker | silnik kolumnowy: przewaga w skanach i agregacji, słabość w odczytach po kluczu i pojedynczych zapisach wyznacza granicę zastosowań Flat-SQL [D1] |
| AlaSQL 4.21.0 | pamięć | SQL w czystym JavaScripcie, bez WebAssembly |
| Dexie.js 4.4.7 | trwały zapis | nakładka na IndexedDB: trwały zapis bez WASM i bez SQL |
| PouchDB 9.0.0, RxDB 17.6.0 (magazyn Dexie) | trwały zapis | bazy local-first z synchronizacją, konkurencja w aplikacjach offline; zapytania odwzorowane na ich API |
| czysty JavaScript (`Map`) | pamięć | dolna granica: koszt warstwy SQL i granicy WASM↔JS |
| serwer HTTP z tym samym silnikiem | – | model Cloud-First z symulowanym RTT (H1) |

Niemierzone: Zeta-Lite [D5], bo kod jest zamknięty i nie ma modułu do pobrania (porównanie opiera się na deklaracjach z preprintu); absurd-sql, bo projekt nie jest rozwijany, a jego podejście reprezentują wa-sqlite i `opfs-sahpool`; GlueSQL i Turso są opisane w przeglądzie, ale nie należą do zestawu punktów odniesienia tej wersji benchmarku (Turso w przeglądarce jest w fazie beta).

## Źródła

- Literatura: [D1] DuckDB-Wasm, [D2] SQLite, [D5] Zeta-Lite, [L24] CRR, [L27] Simba, [S10] checkpointy, [B3] metodyka – [references/README.md](../references/README.md).
- SQLite WASM – trwałość i VFS OPFS: <https://sqlite.org/wasm/doc/tip/persistence.md> ([Q7])
- SQLite w przeglądarce z OPFS (`opfs` vs `opfs-sahpool`): <https://blog.openreplay.com/sqlite-browser-opfs/>
- sql.js: <https://github.com/sql-js/sql.js>
- wa-sqlite: <https://github.com/rhashimoto/wa-sqlite>; absurd-sql: <https://github.com/jlongster/absurd-sql>
- PGlite: <https://github.com/electric-sql/pglite>, <https://pglite.dev>
- DuckDB-Wasm: <https://duckdb.org/2021/10/29/duckdb-wasm>
- GlueSQL: <https://github.com/gluesql/gluesql>
- Turso: <https://turso.tech/blog/introducing-limbo-a-complete-rewrite-of-sqlite-in-rust>, <https://turso.tech/blog/introducing-turso-in-the-browser>
- cr-sqlite: <https://github.com/vlcn-io/cr-sqlite>; ElectricSQL: <https://electric-sql.com>; PowerSync: <https://www.powersync.com>
- Dexie.js: <https://dexie.org>; RxDB: <https://rxdb.info>; PouchDB: <https://pouchdb.com>
- Notion – SQLite WASM, `SharedWorker` i Web Locks: <https://www.notion.com/blog/how-we-sped-up-notion-in-the-browser-with-wasm-sqlite> ([Q12])
