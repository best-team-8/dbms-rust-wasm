# ADR-0002: Local-First – praca offline, synchronizacja w tle, serwer koordynujący

| | |
|---|---|
| Status | przyjęta |
| Data | 2026-06-02 (README); model serwera doprecyzowany 2026-10-18 ([use-cases.md §1](../use-cases.md)) |
| Rodzaj | założenie |
| Powiązane | ADR-0001, ADR-0015–ADR-0018; future §2, §3 |

## Kontekst

README zakłada „pełną funkcjonalność offline połączoną z asynchroniczną synchronizacją stanu”. Trzeba było rozstrzygnąć, czy repliki synchronizują się bezpośrednio (peer-to-peer), czy przez serwer, i kto jest źródłem prawdy dla danych wspólnych.

## Decyzja

1. Aplikacja działa w pełni bez sieci; synchronizacja jest dodatkiem, nie warunkiem działania – brak sieci oznacza tylko, że rośnie outbox.
2. Repliki synchronizują się przez serwer (topologia gwiazdy, docelowo drzewa przez węzły edge – future §7).
3. Dla danych wspólnych (rankingi, ceny, płatności, uprawnienia) źródłem prawdy jest serwer; urządzenie trzyma kopię potrzebnych danych. Flat-SQL uzupełnia bazę serwerową, nie zastępuje jej.

## Uzasadnienie

- Klient pracujący offline z trwałym logiem zmian i reintegracją po połączeniu to sprawdzony wzorzec ([C10], [C3]); aplikacje mobilne bez takiego mechanizmu tracą dane [L27].
- W praktyce wykonalne są modele hybrydowe z minimalną koordynacją serwera, w pełni bezserwerowe systemy rzadko [L20]; tak działają też analizowane wdrożenia (Actual Budget, Linear, PowerSync – [real-world-usage.md](../research/real-world-usage.md)).
- Część operacji wymaga koordynacji z natury: zapobieganie podwójnemu wydaniu środków, unikalność między replikami ([C8], [C9], [P1] rozdz. 9).

## Odrzucone alternatywy

- **Synchronizacja peer-to-peer bez serwera** (Merkle-CRDT, sieci odporne na uczestników bizantyjskich) – trudniejsza kontrola dostępu i brak autorytetu dla danych wspólnych; prace przeniesione poza rdzeń literatury.
- **Silna spójność (każdy zapis potwierdzany przez serwer)** – przeczy pracy offline i ADR-0001.

## Konsekwencje

- Konflikty są nieuniknione i potrzebna jest deterministyczna reguła ich rozstrzygania (ADR-0015).
- Dane wspólne i operacje wymagające koordynacji realizuje się przez zapisy wstępne zatwierdzane przez serwer (future 2.1–2.4, wzór: replikacja dwuwarstwowa [C9]).
- Serwer musi mieć trwały log (todo 3.2), reguły i uwierzytelnianie (future §2, 4.1).

## Źródła

[L1], [L20], [L27], [C3], [C8], [C9], [C10], [P1] rozdz. 5 i 9.
