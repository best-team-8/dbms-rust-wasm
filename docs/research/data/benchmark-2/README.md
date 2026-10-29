# Benchmark 2: dane

Surowe wyniki pomiaru prototypu Flat-SQL (tag `v1-prototyp`) razem z wersją po optymalizacjach (tag `v2-optymalizacja`) według tej samej [metodyki](../../benchmark-scenario.md) co Benchmark 1. Opis wyników: [benchmark-results.md §2](../../benchmark-results.md#2-benchmark-2-wersja-po-optymalizacjach), miejsce wersji po optymalizacjach wśród silników z Benchmarku 1: [§2.9](../../benchmark-results.md#29-flat-sql-po-optymalizacjach-na-tle-innych-silników).

| Ścieżka | Zawartość |
|---|---|
| `manifest.json` | obie wersje (nazwy `prototyp` i `po-2.5` w manifestach i raportach to nazwy tagów z dnia pomiaru; tagi `v1-prototyp` i `v2-optymalizacja` wskazują te same commity; (opis `git describe`, commit, `codeTreeSha256`, rozmiar modułu WebAssembly), opcje pomiaru, obciążenie procesora przed pomiarem, czas rozpoczęcia i zakończenia |
| `before/raw/`, `after/raw/` | wyniki prototypu i wersji po optymalizacjach w Windows: `<środowisko>/<rodzaj>/<silnik>/NN.json`, jeden plik na proces; środowiska `node`, `native`, `browser`, rodzaje `workload`, `cold`, `memory`, `checkpoint`, `network`, `fsync` |
| `linux/` | koszt utrwalania plików w Linuksie (WSL 2): `manifest.json` z opisem środowiska (`note`), `before/raw/` i `after/raw/` z rodzajem `fsync` w środowiskach `node` i `native` |
| `compare.md` | zestawienie narzędzia pomiarowego dla Windows (mediana z kwartylami, zmiana, rozłączność przedziałów ufności, zgodność sum kontrolnych) |
| `make-report.mjs` | samodzielny skrypt (tylko moduły wbudowane Node.js): czyta wyniki obu wersji z Windows i Linuksa, tworzy `tables.md`, `summary.json` i `figures/*.svg` |
| `tables.md` | wszystkie tabele: mediana z 10 uruchomień, w nawiasie kwartyle, zmiana mediany, krotność, rozłączność 95% przedziałów ufności; zgodność sum kontrolnych wyników obu wersji |
| `summary.json` | wartości zagregowane, także średnia z 95% przedziałem ufności i współczynnik zmienności |
| `figures/` | wykresy SVG do pracy: `01`–`06` prototyp a wersja po optymalizacjach, `07`–`12` wersja po optymalizacjach wśród silników z Benchmarku 1 |
| `engines-report.mjs` | samodzielny skrypt: wersja po optymalizacjach (`after/raw`) wśród silników odniesienia z `../benchmark-1/raw`; różnica sesji zmierzona na prototypie z obu pomiarów; tworzy `engines.md`, `engines.json` i `figures/07`–`12` |
| `engines.md` | tabele porównania z silnikami: mediana z kwartylami, rola silnika, miejsce Flat-SQL przed i po optymalizacjach, iloraz czasu Flat-SQL i najszybszego pokrewnego silnika, różnica między sesjami |
| `engines.json` | wartości zagregowane porównania, miejsca i wszystkie pary różnicy sesji |
| `linux.sh` | pomiar w WSL: kopie robocze obu wersji, CLI dla `x86_64-unknown-linux-musl`, pakiety WebAssembly skopiowane z kompilacji w Windows |

Pomiar w Windows (obie wersje zbudowane przez `bench/build-version.js`):

```bash
node bench/build-version.js v1-prototyp
node bench/build-version.js v2-optymalizacja
node bench/compare.js --before bench/versions/v1-prototyp --after bench/versions/v2-optymalizacja --out docs/research/data/benchmark-2 --runs 10 --targets node,native,browser --kinds workload,cold,memory,network,fsync,checkpoint --engines flatsql,flatsql-json,flatsql-idb,flatsql-idb-strict,flatsql-worker,flatsql-worker-idb
```

Pomiar w Linuksie (WSL, klon repozytorium w `~/flatsql-bench` z katalogiem Windows jako `origin`), potem skopiowanie katalogu wyników do `linux/`:

```bash
bash docs/research/data/benchmark-2/linux.sh /mnt/c/<ścieżka do repozytorium> ~/b2linux
```

Odtworzenie tabel i wykresów:

```bash
node docs/research/data/benchmark-2/make-report.mjs
node docs/research/data/benchmark-2/engines-report.mjs
```

Silników odniesienia w Benchmarku 2 nie mierzono ponownie: ich adaptery i wersje pakietów są takie jak w Benchmarku 1. `engines-report.mjs` zestawia je z wersją po optymalizacjach i podaje, o ile prototyp zmierzony w obu sesjach różni się między nimi (mediana −2,3%, zakres od −12% do +4%); miejsce bliższe innemu silnikowi niż 10% jest oznaczone jako niepewne.

Obciążenie ma identyfikator `benchmark-2`: operacje Benchmarku 1 bez zmian i na końcu `insert_many` (N wierszy do drugiej tabeli jednym wywołaniem; prototyp wykonuje je pętlą pojedynczych instrukcji). Benchmark natywny prototypu zgłasza obciążenie `benchmark-1`, zgodne z `benchmark-2` w operacjach, które oba zawierają. Raport `compare.md` z pomiaru w WSL usunięto, bo narzędzie w tamtej kopii nie odczytywało jeszcze wyników zapisu plików natywnego CLI; te same wyniki zawiera `tables.md`.
