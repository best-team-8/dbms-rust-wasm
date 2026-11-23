# Testy

Testy sprawdzają gwarancje silnika: że zatwierdzone dane przetrwają awarię, że urządzenia dochodzą do tego samego stanu i że optymalizacje nie zmieniają odpowiedzi. Czasu operacji nie mierzą. To robią [benchmarki](../research/benchmark-results.md).

```bash
cargo test --workspace
```

```bash
npm run build && npm test
```

Rdzeń ma 118 testów w Rust, a most JavaScript, serwer i aplikacja demonstracyjna 37 testów w Node.js. Wszystkie przechodzą.

## Jak testujemy

Test leży na najniższym poziomie, który obejmuje sprawdzany mechanizm. Testy jednostkowe są w plikach `src/`. Testy rdzenia przez jego publiczne API są w `crates/flatsql-core/tests/`. Testy modułu WebAssembly, mostu i prawdziwego serwera HTTP są w `test/`. Zachowania prawdziwej przeglądarki (Service Worker, IndexedDB) sprawdza się ręcznie według opisu krok po kroku.

Testy są deterministyczne. Zegar, system plików i losowość są wstrzykiwane, a ziarna stałe, więc każdy błąd odtwarza jedno polecenie. Wzorzec poprawnej odpowiedzi nie zależy od testowanego kodu: to prosty model w pamięci, pełny przegląd tabeli albo interpreter wyrażeń. Gdy przestrzeń przypadków jest skończona (każdy bajt WAL, każdy stan dysku po awarii), test sprawdza ją w całości. Gdy jest za duża, losuje przypadki, a pełny przebieg uruchamia się ręcznie.

Duży test ma kontrolę negatywną: na kodzie sprzed poprawki albo z celowo wprowadzonym błędem musi zawieść. Test, który nigdy nie zawiódł, niczego nie dowodzi.

## Duże testy

| Gwarancja | Opis |
|---|---|
| Po awarii w dowolnym miejscu zapisu na pliki baza wraca do stanu zatwierdzonego | [awarie zapisu](crash-recovery.md) |
| Urządzenia dochodzą do tego samego stanu mimo przesuniętych zegarów, podziałów sieci i restartów | [zbieżność urządzeń](replica-convergence.md) |
| Aplikacja działa bez serwera, a zmiany docierają, gdy serwer wróci | [praca offline](offline-work.md) |

## Pozostałe testy

| Co sprawdza | Gdzie |
|---|---|
| Typy, konwersje, logika trójwartościowa, agregaty, transakcje i błędy języka | `crates/flatsql-core/tests/sql.rs` |
| Ucięcie WAL w każdym bajcie daje stan po ostatniej pełnej transakcji; urwany zapis i uszkodzony snapshot są wykrywane | `crates/flatsql-core/tests/storage.rs` |
| Tryb trwałości z `FlatDB.open` trafia do każdej transakcji IndexedDB | `test/browser.test.js` i ręcznie w przeglądarce |
| Bez wskazania magazynu baza używa IndexedDB w przeglądarce, a w Node.js zgłasza błąd | `test/browser.test.js` |
| Baza w IndexedDB prosi przeglądarkę o trwały magazyn | `test/browser.test.js` |
| Niewysłane zmiany przetrwają restart, a potwierdzone nie są wysyłane ponownie | `crates/flatsql-core/tests/sync.rs` |
| Konflikty edycji i usunięć rozstrzyga późniejszy znacznik | `crates/flatsql-core/tests/sync.rs` |
| Różne definicje tej samej tabeli dają jeden konflikt schematu z obiema definicjami | `crates/flatsql-core/tests/sync.rs` |
| Kolejka zmian prowadzona od włączenia synchronizacji wysyła to samo co kolejka prowadzona od początku, także po restarcie | `crates/flatsql-core/tests/outbox.rs` |
| Te same dane dają te same bajty niezależnie od postaci bitowej NaN | `crates/flatsql-core/tests/format.rs` |
| Każda ścieżka dostępu, instrukcja przygotowana, grupowanie i binarny format wyniku dają odpowiedzi pełnego przeglądu tabeli | `crates/flatsql-core/tests/planner.rs` |
| Warunki skompilowane dają ten sam wynik i ten sam błąd co interpreter | `crates/flatsql-core/tests/compiled.rs` |
| `UPDATE` w miejscu zostawia indeksy, wycofanie i WAL w stanie zgodnym | `crates/flatsql-core/tests/update.rs` |
| `executeMany` daje te same wiersze i WAL co osobne wykonania, jako jedna instrukcja | `crates/flatsql-core/tests/many.rs`, `test/bridge.test.js` |
| Snapshot kodowany krokami jest stanem z chwili rozpoczęcia mimo zapisów między krokami | `crates/flatsql-core/tests/snapshot_steps.rs`, `test/bridge.test.js` |
| Tekst w samej wartości i na stercie ma tę samą równość, porządek i skrót | testy jednostkowe w `crates/flatsql-core/src/value.rs` |
| Wynik instrukcji przygotowanej ma własne nazwy kolumn, które nadążają za zmianą schematu | `test/bridge.test.js` |
| Wartości każdego typu przechodzą przez binarny format wyniku bez zmian | `test/bridge.test.js` |

## Czego testy nie obejmują

- Prawdziwego odcięcia zasilania. Testy awarii zapisu używają modelu systemu plików.
- Innych przeglądarek niż Chromium i urządzeń mobilnych.
- Serwera z błędnym zegarem i restartu serwera w testach losowych synchronizacji.
- Zmian struktury tabel w testach losowych synchronizacji.

Środowisko: Windows 11 (NTFS), AMD Ryzen 5 4600H, Rust 1.99, Node.js 24 lokalnie i 22 w CI, Chromium 152. CI uruchamia testy w Linuksie (`.github/workflows/build-test.yml`).
