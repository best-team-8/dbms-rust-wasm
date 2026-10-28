# ADR-0049: Checkpoint kodowany krokami z zachowaniem wierszy zmienianych w trakcie, snapshot w IndexedDB jako Blob

| | |
|---|---|
| Status | przyjęta |
| Data | 2026-10-28 |
| Rodzaj | architektura |
| Powiązane | ADR-0008, ADR-0010, ADR-0013, ADR-0033, ADR-0039; zadanie todo 2.6 |

## Kontekst

Checkpoint (snapshot całej bazy, zapis do magazynu, obcięcie WAL) blokował wątek główny przeglądarki na 40 ms przy 100 000 wierszy w Benchmarku 1 i na 28 ms po zmianach z todo 2.1–2.7: snapshot był kodowany jednym wywołaniem, a cała jego suma CRC liczona na końcu. Rozkład pracy (skrypt `stages.mjs`, `idb-put.mjs`): kodowanie wierszy około 9 ms, CRC około 9 ms, kopia bajtów do JavaScriptu około 1 ms, `put` bajtów do IndexedDB 4,4 ms synchronicznie. ADR-0013 trzyma silnik w wątku głównym, więc pauzę trzeba skrócić bez wątków.

## Decyzja

1. Rdzeń koduje snapshot krokami: `snapshot_begin`, `snapshot_step(rows)`, `snapshot_finish`. Snapshot jest stanem z chwili rozpoczęcia, z LSN tej chwili: nagłówek, tombstones i outbox kodowane od razu, wiersze krok po kroku w kolejności kluczy, suma CRC liczona po każdym kroku. Przed zmianą wiersza, do którego kroki jeszcze nie doszły, zachowywany jest jego stan z chwili rozpoczęcia (kopia przy zmianie), więc zapisy między krokami nie trafiają do snapshotu; WAL zapisany po rozpoczęciu jest odtwarzany na snapshot jak dotąd (ramki z LSN nie większym niż snapshotu są pomijane). Zmiana schematu albo `rebase_clock` (zmienia znaczniki bez WAL) rozpoczyna snapshot od nowa, poza otwartą transakcją. `snapshot()` używa tego samego kodu jednym krokiem, więc daje te same bajty.
2. `FlatDB` wykonuje checkpoint krokami `checkpointStepRows` wierszy (domyślnie 4096; `Infinity` jednym wywołaniem), z obrotem pętli zdarzeń między krokami, przed skopiowaniem bajtów i przed zapisem do magazynu. Obrót to wiadomość do samego siebie (`MessageChannel`), w Node.js `setImmediate`.
3. `IndexedDbStore` zapisuje snapshot jako `Blob`; odczyt przyjmuje też bajty zapisane przez wcześniejsze wersje.
4. Wycofanie `UPDATE` nie zmienia wersji katalogu (dotąd traktowane jak zmiana schematu, co niepotrzebnie unieważniało plany i rozpoczynałoby snapshot od nowa).

## Uzasadnienie

- Pomiar ([optimization-results.md §15](../research/optimization-results.md#15-checkpoint-krokami)): najdłuższa pauza pętli zdarzeń przy checkpoincie 100 000 wierszy 27,8 → 1,70 ms (poziom sondy w bezczynności, około 2 ms), przy 50 000 14,7 → 1,38 ms, cały checkpoint bez zmiany (34,0 → 33,7 ms).
- `put` bajtów serializuje je w wątku wywołującym (4,4 ms dla 4,5 MB w Edge), `put` obiektu `Blob` z utworzeniem go 1,6 ms.
- Kopia przy zmianie zachowuje semantykę snapshotu jako stanu z jednej chwili bez blokowania zapisów aplikacji, których API jest synchroniczne.

## Odrzucone alternatywy

- **Snapshot przyrostowy (tylko wiersze zmienione od poprzedniego)** – zmienia format odtwarzania (łańcuch plików przyrostowych) i model awarii testowany dla checkpointu (ADR-0033).
- **Worker** – silnik w wątku głównym jest decyzją ADR-0013; przeniesienie snapshotu do wątku wymagałoby kopii bazy albo pamięci współdzielonej z nagłówkami COOP/COEP.
- **Wiersze współdzielone (`Arc`) z kopią przy zapisie podczas snapshotu** – koszt licznika referencji przy każdym odczycie i zapisie.
- **Blokowanie zapisów na czas checkpointu** – API zapisu `FlatDB` jest synchroniczne.
- **`scheduler.yield()`** – kontynuacja ma pierwszeństwo przed innymi zadaniami strony, więc te czekałyby do końca checkpointu; **`setTimeout(0)`** – zagnieżdżone odliczania są wydłużane do 4 ms.
- **Snapshot w IndexedDB w porcjach zapisywanych w kolejnych zadaniach** – zmienia układ magazynu i komplikuje odczyt; `Blob` wystarcza.

## Konsekwencje

- Wiersze zmieniane w trakcie checkpointu zajmują pamięć kopii do jego końca (proporcjonalnie do liczby zmienionych wierszy).
- Snapshot nie zawiera zapisów wykonanych w trakcie checkpointu; trafiają do WAL zapisywanego po nim.
- Moduł WebAssembly większy o 5,1 KiB po kompresji gzip.
- Magazyn IndexedDB zapisany tą wersją nie jest czytelny dla wersji wcześniejszych (snapshot jako `Blob`).
- Poprawność: [tests-results.md §3.11](../research/tests-results.md#311-snapshot-kodowany-krokami).

## Źródła

[optimization-results.md §15](../research/optimization-results.md#15-checkpoint-krokami); [benchmark-results.md §1.7](../research/benchmark-results.md#17-pauza-checkpointu).
