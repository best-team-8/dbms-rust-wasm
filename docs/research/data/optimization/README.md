# Optymalizacje silnika: dane pomiarów przed zmianą i po niej

Surowe wyniki pomiarów każdej optymalizacji opisanej w [optimization-results.md](../../optimization-results.md). Każdy katalog odpowiada jednej zmianie (numer to kolejność wykonania) i zawiera:

| Ścieżka | Zawartość |
|---|---|
| `manifest.json` | obie wersje (opis `git describe`, commit, temat commita oraz `codeTreeSha256`: skrót SHA-256 listy plików `crates/`, `bridge/` i `server/` z ich skrótami git, który identyfikuje mierzony kod także przed zatwierdzeniem i jest równy skrótowi drzewa późniejszego commita: `node bench/code-tree.js <tag>`), obciążenie procesora przed pomiarem, rozmiar modułu WebAssembly (bajty i gzip -9) |
| `before/`, `after/` | wyniki `bench/run.js` dla wersji przed zmianą i po niej: `raw/<środowisko>/<rodzaj>/<silnik>/NN.json` (jeden plik na proces), `manifest.json` runnera |
| `compare.md` | tabela: mediana z kwartylami przed i po, zmiana mediany, czy 95% przedziały ufności średnich są rozłączne; zgodność sum kontrolnych wyników obu wersji |
| `cost-split-*.txt`, `allocations-*.txt` | natywne rozbicie kosztów i liczba alokacji tych samych operacji w obu wersjach (program z [benchmark-1/analysis/cost-split](../benchmark-1/analysis/cost-split/)) |
| podkatalogi z `compare.md` | pomiary dodatkowe tym samym skryptem: `first/` – pierwsza wersja zmiany, poprawiona po pomiarze; `native-20/` – benchmark natywny z 20 uruchomieniami na wersję, gdy 10 nie rozstrzygało; `network/` – porównanie Local-First (`--kinds network`) |
| skrypty `*.mjs` i ich wyniki `*-before-N.txt`, `*-after-N.txt` | pomiary operacji spoza obciążenia (np. warstwa mostu `FlatDB`), uruchamiane na przemian dla obu wersji: `node <skrypt> <katalog wersji> --out <plik>` |

Pomiar przed i po wykonuje skrypt `bench/compare.js`: obie wersje silnika mierzy to samo narzędzie, na przemian (uruchomienie 1 wersji „przed”, uruchomienie 1 wersji „po”, potem w odwrotnej kolejności), każda w 10 nowych procesach. Wersja „przed” to worktree poprzedniego stanu zbudowany przez `bench/build-version.js`, wersja „po” to katalog roboczy ze zmianą przed jej zatwierdzeniem:

```bash
node bench/build-version.js <commit poprzedniej zmiany> --name <nazwa>
node bench/compare.js --before bench/versions/<nazwa> --after . --out docs/research/data/optimization/<katalog> --runs 10
node docs/research/data/benchmark-1/analysis/cost-split/run.mjs --flatsql bench/versions/<nazwa> --out <katalog>/cost-split-before.txt
node docs/research/data/benchmark-1/analysis/cost-split/run.mjs --out <katalog>/cost-split-after.txt
```

Domyślnie mierzone są konfiguracje „Flat-SQL” i „Flat-SQL (JSON)” w Node.js oraz benchmark natywny; zadanie dotyczące innej grupy dokłada `--targets browser` albo `--kinds`. Pomiar jest krótszy niż benchmark pełny (jedno środowisko JavaScript i dwie konfiguracje), więc służy do oceny pojedynczej zmiany; wersja końcowa i prototyp są mierzone pełnym benchmarkiem.

## Wersje mierzone w czasie optymalizacji

W czasie pomiarów każdą zmianę oznaczał tag `po-2.N` (numer zadania), a prototyp tag `prototyp`. Po zakończeniu prac zostały tylko tagi `v1-prototyp` i `v2-optymalizacja`, a manifesty (`describe`), raporty `compare.md` i część skryptów w tym katalogu podają dawne nazwy. Tabela przypisuje je do commitów w kolejności wykonania zmian. Kopię roboczą pod dawną nazwą, potrzebną skryptom z odwołaniem do `bench/versions/po-2.N`, tworzy `node bench/build-version.js <commit> --name po-2.N`; warianty kompilacji z pomiaru 16 (`bench/versions/v25-*`) tworzy skrypt `16-build-options/variants.py`.

| Dawny tag | Commit | Temat commita |
|---|---|---|
| `prototyp` (`v1-prototyp`) | `261d78f` | bench: module size of Worker engines, plain values from DuckDB, table alignment in the report |
| `po-2.1` | `4845bac` | core: count rows by the access path when it guarantees the whole WHERE clause |
| `po-2.2` | `6839b70` | core: skip the conjuncts of WHERE that the access path guarantees |
| `po-2.9` | `54ba1cf` | core: compile statements once and keep the plan in prepared statements |
| `po-2.10` | `5639f81` | core: compile WHERE conjuncts and SET expressions specialised to column types |
| `po-2.11` | `f6d99ad` | core: keep the outbox only once synchronisation is enabled, compact outbox in snapshots |
| `po-2.8` | `a273b5d` | core: update rows in place, touch only changed indexes, read the clock once per statement |
| `po-2.12` | `d0a2e22` | core: build secondary indexes in bulk from rows grouped by value |
| `po-2.15` | `17d30dc` | core: group rows by one column through a hash table |
| `po-2.16` | `b94377b` | bridge: keep the WAL file of FsStore open between checkpoints |
| `po-2.3` | `327a0ce` | docs: profile of a primary key lookup at the JavaScript and WebAssembly boundary |
| `po-2.13` | `2f9d972` | wasm: build the result object in JavaScript and keep column names per plan |
| `po-2.4` | `16f547e` | core/wasm: encode results straight from the table in a binary format decoded by JavaScript |
| `po-2.14` | `fc2703f` | core/wasm: execute a prepared write for many parameter sets in one call |
| `po-2.7` | `140836b` | core: keep short texts inside the value |
| `po-2.6` | `a397241` | core/bridge: encode checkpoints in steps |
| `po-2.5` (`v2-optymalizacja`) | `45f581d` | docs: compare build options of the WebAssembly module |
