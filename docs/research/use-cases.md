# Zastosowania Flat-SQL

Dokument opiera zastosowania Flat-SQL na analizie rzeczywistych projektów, które korzystają z baz porównywanych w [wasm-engines-comparison.md](wasm-engines-comparison.md) (SQLite WASM, sql.js, wa-sqlite/absurd-sql, PGlite, DuckDB-Wasm, Zeta-Lite) oraz z pokrewnych rozwiązań: baz na IndexedDB (Dexie, RxDB, PouchDB), silników synchronizacji (PowerSync, ElectricSQL, Replicache/Zero, LiveStore) i replik na brzegu sieci (LiteFS, Turso, Cloudflare Durable Objects). Analizę wykonano 18 października 2026; części 1–5 opisują jej wynik, część 6 rozpisuje wybrane scenariusze, a część 7 wymienia zastosowania, których Flat-SQL nie powinien obsługiwać.

**Metoda.** Dla każdego projektu ustalono: do czego służy lokalna baza, która jej cecha zdecydowała o wyborze oraz czy sprostałby temu Flat-SQL w dwóch wersjach:

- **prototyp** – stan z dnia analizy, przed poprawkami trwałości i synchronizacji oraz przed optymalizacjami;
- **dojrzały Flat-SQL** – prototyp uzupełniony o wszystko, czego brakuje wyłącznie z powodu niedojrzałości projektu, **pod warunkiem, że nie traci swoich atutów**. Obejmuje kierunki rozwoju z [future/README.md](../future/README.md) (m.in. LWW per kolumna i liczniki `MERGE ADD`, zapytania reaktywne, szyfrowanie end-to-end, uwierzytelnianie i reguły serwera, replikacja częściowa, węzły edge z bazą główną, obsługa wielu kart, interfejs C, wyszukiwanie pełnotekstowe i typ wektorowy).

**Atuty, których dojrzały Flat-SQL nie może stracić:** mały moduł (setki KiB), praca w wątku głównym bez Web Workera i nagłówków COOP/COEP, zapytania lokalne w mikrosekundach, przyrostowa trwałość (WAL), wbudowana synchronizacja, płaski model tabel adresowanych kluczem (proste i przewidywalne wykonanie, replikacja per klucz) oraz jeden rdzeń bez I/O działający wszędzie.

**Replikacja częściowa** oznacza, że klient dostaje tylko potrzebną mu część bazy (np. swoje zgłoszenia albo swój region), a nie całą bazę. Dziś Flat-SQL wysyła każdemu klientowi wszystkie zmiany wszystkich użytkowników.

**Ograniczenia.** Źródła to głównie blogi firm, dokumentacja i wypowiedzi twórców – często z elementem marketingu. Listy klientów publikowane przez producentów (np. logotypy na stronie Dexie) nie zostały zweryfikowane i nie są tu traktowane jako dowód. Brak danych o skali wdrożeń.

## 1. Czym Flat-SQL jest, a czym nie jest

> **Flat-SQL to baza danych po stronie klienta z wbudowaną synchronizacją: daje natychmiastowe działanie i pracę offline, a serwer z bazą główną pozostaje źródłem prawdy dla danych wspólnych.**

```text
urządzenie użytkownika: Flat-SQL (dane pod ręką, praca offline)
        ↕  synchronizacja w tle (paczki zmian)
serwer: baza główna – w prototypie ten sam silnik, docelowo także np. PostgreSQL
```

Flat-SQL **nie zastępuje** PostgreSQL na serwerze – uzupełnia go. PostgreSQL jest magazynem wspólnym dla wszystkich, Flat-SQL – kopią potrzebnych danych, którą użytkownik ma zawsze przy sobie. Tak działają m.in. PowerSync i ElectricSQL (PostgreSQL na serwerze, lokalna baza u klienta, synchronizacja między nimi – [Q11]), a z aplikacji m.in. Linear, który trzyma dane w przeglądarce i synchronizuje je w tle.

| | Serwerowa baza + każda akcja przez sieć | Flat-SQL lokalnie + synchronizacja |
|---|---|---|
| Czas reakcji na akcję | 20–100 ms (RTT) | ok. 0,01 ms (pomiar H1) |
| Brak internetu | aplikacja nie działa | aplikacja działa |
| Obciążenie serwera | żądanie przy każdej akcji | paczki zmian co kilka sekund |
| Dane wspólne (rankingi, rynek) | naturalnie | wymaga serwera |
| Ochrona przed oszustwami | serwer decyduje | klient może zmienić własne dane |
| Analityka wszystkich użytkowników | tak | nie |

## 2. Zestawienie projektów

Oznaczenia w kolumnie „Dojrzały Flat-SQL”: **tak** – nadaje się bez utraty atutów; **tak\*** – nadaje się, ale wymaga rozszerzenia zgodnego z koncepcją, którego nie ma jeszcze w planach; **częściowo** – tylko część zadania mieści się w koncepcji; **nie** – przeszkodą jest sama koncepcja, nie niedojrzałość.

| Projekt | Baza | Do czego | Co zdecydowało o wyborze | Prototyp | Dojrzały Flat-SQL (bez utraty atutów) |
|---|---|---|---|---|---|
| **Notion** (web) | SQLite WASM na OPFS, w Workerze | pamięć podręczna stron pobranych z serwera; serwer pozostaje źródłem prawdy | ta sama baza co w aplikacjach desktopowych i mobilnych; nawigacja szybsza o ok. 20%, w regionach z wolnym API o 28–33% | **nie** – brak obsługi wielu kart i replikacji częściowej | **tak** – oba braki to niedojrzałość; granica koncepcji: pamięć podręczna musi mieścić się w pamięci urządzenia |
| **Actual Budget** (budżet domowy, open source) | SQLite w przeglądarce (dawniej absurd-sql na IndexedDB) | kopia robocza budżetu w przeglądarce, synchronizacja z serwerem kontrolowanym przez użytkownika; zegary HLC | lokalne zapytania SQL, praca offline, własny serwer synchronizacji | **częściowo** – działa dla jednego użytkownika na kilku urządzeniach; brak uwierzytelniania i rozdzielenia użytkowników | **tak** – architektura niemal identyczna z Flat-SQL |
| **Linear** (zarządzanie zadaniami) | IndexedDB + graf obiektów w pamięci (własny silnik) | wszystkie zmiany najpierw lokalnie, wysyłane w tle; start z jednego pobrania stanu | natychmiastowy interfejs, offline, synchronizacja częściowa i uprawnienia | **nie** – brak replikacji częściowej, uprawnień i zapytań reaktywnych | **tak** – wszystkie braki to niedojrzałość; powiązania (zadanie–projekt) składane w aplikacji albo złączeniami tylko do odczytu |
| **Productlane** (Replicache → Zero) | silnik synchronizacji Rocicorp | dane przestrzeni roboczej u klienta | po wzroście klientów pobieranie *całej* przestrzeni na urządzenie przestało się skalować – migracja do synchronizacji częściowej | **nie** – dziś replikowana jest cała baza | **tak** – replikacja częściowa to niedojrzałość, nie koncepcja |
| **Community Health Toolkit** (pracownicy ochrony zdrowia, skala krajowa) | PouchDB na urządzeniu, CouchDB na serwerze | zbieranie danych offline w terenie o słabym zasięgu; analityka po przeniesieniu danych do PostgreSQL | offline-first, synchronizacja dwukierunkowa przy przerywanej łączności | **częściowo** – rdzeń (offline, outbox, synchronizacja) jest; brak rozdzielenia danych i uwierzytelniania | **tak** – replikacja częściowa, uwierzytelnianie i eksport do PostgreSQL to niedojrzałość |
| **DHIS2 Capture**, **KoboToolbox (Enketo)** | dane w przeglądarce / na urządzeniu | formularze i ewidencja offline, wysyłka po odzyskaniu sieci | praca bez internetu w terenie | **częściowo** – jak wyżej | **tak** – jak wyżej |
| **JourneyApps → PowerSync** (aplikacje przemysłowe, wg producenta m.in. GE, Halliburton, ExxonMobil) | SQLite na urządzeniu + baza serwerowa | aplikacje terenowe; zapisy offline kolejkowane i wysyłane później | lokalny SQLite zsynchronizowany z bazą firmy | **nie** – brak łącznika z bazą firmy | **tak** – łącznik z PostgreSQL to integracja, nie zmiana koncepcji |
| **WCPOS, OpenPOS, Odoo POS offline** (kasy w sklepach) | IndexedDB | katalog produktów i klientów lokalnie, koszyk offline, zamówienia w kolejce; płatność wymaga serwera | praca kasy bez internetu | **częściowo** – katalog i kolejka zamówień działają; brak reguł serwera do zatwierdzania zamówień | **tak** – katalog jako tabele serwera, zamówienia jako „zamiary” zatwierdzane przez serwer; płatność zawsze online (cecha każdego systemu offline) |
| **Dexie Cloud** – Routickr (nawyki), Fablehenge (pisanie opowiadań) | IndexedDB (Dexie) + usługa synchronizacji | aplikacje osobiste na wielu urządzeniach „bez własnego backendu” | gotowa synchronizacja | **częściowo** – brak uwierzytelniania i rozdzielenia użytkowników | **tak** |
| **js-dos** (gry DOS w przeglądarce), **Adventures in Lestoria** (gra indie) | IndexedDB + płatny / własny zapis w chmurze | zapisy gier synchronizowane między urządzeniami | przeglądarka może usunąć dane; gracze chcą postępu na wielu urządzeniach | **częściowo** – zapis lokalny i synchronizacja są; brak rozdzielenia graczy i migracji | **tak** – zapis w chmurze, migracje wersji, eksport pliku |
| **tldraw**, **Excalidraw** (tablice do rysowania) | IndexedDB (tldraw: także synchronizacja między kartami) | lokalna trwałość dokumentu; współpraca przez osobną warstwę synchronizacji, w Excalidraw pokoje przechowywane jako szyfrogram | dokument jako całość, współpraca w czasie rzeczywistym | **nie** | **częściowo** – trwałość, przekazywanie zmian i szyfrowany przekaźnik tak; scalanie współbieżnej edycji wymaga algorytmu CRDT spoza modelu LWW (zewnętrzna biblioteka na Flat-SQL jako magazynie i transporcie) |
| **SmrutiCortex** (rozszerzenie: wyszukiwanie w historii), biblioteki dla rozszerzeń (Dexie, WebExtension DB) | IndexedDB | lokalne indeksowanie i wyszukiwanie | brak serwera, prywatność | **częściowo** – brak wyszukiwania pełnotekstowego | **tak\*** – indeks pełnotekstowy pasuje do koncepcji, kosztem kilkudziesięciu KiB modułu |
| **Evidence**, **MotherDuck**, **lakeFS** (analityka, BI) | DuckDB-Wasm | filtrowanie i agregacje w pulpitach, podgląd plików Parquet | przetwarzanie kolumnowe dużych danych | **nie** | **nie** – silnik kolumnowy dla dużych plików to drugi silnik; stracony mały rozmiar i dane w pamięci |
| **database.build**, Supabase Studio, Bolt.new, LiveCodes, Codapi | PGlite | piaskownice, interaktywna dokumentacja, aplikacje generowane w przeglądarce | pełna zgodność z PostgreSQL | **nie** | **nie** – własny dialekt z definicji; zgodność z PostgreSQL oznaczałaby stanie się PostgreSQL (moduł w MB) |
| Prisma, Drizzle, Netlify, Firebase, AWS Blocks | PGlite (głównie Node.js) | lokalna baza deweloperska i testy bez Dockera | zgodność z PostgreSQL, szybki start | **nie** | **nie** – jak wyżej |
| SQL Murder Mystery, SQLime, kursy SQL | sql.js / SQLite WASM | nauka SQL w przeglądarce | standardowy SQL ze złączeniami | **nie** | **nie** – nauka standardowego SQL wymaga standardowego dialektu; możliwa tylko nauka samego Flat-SQL |
| Wtyczki do notatek z wyszukiwaniem semantycznym, demonstracje Transformers.js, frameworki agentów | PGlite + pgvector | wyszukiwanie po podobieństwie, lokalna pamięć asystentów | rozszerzenie wektorowe | **nie** – brak typu wektorowego | **tak\*** – typ wektorowy i wyszukiwanie podobieństwa pasują do płaskiego modelu (wiersz = wpis z wektorem); dla dużych zbiorów potrzebny indeks przybliżony |
| **Zeta-Lite** [D5] | własny silnik WASM | pamięć agentów w przeglądarce (preprint) | MVCC, rozgałęzianie bazy, wektory | **nie** | **częściowo** – wektory i punkty przywracania (na bazie numerów LSN w WAL) tak; wiele jednoczesnych, przeplatanych transakcji – nie (jednowątkowe wykonanie to część koncepcji); izolację pracy agentów i niepodzielne scalanie dają gałęzie bazy ([§3 H](#h-wyszukiwanie-semantyczne-i-pamięć-asystentów)) |
| **LiteFS** (Fly.io) | SQLite z replikami | kopie danych aplikacji blisko użytkowników, dane konfiguracyjne i geolokalizacyjne; zapisy przekazywane do węzła głównego | szybkie odczyty lokalne w wielu regionach | **nie** – brak węzłów edge z bazą główną | **tak** – węzły edge (todo_future 7.1); dla danych zmienianych w wielu miejscach LWW zamiast jednego węzła zapisującego |
| **Turso** (repliki osadzone) | SQLite / libSQL | lokalne odczyty w µs, aplikacje mobilne, kasy, zbieranie danych w terenie, IoT | lokalna kopia bazy głównej, synchronizacja | **częściowo** – lokalna kopia i synchronizacja są; brak replikacji częściowej i hierarchii węzłów | **tak** |
| **Cloudflare Durable Objects** | SQLite w każdym obiekcie | jeden obiekt na pokój czatu, grę lub użytkownika | stan i baza przy kodzie, bez osobnego serwera bazy | **nie** – brak backendu `Storage` dla platformy | **tak** – jednowątkowy obiekt ze stanem w pamięci odpowiada modelowi Flat-SQL (todo_future 7.3) |

### Co jest niedojrzałością, a co granicą koncepcji

| Brak | Rodzaj | Czy dojrzały Flat-SQL go usuwa |
|---|---|---|
| Replikacja częściowa, rozdzielenie użytkowników | niedojrzałość | tak ([future/todo.md §3](../future/todo.md)) |
| Uwierzytelnianie, reguły serwera, tabele tylko do odczytu | niedojrzałość | tak ([future/todo.md §2, §4](../future/todo.md)) |
| Obsługa wielu kart | niedojrzałość | tak ([future/todo.md 1.1](../future/todo.md)) |
| Zapytania reaktywne | niedojrzałość | tak ([future/todo.md 5.1](../future/todo.md)) |
| Interfejs C | niedojrzałość (integracja) | tak ([future/todo.md §9](../future/todo.md)) |
| Łącznik z PostgreSQL | integracja | nie – poza zakresem projektu ([future/todo.md – poza zakresem](../future/todo.md#poza-zakresem-decyzja-autora)) |
| Węzły edge z bazą główną | niedojrzałość | tak ([future/todo.md §7](../future/todo.md)) |
| Wyszukiwanie pełnotekstowe, typ wektorowy | niedojrzałość (rozszerzenie zgodne z koncepcją) | tak, kosztem niewielkiego wzrostu modułu ([future/todo.md 5.5–5.6](../future/todo.md)) |
| Złączenia | koncepcja „bez JOIN” | nie – złączenia tylko do odczytu nie psułyby pozostałych atutów (replikacja dalej per wiersz), ale zmieniłyby tożsamość dialektu; decyzja autora ([future/todo.md – poza zakresem](../future/todo.md#poza-zakresem-decyzja-autora)) |
| Zgodność z dialektem PostgreSQL / SQLite | koncepcja (własny dialekt) | nie |
| Przetwarzanie kolumnowe dużych zbiorów | koncepcja (silnik wierszowy w pamięci, mały moduł) | nie |
| Dane większe niż pamięć urządzenia | koncepcja (dane w pamięci) | nie – stronicowanie na OPFS wymaga Web Workera i zmienia architekturę |
| Scalanie współbieżnej edycji tekstu | koncepcja (LWW na wartościach) | nie w silniku – możliwe przez zewnętrzną bibliotekę CRDT |
| Wiele jednoczesnych transakcji | koncepcja (wykonanie jednowątkowe) | nie; na jednym urządzeniu izolację i scalanie wszystko albo nic dają gałęzie bazy ([future/todo.md 5.7](../future/todo.md)) |
| Rozgałęzianie bazy | niedojrzałość (rozszerzenie zgodne z koncepcją) | tak, gałąź jako replika scalana jak zmiany z serwera ([future/todo.md 5.4, 5.7](../future/todo.md)) |

## 3. Powtarzające się wzorce

### A. Lokalna pamięć podręczna danych serwera

Notion trzyma w przeglądarce kopię stron pobranych z serwera i „ściga” odczyt lokalny z zapytaniem do API. Serwer pozostaje źródłem prawdy. **Flat-SQL pasuje:** tabele prowadzone przez serwer i tylko czytane przez klienta, szybkie odczyty, mały moduł. Notion musiał ładować bibliotekę SQLite asynchronicznie, bo kilkaset KB blokowało ładowanie strony – 287 KiB Flat-SQL (moduł z mostem JavaScript po kompresji) to tu realna przewaga, choć ładowanie asynchroniczne i tak jest wskazane. **Brakuje:** obsługi wielu kart (Notion zbudował do tego SharedWorker i Web Locks) i replikacji częściowej.

### B. Aplikacje local-first z własnym serwerem synchronizacji

Actual Budget (SQLite w przeglądarce, zegary HLC, serwer kontrolowany przez użytkownika), Linear (IndexedDB, zmiany najpierw lokalnie), aplikacje na Dexie Cloud. **To najbliższy odpowiednik architektury Flat-SQL – i dowód, że ma ona sens produkcyjnie.** **Brakuje:** zapytań reaktywnych, uprawnień, replikacji częściowej. Doświadczenie Productlane pokazuje, że replikacja całej bazy przestaje działać przy dużych przestrzeniach roboczych.

### C. Zbieranie danych w terenie przy słabym zasięgu

Community Health Toolkit (skala krajowa), DHIS2, KoboToolbox, aplikacje przemysłowe na PowerSync. **Najsilniejsze dowody spośród wszystkich wzorców** – działające wdrożenia z prawdziwą potrzebą pracy offline. **Flat-SQL pasuje** w rdzeniu (offline, trwały outbox, synchronizacja). **Brakuje:** replikacji częściowej (pracownik widzi tylko swoje dane), uwierzytelniania, reguł serwera i przekazywania danych do PostgreSQL na potrzeby analityki – CHT robi to osobnym narzędziem.

### D. Kasy i sprzedaż offline

WCPOS, OpenPOS, rozszerzenie Odoo POS; Turso wymienia kasy wśród zastosowań. Wspólny wzorzec: katalog lokalnie, koszyk offline, zamówienia w kolejce, **płatność wymaga serwera**. **Flat-SQL pasuje** przy podziale: katalog jako tabele serwera, zamówienia jako „zamiary” zatwierdzane przez serwer ([future/todo.md 2.1, 2.4](../future/todo.md)). Zgodne z replikacją dwuwarstwową Graya i in.: urządzenie offline wykonuje transakcje wstępne, a ostateczne zatwierdza właściciel kopii głównej [C9].

### E. Repliki danych blisko użytkownika (edge)

LiteFS (dane aplikacji, konfiguracja, geolokalizacja w wielu regionach), Turso (lokalne odczyty w µs), Durable Objects (baza w obiekcie na użytkownika lub pokój). **Potwierdza założenie z prezentacji** o węzłach edge synchronizowanych z bazą główną i model „baza na użytkownika/pokój”. **Różnica:** LiteFS przekazuje zapisy do jednego węzła głównego, a Flat-SQL rozstrzygałby konflikty LWW na węzłach – prostsze dla danych tylko do odczytu (flagi, konfiguracja), ryzykowniejsze dla danych zmienianych w wielu miejscach.

### F. Zapisy gier w chmurze

js-dos sprzedaje zapis w chmurze jako płatną usługę, twórcy gier indie opisują jego samodzielne dodawanie; poradniki zalecają wersjonowanie zapisów i eksport do pliku. **Dowód, że potrzeba istnieje**, choć w niszy. **Flat-SQL pasuje** (zapis lokalny, synchronizacja, migracje, eksport).

### G. Rozszerzenia przeglądarki

Rozszerzenia do wyszukiwania w historii i biblioteki dla rozszerzeń korzystają z IndexedDB. **Flat-SQL pasuje** dzięki brakowi wymagań co do Workera, ale typowym zadaniem jest tu wyszukiwanie tekstu – potrzebne wyszukiwanie pełnotekstowe.

### H. Wyszukiwanie semantyczne i pamięć asystentów

Wyraźnie rosnąca kategoria (PGlite + pgvector w wielu projektach, Zeta-Lite). **Bez typu wektorowego Flat-SQL nie ma tu czego szukać** – to warunek konieczny, a nie dodatek.

Po zadaniach zaplanowanych dla tej kategorii Flat-SQL nadawałby się do niej bez utraty atutów w pozostałych zastosowaniach. Potrzebne są typ wektorowy z wyszukiwaniem podobieństwa, wyszukiwanie pełnotekstowe, szyfrowanie end-to-end i gałęzie bazy ([future/todo.md 4.3, 4.4, 5.4–5.8](../future/todo.md)). Zeta-Lite izoluje pracę agentów przez wiele wersji wierszy i rozgałęzianie bazy. W Flat-SQL gałąź może być osobną repliką: powstaje ze snapshotu, zbiera własne zmiany i jest scalana tą samą ścieżką co zmiany z serwera. Scalanie sprawdza, czy wiersze przeczytane i zmienione w gałęzi nie zmieniły się w bazie głównej, i przyjmuje albo wszystkie zmiany gałęzi, albo żadnej. Na jednym urządzeniu daje to wykrywanie konfliktów i niepodzielność, które w Zeta-Lite zapewnia zatwierdzenie transakcji. Odczyty nie sprawdzają przy tym widoczności wersji, więc gry, kasy i zbieranie danych w terenie nie zwalniają. Gałęzie można też trzymać obok siebie i porównywać zapytaniami. Funkcje dla agentów mogą być włączane flagami przy budowie modułu, dzięki czemu moduł podstawowy nie rośnie.

Część ograniczeń zostaje. Utworzenie gałęzi kosztuje kopię bazy, przy 100 000 wierszy około 70 ms i podwójną pamięć. Wielu długich transakcji otwartych naraz na wspólnym stanie nie będzie, a między replikami atomowy pozostaje pojedynczy wiersz. Dopasowanie dojrzałego Flat-SQL jest więc wysokie dla pamięci asystenta jednego użytkownika na kilku urządzeniach, w której liczą się trwałość każdego zapisu, synchronizacja i mały moduł. Dla wielu agentów badających równolegle warianty na wspólnym stanie lepszy pozostaje silnik z wieloma wersjami wierszy. Wektory powiększają snapshot, a czas otwarcia bazy rośnie z jego rozmiarem, dlatego w tej kategorii przydaje się odtwarzanie bazy krokami ([future/todo.md 12.11](../future/todo.md)).

### I. Kategorie, w których Flat-SQL nie powinien konkurować

- **Analityka i BI** – DuckDB-Wasm (przetwarzanie kolumnowe).
- **Piaskownice, testy, interaktywna dokumentacja, nauka SQL** – PGlite i sql.js; o wyborze decyduje zgodność z PostgreSQL lub SQLite, której własny dialekt z definicji nie zapewni.
- **Współpraca na żywo nad dokumentem** (tablice, edytory) – CRDT i dedykowane warstwy synchronizacji.

## 4. Wnioski projektowe z cudzych wdrożeń

1. **Obsługa wielu kart jest obowiązkowa** – Notion zbudował do tego SharedWorker i Web Locks, tldraw synchronizuje karty. W Flat-SQL to zadanie [future/todo.md 1.1](../future/todo.md).
2. **Replikacja częściowa jest obowiązkowa przy wzroście danych** – Productlane musiało zmienić silnik, bo pobieranie całej przestrzeni roboczej przestało się skalować; Linear i Zero synchronizują częściowo. W Flat-SQL: [future/todo.md 3.2](../future/todo.md).
3. **Rozmiar biblioteki ma znaczenie** – Notion ładuje SQLite asynchronicznie, bo blokował start strony. To wzmacnia argument małego modułu Flat-SQL.
4. **Na wolnych urządzeniach lokalny odczyt bywa wolniejszy od sieci** – Notion „ściga” odczyt lokalny z API. Warto to uwzględnić w benchmarku na telefonie ([todo.md 4.7](../todo.md)).
5. **Przeglądarka może usunąć dane** – kasy dodają zapis na serwerze lub kopie do pliku, gry eksport zapisu, Logseq ostrzega przed czyszczeniem pamięci podręcznej. W Flat-SQL: `navigator.storage.persist()` i synchronizacja jako kopia ([todo.md 1.5](../todo.md)).
6. **Analityka trafia do bazy serwerowej** – CHT przenosi dane do PostgreSQL. W zastosowaniach C i D analityka wymaga więc osobnego narzędzia, bo łącznik z PostgreSQL jest poza zakresem projektu ([future/todo.md – poza zakresem](../future/todo.md#poza-zakresem-decyzja-autora)).
7. **Wybór zegarów HLC jest zgodny z praktyką** – stosuje je Actual Budget.
8. **Płatności zawsze przechodzą przez serwer** – we wszystkich znalezionych systemach kasowych.

## 5. Zastosowania Flat-SQL potwierdzone analizą

| # | Zastosowanie | Dowody | Dopasowanie dojrzałego Flat-SQL | Warunki |
|---|---|---|---|---|
| 1 | Zbieranie danych w terenie offline | **mocne** (CHT, DHIS2, KoboToolbox, PowerSync) | wysokie | replikacja częściowa, uwierzytelnianie, reguły serwera, eksport do PostgreSQL |
| 2 | Aplikacje osobiste na wielu urządzeniach (budżet, nawyki, notatki) | **mocne** (Actual Budget, Dexie Cloud) | wysokie – architektura jak w Actual Budget | szyfrowanie end-to-end dla danych wrażliwych |
| 3 | Lokalna pamięć podręczna danych serwera w aplikacjach SaaS | **mocne** (Notion, Linear) | wysokie | wiele kart, replikacja częściowa, ładowanie asynchroniczne |
| 4 | Kasy i sprzedaż offline | średnie (WCPOS, OpenPOS, Odoo, Turso) | średnie | serwer zatwierdza zamówienia i płatności |
| 5 | Repliki konfiguracji i danych pomocniczych na edge | średnie (LiteFS, Turso, Durable Objects) | średnie | węzły edge z bazą główną ([future/todo.md §7](../future/todo.md)) |
| 6 | Zapisy gier w chmurze | słabe–średnie (js-dos, twórcy indie) | wysokie | nisza; potrzebne migracje i eksport |
| 7 | Rozszerzenia przeglądarki | słabe (pojedyncze projekty) | średnie | wyszukiwanie pełnotekstowe |
| 8 | Wyszukiwanie semantyczne i pamięć asystentów | mocne dla kategorii, ale nie dla Flat-SQL | wysokie dla pamięci jednego użytkownika na kilku urządzeniach, średnie dla wielu agentów równolegle na wspólnym stanie ([§3 H](#h-wyszukiwanie-semantyczne-i-pamięć-asystentów)) | typ wektorowy, wyszukiwanie pełnotekstowe, gałęzie bazy ze scalaniem warunkowym, szyfrowanie end-to-end ([future/todo.md 4.3, 4.4, 5.4–5.8](../future/todo.md)) |

**Nie celować:** analityka i BI, piaskownice i testy wymagające zgodności z PostgreSQL/SQLite, nauka SQL, współpraca na żywo nad dokumentami.

### Pierwotna lista zastosowań a wynik analizy

Pierwotną listę zastosowań zawierała [prezentacja projektu](../presentation/v1.pdf) i pierwsza wersja tego dokumentu.

| Pierwotne zastosowanie | Wynik analizy |
|---|---|
| Aplikacje terenowe offline | **potwierdzone** – najsilniejsze dowody |
| Aplikacje osobiste na wielu urządzeniach | **potwierdzone** (Actual Budget) |
| Lokalna kopia danych referencyjnych | **potwierdzone** w szerszej formie: pamięć podręczna danych serwera (Notion) i katalogi w kasach |
| Feature flags, personalizacja na edge | **potwierdzone pośrednio** – LiteFS wymienia dane konfiguracyjne i geolokalizacyjne; brak przykładów z flagami wprost |
| Gry przeglądarkowe – zapis w chmurze | **potwierdzone jako nisza** |
| Rozszerzenia przeglądarki | **słabo potwierdzone** |
| Telemetria na edge, sesje na edge | **nie znaleziono przykładów** – traktować jako hipotezy |
| Portfele kryptowalut | **nie badano** – bez dowodów, traktować jako hipotezę |
| Autozapis formularzy | **odrzucone** (przeglądarki i proste mechanizmy wystarczają) |
| Analityka CSV 500 MB | **potwierdzone, że nie dla Flat-SQL** – to domena DuckDB-Wasm |
| Edytory graficzne – katalog zasobów | **nie badano** – metadane i filtrowanie po indeksach pasują, relacje wiele-do-wielu (tagi) trzeba modelować bez złączeń |
| Kasy offline | **nowe** – wcześniej traktowane tylko jako przykład ograniczeń |
| Pamięć podręczna danych serwera (SaaS) | **nowe** – najmocniejszy wzorzec obok aplikacji terenowych |

## 6. Scenariusze zastosowań

### 6.1 Gry przeglądarkowe

**Nadaje się:**

- **gry jednoosobowe z zapisem w chmurze** – gry „idle”, RPG, roguelike, strategie: postęp zapisywany lokalnie przy każdej akcji, gra działa offline, ten sam zapis na telefonie i komputerze;
- **zabezpieczenie postępu** – kopia na serwerze chroni przed usunięciem danych przez przeglądarkę (Safari po 7 dniach bez interakcji – [Q2]);
- **gry turowe i asynchroniczne** – ruch raz na jakiś czas (np. szachy korespondencyjne); replikacja dla słabo połączonych urządzeń sprawdza się przy współpracy asynchronicznej [C3];
- **mała kooperacja** – np. wspólna skrzynia kilku graczy, po dodaniu liczników `MERGE ADD`;
- **narzędzia twórców gier** – edytory poziomów, katalogi zasobów.

**Nie nadaje się:**

- **rozgrywka wieloosobowa w czasie rzeczywistym** (pozycje, walka) – to zadanie serwera gry i komunikacji przez WebSocket, a nie bazy danych;
- **rankingi, rynek, ekonomia gry** – dane wspólne i podatne na oszustwa; autorytetem musi być serwer.

**Przykładowy podział danych w grze:**

| Dane | Gdzie | Dlaczego |
|---|---|---|
| Zapis stanu gry, ekwipunek, ustawienia | Flat-SQL u gracza + synchronizacja | natychmiastowy dostęp, praca offline, ten sam zapis na wielu urządzeniach |
| Ranking, rynek, waluta premium | serwer (np. PostgreSQL) | dane wspólne, ochrona przed oszustwami |
| Pozycje graczy w czasie rzeczywistym | serwer gry (WebSocket) | stan ulotny, nie jest to zadanie bazy danych |
| Katalog przedmiotów, konfiguracja, flagi | baza główna → kopia u gracza (tylko do odczytu) | odczyty w µs, aktualizacje rozsyłane z serwera |

### 6.2 Portfele kryptowalut

W portfelu są dwie różne rzeczy: **klucze** (czyli środki) i **dane** (historia, saldo, książka adresowa). Flat-SQL nadaje się do danych, **nigdy do kluczy**: dane nie są szyfrowane, IndexedDB może odczytać każdy skrypt strony (np. przy XSS lub złośliwej bibliotece), izolacja WebAssembly nie chroni danych wewnątrz modułu ([W2]), a przeglądarka może dane usunąć – utrata klucza to utrata środków.

Przepis:

1. **Klucze poza bazą** – portfel sprzętowy albo zaszyfrowany sejf (hasło → klucz przez PBKDF2 → AES-GCM w WebCrypto), odszyfrowywany tylko na czas podpisu. WebCrypto nie obsługuje krzywej secp256k1 (Bitcoin, Ethereum), więc podpis wykonuje osobna biblioteka, a WebCrypto szyfruje sejf.
2. **Blockchain jako źródło prawdy, Flat-SQL jako lokalny indeks** – tabela `tx` z kluczem = hash transakcji (niezmienny, więc bez konfliktów), indeksy po tokenie, dacie i adresie, podsumowania portfela przez `GROUP BY`.
3. **Transakcje oczekujące** – status „oczekująca → potwierdzona/odrzucona”, jak zapisy wstępne w Bayou [C3].
4. **Synchronizacja tylko metadanych użytkownika** (etykiety adresów, notatki, książka adresowa, ustawienia) – z szyfrowaniem end-to-end, bo etykiety ujawniają finanse; historię zawsze można pobrać ponownie z blockchaina.
5. **Szyfrowanie lokalne i `navigator.storage.persist()`** ([future/todo.md 4.4](../future/todo.md), [todo.md 1.5](../todo.md)).
6. **„Weryfikacja na edge” nie jest dowodem** – wymaga zaufania do węzła; dane z węzła RPC warto sprawdzać w kilku źródłach.
7. **Forma: rozszerzenie przeglądarki** – tu mały moduł działający bez Workera i nagłówków to realna przewaga.

Proponowane sformułowanie: *„Portfele kryptowalut: szybki lokalny indeks historii transakcji i prywatne metadane synchronizowane end-to-end. Klucze w portfelu sprzętowym lub zaszyfrowanym sejfie, blockchain jako źródło prawdy.”*

### 6.3 Węzły edge zsynchronizowane z bazą główną

Założenie z prezentacji: baza działa na serwerach brzegowych blisko użytkowników i synchronizuje się z bazą główną.

```text
przeglądarki ─► węzeł edge (np. Warszawa) ─┐
przeglądarki ─► węzeł edge (np. Berlin)  ──┴─► baza główna
```

**Stan prototypu:** jest jeden serwer ([server/server.js](../../server/server.js)), który jest jednocześnie hubem synchronizacji, API do porównań w H1 i bazą główną; węzłów edge zsynchronizowanych z bazą główną jeszcze nie ma ([future/todo.md §7](../future/todo.md)). W literaturze to replikacja dwuwarstwowa z węzłami bazowymi [C9] albo multi-leader w topologii drzewa ([P1] rozdz. 5) i scenariusz replikacji tabel relacyjnych bez konfliktów (Yu, Ignat, 2020, praca poza rdzeniem); węzeł edge może obsługiwać użytkowników przy niedostępnej centrali, jak klient Coda przy niedostępnym serwerze [C10].

| Scenariusz | Ocena | Uwagi |
|---|---|---|
| Feature flags, testy A/B | bardzo dobre | zmienia centrala, węzły tylko czytają |
| Personalizacja wg regionu | bardzo dobre | jak wyżej |
| Telemetria | bardzo dobre | węzeł zbiera zdarzenia i wysyła paczkami w górę |
| Sesje i autoryzacja | ryzykowne | unieważnienie dociera z opóźnieniem; bezpieczniej: krótki czas życia sesji, natychmiastowe rozsyłanie unieważnień, węzeł jako replika tylko do odczytu |

### 6.4 Flat-SQL obok backendu (np. FastAPI w Pythonie)

```text
przeglądarka (FlatDB + SyncClient)
      │ synchronizacja + token
      ▼
backend (logowanie, uprawnienia) ──przekazuje──► serwer Flat-SQL (kontener)
      │                                                   ▲
      └── logika biznesowa przez API ─────────────────────┘
```

| Wariant | Opis | Stan |
|---|---|---|
| A. Kontener obok backendu | obecny serwer Flat-SQL w osobnym kontenerze, backend jako bramka | do testów działa już dziś |
| B. Silnik w procesie backendu | rdzeń jako moduł Pythona przez interfejs C, synchronizacja w FastAPI | poza planami: integracja z backendem tylko przez HTTP |

Już działa: nowy klient bez kursora dostaje pełny stan bazy, a potem `SyncClient` synchronizuje zmiany w obie strony; zapisy backendu przez `/api/query` docierają do klientów.

**Jak dzielić dane między użytkowników:**

| Sposób | Kiedy | Plusy | Minusy |
|---|---|---|---|
| Jedna baza główna + reguły, kto dostaje które wiersze (domyślnie) | dane wspólne: rankingi, rynek, zespoły | zapytania przekrojowe, jedno źródło prawdy | wymaga replikacji częściowej i reguł ([future/todo.md 3.2, §2](../future/todo.md)) |
| Osobna baza na użytkownika | dane niezależne: zapisy gry, notatki | izolacja, łatwe usunięcie konta i eksport danych | brak zapytań między użytkownikami (do tego osobna baza, np. PostgreSQL) |

Baza Flat-SQL to struktura w pamięci i dwa pliki, a nie osobny serwer ani proces. Szacunek: ekwipunek 200 wierszy to kilkadziesiąt KB, 100 tys. graczy – ok. 2 GB na dysku, a w pamięci tylko aktywni użytkownicy. Model „baza na użytkownika” stosują m.in. Cloudflare Durable Objects, Turso i Apple iCloud (praca o FoundationDB Record Layer, SIGMOD 2019 – do doczytania).

Czego brakuje do realnego użycia: rozdzielenia danych użytkowników, uwierzytelniania, reguł po stronie serwera, obrazu Dockera ([future/todo.md](../future/todo.md)). `/api/query` nie wolno wystawiać publicznie bez logowania.

### 6.5 Synchronizacja z innymi bazami (PostgreSQL, SQLite)

| Sytuacja | Czy potrzebna synchronizacja z inną bazą | Jak |
|---|---|---|
| Zapis w chmurze dla jednego użytkownika, aplikacje osobiste, aplikacje terenowe z własnym serwerem | nie – baza główna może być Flat-SQL | obecny protokół synchronizacji |
| Dane wspólne (rankingi, rynek), analityka wszystkich użytkowników | tak – zwykle PostgreSQL jako baza główna | łącznik przechwytujący zmiany PostgreSQL (CDC, [P1] rozdz. 11) i zapisujący do niego zmiany klientów po walidacji – poza zakresem projektu ([future/todo.md – poza zakresem](../future/todo.md#poza-zakresem-decyzja-autora)) |
| Istniejący backend z PostgreSQL | tak | jak wyżej; to dokładnie zakres PowerSync i ElectricSQL [Q11] |
| Aplikacje natywne (mobilne, desktopowe) używające SQLite | zależy | albo łącznik SQLite ↔ Flat-SQL, albo natywny Flat-SQL zamiast SQLite przez interfejs C ([future/todo.md §9](../future/todo.md)) – wtedy synchronizacja działa bez tłumaczenia |
| Serwer lub edge na SQLite (np. Durable Objects, Turso) | zależy | backend `Storage` dla magazynu platformy albo łącznik jak dla PostgreSQL |

### 6.6 Lokalna pamięć asystentów i agentów programowych

Trend opisany m.in. w pracy o Zeta-Lite [D5] (2026) i widoczny w analizie wdrożeń ([§3 H](#h-wyszukiwanie-semantyczne-i-pamięć-asystentów)): lokalna baza w przeglądarce jako trwała pamięć asystentów – ustawienia, fakty, notatki, stan zadań i wyniki narzędzi. Asystent działa zwykle na serwerze dostawcy, a aplikacja przekazuje mu tylko potrzebne fragmenty danych.

Sposoby użycia Flat-SQL:

1. **Własna aplikacja webowa z asystentem** – pamięć i stan zadań lokalnie we Flat-SQL, do asystenta trafiają tylko potrzebne fragmenty.
2. **Narzędzie dla asystentów na komputerze** – natywny Flat-SQL udostępniony przez standardowy protokół podłączania narzędzi (MCP); ten sam format plików co w przeglądarce, więc z synchronizacją pamięć jest wspólna dla obu.
3. **W pełni lokalnie** – asystent działający w przeglądarce (WebGPU) i Flat-SQL jako jego pamięć; dane nie opuszczają urządzenia.

| Cecha | Flat-SQL dziś | Zeta-Lite [D5] |
|---|---|---|
| Wyszukiwanie po podobieństwie (wektory) | brak | jest |
| Wyszukiwanie pełnotekstowe | tylko `LIKE` | jest |
| Każdy zapis od razu trwały | tak (WAL) | nie (snapshot na żądanie) |
| Synchronizacja między urządzeniami | wbudowana | brak |
| Rozmiar modułu (gzip) | 287 KiB | 2,87 MB |
| Rozgałęzianie bazy | tylko `ROLLBACK` | jest |

Możliwa unikalna propozycja: **pamięć trzymana na urządzeniu, trwała przy każdym zapisie, synchronizowana między urządzeniami z szyfrowaniem end-to-end (serwer jej nie widzi), w module o rozmiarze kilkuset KiB.** Wymaga: typu wektorowego i wyszukiwania podobieństwa, prostego wyszukiwania pełnotekstowego, szyfrowania end-to-end, `navigator.storage.persist()`, zapytań reaktywnych i punktów przywracania. Zasada bezpieczeństwa: zapisywać źródło każdego wpisu i dać użytkownikowi podgląd oraz usuwanie, bo treść z obcej strony zapisana jako wspomnienie może później wpływać na działanie asystenta (przejrzystość – [V1]).

## 7. Czego nie polecać i dlaczego

| Ograniczenie | Źródło | Dlaczego | Obejście |
|---|---|---|---|
| Duża analityka (np. CSV 500 MB) | implementacja i platforma (nie język – plik CSV to jedna tabela) | każdy wiersz to osobny obiekt w pamięci, przetwarzanie wiersz po wierszu zamiast kolumnami, limit 4 GiB `wasm32` | DuckDB-Wasm [D1]; tabele kolumnowe w Flat-SQL byłyby budową drugiego silnika |
| Płatności jako jedyne źródło prawdy | założenie Local-First (każdy system offline) | urządzenie offline nie wie, czy środki nie zostały wydane na innym urządzeniu; zapobieganie podwójnemu wydaniu wymaga koordynacji ([C9], [P1] rozdz. 9) | serwer lub blockchain jako źródło prawdy, lokalne operacje jako wstępne do potwierdzenia [C3]; ewentualnie limit do wydania offline przydzielony z góry (metoda escrow – liczniki ograniczone w Antidote SQL, praca poza rdzeniem) |
| Wspólna edycja tekstu na żywo | model danych (pole jako całość + LWW) i implementacja (pobieranie zmian co 5 s) | edytory współdzielone śledzą każdy znak osobno (CRDT tekstu, [L1]) | każda edycja jako osobny niezmienny wiersz, tekst składany biblioteką CRDT (np. Yjs) w aplikacji, powiadomienia push |
| Dane większe niż pamięć urządzenia | implementacja (decyzja „dane w pamięci”) i platforma | cała baza musi zmieścić się w pamięci modułu | magazyn stronicowany na OPFS z buforem stron (duży nakład); 64-bitowa pamięć WASM kosztuje wydajność [W6], [Q6] |
| Rozgrywka w czasie rzeczywistym, rankingi i rynek bez serwera | założenie Local-First | dane wspólne i podatne na oszustwa | serwer gry i baza serwerowa (§6.1) |
| Przechowywanie kluczy prywatnych | bezpieczeństwo przeglądarki | brak szyfrowania, dostęp skryptów strony, możliwe usunięcie danych | portfel sprzętowy lub zaszyfrowany sejf (§6.2) |

## 8. Źródła

- Notion: [How we sped up Notion in the browser with WASM SQLite](https://www.notion.com/blog/how-we-sped-up-notion-in-the-browser-with-wasm-sqlite)
- Actual Budget: [localfirst.fm #7 – James Long: Actual Budget, Hybrid Logical Clocks & absurd-sql](https://metacast.app/podcast/localfirst-fm/0uiP5YFv/7---james-long-actual-budget-hybrid-logical-clocks-and-absurd-sql/BSZTYxzt); [opis wdrożenia Actual Budget](https://railway.com/deploy/actual-budget-railway)
- Linear (analizy zewnętrzne): [Reverse Linear Sync Engine Study](https://docsearch.algolia.com/mcp/docs/repo/wzhudev/reverse-linear-sync-engine)
- Productlane: [Moving to Zero: Productlane's new Sync Engine](https://feedback.clerk.com/blog/zero-migration)
- Community Health Toolkit: [Overview of CHT Components](https://docs.communityhealthtoolkit.org/core/overview/), [Data Flows for Analytics](https://docs.communityhealthtoolkit.org/core/overview/data-flows-for-analytics)
- DHIS2: [Android Capture – overview](https://developers.dhis2.org/docs/mobile/android-capture-app/overview)
- KoboToolbox: [Data collection through web forms](https://support.kobotoolbox.org/data_through_webforms.html)
- PowerSync: [o firmie i JourneyApps](https://powersync.com/company), [dokumentacja](https://docs.powersync.com/overview/readme-1)
- Kasy: [WCPOS – Offline Functionality](https://docs.wcpos.com/getting-started/offline), [OpenPOS](https://github.com/tasmon/OpenPOS)
- Dexie Cloud: [dexie.org/cloud](https://dexie.org/cloud)
- Gry: [js-dos – cloud saves](https://js-dos.com/cloud-overview.html), [Adventures in Lestoria – cloud saving](https://sigonasr2.itch.io/adventures-in-lestoria/devlog/672554/cloud-saving-web-browser-saving), [Game save best practices (web)](https://bugnet.io/blog/game-save-best-practices-web)
- Tablice: [tldraw – persistence](https://tldraw.dev/docs/persistence)
- Rozszerzenia: [SmrutiCortex](https://chromeboard.com/extension/smruticortex-ecnkiihcifbfnhjblicfbppplobiicoi)
- DuckDB-Wasm: [DuckDB WASM in the browser (MotherDuck)](https://motherduck.com/blog/duckdb-wasm-in-browser/)
- PGlite: [PGlite reaches 10 million weekly downloads](https://electric.ax/blog/2026/06/25/pglite-reaches-10-million-weekly-downloads), [database.build](https://github.com/supabase-community/database-build), [wyszukiwanie semantyczne w przeglądarce z PGlite](https://supabase.com/blog/in-browser-semantic-search-pglite)
- sql.js: [SQL Murder Mystery](https://mystery.knightlab.com), [SQLite playground (SQLime)](https://antonz.org/sqlite-playground/)
- LiteFS: [Introducing LiteFS](https://fly.io/blog/introducing-litefs/), [LiteFS FAQ](https://fly.io/docs/litefs/faq)
- Turso: [Embedded replicas](https://docs.turso.tech/features/embedded-replicas), [Offline sync public beta](https://turso.tech/blog/turso-offline-sync-public-beta)
- Cloudflare: [Durable Objects](https://www.cloudflare.com/products/durable-objects/)
- Logseq: [README (wersja DB, ostrzeżenie o kopiach)](https://fossies.org/linux/logseq/README.md)
