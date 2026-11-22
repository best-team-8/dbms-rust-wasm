# 0002. Praca offline, synchronizacja w tle przez serwer

- Status: obowiązuje
- Ostatnia zmiana: 2026-10-20

## Kontekst

Projekt obiecuje pełną pracę bez sieci i wymianę zmian w tle (model Local-First). Trzeba było rozstrzygnąć, czy urządzenia wymieniają zmiany bezpośrednio między sobą, czy przez serwer, i kto jest źródłem prawdy dla danych wspólnych.

## Decyzja

1. Aplikacja działa w pełni bez sieci. Brak połączenia oznacza tylko, że rośnie kolejka zmian czekających na wysłanie (outbox).
2. Urządzenia wymieniają zmiany przez serwer.
3. Dla danych wspólnych, takich jak ceny, rankingi czy uprawnienia, źródłem prawdy jest serwer. Urządzenie trzyma kopię potrzebnych danych. Flat-SQL uzupełnia bazę serwerową, a nie ją zastępuje.

## Uzasadnienie

- Klient z trwałym logiem zmian, który po połączeniu odtwarza je na serwerze, to sprawdzony wzorzec (Kistler i Satyanarayanan, 1992; Terry i in., 1995). Aplikacje mobilne bez takiego mechanizmu tracą dane przy przerwie w sieci (Go i in., 2015).
- Tak działają też analizowane wdrożenia: Actual Budget, Linear, PowerSync.
- Część operacji wymaga koordynacji z natury, na przykład zapobieganie podwójnemu wydaniu środków albo unikalność między urządzeniami (Gray i in., 1996; Kleppmann, 2017, rozdz. 9).

## Odrzucone warianty

- Wymiana zmian bezpośrednio między urządzeniami. Trudniej kontrolować dostęp i nie ma autorytetu dla danych wspólnych.
- Każdy zapis potwierdzany przez serwer. Przeczy pracy offline.

## Konsekwencje

- Konflikty są nieuniknione, więc potrzebna jest reguła ich rozstrzygania dająca ten sam wynik na każdym urządzeniu (decyzja 0019).
- Operacje wymagające koordynacji trzeba robić przez zapisy wstępne, które serwer zatwierdza (decyzja 0024).
- Serwer potrzebuje trwałego logu, uwierzytelniania i reguł. Prototyp ich nie ma.

## Historia

- 2026-06-02: założenie z pierwotnego opisu projektu.
- 2026-10-20: dopisano rolę serwera jako źródła prawdy dla danych wspólnych.
