# ADR-0043: Outbox wierszy prowadzony od włączenia synchronizacji, zwarty zapis w snapshocie

| | |
|---|---|
| Status | przyjęta |
| Data | 2026-10-25 |
| Rodzaj | architektura |
| Powiązane | ADR-0002, ADR-0009, ADR-0011, ADR-0016 (zmienia), ADR-0039; zadanie todo 2.11 |

## Kontekst

ADR-0016 przyjął outbox jako referencje `(tabela, klucz) → seq` dla każdej lokalnej zmiany wiersza, odbudowywane z WAL i zapisywane w snapshocie. Benchmark 1 i rozbicie kosztów pokazały koszt tej decyzji w aplikacji, która w ogóle nie synchronizuje: po wstawieniu 100 000 wierszy outbox ma 100 002 wpisy, a snapshot zapisuje przy każdym nazwę tabeli. Outbox to 25% rozmiaru snapshotu (5,55 wobec 4,17 MB), około jednej trzeciej czasu snapshotu, połowa czasu odtworzenia (natywnie 118 wobec 59 ms) i 42% jego alokacji. Ten koszt płaci każda aplikacja, w tym serwer synchronizacji, który własnego outboxa używa tylko do zapisów przez `/api/query`.

## Decyzja

1. Baza ma flagę synchronizacji (`Database::enable_sync`, w JavaScripcie `enableSync`), zapisywaną w WAL nową operacją `Sync(seq)` i w nagłówku snapshotu. Dopóki synchronizacja jest wyłączona, outbox przyjmuje tylko zmiany schematu (są rzadkie i zachowują np. usunięcie tabeli przed pierwszą synchronizacją); lokalne zmiany wierszy nie tworzą wpisów, choć nadal dostają numer `seq` w WAL.
2. Włączenie synchronizacji oznacza każdy wiersz i każdy tombstone jako oczekujący, z numerami od `seq` zapisanego w operacji WAL, w kolejności katalogu i kluczy. Pierwsza wysyłka zawiera więc cały stan, czyli dokładnie to, co zawierałby outbox bazy prowadzony od początku (wpisy są referencjami, a wiele zmian jednego wiersza i tak daje jedną zmianę). Odtworzenie z WAL powtarza ten krok w tym samym miejscu historii z tymi samymi numerami.
3. `SyncClient` włącza synchronizację przy utworzeniu, serwer synchronizacji przy starcie. Pozostałe punkty ADR-0016 (referencje zamiast kopii, potwierdzenia `Ack` w WAL, brak echa zmian z serwera) obowiązują bez zmian.
4. Snapshot ma format 2: bajt flag po nagłówku, a outbox w dwóch częściach: zmiany schematu (`seq`, zmiana) oraz wiersze pogrupowane po tabeli (nazwa raz, potem pary `seq`, klucz). W pamięci oczekujące wiersze są mapą klucz → `seq` dla każdej tabeli. Format 1 jest nadal czytany; baza w formacie 1 zawsze prowadziła outbox, więc ma synchronizację włączoną.

## Uzasadnienie

- Pomiar przed zmianą i po niej ([optimization-results.md §5](../research/optimization-results.md)): bez synchronizacji snapshot 5,55 → 4,17 MB, alokacje odtworzenia 559 268 → 325 937; z włączoną synchronizacją zwarty zapis daje snapshot 4,86 MB i odtworzenie natywnie 82–87 ms zamiast 112–129 ms.
- Log zmian offline, w którym nowszy zapis unieważnia wcześniejsze wpisy [C10], i wysyłanie stanu zmienionych wierszy (delta-state [L28]) zostają; zmienia się tylko moment, od którego log jest prowadzony. Dla bazy, która jeszcze nie synchronizowała, każdy wiersz i tak jest zmianą do wysłania, więc pełny stan przy włączeniu jest semantycznie tym samym zbiorem.
- Brak synchronizacji jest też przypadkiem domyślnym serwera bez zapisów przez `/api/query` i aplikacji używających Flat-SQL jako lokalnej bazy bez serwera (ADR-0001).

## Odrzucone alternatywy

- **Odtworzenie zmian sprzed włączenia z WAL** – WAL jest obcinany przy checkpoincie (ADR-0009), więc zmian sprzed ostatniego snapshotu w nim nie ma; trzeba by trzymać cały WAL od utworzenia bazy.
- **Sam zwarty zapis bez flagi** – zmniejsza snapshot i odtworzenie z outboxem, ale baza bez synchronizacji nadal trzymałaby wpis dla każdego wiersza.
- **Wyłączona synchronizacja jako stan bez zmian schematu w outboxie** – zmiana schematu wykonana przed włączeniem (np. usunięcie tabeli) zginęłaby; zmiany schematu są rzadkie, więc ich przechowywanie nic nie kosztuje.
- **Włączanie przy pierwszym `export_outbox`** – eksport nie zmienia stanu bazy i nie jest zapisywany w WAL; jawny krok jest trwały i widoczny w statystykach (`sync`).

## Konsekwencje

- Zmiana API: aplikacja, która wysyła outbox inaczej niż przez `SyncClient`, wywołuje `enableSync()`; testy synchronizacji w rdzeniu włączają ją przy tworzeniu replik.
- Pierwsza synchronizacja bazy z dużą liczbą wierszy wysyła pełny stan, tak jak przed zmianą.
- Format snapshotu ma wersję 2; odczyt wersji 1 sprawdza test jednostkowy z koderem starego formatu.
- Zmienia ADR-0016 w zakresie momentu prowadzenia outboxa i formatu jego zapisu.

## Źródła

[C10], [L28]; [optimization-results.md §5](../research/optimization-results.md), [tests-results.md §2.4](../research/tests-results.md), [benchmark-results.md §1.12](../research/benchmark-results.md#112-rozbicie-kosztów-silnika-pomiar-pomocniczy).
