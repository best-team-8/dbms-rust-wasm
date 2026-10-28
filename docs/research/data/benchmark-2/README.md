# Benchmark 2: dane

Surowe wyniki pomiaru prototypu Flat-SQL (tag `prototyp`) razem z wersją po optymalizacjach (tag `po-2.5`) według tej samej [metodyki](../../benchmark-scenario.md) co Benchmark 1. Opis wyników: [benchmark-results.md §3](../../benchmark-results.md#3-benchmark-2-wersja-po-optymalizacjach).

| Ścieżka | Zawartość |
|---|---|
| `manifest.json` | obie wersje (opis `git describe`, commit, `codeTreeSha256`, rozmiar modułu WebAssembly), opcje pomiaru, obciążenie procesora przed pomiarem, czas rozpoczęcia i zakończenia |
| `before/raw/`, `after/raw/` | wyniki prototypu i wersji po optymalizacjach w Windows: `<środowisko>/<rodzaj>/<silnik>/NN.json`, jeden plik na proces; środowiska `node`, `native`, `browser`, rodzaje `workload`, `cold`, `memory`, `checkpoint`, `network`, `fsync` |
| `linux/` | koszt utrwalania plików w Linuksie (WSL 2): `manifest.json` z opisem środowiska (`note`), `before/raw/` i `after/raw/` z rodzajem `fsync` w środowiskach `node` i `native` |
| `compare.md` | zestawienie narzędzia pomiarowego dla Windows (mediana z kwartylami, zmiana, rozłączność przedziałów ufności, zgodność sum kontrolnych) |
| `make-report.mjs` | samodzielny skrypt (tylko moduły wbudowane Node.js): czyta wyniki obu wersji z Windows i Linuksa, tworzy `tables.md`, `summary.json` i `figures/*.svg` |
| `tables.md` | wszystkie tabele: mediana z 10 uruchomień, w nawiasie kwartyle, zmiana mediany, krotność, rozłączność 95% przedziałów ufności; zgodność sum kontrolnych wyników obu wersji |
| `summary.json` | wartości zagregowane, także średnia z 95% przedziałem ufności i współczynnik zmienności |
| `figures/` | wykresy SVG do pracy |
| `linux.sh` | pomiar w WSL: kopie robocze obu wersji, CLI dla `x86_64-unknown-linux-musl`, pakiety WebAssembly skopiowane z kompilacji w Windows |

Pomiar w Windows (obie wersje zbudowane przez `bench/build-version.js`):

```bash
node bench/build-version.js prototyp
node bench/build-version.js po-2.5
node bench/compare.js --before bench/versions/prototyp --after bench/versions/po-2.5 --out docs/research/data/benchmark-2 --runs 10 --targets node,native,browser --kinds workload,cold,memory,network,fsync,checkpoint --engines flatsql,flatsql-json,flatsql-idb,flatsql-idb-strict,flatsql-worker,flatsql-worker-idb
```

Pomiar w Linuksie (WSL, klon repozytorium w `~/flatsql-bench` z katalogiem Windows jako `origin`), potem skopiowanie katalogu wyników do `linux/`:

```bash
bash docs/research/data/benchmark-2/linux.sh /mnt/c/<ścieżka do repozytorium> ~/b2linux
```

Odtworzenie tabel i wykresów:

```bash
node docs/research/data/benchmark-2/make-report.mjs
```

Obciążenie ma identyfikator `benchmark-2`: operacje Benchmarku 1 bez zmian i na końcu `insert_many` (N wierszy do drugiej tabeli jednym wywołaniem; prototyp wykonuje je pętlą pojedynczych instrukcji). Benchmark natywny prototypu zgłasza obciążenie `benchmark-1`, zgodne z `benchmark-2` w operacjach, które oba zawierają. Raport `compare.md` z pomiaru w WSL usunięto, bo narzędzie w tamtej kopii nie odczytywało jeszcze wyników zapisu plików natywnego CLI; te same wyniki zawiera `tables.md`.
