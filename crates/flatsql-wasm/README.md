# flatsql-wasm – wiązania WebAssembly

Cienka warstwa nad [rdzeniem](../flatsql-core/), która udostępnia silnik JavaScriptowi przez `wasm-bindgen`: wykonanie SQL, instrukcje przygotowane, bajty WAL i snapshotu oraz operacje synchronizacji. Nie zapisuje danych i nie korzysta z sieci; trwałością i synchronizacją zarządza most JavaScript ([bridge/](../../bridge/)). Moduł kompiluje się do `pkg/flatsql.wasm` z kodem wiązań `pkg/flatsql.js`.

## Budowanie

```bash
npm run build
```

Polecenie uruchamia `wasm-pack build crates/flatsql-wasm --release --target web` i zapisuje wynik w `pkg/`. Kompilacja używa `opt-level = 3` i `wasm-opt -O3`; mniejsze warianty (`"s"`, `"z"`) wydłużały operacje o 25–78% ([wyniki optymalizacji](../../docs/research/optimization-results.md#16-ustawienia-kompilacji), [decyzja 0009](../../docs/adr/0009-ustawienia-kompilacji.md)). Moduł ma po kompresji gzip 274 098 B. `npm run build:dev` buduje wersję bez optymalizacji.

## Interfejs

| Eksport | Opis |
|---|---|
| `Database` | baza: `exec`, `execAll`, `execJson`, `prepare`, `run`, `runJson`, `execute`, `executeMany`, `takeWal`, `unshiftWal`, `walPending`, `snapshot`, `snapshotBegin` / `snapshotStep` / `snapshotFinish`, `restore`, `recoveryReport`, `stats`, `tableNames`, `inTransaction`, `nodeId` |
| synchronizacja na `Database` | `enableSync`, `syncEnabled`, `outboxLength`, `exportOutbox`, `ackOutbox`, `applyChanges`, `exportState`, `clockAhead`, `rebaseClock` |
| `PreparedStatement` | instrukcja przygotowana: `paramCount`, `readOnly` |
| `OutboxBatch` | paczka zmian do wysłania: `bytes`, `lastSeq`, `count` |
| `describeChanges`, `maxDriftMs`, `version` | opis paczki zmian (do podglądu), granica dryfu zegara, wersja silnika |

Pełne typy z opisami: [bridge/index.d.ts](../../bridge/index.d.ts). Aplikacja korzysta zwykle z `FlatDB` z mostu, który opakowuje te klasy.

## Przekazywanie wyników

Wynik zapytania trafia do JavaScriptu na dwa sposoby, porównywane w badaniach:

- `exec` / `run` zwracają obiekty JavaScript zdekodowane z binarnego formatu wyniku. Rdzeń koduje wiersze SELECT wprost z tabeli do bufora w pamięci modułu, a funkcja JavaScriptu dołączona do modułu tworzy obiekt wyniku jednym wywołaniem: liczby czyta z bajtów, a wszystkie teksty dekoduje jednym wywołaniem `TextDecoder`. Nazwy kolumn instrukcji przygotowanej powstają raz na plan.
- `execJson` / `runJson` zwracają wynik zakodowany w Rust jako JSON, parsowany przez `JSON.parse`.

Dla zapisów masowych `execute` zwraca tylko liczbę zmienionych wierszy, a `executeMany(statement, rows)` wykonuje przygotowany zapis dla wielu tablic parametrów jako jedną instrukcję: JavaScript koduje parametry do bufora w porcjach około 64 KiB, a rdzeń wykonuje je z jednym odczytem zegara. Komunikat (`message`) mają tylko instrukcje zmieniające dane lub schemat.

Błędy są obiektami `Error` z polem `kind` (`parse`, `schema`, `constraint`, `type`, `eval`, `transaction`, `storage`, `unsupported`). Panika w Rust trafia do konsoli przez `console_error_panic_hook`.

## Decyzje

- [wynik w jednym buforze binarnym](../../docs/adr/0015-wynik-binarny-dla-js.md)
- [wiele wierszy jednym wywołaniem](../../docs/adr/0013-sciezka-zapisu.md)
- [ustawienia kompilacji](../../docs/adr/0009-ustawienia-kompilacji.md)

## Testy

Wiązania nie mają własnych testów w Rust; sprawdzają je testy mostu w [test/](../../test/), które działają na zbudowanym `pkg/`.

## Ograniczenia

Każde wywołanie przechodzi granicę między JavaScriptem a modułem, więc odczyt po kluczu jest 2,7–3,6 raza wolniejszy niż natywnie, choć operacje wykonywane w całości w silniku mają w WebAssembly czas natywny ([wyniki benchmarków](../../docs/research/benchmark-results.md#narzut-webassembly)). Kierunki: [na później](../../docs/management/future.md).
