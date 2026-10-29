# Testy integracyjne rdzenia

Testy w tym katalogu sprawdzają gwarancje silnika przez jego publiczne API: trwałość zatwierdzonych danych, zbieżność replik, semantykę zapytań i stały format plików. Są deterministyczne (stałe ziarna generatorów, symulowany zegar i system plików), więc każde uruchomienie daje ten sam wynik. Czas operacji mierzą benchmarki, nie testy ([metodyka testów](../../../docs/research/tests-methodology.md)). Testy jednostkowe pojedynczych funkcji są w plikach `src/` (`#[cfg(test)]`), a testy mostu JavaScript i serwera w [test/](../../../test/).

## Uruchomienie

```bash
cargo test -p flatsql-core
```

```bash
cargo test -p flatsql-core --test sync_faults
```

## Pliki

Kolumna „Opis” odsyła do [tests-results.md](../../../docs/research/tests-results.md), gdzie każda grupa testów ma pytanie, model awarii, procedurę, kontrolę negatywną i wynik.

| Plik | Testów | Co sprawdza | Opis |
|---|---:|---|---|
| `sql.rs` | 20 | zachowanie języka od tekstu SQL do wyniku: typy, koercja, logika trójwartościowa, agregaty, transakcje, błędy | – |
| `storage.rs` | 9 | odtwarzanie z WAL i snapshotu, ucięcie WAL w każdym bajcie, urwane zapisy, uszkodzony snapshot, magazyn plikowy | – |
| `checkpoint.rs` | 4 | stany dysku po awarii w dowolnym miejscu checkpointu na plikach, utrwalanie katalogu w Windows | [1.1](../../../docs/research/tests-results.md#11-utrwalenie-zmiany-nazwy-snapshotu-przed-obcięciem-wal), [1.2](../../../docs/research/tests-results.md#12-eksperyment-synchronizacja-katalogu-w-windows), [1.4](../../../docs/research/tests-results.md#14-wszystkie-stany-po-awarii-dopuszczone-przez-model) |
| `sync.rs` | 12 | outbox, potwierdzenia, „ostatni zapis wygrywa”, znaczniki usunięcia, granica dryfu zegara, konflikt schematu, losowe sesje offline trzech replik | [2.1](../../../docs/research/tests-results.md#21-granica-dryfu-zegara), [2.2](../../../docs/research/tests-results.md#22-konflikt-definicji-tabeli) |
| `sync_faults.rs` | 1 | zbieżność czterech urządzeń przy przesuniętych zegarach, podziałach sieci, utraconych potwierdzeniach i kursorach oraz restartach (1000 przebiegów) | [2.3](../../../docs/research/tests-results.md#23-zbieżność-w-trudnych-warunkach) |
| `outbox.rs` | 2 | outbox prowadzony od włączenia synchronizacji daje to samo co outbox prowadzony od początku, także po restarcie | [2.4](../../../docs/research/tests-results.md#24-outbox-od-włączenia-synchronizacji) |
| `format.rs` | 2 | te same dane dają te same bajty snapshotu, WAL i paczek niezależnie od bitów NaN | [3.1](../../../docs/research/tests-results.md#31-ten-sam-zapis-każdego-nan) |
| `planner.rs` | 8 | testy różnicowe: każda ścieżka dostępu, instrukcja przygotowana, grupowanie przez tablicę mieszającą i binarny format wyniku dają te same odpowiedzi co pełny przegląd | [3.2](../../../docs/research/tests-results.md#32-te-same-odpowiedzi-przy-każdej-ścieżce-dostępu), [3.3](../../../docs/research/tests-results.md#33-instrukcje-przygotowane-z-zapamiętanym-planem), [3.6](../../../docs/research/tests-results.md#36-grupowanie-przez-tablicę-mieszającą), [3.8](../../../docs/research/tests-results.md#38-binarny-format-wyniku) |
| `compiled.rs` | 1 | wyrażenia skompilowane dają te same wyniki i błędy co interpreter | [3.4](../../../docs/research/tests-results.md#34-wyrażenia-skompilowane-a-interpreter) |
| `update.rs` | 2 | `UPDATE` w miejscu zostawia indeksy, wycofanie i WAL w stanie zgodnym | [3.5](../../../docs/research/tests-results.md#35-update-w-miejscu) |
| `many.rs` | 3 | zapis dla wielu zestawów parametrów daje te same wiersze i WAL co osobne wykonania | [3.9](../../../docs/research/tests-results.md#39-wiele-zestawów-parametrów-jednym-wywołaniem) |
| `snapshot_steps.rs` | 4 | snapshot kodowany krokami jest stanem z chwili rozpoczęcia, mimo zmian między krokami | [3.11](../../../docs/research/tests-results.md#311-snapshot-kodowany-krokami) |

Nowy test opisuje się w `tests-results.md` według [metodyki](../../../docs/research/tests-methodology.md#5-opis-testu), gdy odpowiada na nowe pytanie o gwarancję silnika.
