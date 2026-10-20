# ADR-0001: Zapytania wykonywane lokalnie, bez udziału serwera

| | |
|---|---|
| Status | przyjęta |
| Data | 2026-06-02 (README, prezentacja projektu) |
| Rodzaj | założenie |
| Powiązane | ADR-0002, ADR-0006, ADR-0013; todo 0.1 |

## Kontekst

W modelu Cloud-First każda akcja użytkownika czeka na odpowiedź serwera: co najmniej czas obiegu sieci (RTT, 20–100 ms), a bez sieci aplikacja nie działa. README nazywa cel projektu „0 ms Latency” i przenosi operacje bazy danych na urządzenie użytkownika (paradygmat edge computing, w którym brzegiem sieci jest samo urządzenie – [E1]).

## Decyzja

Każde zapytanie i każdy zapis wykonuje się na urządzeniu użytkownika, bez udziału serwera. „0 ms latencji” oznacza **brak opóźnienia sieci i brak zależności od serwera przy zapytaniu**, a nie dosłownie zerowy czas wykonania. Mikrosekundowe czasy lokalnych zapytań są skutkiem tej decyzji, nie jej celem.

## Uzasadnienie

- Czas zapytania przez sieć jest ograniczony od dołu przez RTT; lokalnie ograniczeniem jest tylko wykonanie (H1: 11 µs lokalnie vs 20,9–102,5 ms przy RTT 20–100 ms).
- Przy budżecie 16 ms na klatkę interfejsu jedno zapytanie przez sieć z RTT 20 ms już go przekracza [D4].
- Aplikacja, która nie pyta serwera, działa także bez sieci – to warunek ADR-0002.

## Odrzucone alternatywy

- **Zapytania przez API serwera (Cloud-First)** – zostaje tylko jako punkt odniesienia w badaniach (`/api/query`, H1).
- **Pamięć podręczna odpowiedzi API** – przyspiesza tylko powtórzone odczyty, nie zapisy; nie daje pracy offline.

## Konsekwencje

- Dane potrzebne do zapytań muszą być na urządzeniu (ADR-0006), a zmiany – synchronizowane w tle (ADR-0002).
- Przeniesienie zapytania do innego wątku na tym samym urządzeniu (Web Worker) nie narusza tego założenia – zob. ADR-0013.
- Teza w README wymaga doprecyzowania: „bez opóźnienia sieci”, nie „zero milisekund” (todo 0.1).

## Źródła

[E1], [D4], [L1]; [benchmark-results.md §4](../research/benchmark-results.md).
