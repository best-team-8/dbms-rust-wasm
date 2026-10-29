# Testy end-to-end

Testy w Node.js (`node:test`), które sprawdzają złożony system: moduł WebAssembly z `pkg/`, most JavaScript, magazyny i prawdziwy proces serwera synchronizacji. Uzupełniają testy rdzenia w [crates/flatsql-core/tests/](../crates/flatsql-core/tests/) tam, gdzie gwarancja zależy od mostu, magazynu albo sieci. Zachowania, których nie da się uruchomić w Node.js (praca Service Workera w przeglądarce, prawdziwa IndexedDB), sprawdza się ręcznie w przeglądarce według opisu w [tests-results.md](../docs/research/tests-results.md).

## Uruchomienie

Testy potrzebują zbudowanego modułu:

```bash
npm run build
```

```bash
npm test
```

```bash
node --test test/offline-sync.test.js
```

## Pliki

| Plik | Testów | Co sprawdza | Opis |
|---|---:|---|---|
| `bridge.test.js` | 15 | `FlatDB` nad magazynami: zatwierdzanie grupowe, odtworzenie, checkpoint krokami, instrukcje przygotowane, wiele zestawów parametrów, nazwy kolumn wyniku | [3.3](../docs/research/tests-results.md#33-instrukcje-przygotowane-z-zapamiętanym-planem), [3.7](../docs/research/tests-results.md#37-nazwy-kolumn-wyniku-instrukcji-przygotowanej), [3.9](../docs/research/tests-results.md#39-wiele-zestawów-parametrów-jednym-wywołaniem) |
| `browser.test.js` | 8 | zachowanie w przeglądarce na IndexedDB w pamięci: tryb trwałości, magazyn domyślny, prośba o trwały magazyn; Service Worker aplikacji demo zapisuje przy instalacji wszystkie pliki ładowane przez stronę, także pliki JavaScript wiązań z `pkg/snippets/` | [1.5](../docs/research/tests-results.md#15-tryb-trwałości-indexeddb), [1.6](../docs/research/tests-results.md#16-magazyn-domyślny), [1.7](../docs/research/tests-results.md#17-prośba-o-trwały-magazyn), [2.5](../docs/research/tests-results.md#25-praca-offline-w-aplikacji-demo) |
| `fs-store.test.js` | 5 | `FsStore`: stany po awarii zbudowane z operacji na symulowanym systemie plików, `fsync` wyłączany dla WAL, otwarty plik WAL | [1.3](../docs/research/tests-results.md#13-utrwalanie-zapisów-w-magazynie-plikowym-nodejs), [1.4](../docs/research/tests-results.md#14-wszystkie-stany-po-awarii-dopuszczone-przez-model) |
| `offline-sync.test.js` | 7 | praca offline z prawdziwym serwerem: zapisy przy wyłączonym serwerze, restart aplikacji z oczekującymi zmianami, ponowne połączenie, konflikt edycji, zapisy przez `/api/query` | [scenariusz §6.2](../docs/research/benchmark-scenario.md#62-testy-end-to-end-nodejs-testoffline-synctestjs) |
| `clock.test.js` | 2 | urządzenie z błędnym zegarem: wstrzymanie synchronizacji i nowe znaczniki po poprawie zegara | [2.1](../docs/research/tests-results.md#21-granica-dryfu-zegara) |

Pomocnicze moduły w `support/`:

| Plik | Rola |
|---|---|
| `crash-fs.js` | symulowany system plików dla testów awarii `FsStore`, z tym samym modelem co `checkpoint.rs` w rdzeniu: wywołania `node:fs/promises` są zapisywane jako operacje, z których powstają możliwe stany dysku po awarii |
| `fake-indexeddb.js` | IndexedDB w pamięci w zakresie używanym przez `IndexedDbStore`; zapisuje tryb i podpowiedź trwałości każdej transakcji |
