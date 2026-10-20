# ADR-0028: Kursor synchronizacji jako numer przyjęcia nadany przez serwer

| | |
|---|---|
| Status | przyjęta |
| Data | 2026-10-21 |
| Rodzaj | architektura |
| Powiązane | ADR-0015, ADR-0017, ADR-0023; todo 3.2, 3.4 |

## Kontekst

Urządzenie pobiera zmiany od swojego kursora (`/sync/pull?since=N`). W prototypie kursor to numer `seq` paczki w logu serwera (ADR-0017). Przy projektowaniu trybów serwera (ADR-0023) rozważono, czy w trybie z silnikiem log jest potrzebny: serwer ma aktualny stan, a każdy wiersz ma znacznik HLC, więc „co nowego” dałoby się wyliczyć ze stanu jako „wiersze ze znacznikiem późniejszym niż T”.

## Decyzja

1. Kursor to **numer nadawany przez serwer w chwili przyjęcia zmiany** (`seq` w logu przestrzeni), a nie znacznik czasu zmiany.
2. Log paczek z tą numeracją jest wspólną podstawą obu trybów serwera; w magazynie paczek jest jedyną formą danych, w trybie z silnikiem istnieje obok bazy przestrzeni.

## Uzasadnienie

- **Znacznik HLC mówi, kiedy zmianę zrobiono na urządzeniu, a nie kiedy dotarła na serwer.** Zmiana wykonana offline o 11:30 i wysłana o 12:30 ma znacznik 11:30. Urządzenie, które wcześniej pobrało wszystko do 12:00, przy kursorze czasowym nigdy by jej nie dostało.
- **Numer przyjęcia to wzorzec z literatury.** W Bayou [C3] serwer główny nadaje zapisom numery w kolejności przyjęcia; w logach zmian konsument śledzi offset w logu ([P1] rozdz. 11). `literature-comparison.md` §4.5 ocenia kursor i epokę jako zgodne z tym wzorcem.
- **Magazyn paczek nie ma stanu**, więc i tak potrzebuje logu – jeden mechanizm dla obu trybów jest prostszy.

## Odrzucone alternatywy

- **Kursor jako znacznik HLC, odpowiedź wyliczana ze stanu** – gubi zmiany wykonane offline i wysłane z opóźnieniem.
- **Stan z numerem przyjęcia przy każdym wierszu, bez logu (tylko tryb z silnikiem)** – poprawny wariant, który oszczędza podwójne przechowywanie danych, ale wymaga indeksu po numerze przyjęcia i nie działa w magazynie paczek. Możliwa późniejsza optymalizacja trybu z silnikiem, bez zadania.

## Konsekwencje

- Log musi być trwały i numeracja nie może się cofać (todo 3.2); po utracie logu zmienia się epoka, a urządzenia dostają pełny stan.
- Kompaktowanie zastępuje stare paczki punktem startowym, ale zachowuje numerację kolejnych paczek; urządzenie z kursorem sprzed kompaktowania dostaje pełny stan i resztę logu.
- Usuwanie tombstone'ów może korzystać z najmniejszego potwierdzonego kursora aktywnych urządzeń (todo 3.4).

## Źródła

[C3], [P1] rozdz. 11; [literature-comparison.md §4.5](../references/literature-comparison.md#45-protokół-synchronizacji).
