# Benchmark 1: dane

Surowe wyniki pomiaru prototypu Flat-SQL (tag `prototyp`) według [metodyki](../../benchmark-scenario.md). Opis wyników: [benchmark-results.md §1](../../benchmark-results.md#1-benchmark-1-prototyp-przed-optymalizacjami).

| Ścieżka | Zawartość |
|---|---|
| `manifest.json` | środowisko (procesor, system, Node.js, przeglądarka, Rust), wersje pakietów, mierzona wersja Flat-SQL, opcje runnera, rejestr konfiguracji silników, pomiar w Linuksie (`linux`) |
| `raw/<środowisko>/<rodzaj>/<silnik>/NN.json` | wynik jednego procesu; środowiska `node`, `native`, `browser`, `node-linux`, `native-linux`; rodzaje `workload`, `cold`, `memory`, `checkpoint`, `network`, `fsync` |
| `make-report.mjs` | samodzielny skrypt (tylko moduły wbudowane Node.js): czyta `raw/` i tworzy `tables.md`, `summary.json` i `figures/*.svg` |
| `tables.md` | wszystkie tabele: mediana z 10 uruchomień, w nawiasie kwartyle; zgodność sum kontrolnych |
| `summary.json` | wartości zagregowane, także średnia z 95% przedziałem ufności i współczynnik zmienności |
| `figures/` | wykresy SVG do pracy |
| `analysis/` | skrypty i wyniki analizy słabych stron: plany zapytań (`explain`), pamięć według sposobu wczytania (`memory`), profil V8 odczytu po kluczu (`profile`), natywne rozbicie kosztów silnika i liczba alokacji na operację (`cost-split`, pakiet Rust z programami `cost-split` i `allocations`; opis: [benchmark-results.md §1.12](../../benchmark-results.md#112-rozbicie-kosztów-silnika-pomiar-pomocniczy)) |

Odtworzenie tabel i wykresów:

```bash
node docs/research/data/benchmark-1/make-report.mjs
```

Odtworzenie analiz (wymaga zbudowanego `pkg/` mierzonej wersji, domyślnie bieżącego katalogu):

```bash
node docs/research/data/benchmark-1/analysis/explain.mjs
node docs/research/data/benchmark-1/analysis/memory.mjs
node docs/research/data/benchmark-1/analysis/profile.mjs
cargo run --release --manifest-path docs/research/data/benchmark-1/analysis/cost-split/Cargo.toml --target-dir target/analysis --bin cost-split
cargo run --release --manifest-path docs/research/data/benchmark-1/analysis/cost-split/Cargo.toml --target-dir target/analysis --bin allocations
```

`cost-split` mierzy bieżący rdzeń z `crates/flatsql-core`. Program działa około 15 s (pierwsza kompilacja około 25 s); wynik zależy od obciążenia procesora, więc uruchamia się go bez innych obciążających aplikacji.

Po Benchmarku 1 program służy też do pomiaru optymalizacji ([optimization-results.md](../../optimization-results.md)), więc zmieniono go w trzech miejscach. Doszły pomiary `COUNT(*)` po indeksie (silnik wobec odczytu samego indeksu), czasu przygotowania każdej instrukcji obciążenia, a w programie `allocations` także `COUNT(*)`, zakresu klucza i przygotowania instrukcji; wyniki z pliku `analysis/cost-split.txt` pochodzą z wersji sprzed tych dodatków. Skrypt `run.mjs` buduje program z rdzeniem dowolnej wersji, np. worktree poprzedniego stanu, żeby te same operacje zmierzyć przed zmianą i po niej:

```bash
node docs/research/data/benchmark-1/analysis/cost-split/run.mjs --flatsql bench/versions/prototyp [--bin allocations] [--out PLIK]
```

Części programu, które korzystają z wewnętrznego API rdzenia zmienionego przez optymalizację, wybiera `build.rs` na podstawie źródeł mierzonego rdzenia; każda taka część jest opisana w komentarzu programu. Od wersji, w której outbox jest prowadzony dopiero po włączeniu synchronizacji, stan „z outboxem” program uzyskuje, włączając synchronizację na kopii bazy, a osobne pozycje mierzą bazę w stanie, w jakim tworzy ją benchmark; dodano też pozycje programu `allocations` dla wstawienia 1000 wierszy i snapshotu oraz odtworzenia bazy z benchmarku. Dla rdzenia z binarnym formatem wyniku obie części mierzą też wynik zakodowany wprost z tabeli wobec wyniku zebranego (odczyt po kluczu i 10 000 wierszy).

Pomiar RxDB w grupie „trwały zapis” (`raw/browser/workload/rxdb/`) powtórzono po poprawce adaptera: pierwsza wersja wymuszała ponowne wykonanie zapytania dodatkowym warunkiem na polu bez indeksu, co zmieniało plan zapytania. Powtórzenie wykonano tą samą wersją silnika i tym samym środowiskiem tego samego dnia; opis w `manifest.json` (`reruns`).
