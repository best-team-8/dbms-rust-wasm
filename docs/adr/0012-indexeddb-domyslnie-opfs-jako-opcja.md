# ADR-0012: IndexedDB jako magazyn domyślny, OPFS jako opcja

| | |
|---|---|
| Status | przyjęta |
| Data | 2026-10-06 (bridge); uzasadnienie uzupełnione 2026-10-20 |
| Rodzaj | architektura |
| Powiązane | ADR-0005, ADR-0010, ADR-0013; todo 1.4, 1.5, 1.10; future 1.5 |

## Kontekst

Przeglądarka ma dwa trwałe magazyny nadające się do bazy: IndexedDB (baza klucz–wartość) i OPFS (prywatny system plików strony). OPFS ma dwa interfejsy: asynchroniczny, dostępny także w wątku głównym, oraz szybki synchroniczny (`createSyncAccessHandle`), dostępny **wyłącznie w Web Workerze**. Pomiary producentów pokazują przewagę OPFS, głównie interfejsu synchronicznego [Q8].

## Decyzja

Domyślnym magazynem przeglądarki jest IndexedDB (`IndexedDbStore`): WAL dopisywany jako kolejne wpisy, checkpoint = zapis snapshotu i wyczyszczenie WAL w **jednej** transakcji IndexedDB. OPFS będzie dodatkowym magazynem, przeznaczonym do trybu Worker (future 1.5).

## Uzasadnienie

1. **Wybór magazynu nie jest wyborem silnika.** Rdzeń nie robi I/O (ADR-0005) – magazyn to wymienny moduł w `bridge/stores/`. Dodanie OPFS nie zmienia silnika ani formatu danych; to różni Flat-SQL od SQLite, w którym magazyn (VFS) czyta strony synchronicznie z wnętrza silnika i dlatego wymusza Workera.
2. **To, co dziś jest wolne, nie zależy od dysku.** Flat-SQL czyta z pamięci, a na dysk zapisuje małe ramki WAL zbierane przez group commit (0,34 ms na trwałe zatwierdzenie). Wolne są kodowanie snapshotu i odtwarzanie, czyli praca procesora – szybszy magazyn ich nie przyspieszy.
3. **Atomowy checkpoint.** IndexedDB zapisuje snapshot i czyści WAL w jednej transakcji. OPFS nie ma transakcji obejmujących kilka plików, więc potrzebny byłby protokół zmiany nazwy pliku z jego pułapkami [S8].
4. **Szybki OPFS wymaga Workera,** a domyślnym trybem jest wątek główny (ADR-0013). Interfejs asynchroniczny zapisuje przez plik tymczasowy podmieniany przy zamknięciu, a dopisanie do istniejącego pliku wymaga skopiowania jego zawartości – niekorzystne dla logu dopisywanego co transakcję (według specyfikacji File System; nie mierzono).
5. IndexedDB jest dostępna wszędzie: w wątku głównym, w Workerach i w workerze tła rozszerzeń.

## Odrzucone alternatywy

- **OPFS jako magazyn domyślny** – z powodów 2–4.
- **Stronicowy magazyn na OPFS dla danych większych niż pamięć** – zmienia ADR-0006.

## Konsekwencje

- Domyślna trwałość IndexedDB w Chrome 121+ to `relaxed` – trzeba to opisać i udostępnić `strict` (todo 1.4) [Q1].
- Przeglądarka może usunąć dane – potrzebne `navigator.storage.persist()` (todo 1.5) [Q2]–[Q4].
- OPFS w trybie Worker (future 1.5) ma sens dla dużych baz i dużych snapshotów.

## Źródła

[Q1]–[Q4], [Q7], [Q8], [S8]; [benchmark-results.md §3](../research/benchmark-results.md).
