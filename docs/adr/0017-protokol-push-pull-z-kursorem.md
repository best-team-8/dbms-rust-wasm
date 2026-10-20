# ADR-0017: Protokół push/pull z kursorem i epoką

| | |
|---|---|
| Status | przyjęta |
| Data | 2026-10-06 (serwer i bridge, [architecture.md §5.3–5.4](../architecture.md)) |
| Rodzaj | architektura |
| Powiązane | ADR-0002, ADR-0016; todo 3.2; future 3.7, 6.2, §7 |

## Kontekst

Klienci synchronizują się przez serwer (ADR-0002) przy przerywanej łączności. Każda runda może się urwać w dowolnym miejscu, a ponowienie nie może zmienić stanu.

## Decyzja

1. Dwa żądania HTTP: `POST /sync/push` (binarna paczka zmian z outboksu, odpowiedź z liczbą przyjętych, zignorowanych i odrzuconych) i `GET /sync/pull?since=N&epoch=E` (paczki po kursorze, bez własnych zmian klienta).
2. Serwer utrzymuje własną replikę z tym samym silnikiem WASM i log przyjętych paczek z numerami `seq`; konkatenacja paczek jest poprawną paczką.
3. Gdy kursor jest nieznany (nowy klient, nowa epoka serwera, log skompaktowany), serwer odsyła pełny stan bazy – bezpieczny do ponownego zastosowania dzięki LWW.
4. Kolejność kroków klienta: push i `ack` dopiero po odpowiedzi 200 → pull i zastosowanie zmian → `flush` WAL → zapis kursora. Przy braku sieci – ponawianie z wykładniczym odstępem (do 30 s) i natychmiast po zdarzeniu `online`.

## Uzasadnienie

- Lokalne zapisy i serwer porządkujący log to wzorzec Bayou [C3]; kursor i pełny stan przy nieznanym kursorze to offsety konsumentów i Change Data Capture ze snapshotem początkowym ([P1] rozdz. 11).
- Idempotencja: powtórzone zmiany mają równe znaczniki i są ignorowane, więc przerwanie w dowolnym miejscu kończy się powtórzeniem rundy ([C7], [P1] rozdz. 12).
- Serwer z tym samym silnikiem nie wymaga osobnej implementacji scalania.

## Odrzucone alternatywy

- **WebSocket od początku** – prostszy pull wystarcza do pracy offline; push dla współpracy na żywo – future 6.2.
- **Uzgadnianie zbiorów zamiast pełnego stanu** – większa złożoność; warunkowo w future 3.7.

## Konsekwencje

- Log serwera jest tylko w pamięci, a epoka losowana przy starcie, więc **każdy restart serwera** wysyła wszystkim klientom pełny stan – trwały log to todo 3.2.
- Serwer przyjmuje każdą poprawną zmianę – reguły i uwierzytelnianie to future §2 i 4.1.
- Każdy klient dostaje całą bazę – replikacja częściowa to future 3.2.

## Źródła

[C3], [C7], [C10], [P1] rozdz. 11–12.
