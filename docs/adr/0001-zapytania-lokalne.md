# 0001. Zapytania wykonywane lokalnie, bez serwera

- Status: obowiązuje
- Ostatnia zmiana: 2026-10-20

## Kontekst

W aplikacji opartej na serwerze każda akcja użytkownika czeka na odpowiedź przez sieć. Trwa to co najmniej tyle, ile podróż pakietu tam i z powrotem, czyli zwykle 20–100 ms. Bez sieci aplikacja nie działa.

## Decyzja

Każde zapytanie i każdy zapis wykonuje się na urządzeniu użytkownika. Hasło „0 ms latencji” z opisu projektu oznacza brak opóźnienia sieci i brak zależności od serwera przy zapytaniu, a nie zerowy czas wykonania.

## Uzasadnienie

- Zapytanie przez sieć nie może być szybsze niż opóźnienie sieci. Lokalny odczyt po kluczu trwa około 10–14 µs, ten sam odczyt przez serwer 20–100 ms, czyli trzy do czterech rzędów wielkości dłużej.
- Interfejs ma około 16 ms na jedną klatkę obrazu (Litt i in., 2023). Jedno zapytanie przez sieć przekracza ten budżet.
- Aplikacja, która nie pyta serwera, działa także bez sieci. To warunek pracy offline (decyzja 0002).

## Odrzucone warianty

- Zapytania przez API serwera. Zostały tylko jako punkt odniesienia w pomiarach (`/api/query`).
- Pamięć podręczna odpowiedzi serwera. Przyspiesza tylko powtarzane odczyty, nie zapisy, i nie daje pracy offline.

## Konsekwencje

- Dane potrzebne do zapytań muszą być na urządzeniu (decyzja 0006), a zmiany trzeba wysyłać na serwer w tle (decyzja 0002).
- Przeniesienie zapytania do innego wątku tego samego urządzenia (Web Worker) nie łamie tej decyzji.

## Historia

- 2026-06-02: założenie z pierwotnego opisu projektu.
- 2026-10-20: doprecyzowano znaczenie „0 ms latencji”.
