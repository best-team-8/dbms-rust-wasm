# Testy integracyjne rdzenia

Testy sprawdzają gwarancje silnika przez jego publiczne API: trwałość zatwierdzonych danych, zbieżność urządzeń, poprawność zapytań i stały format plików. Są deterministyczne (stałe ziarna, symulowany zegar i system plików), więc każde uruchomienie daje ten sam wynik. Testy jednostkowe są w plikach `src/`, a testy mostu JavaScript i serwera w [test/](../../../test/README.md). Co i jak testujemy w całym projekcie: [testy](../../../docs/testing/README.md).

```bash
cargo test -p flatsql-core
```

```bash
cargo test -p flatsql-core --test sync_faults
```

| Plik | Testów | Co sprawdza |
|---|---:|---|
| `sql.rs` | 20 | język od tekstu SQL do wyniku: typy, konwersje, logika trójwartościowa, agregaty, transakcje, błędy |
| `storage.rs` | 9 | odtwarzanie z WAL i snapshotu, ucięcie WAL w każdym bajcie, urwane zapisy, uszkodzony snapshot, magazyn plikowy |
| `checkpoint.rs` | 4 | każdy stan dysku po awarii checkpointu na plikach, utrwalanie katalogu w Windows ([opis](../../../docs/testing/crash-recovery.md)) |
| `sync.rs` | 12 | kolejka zmian, potwierdzenia, „ostatni zapis wygrywa”, znaczniki usunięcia, granica dryfu zegara, konflikt schematu, losowe sesje offline trzech urządzeń |
| `sync_faults.rs` | 1 | zbieżność czterech urządzeń przy przesuniętych zegarach, podziałach sieci, utraconych potwierdzeniach i restartach ([opis](../../../docs/testing/replica-convergence.md)) |
| `outbox.rs` | 2 | kolejka prowadzona od włączenia synchronizacji wysyła to samo co kolejka prowadzona od początku, także po restarcie |
| `format.rs` | 2 | te same dane dają te same bajty snapshotu, WAL i paczek niezależnie od postaci bitowej NaN |
| `planner.rs` | 8 | każda ścieżka dostępu, instrukcja przygotowana, grupowanie i binarny format wyniku dają odpowiedzi pełnego przeglądu tabeli |
| `compiled.rs` | 1 | warunki skompilowane dają te same wyniki i błędy co interpreter |
| `update.rs` | 2 | `UPDATE` w miejscu zostawia indeksy, wycofanie i WAL w stanie zgodnym |
| `many.rs` | 3 | zapis dla wielu zestawów parametrów daje te same wiersze i WAL co osobne wykonania |
| `snapshot_steps.rs` | 4 | snapshot kodowany krokami jest stanem z chwili rozpoczęcia mimo zmian między krokami |
