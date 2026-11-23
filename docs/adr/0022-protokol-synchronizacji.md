# 0022. Protokół wysyłania i pobierania zmian z kursorem nadawanym przez serwer

- Status: obowiązuje
- Ostatnia zmiana: 2026-10-21

## Kontekst

Urządzenia synchronizują się przez serwer (decyzja 0002) przy przerywanej łączności. Każda runda może się urwać w dowolnym miejscu, a powtórzenie nie może zmienić stanu.

## Decyzja

1. Dwa żądania HTTP. `POST /sync/push` wysyła binarną paczkę zmian z kolejki. `GET /sync/pull?since=N&epoch=E` pobiera paczki innych urządzeń od kursora.
2. Kursor to numer, który serwer nadaje paczce w chwili przyjęcia, a nie znacznik czasu zmiany.
3. Gdy serwer nie zna kursora (nowe urządzenie, restart serwera, skrócony log), odsyła pełny stan bazy. Ponowne zastosowanie stanu jest bezpieczne dzięki regule „ostatni zapis wygrywa”.
4. Kolejność kroków klienta: wysłanie i potwierdzenie dopiero po odpowiedzi 200, pobranie i zastosowanie zmian, zapis WAL, zapis kursora. Bez sieci klient ponawia z rosnącym odstępem (do 30 s) i od razu po zdarzeniu `online`.

## Uzasadnienie

- Lokalne zapisy i serwer porządkujący log to wzorzec z systemu Bayou (Terry i in., 1995). Kursor jako pozycja w logu i pełny stan przy nieznanym kursorze to wzorzec strumieni zmian (Kleppmann, 2017, rozdz. 11).
- Znacznik czasu mówi, kiedy zmianę zrobiono na urządzeniu, a nie kiedy dotarła na serwer. Zmiana zrobiona offline o 11:30 i wysłana o 12:30 ma znacznik 11:30. Urządzenie, które pobrało już wszystko do 12:00, przy kursorze czasowym nigdy by jej nie dostało.
- Powtórzone zmiany mają równe znaczniki i są ignorowane, więc przerwanie w dowolnym miejscu kończy się powtórzeniem rundy (Helland, 2007).

## Odrzucone warianty

- Kursor jako znacznik czasu i odpowiedź wyliczana ze stanu bazy. Gubi zmiany wysłane z opóźnieniem.
- Połączenie WebSocket od początku. Odpytywanie wystarcza do pracy offline.
- Uzgadnianie zbiorów zamiast pełnego stanu. Większa złożoność.

## Konsekwencje

- W prototypie log serwera jest tylko w pamięci, więc każdy restart serwera wysyła wszystkim pełny stan. Trwały log to zadanie na później.
- Serwer przyjmuje każdą poprawną zmianę, a każde urządzenie dostaje całą bazę.

## Historia

- 2026-10-06: protokół pierwszej wersji serwera.
- 2026-10-21: kursor jako numer przyjęcia także w projekcie docelowego serwera.
