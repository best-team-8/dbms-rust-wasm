# ADR-0053: Pamięć agentów przez gałęzie bazy ze scalaniem warunkowym, bez wielu wersji wierszy

| | |
|---|---|
| Status | przyjęta |
| Data | 2026-10-28 |
| Rodzaj | architektura |
| Powiązane | ADR-0005, ADR-0006, ADR-0008, ADR-0013, ADR-0015, ADR-0018, ADR-0043; future 5.4–5.8, 12.11–12.15; [use-cases.md H](../research/use-cases.md#h-wyszukiwanie-semantyczne-i-pamięć-asystentów), [use-cases.md](../research/use-cases.md) |

## Kontekst

Pamięć asystentów i agentów programowych w przeglądarce to rosnąca kategoria zastosowań (PGlite z pgvector, Zeta-Lite [D5]). Zeta-Lite izoluje pracę agentów przez wiele wersji wierszy (MVCC) i rozgałęzianie bazy. Flat-SQL wykonuje transakcje szeregowo, bez wersji wierszy (ADR-0008), a ADR-0008 odrzuciło MVCC jako „złożoność bez korzyści w jednym wątku”, bez oceny potrzeb agentów. Autor zapytał, co musiałoby się zmienić, żeby Flat-SQL nadawał się do pamięci agentów, i czy dałoby się to osiągnąć bez straty dla pozostałych zastosowań z analizy wdrożeń.

## Decyzja

1. Silnik domyślny zostaje szeregowy, bez wersji wierszy; ADR-0008 obowiązuje bez zmian.
2. Izolację pracy agentów i porzucanie nieudanej pracy dają gałęzie bazy jako repliki (future 5.7): gałąź powstaje ze snapshotu jako osobna instancja z własnym identyfikatorem węzła, zbiera własne zmiany i jest scalana tą samą ścieżką co zmiany z serwera (LWW per wiersz, ADR-0015).
3. Scalanie warunkowe sprawdza znaczniki wierszy, które gałąź przeczytała i zmieniła. Jeśli któryś zmienił się w bazie głównej, scalenie jest odrzucane w całości, w innym wypadku zmiany wchodzą jedną lokalną transakcją.
4. Funkcje dla agentów (wyszukiwanie pełnotekstowe, typ wektorowy, gałęzie) są dokładane poza gorącymi ścieżkami silnika i mogą być wyłączane flagami kompilacji (future 12.15).
5. Najpierw powstaje port warstwy wierszy z dzisiejszą implementacją jako jedynym adapterem (future 12.13). Drzewo trwałe (future 12.14) i wariant z wieloma wersjami wierszy (kierunki na później) mogą powstać tylko jako adaptery za tym portem, wybierane przy budowie.

## Uzasadnienie

- **Koszt MVCC dla silnika.** Analiza kodu: zmiana obejmuje warstwę wierszy i transakcji, czyli `table.rs` (362 linie), `db.rs` (1295), ścieżki dostępu w `plan.rs` i zapisy w `exec.rs` oraz część `snapshot.rs`. To około 25–35% rdzenia (9042 linie); lekser, parser, wyrażenia, kodek, format WAL i snapshotu oraz protokół synchronizacji zostają. Trudność leży w poprawności (wykrywanie konfliktów, usuwanie starych wersji, zmiany schematu przy otwartych transakcjach) i w API: zatwierdzenie może zostać odrzucone, więc aplikacja musi ponawiać.
- **Koszt MVCC dla innych zastosowań.** Każdy odczyt sprawdzałby widoczność wersji (odczyt po kluczu 2,45 µs w Node.js w Benchmarku 2), `COUNT` po indeksie bez czytania wierszy (optymalizacja §1) i UPDATE w miejscu (ADR-0044) przestałyby działać w obecnej postaci, a stare wersje zwiększyłyby pamięć, która jest głównym ograniczeniem modelu w pamięci (51,8 MiB po wczytaniu 100 000 wierszy, ADR-0006). Na wzorce z analizy wdrożeń: pamięć podręczna danych serwera, aplikacje osobiste, zbieranie danych w terenie, kasy i rozszerzenia nic nie zyskują, gry tracą na odczytach w każdej klatce. Zyskuje tylko pamięć agentów, a w serwerze edge zysk jest wątpliwy, bo model Durable Objects to wykonanie szeregowe.
- **Korzyść MVCC w jednym wątku.** Przeplatają się tylko transakcje trwające przez `await`, np. agent czekający na odpowiedź modelu wewnątrz transakcji. Agent, który najpierw rozumuje, a potem wykonuje krótką transakcję, ma przy wykonaniu szeregowym pełną izolację.
- **Gałęzie wykorzystują to, co jest.** Snapshot, odtworzenie i nakładanie paczek zmian już istnieją; LWW zbiega się przy dowolnej kolejności scaleń [L2]. Gałęzie można trzymać obok siebie, zapisać i porównywać zapytaniami, czego transakcja MVCC po zatwierdzeniu nie daje. Koszt utworzenia to kopia bazy: snapshot 22,6 ms i odtworzenie 46,4 ms w przeglądarce przy 100 000 wierszy (Benchmark 2), przy pamięci agenta z kilkoma tysiącami wpisów kilka ms.
- **Scalanie warunkowe zamyka luki LWW.** Same gałęzie z LWW tracą wykrywanie konfliktu (późniejszy znacznik wygrywa po cichu), decyzje oparte na nieaktualnym odczycie (dwa agenty przejmują to samo zadanie) i niepodzielność (część wierszy gałęzi przegrywa z nowszymi w bazie głównej). Sprawdzenie znaczników przy scaleniu to sterowanie optymistyczne, które MVCC wykonuje przy zatwierdzeniu; znaczniki HLC są już w każdym wierszu, więc nie trzeba wersji ani zmian w odczytach.

## Odrzucone alternatywy

- **MVCC w silniku domyślnym** – koszt dla odczytów, pamięci i optymalizacji, zysk dla jednej kategorii zastosowań.
- **MVCC od razu jako wariant kompilacji** – wykonalne za portem (format plików i protokół bez zmian), ale to osobny duży projekt z podwójnym utrzymaniem warstwy wierszy; odłożone do potrzeby, której gałęzie nie pokryją.
- **Gałęzie scalane samym LWW** – ciche nadpisania i częściowe scalenia, groźne właśnie dla agentów działających na wspólnym stanie.
- **Drzewo trwałe od razu** – tanie gałęzie, ale kopiowanie ścieżek spowalnia zapisy we wszystkich zastosowaniach; tylko jako adapter i tylko po pomiarze kosztu kopii (future 5.8).

## Konsekwencje

- Nowe zadania w [future/todo.md](../future/todo.md): 5.7 (gałęzie i scalanie warunkowe), 5.8 (przykład pamięci asystenta), 12.11 (odtwarzanie bazy krokami, bo wektory powiększą snapshot), 12.12 (szybsza suma CRC32), 12.13 (port warstwy wierszy), 12.14 (drzewo trwałe, warunkowo), 12.15 (warianty modułu); MVCC jako wariant kompilacji w kierunkach na później.
- Tracimy wobec MVCC tani start gałęzi (do czasu 12.14) i wiele długich transakcji naraz na żywym stanie; gwarancje scalania warunkowego obejmują jedno urządzenie, między replikami atomowy pozostaje wiersz (ADR-0018).
- [use-cases.md](../research/use-cases.md) opisuje dopasowanie dojrzałego Flat-SQL do pamięci asystentów po tych zadaniach.
- Wszystko to jest pracą po pracy inżynierskiej; w pracy dyplomowej kierunek rozwoju opisuje się słowami.

## Źródła

[D5], [L2]; [benchmark-results.md §2](../research/benchmark-results.md#2-benchmark-2-wersja-po-optymalizacjach), [optimization-results.md §1, §6, §15](../research/optimization-results.md), [use-cases.md](../research/use-cases.md).
