# Testy end-to-end

Testy w Node.js (`node:test`), które sprawdzają złożony system: moduł WebAssembly z `pkg/`, most JavaScript, magazyny i prawdziwy proces serwera synchronizacji. Uzupełniają testy rdzenia w [crates/flatsql-core/tests/](../crates/flatsql-core/tests/) tam, gdzie gwarancja zależy od mostu, magazynu albo sieci. Zachowania, których nie da się uruchomić w Node.js (praca Service Workera w przeglądarce, prawdziwa IndexedDB), sprawdza się ręcznie w przeglądarce według opisu w [teście pracy offline](../docs/testing/offline-work.md).

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

| Plik | Testów | Co sprawdza |
|---|---:|---|
| `bridge.test.js` | 15 | `FlatDB` nad magazynami: zatwierdzanie grupowe, odtworzenie, checkpoint krokami, instrukcje przygotowane, wiele zestawów parametrów, nazwy kolumn wyniku |
| `browser.test.js` | 8 | zachowanie w przeglądarce na IndexedDB w pamięci: tryb trwałości, magazyn domyślny, prośba o trwały magazyn; Service Worker aplikacji demo zapisuje przy instalacji wszystkie pliki ładowane przez stronę, także pliki JavaScript wiązań z `pkg/snippets/` |
| `fs-store.test.js` | 5 | `FsStore`: stany po awarii zbudowane z operacji na symulowanym systemie plików, `fsync` wyłączany dla WAL, otwarty plik WAL ([opis](../docs/testing/crash-recovery.md)) |
| `offline-sync.test.js` | 7 | praca offline z prawdziwym serwerem: zapisy przy wyłączonym serwerze, restart aplikacji z oczekującymi zmianami, ponowne połączenie, konflikt edycji, zapisy przez `/api/query` ([opis](../docs/testing/offline-work.md)) |
| `clock.test.js` | 2 | urządzenie z błędnym zegarem: wstrzymanie synchronizacji i nowe znaczniki po poprawie zegara ([opis](../docs/testing/replica-convergence.md)) |

Pomocnicze moduły w `support/`:

| Plik | Rola |
|---|---|
| `crash-fs.js` | symulowany system plików dla testów awarii `FsStore`, z tym samym modelem co `checkpoint.rs` w rdzeniu: wywołania `node:fs/promises` są zapisywane jako operacje, z których powstają możliwe stany dysku po awarii |
| `fake-indexeddb.js` | IndexedDB w pamięci w zakresie używanym przez `IndexedDbStore`; zapisuje tryb i podpowiedź trwałości każdej transakcji |
