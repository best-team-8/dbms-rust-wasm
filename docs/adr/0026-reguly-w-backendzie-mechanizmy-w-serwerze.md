# ADR-0026: Reguły w backendzie, mechanizmy w serwerze synchronizacji

| | |
|---|---|
| Status | przyjęta |
| Data | 2026-10-20 |
| Rodzaj | architektura |
| Powiązane | ADR-0002, ADR-0023, ADR-0025; future 2.1–2.4, 2.6 |

## Kontekst

Synchronizacja jest automatyczna: każda zmiana w lokalnej bazie urządzenia trafia paczką na serwer, z pominięciem endpointów backendu. Bramka (ADR-0025) przekazuje paczki bez czytania. Urządzeniu nie można ufać – użytkownik może zmienić lokalną bazę w narzędziach przeglądarki (np. ceny w sklepie albo złoto w grze wieloosobowej).

Plan w [todo_future.md](../todo_future.md) §2 zakładał walidację zmian według reguł zapisanych w samym serwerze synchronizacji. Reguły biznesowe (co jest oszustwem, czy zamówienie jest poprawne) zależą jednak od aplikacji i należą do backendu programisty.

## Decyzja

Serwer synchronizacji nie zawiera logiki biznesowej. Daje backendowi dwa mechanizmy, oba w trybie z silnikiem (ADR-0023) i oba opcjonalne, konfigurowane dla przestrzeni:

1. **Tabele tylko do odczytu dla klientów.** W konfiguracji przestrzeni programista wymienia tabele, których klienci nie mogą zmieniać (np. `"clientReadOnly": ["prices"]`). Serwer odrzuca paczki klientów zmieniające te tabele. Zapisuje je tylko backend przez wewnętrzne `/api/query`, a zmiany trafiają do urządzeń jak każde inne. Bez kodu po stronie programisty.
2. **Webhook do backendu.** W konfiguracji przestrzeni programista podaje adres backendu (np. `"validate": "http://backend:3000/flatsql/validate"`). Przed przyjęciem paczki serwer wysyła pod ten adres zmiany jako JSON (tabela, klucz, stan przed i po) – jedno żądanie na paczkę. Backend odpowiada listą odrzuconych zmian. Serwer ich nie zapisuje, nie rozsyła i odsyła urządzeniu poprawny stan.

Przestrzeń w trybie magazynu paczek nie ma reguł – serwer tylko przechowuje i rozsyła paczki.

## Uzasadnienie

- **Podział odpowiedzialności.** Backend i tak ma konta, sesje i logikę aplikacji. Serwer zostaje ogólnym narzędziem – nie zna pojęć takich jak „złoto” czy „oszustwo”.
- **Bez nakładek.** Oba mechanizmy działają przez HTTP i JSON, więc backend w dowolnym języku z nich korzysta (ADR-0025).
- **Tabele tylko do odczytu zamykają obejście.** Gdy dane kontrolowane zapisuje tylko backend, użytkownik i tak mógłby zmienić lokalną kopię i wysłać ją synchronizacją. Odrzucenie takich paczek to uprawnienie (jak „tylko do odczytu” na pliku), nie logika biznesowa.
- **Webhook dla zmian offline.** Tam, gdzie urządzenie zmienia dane offline, a nie wolno mu ufać, backend sprawdza zmiany po synchronizacji. To transakcje wstępne zatwierdzane przez właściciela kopii głównej z replikacji dwuwarstwowej [C9] i zapisy wstępne w Bayou [C3].
- **Koszt mały.** `SyncClient` grupuje zmiany w paczki (do 1000 zmian), więc backend dostaje jedno żądanie na paczkę, nie na zmianę. Użytkownik nie czeka – synchronizacja działa w tle.
- **Potwierdzenie w praktyce.** W PowerSync zapisy trafiają do backendu programisty, który je sprawdza; w Firebase uprawnienia deklaruje się w konfiguracji usługi.

Zastosowania z [real-world-usage.md](../research/real-world-usage.md), w których webhook się przydaje: kasy offline (stan magazynu i ceny zamówień zrobionych offline), pamięć podręczna SaaS (uprawnienia w zespole), gry wieloosobowe (oszustwa), czasem zbieranie danych w terenie (poprawność formularzy). Zbędny w aplikacjach osobistych i zapisach gier jednoosobowych.

## Odrzucone alternatywy

- **Reguły biznesowe zapisane w serwerze synchronizacji** (dawne future 2.2: zakresy wartości, właściciel wiersza w konfiguracji) – serwer musiałby rozumieć logikę każdej aplikacji; prosty język reguł szybko staje się za słaby.
- **Kod reguł uruchamiany w serwerze** (funkcje JavaScript wgrywane do serwera) – wiąże programistę z JavaScriptem i miesza kod aplikacji z serwerem.
- **Brak mechanizmów, wszystkie kontrolowane zapisy przez endpointy backendu** – nie zamyka obejścia przez lokalną bazę i automatyczną synchronizację; zmiany offline nie mogłyby być sprawdzone.

## Konsekwencje

- Dane kontrolowane mają dwie drogi: zapis tylko przez backend (tabela tylko do odczytu, wymaga sieci) albo zapis lokalny sprawdzany później przez webhook (działa offline, odrzucona zmiana znika z urządzenia po synchronizacji).
- Zapytania działają w obrębie jednej przestrzeni. Dane porównywane między użytkownikami (np. ranking) programista trzyma w bazie backendu albo we wspólnej przestrzeni, do której zapisuje backend.
- Zadania: future 2.1 (tabele tylko do odczytu), 2.2 (webhook), 2.3 (odesłanie poprawnego stanu po odrzuceniu), 2.4 (zamiary sprawdzane przez backend).

## Źródła

[C3], [C9]; [real-world-usage.md](../research/real-world-usage.md) §1–4; [PowerSync – dokumentacja](https://docs.powersync.com/overview/readme-1).
