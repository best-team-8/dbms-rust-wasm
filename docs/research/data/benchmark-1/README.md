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
| `analysis/` | skrypty i wyniki analizy słabych stron: plany zapytań (`explain`), pamięć według sposobu wczytania (`memory`), profil V8 odczytu po kluczu (`profile`), natywne rozbicie kosztów silnika (`cost-split`, program w Rust; opis: [benchmark-results.md §1.12](../../benchmark-results.md#112-rozbicie-kosztów-silnika-pomiar-pomocniczy)) |

Odtworzenie tabel i wykresów:

```bash
node docs/research/data/benchmark-1/make-report.mjs
```

Odtworzenie analiz (wymaga zbudowanego `pkg/` mierzonej wersji, domyślnie bieżącego katalogu):

```bash
node docs/research/data/benchmark-1/analysis/explain.mjs
node docs/research/data/benchmark-1/analysis/memory.mjs
node docs/research/data/benchmark-1/analysis/profile.mjs
cargo run --release --manifest-path docs/research/data/benchmark-1/analysis/cost-split/Cargo.toml --target-dir target/analysis
```

`cost-split` mierzy bieżący rdzeń z `crates/flatsql-core`; dla innej wersji zmienia się ścieżkę zależności w jego `Cargo.toml`. Wyjście zapisane w `analysis/cost-split.txt` pochodzi z uruchomienia przy procesorze obciążonym inną aplikacją (wartości 1,4–2 razy wyższe niż w pierwszym uruchomieniu, udziały te same).

Pomiar RxDB w grupie „trwały zapis” (`raw/browser/workload/rxdb/`) powtórzono po poprawce adaptera: pierwsza wersja wymuszała ponowne wykonanie zapytania dodatkowym warunkiem na polu bez indeksu, co zmieniało plan zapytania. Powtórzenie wykonano tą samą wersją silnika i tym samym środowiskiem tego samego dnia; opis w `manifest.json` (`reruns`).
