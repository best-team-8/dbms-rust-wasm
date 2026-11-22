# 0024. Docelowy serwer synchronizacji: dwa tryby, za bramką backendu, reguły w backendzie

- Status: proponowana (projekt docelowy, prototyp go nie realizuje)
- Ostatnia zmiana: 2026-10-21

## Kontekst

Prototypowy serwer trzyma jedną bazę dla wszystkich klientów, przyjmuje każdą zmianę i nie zna użytkowników. Analiza wdrożeń pokazała dwie grupy zastosowań. Dane jednej osoby (aplikacje osobiste, zapisy gier) wymagają od serwera tylko przechowania i rozesłania zmian. Dane wspólne (kasy, rankingi, dane serwera w aplikacjach SaaS) wymagają, żeby serwer rozumiał wiersze i je sprawdzał. Urządzeniu nie można ufać: użytkownik może zmienić lokalną bazę w narzędziach przeglądarki.

## Decyzja

1. Dane dzielą się na przestrzenie (użytkownik, zespół, rozgrywka). Każda przestrzeń ma tryb. Domyślny to magazyn paczek: serwer dopisuje paczki do trwałego logu i oddaje je od kursora, bez czytania treści. Opcją jest tryb z silnikiem: przestrzeń ma na serwerze bazę Flat-SQL, a serwer może zmiany sprawdzać, odrzucać i filtrować. Protokół jest w obu trybach ten sam.
2. Serwer nie zna użytkowników i działa tylko w sieci wewnętrznej, za backendem aplikacji. Backend sprawdza sesję, wybiera przestrzeń i przekazuje żądanie z kluczem bramki. Integracja odbywa się wyłącznie przez HTTP, więc backend może być w dowolnym języku.
3. Serwer nie zawiera logiki biznesowej. Daje backendowi dwa mechanizmy w trybie z silnikiem: tabele tylko do odczytu dla klientów i webhook, który przed przyjęciem paczki pyta backend, które zmiany odrzucić.
4. Serwer ma jedną implementację, w Node.js. Wiązań dla Pythona i synchronizacji w procesie backendu nie ma w planach.

## Uzasadnienie

- Replikacja dwuwarstwowa rozdziela dane, które urządzenie zmienia samo, od danych z kopią główną zatwierdzającą zmiany (Gray i in., 1996). Odpowiadają im dwa tryby. Webhook to zapisy wstępne zatwierdzane przez właściciela kopii głównej (Gray i in., 1996; Terry i in., 1995).
- Magazyn paczek nie trzyma niczego w pamięci, więc skaluje się jak przechowywanie plików. Tylko w nim możliwe jest szyfrowanie end-to-end.
- Uwierzytelnianie i tak jest w backendzie aplikacji. Do internetu wystawiony jest tylko backend. Tak samo zaleca ElectricSQL.
- Reguły biznesowe zależą od aplikacji, więc należą do backendu. Serwer zostaje ogólnym narzędziem.

## Odrzucone warianty

- Tylko magazyn paczek. Odpadają kasy, rankingi i dane serwera.
- Tylko silnik na serwerze. Każda aktywna przestrzeń to baza w pamięci i brak szyfrowania end-to-end.
- Publiczny serwer sprawdzający przepustki JWT jako tryb domyślny. Więcej pracy i większa powierzchnia ataku. Zostaje jako rozszerzenie.
- Konta wbudowane w serwer. Drugi system kont obok kont aplikacji.
- Reguły zapisane w serwerze albo kod reguł wgrywany do serwera. Serwer musiałby rozumieć logikę każdej aplikacji.
- Druga implementacja serwera w Pythonie. Podwójne utrzymanie bez potwierdzonej potrzeby.

## Konsekwencje

- Kolejność prac: przestrzenie i trwały log, potem magazyn paczek, potem silnik dla przestrzeni, na końcu reguły. Zadania: [na później](../management/future.md#serwer-synchronizacji).
- Każda paczka przechodzi przez backend. Aplikacje bez backendu muszą postawić bramkę.

## Historia

- 2026-10-20: tryby serwera, bramka backendu, reguły w backendzie, rezygnacja z wiązań dla Pythona.
- 2026-10-21: kursor jako numer przyjęcia wspólny dla obu trybów (decyzja 0022).
