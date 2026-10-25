# Optymalizacje silnika: dane pomiarów przed zmianą i po niej

Surowe wyniki pomiarów każdej optymalizacji opisanej w [optimization-results.md](../../optimization-results.md). Każdy katalog odpowiada jednej zmianie (numer to kolejność wykonania) i zawiera:

| Ścieżka | Zawartość |
|---|---|
| `manifest.json` | obie wersje (opis `git describe`, commit, temat commita oraz `codeTreeSha256`: skrót SHA-256 listy plików `crates/`, `bridge/` i `server/` z ich skrótami git, który identyfikuje mierzony kod także przed zatwierdzeniem i jest równy skrótowi drzewa późniejszego commita: `node bench/code-tree.js <tag>`), obciążenie procesora przed pomiarem, rozmiar modułu WebAssembly (bajty i gzip -9) |
| `before/`, `after/` | wyniki `bench/run.js` dla wersji przed zmianą i po niej: `raw/<środowisko>/<rodzaj>/<silnik>/NN.json` (jeden plik na proces), `manifest.json` runnera |
| `compare.md` | tabela: mediana z kwartylami przed i po, zmiana mediany, czy 95% przedziały ufności średnich są rozłączne; zgodność sum kontrolnych wyników obu wersji |
| `cost-split-*.txt`, `allocations-*.txt` | natywne rozbicie kosztów i liczba alokacji tych samych operacji w obu wersjach (program z [benchmark-1/analysis/cost-split](../benchmark-1/analysis/cost-split/)) |

Pomiar przed i po wykonuje skrypt `bench/compare.js`: obie wersje silnika mierzy to samo narzędzie, na przemian (uruchomienie 1 wersji „przed”, uruchomienie 1 wersji „po”, potem w odwrotnej kolejności), każda w 10 nowych procesach. Wersja „przed” to worktree poprzedniego stanu zbudowany przez `bench/build-version.js`, wersja „po” to katalog roboczy ze zmianą przed jej zatwierdzeniem:

```bash
node bench/build-version.js <poprzedni-tag>
node bench/compare.js --before bench/versions/<poprzedni-tag> --after . --out docs/research/data/optimization/<katalog> --runs 10
node docs/research/data/benchmark-1/analysis/cost-split/run.mjs --flatsql bench/versions/<poprzedni-tag> --out <katalog>/cost-split-before.txt
node docs/research/data/benchmark-1/analysis/cost-split/run.mjs --out <katalog>/cost-split-after.txt
```

Domyślnie mierzone są konfiguracje „Flat-SQL” i „Flat-SQL (JSON)” w Node.js oraz benchmark natywny; zadanie dotyczące innej grupy dokłada `--targets browser` albo `--kinds`. Pomiar jest krótszy niż benchmark pełny (jedno środowisko JavaScript i dwie konfiguracje), więc służy do oceny pojedynczej zmiany; wersja końcowa i prototyp są mierzone pełnym benchmarkiem.
