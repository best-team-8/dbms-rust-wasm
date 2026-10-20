# ADR-0024: Bez wiązań dla Pythona i synchronizacji w procesie backendu

| | |
|---|---|
| Status | przyjęta |
| Data | 2026-10-20 |
| Rodzaj | proces |
| Powiązane | ADR-0005, ADR-0022, ADR-0023; future 9.1, 9.5, 9.6, dawne 9.2–9.4 |

## Kontekst

[todo_future.md](../todo_future.md) §9 planował łańcuch zadań nad interfejsem C (9.1): pakiet Pythona (9.2), obsługę `/sync/push` i `/sync/pull` w FastAPI na tych wiązaniach (9.3) i testy zgodności serwera w Node.js i w Pythonie (9.4). Dawałoby to drugą implementację serwera synchronizacji – w procesie backendu w Pythonie, bez osobnego serwera Node.js („wariant B” w [use-cases.md §8](../use-cases.md)). FastAPI pojawiało się w dokumentacji tylko jako przykład backendu, a nie jako wybrana technologia.

## Decyzja

Wiązania dla Pythona i synchronizacja w procesie backendu wypadają z planów (decyzja autora). Serwer synchronizacji ma jedną implementację – program w Node.js z silnikiem WASM, z trybami z ADR-0023. Interfejs C (9.1) zostaje, bo opierają się na nim aplikacje natywne (9.5) i panel administracyjny WinUI 3 (9.6). Numery 9.2–9.4 nie są używane ponownie.

## Uzasadnienie

- Projekt nie potrzebuje backendu w Pythonie; był to tylko przykład możliwego użycia.
- Druga implementacja serwera dublowałaby log, epoki, kursory, kompaktowanie i tryby z ADR-0023 oraz wymagała stałego testowania zgodności (dawne 9.4).
- Protokół HTTP i binarny format paczek są niezależne od języka, więc backend w dowolnym języku może współpracować z serwerem synchronizacji bez wiązań.

## Odrzucone alternatywy

- **Zachowanie 9.2–9.4 jako rozszerzenia** – koszt utrzymania drugiej implementacji serwera bez potwierdzonej potrzeby.

## Konsekwencje

- Serwer synchronizacji działa jako osobny proces Node.js; backend aplikacji komunikuje się z nim przez HTTP.
- Wiązania dla innych języków nadal są możliwe przez interfejs C (9.5), ale nie obejmują serwera synchronizacji.

## Źródła

Decyzja autora; [use-cases.md §8](../use-cases.md).
