# Narzędzie pomiarowe

Kod benchmarku Flat-SQL i silników odniesienia. Metodyka, procedura i zagrożenia dla trafności: [docs/research/benchmark-scenario.md](../docs/research/benchmark-scenario.md). Wyniki: [docs/research/benchmark-results.md](../docs/research/benchmark-results.md).

## Pliki

| Plik | Rola |
|---|---|
| `suite.js` | obciążenie (`WORKLOAD = 'benchmark-1'`): dane, zapytania, ziarna parametrów, suma kontrolna wyników, przebieg jednego silnika |
| `stats.js` | mediana, kwartyle, średnia z 95% przedziałem ufności |
| `engines/` | adaptery silników o wspólnym interfejsie; rejestr konfiguracji i grup warunków w `engines/index.js` |
| `node/child.js` | jeden mierzony proces Node.js (obciążenie, zimny start, pamięć, sieć, `fsync`) |
| `browser/` | strona pomiarowa, Worker, serwer z nagłówkami COOP/COEP, sterowanie przeglądarką przez playwright-core, tryb ręczny |
| `network.js` | Local-First a Cloud-First (zapytanie lokalne i przez `POST /api/query`) |
| `wasm-memory.js` | sonda rozmiaru pamięci liniowej WebAssembly dowolnego silnika |
| `run.js` | runner: powtarza pomiary w nowych procesach, zapisuje surowe wyniki |
| `build-version.js` | worktree wskazanej wersji Flat-SQL z gotowym `pkg/` i CLI |
| `compare.js` | pomiar optymalizacji: dwie wersje silnika na przemian tym samym runnerem, tabela przed i po z kryterium przedziałów ufności, zgodność wyników ([docs/research/data/optimization/](../docs/research/data/optimization/)) |
| `../crates/flatsql-cli/src/bench.rs` | to samo obciążenie natywnie (`flatsql bench`), ta sama suma kontrolna |

## Uruchomienie

```bash
npm ci
npm run build
cargo build --release -p flatsql-cli
node bench/run.js --targets node,native,browser --runs 10 --out bench/results/benchmark-1
node docs/research/data/benchmark-1/make-report.mjs bench/results/benchmark-1
```

Opcje `run.js`: `--targets node,native,browser`, `--runs N` (domyślnie 10), `--rows N` (domyślnie 100 000), `--kinds workload,cold,memory,network,fsync,checkpoint`, `--engines id,…` (identyfikatory z `engines/index.js`), `--flatsql DIR` (mierzona wersja), `--out DIR`, `--browser-channel msedge` albo `--browser-exe ŚCIEŻKA`, `--headed`. Przerwany pomiar wznawia się tym samym poleceniem z tym samym `--out`: istniejące pliki wyników są pomijane.

Przeglądarka: playwright-core uruchamia zainstalowaną przeglądarkę (domyślnie Microsoft Edge); każdy pomiar to nowy proces przeglądarki z pustym profilem.

## Pomiar innej wersji silnika

Narzędzie pochodzi zawsze z bieżącego kodu, a mierzony silnik z katalogu `--flatsql`. Stan sprzed optymalizacji (tag `prototyp`) i stan bieżący mierzy się więc tym samym narzędziem:

```bash
node bench/build-version.js prototyp
node bench/run.js --flatsql bench/versions/prototyp --label prototyp --runs 10
node bench/run.js --label biezacy --runs 10
```

Adaptery Flat-SQL używają tylko API obecnego w wersji `prototyp`. Natywny benchmark jest częścią mierzonej wersji (`flatsql bench` z jej `target/release`), więc jego obciążenie musi pozostać zgodne: runner przerywa pomiar, gdy CLI zgłasza inne `workload` niż `suite.js`.

## Tryb ręczny (inna przeglądarka, telefon)

```bash
node bench/browser/serve.js
```

Polecenie wypisuje adres strony `manual.html`, na której wybiera się zadanie i silnik; wynik zapisuje się jako plik JSON w formacie surowych wyników (`raw/browser/<rodzaj>/<silnik>/NN.json`). Precyzyjny zegar, OPFS i izolacja cross-origin wymagają bezpiecznego kontekstu, czyli adresu `localhost`. Telefon z Androidem podłączony przez USB: `adb reverse tcp:8090 tcp:8090` i `adb reverse tcp:8091 tcp:8091`, potem `http://localhost:8090/bench/browser/manual.html?sync=http%3A%2F%2Flocalhost%3A8091` w Chrome.

## Dodanie silnika

Adapter w `engines/` eksportuje `create(env, options)` i zwraca obiekt z metodami `open`, `load`, `prepare`, `query`, `write`, `verify`, `close` (opcjonalnie `finalize`, `snapshot`/`restore`, a dla magazynów trwałych `commit`, `commitGroup`, `reopen`, `checkpoint`). Wyniki zapytań to tablice wierszy, a każdy wiersz to tablica wartości w kolejności kolumn zapytania. Konfigurację dopisuje się do `ENGINES` w `engines/index.js` z grupą warunków, wymaganiami i środowiskami; wersję pakietu przypina się dokładnie w `package.json`. Sumy kontrolne pokażą, czy odwzorowanie zapytań jest poprawne.
