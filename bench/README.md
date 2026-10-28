# Odtworzenie badań

Ten katalog zawiera narzędzie, którym zmierzono Flat-SQL i silniki odniesienia. Instrukcja prowadzi przez trzy poziomy sprawdzenia, od kilku sekund do kilku godzin: odtworzenie tabel i wykresów z zapisanych wyników, krótki pomiar kontrolny i pełne powtórzenie pomiarów. Metodykę (hipotezy, obciążenie, statystykę, zagrożenia dla trafności) opisuje [scenariusz badawczy](../docs/research/benchmark-scenario.md), a wyniki [benchmark-results.md](../docs/research/benchmark-results.md). Testy poprawności uruchamia się osobno: `cargo test --workspace` i `npm test` ([tests-results.md](../docs/research/tests-results.md)).

## Co zmierzono

| Pomiar | Co porównuje | Wyniki |
|---|---|---|
| Benchmark 1 | prototyp Flat-SQL i 12 innych silników oraz konfiguracji (sql.js, SQLite WASM, wa-sqlite, PGlite, DuckDB-Wasm, AlaSQL, Dexie.js, PouchDB, RxDB, czysty JavaScript) w Node.js, w przeglądarce i natywnie | [benchmark-results.md §1](../docs/research/benchmark-results.md#1-benchmark-1-prototyp-przed-optymalizacjami), dane: [data/benchmark-1/](../docs/research/data/benchmark-1/) |
| Benchmark 2 | prototyp i wersję po optymalizacjach, wszystkie konfiguracje Flat-SQL, na przemian tym samym narzędziem; silników odniesienia nie mierzono ponownie | [benchmark-results.md §2](../docs/research/benchmark-results.md#2-benchmark-2-wersja-po-optymalizacjach), dane: [data/benchmark-2/](../docs/research/data/benchmark-2/) |

Każdy pomiar to tabela 100 000 wierszy i 10 powtórzeń w nowych procesach, a wynikiem jest mediana z kwartylami. Wszystkie silniki muszą zwrócić te same wyniki zapytań: narzędzie liczy sumę kontrolną każdej odpowiedzi.

## Wersje silnika

| Tag | Co to jest | Gdzie mierzony |
|---|---|---|
| `v1-prototyp` | prototyp przed optymalizacjami | Benchmark 1; Benchmark 2 jako wersja „przed” |
| `v2-optymalizacja` | wersja po 15 optymalizacjach silnika i granicy JavaScript–WebAssembly | Benchmark 2 jako wersja „po” |

Narzędzie pochodzi zawsze z bieżącego kodu, a mierzona wersja silnika z osobnej kopii roboczej w `bench/versions/<tag>/`, którą tworzy `node bench/build-version.js <tag>`. Dzięki temu obie wersje mierzy to samo narzędzie. Pomiary pojedynczych optymalizacji używały wersji pośrednich; ich commity wymienia [data/optimization/README.md](../docs/research/data/optimization/README.md#wersje-mierzone-w-czasie-optymalizacji).

## Wymagania

| Narzędzie | Wersja w pomiarach | Sprawdzenie |
|---|---|---|
| Git | dowolna | `git --version` |
| Node.js | 24.21.0 (wymagane co najmniej 22) | `node -v` |
| Rust z celem `wasm32-unknown-unknown` | 1.99.0; plik `rust-toolchain.toml` każe `rustup` zainstalować tę wersję z celem przy pierwszym `cargo` | `rustc -V` |
| wasm-pack | 0.15.0 (`cargo install wasm-pack`) | `wasm-pack -V` |
| Microsoft Edge | 156 (tylko pomiary w przeglądarce; Chrome przez `--browser-exe`) | – |
| Build Tools for Visual Studio | linker MSVC, tylko w Windows | – |

Sprzęt, na którym wykonano pomiary: laptop z AMD Ryzen 5 4600H (6 rdzeni, 12 wątków), 16 GB RAM i dyskiem SSD NVMe, Windows 11 Home, zasilanie sieciowe; koszt zapisu plików dodatkowo w Linuksie (WSL 2, Ubuntu 24.04). Pakiety npm zajmują około 0,5 GB, a każda zbudowana wersja silnika w `bench/versions/` około 130 MB.

| Krok | Czas na sprzęcie z pomiarów |
|---|---|
| poziom 1: tabele i wykresy z zapisanych wyników | kilka sekund |
| poziom 2: pomiar kontrolny | około minuty (plus przygotowanie) |
| poziom 3: Benchmark 2 | około 45 minut pomiaru plus zbudowanie dwóch wersji |
| poziom 3: Benchmark 1 | około 6,5 godziny (5,6 godziny przeglądarka) |

## Przygotowanie

```bash
git clone https://github.com/best-team-8/dbms-rust-wasm.git
```

```bash
cd dbms-rust-wasm
```

```bash
git fetch --tags
```

```bash
npm ci
```

```bash
npm run build
```

```bash
cargo build --release -p flatsql-cli
```

Po tych krokach `pkg/` zawiera moduł WebAssembly, a `target/release/` program `flatsql`. Poziom 1 nie wymaga żadnego z nich, tylko Node.js.

## Poziom 1: tabele i wykresy z zapisanych wyników

Repozytorium zawiera surowe wyniki każdego zmierzonego procesu. Skrypty raportów korzystają tylko z modułów wbudowanych Node.js i tworzą z nich wszystkie tabele i wykresy pracy:

```bash
node docs/research/data/benchmark-1/make-report.mjs
```

```bash
node docs/research/data/benchmark-2/make-report.mjs
```

```bash
node docs/research/data/benchmark-2/engines-report.mjs
```

```bash
git diff --stat -- docs/research/data
```

Oczekiwany wynik: ostatnie polecenie nie wypisuje żadnego pliku, czyli tabele (`tables.md`, `engines.md`), wartości zagregowane (`summary.json`, `engines.json`) i wykresy (`figures/*.svg`) są identyczne z opublikowanymi. W Windows `git status` może mimo to pokazać pliki jako zmienione, bo skrypty zapisują końce linii LF; `git diff` porównuje treść.

## Poziom 2: pomiar kontrolny

Jedno uruchomienie na małej tabeli sprawdza, że narzędzie, moduł i wszystkie silniki działają na danym komputerze. Wartości z tego pomiaru nie są porównywalne z wynikami pracy.

```bash
node bench/run.js --targets node --runs 1 --rows 5000 --kinds workload --out bench/results/kontrola
```

Oczekiwany wynik: tabela czasów wszystkich silników w konsoli i w `bench/results/kontrola/summary.md`, kod wyjścia 0. Niezgodna suma kontrolna któregoś silnika pojawia się w `summary.md` jako `checksum mismatch` i kończy polecenie kodem 1. Ten sam pomiar wykonuje CI przy każdej zmianie.

## Poziom 3: pełne powtórzenie

Przed pomiarem trzeba zamknąć inne aplikacje obciążające procesor, podłączyć zasilanie i wyłączyć usypianie komputera. Przerwany pomiar wznawia się tym samym poleceniem z tym samym `--out`: pliki wyników, które już istnieją, są pomijane.

### Benchmark 2

```bash
node bench/build-version.js v1-prototyp
```

```bash
node bench/build-version.js v2-optymalizacja
```

```bash
node bench/compare.js --before bench/versions/v1-prototyp --after bench/versions/v2-optymalizacja --out bench/results/benchmark-2 --runs 10 --targets node,native,browser --kinds workload,cold,memory,network,fsync,checkpoint --engines flatsql,flatsql-json,flatsql-idb,flatsql-idb-strict,flatsql-worker,flatsql-worker-idb
```

```bash
node docs/research/data/benchmark-2/make-report.mjs bench/results/benchmark-2
```

`compare.js` mierzy obie wersje na przemian, każdą w 10 nowych procesach dla każdej konfiguracji, i zapisuje `compare.md` z medianami, zmianą i tym, czy 95% przedziały ufności średnich są rozłączne. Skrypt raportu tworzy w tym samym katalogu `tables.md` i wykresy do porównania z [data/benchmark-2/tables.md](../docs/research/data/benchmark-2/tables.md). Pomiar zapisu plików w Linuksie opisuje [data/benchmark-2/README.md](../docs/research/data/benchmark-2/README.md).

### Benchmark 1

Benchmark 1 zmierzono narzędziem z wersji `v1-prototyp`, więc dokładne powtórzenie uruchamia narzędzie z tej samej kopii roboczej, w której jest zbudowany silnik:

```bash
node bench/build-version.js v1-prototyp
```

```bash
cd bench/versions/v1-prototyp
```

```bash
npm ci
```

```bash
node bench/run.js --targets node,native,browser --runs 10 --out bench/results/benchmark-1
```

```bash
cd ../../..
```

```bash
node docs/research/data/benchmark-1/make-report.mjs bench/versions/v1-prototyp/bench/results/benchmark-1
```

Tabele trafiają do `bench/versions/v1-prototyp/bench/results/benchmark-1/tables.md` i porównuje się je z [data/benchmark-1/tables.md](../docs/research/data/benchmark-1/tables.md). Analizy pomocnicze (plany zapytań, pamięć, profil, rozbicie kosztów) uruchamia się według [data/benchmark-1/README.md](../docs/research/data/benchmark-1/README.md).

## Porównanie z opublikowanymi wynikami

- Na innym komputerze czasy będą inne. Porównuje się stosunki (Flat-SQL wobec SQLite, prototyp wobec wersji po optymalizacjach, WebAssembly wobec kodu natywnego) i kolejność silników; same mikrosekundy zależą od sprzętu.
- Sumy kontrolne muszą się zgadzać. Różnica oznacza błąd silnika albo adaptera; rozrzut pomiaru jej nie wyjaśnia.
- Rozrzut jest opisany w danych: mediana współczynnika zmienności wszystkich par silnik–operacja w Benchmarku 1 wynosi 4,1%, a pojedyncze zapisy w przeglądarce potrafią trwać kilka razy dłużej niż zwykle. Mediana jest na takie uruchomienia odporna.
- Zmianę między wersjami uznaje się za wykazaną, gdy 95% przedziały ufności średnich są rozłączne (kolumna w `compare.md`).

## Wyniki pracy a pliki i polecenia

| Wynik | Opis | Dane | Polecenie |
|---|---|---|---|
| Benchmark 1: tabele i wykresy | [benchmark-results.md §1](../docs/research/benchmark-results.md#1-benchmark-1-prototyp-przed-optymalizacjami) | [data/benchmark-1/](../docs/research/data/benchmark-1/) | `make-report.mjs` |
| rozbicie kosztów silnika | [§1.12](../docs/research/benchmark-results.md#112-rozbicie-kosztów-silnika-pomiar-pomocniczy) | [data/benchmark-1/analysis/](../docs/research/data/benchmark-1/analysis/) | [data/benchmark-1/README.md](../docs/research/data/benchmark-1/README.md) |
| Benchmark 2: prototyp a wersja po optymalizacjach | [§2](../docs/research/benchmark-results.md#2-benchmark-2-wersja-po-optymalizacjach) | [data/benchmark-2/](../docs/research/data/benchmark-2/), wykresy 01–06 | `make-report.mjs` |
| Benchmark 2: na tle silników z Benchmarku 1 | [§2.9](../docs/research/benchmark-results.md#29-flat-sql-po-optymalizacjach-na-tle-innych-silników) | `engines.md`, wykresy 07–12 | `engines-report.mjs` |
| pojedyncze optymalizacje | [optimization-results.md](../docs/research/optimization-results.md) | [data/optimization/](../docs/research/data/optimization/) | [data/optimization/README.md](../docs/research/data/optimization/README.md) |
| testy poprawności | [tests-results.md](../docs/research/tests-results.md) | – | `cargo test --workspace`, `npm test` |

## Typowe problemy

| Objaw | Rozwiązanie |
|---|---|
| `build-version.js` nie znajduje tagu | `git fetch --tags` |
| Windows: `link.exe` nie znaleziony przy `cargo build` | zainstalować Build Tools for Visual Studio z obsługą C++ |
| przeglądarka się nie uruchamia | domyślnie używany jest zainstalowany Microsoft Edge (`--browser-channel msedge`); inna przeglądarka: `--browser-exe ŚCIEŻKA` |
| pomiar przerwany przez uśpienie komputera | wyłączyć usypianie i uruchomić to samo polecenie z tym samym `--out` |
| duży rozrzut wyników | zamknąć inne aplikacje; manifest pomiaru zapisuje obciążenie procesora przed jego rozpoczęciem |

## Budowa narzędzia

Ta część jest dla osoby, która zmienia narzędzie albo dodaje silnik.

| Plik | Rola |
|---|---|
| `suite.js` | obciążenie (`WORKLOAD = 'benchmark-2'`: operacje Benchmarku 1 i na końcu `insert_many`): dane, zapytania, ziarna parametrów, suma kontrolna wyników, przebieg jednego silnika |
| `stats.js` | mediana, kwartyle, średnia z 95% przedziałem ufności |
| `engines/` | adaptery silników o wspólnym interfejsie; rejestr konfiguracji i grup warunków w `engines/index.js` |
| `node/child.js` | jeden mierzony proces Node.js (obciążenie, zimny start, pamięć, sieć, `fsync`) |
| `browser/` | strona pomiarowa, Worker, serwer z nagłówkami COOP/COEP, sterowanie przeglądarką przez playwright-core, tryb ręczny |
| `network.js` | Local-First a Cloud-First: zapytanie lokalne i przez `POST /api/query` |
| `wasm-memory.js` | rozmiar pamięci liniowej WebAssembly dowolnego silnika |
| `run.js` | runner: powtarza pomiary w nowych procesach, zapisuje surowe wyniki i `summary.md` |
| `compare.js` | dwie wersje silnika na przemian tym samym runnerem, tabela przed i po z kryterium przedziałów ufności i zgodnością wyników |
| `build-version.js` | kopia robocza wskazanej wersji z gotowym `pkg/` i CLI |
| `code-tree.js` | skrót plików kodu silnika (`crates/`, `bridge/`, `server/`) katalogu roboczego albo commita; identyfikuje mierzoną wersję przed zatwierdzeniem |
| `../crates/flatsql-cli/src/bench.rs` | to samo obciążenie natywnie (`flatsql bench`), ta sama suma kontrolna |

Opcje `run.js`: `--targets node,native,browser`, `--runs N` (domyślnie 10), `--rows N` (domyślnie 100 000), `--kinds workload,cold,memory,network,fsync,checkpoint`, `--engines id,…` (identyfikatory z `engines/index.js`), `--flatsql KATALOG` (mierzona wersja), `--label NAZWA`, `--out KATALOG`, `--browser-channel msedge` albo `--browser-exe ŚCIEŻKA`, `--headed`. Każdy pomiar w przeglądarce to nowy proces z pustym profilem.

Adaptery Flat-SQL używają tylko API obecnego w `v1-prototyp`, z wyjątkiem operacji `insert_many`, która korzysta z `executeMany`, gdy mierzona wersja je ma, a w innym wypadku z pętli `execute`. Natywny benchmark jest częścią mierzonej wersji (`flatsql bench` z jej `target/release`), więc runner przerywa pomiar, gdy CLI zgłasza obciążenie spoza listy `COMPATIBLE_WORKLOADS` (prototyp zgłasza `benchmark-1`, zgodne z `benchmark-2` w operacjach, które oba zawierają). Pomiar pojedynczej zmiany silnika opisuje [data/optimization/README.md](../docs/research/data/optimization/README.md).

Tryb ręczny (inna przeglądarka, telefon): `node bench/browser/serve.js` wypisuje adres strony `manual.html`, na której wybiera się zadanie i silnik; wynik zapisuje się jako plik JSON w formacie surowych wyników (`raw/browser/<rodzaj>/<silnik>/NN.json`). Precyzyjny zegar, OPFS i izolacja strony wymagają bezpiecznego kontekstu, czyli adresu `localhost`. Telefon z Androidem podłączony przez USB: `adb reverse tcp:8090 tcp:8090` i `adb reverse tcp:8091 tcp:8091`, potem `http://localhost:8090/bench/browser/manual.html?sync=http%3A%2F%2Flocalhost%3A8091` w Chrome.

Dodanie silnika: adapter w `engines/` eksportuje `create(env, options)` i zwraca obiekt z metodami `open`, `load`, `prepare`, `query`, `write`, `verify`, `close` (opcjonalnie `finalize`, `snapshot` / `restore`, a dla magazynów trwałych `commit`, `commitGroup`, `reopen`, `checkpoint`). Wyniki zapytań to tablice wierszy, a każdy wiersz to tablica wartości w kolejności kolumn zapytania. Konfigurację dopisuje się do `ENGINES` w `engines/index.js` z grupą warunków, wymaganiami i środowiskami, a wersję pakietu przypina dokładnie w `package.json`. Sumy kontrolne pokażą, czy odwzorowanie zapytań jest poprawne.
