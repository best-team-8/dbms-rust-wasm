# Kto naprawdę używa baz danych w przeglądarce i na brzegu sieci

Analiza rzeczywistych projektów korzystających z baz porównywanych w [wasm-engines-comparison.md](wasm-engines-comparison.md) (SQLite WASM, sql.js, wa-sqlite/absurd-sql, PGlite, DuckDB-Wasm, Zeta-Lite) oraz z pokrewnych rozwiązań: baz na IndexedDB (Dexie, RxDB, PouchDB), silników synchronizacji (PowerSync, ElectricSQL, Replicache/Zero, LiveStore) i replik na brzegu sieci (LiteFS, Turso, Cloudflare Durable Objects). Celem jest oprzeć zastosowania Flat-SQL na tym, co ludzie faktycznie budują, a nie na przypuszczeniach. Stan na 18 października 2026.

**Metoda.** Dla każdego projektu ustalono: do czego służy lokalna baza, która jej cecha zdecydowała o wyborze oraz czy sprostałby temu Flat-SQL w dwóch wersjach:

- **prototyp dziś** – stan gałęzi `dbms-2-engine-prototype`;
- **dojrzały Flat-SQL** – prototyp uzupełniony o wszystko, czego brakuje wyłącznie z powodu niedojrzałości projektu, **pod warunkiem, że nie traci swoich atutów**. Obejmuje zadania z [todo.md](../todo.md) i [todo_future.md](../todo_future.md) (m.in. LWW per kolumna i liczniki `MERGE ADD`, zapytania reaktywne, szyfrowanie end-to-end, uwierzytelnianie i reguły serwera, replikacja częściowa, węzły edge z bazą główną, obsługa wielu kart, interfejs C, łącznik z PostgreSQL) oraz rozszerzenia zgodne z koncepcją, które w chwili analizy nie były zaplanowane (wyszukiwanie pełnotekstowe, typ wektorowy – dziś [todo_future.md 5.5–5.6](../todo_future.md)).

**Atuty, których dojrzały Flat-SQL nie może stracić:** mały moduł (setki KiB), praca w wątku głównym bez Web Workera i nagłówków COOP/COEP, zapytania lokalne w mikrosekundach, przyrostowa trwałość (WAL), wbudowana synchronizacja, płaski model tabel adresowanych kluczem (proste i przewidywalne wykonanie, replikacja per klucz) oraz jeden rdzeń bez I/O działający wszędzie.

**Replikacja częściowa** oznacza, że klient dostaje tylko potrzebną mu część bazy (np. swoje zgłoszenia albo swój region), a nie całą bazę. Dziś Flat-SQL wysyła każdemu klientowi wszystkie zmiany wszystkich użytkowników.

**Ograniczenia.** Źródła to głównie blogi firm, dokumentacja i wypowiedzi twórców – często z elementem marketingu. Listy klientów publikowane przez producentów (np. logotypy na stronie Dexie) nie zostały zweryfikowane i nie są tu traktowane jako dowód. Brak danych o skali wdrożeń.

## 1. Zestawienie projektów

Oznaczenia w kolumnie „Dojrzały Flat-SQL”: **tak** – nadaje się bez utraty atutów; **tak\*** – nadaje się, ale wymaga rozszerzenia zgodnego z koncepcją, którego nie ma jeszcze w planach; **częściowo** – tylko część zadania mieści się w koncepcji; **nie** – przeszkodą jest sama koncepcja, nie niedojrzałość.

| Projekt | Baza | Do czego | Co zdecydowało o wyborze | Prototyp dziś | Dojrzały Flat-SQL (bez utraty atutów) |
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
| **Zeta-Lite** [D5] | własny silnik WASM | pamięć agentów w przeglądarce (preprint) | MVCC, rozgałęzianie bazy, wektory | **nie** | **częściowo** – wektory i punkty przywracania (na bazie numerów LSN w WAL) tak; wiele jednoczesnych, przeplatanych transakcji – nie (jednowątkowe wykonanie to część koncepcji) |
| **LiteFS** (Fly.io) | SQLite z replikami | kopie danych aplikacji blisko użytkowników, dane konfiguracyjne i geolokalizacyjne; zapisy przekazywane do węzła głównego | szybkie odczyty lokalne w wielu regionach | **nie** – brak węzłów edge z bazą główną | **tak** – węzły edge (todo_future 7.1); dla danych zmienianych w wielu miejscach LWW zamiast jednego węzła zapisującego |
| **Turso** (repliki osadzone) | SQLite / libSQL | lokalne odczyty w µs, aplikacje mobilne, kasy, zbieranie danych w terenie, IoT | lokalna kopia bazy głównej, synchronizacja | **częściowo** – lokalna kopia i synchronizacja są; brak replikacji częściowej i hierarchii węzłów | **tak** |
| **Cloudflare Durable Objects** | SQLite w każdym obiekcie | jeden obiekt na pokój czatu, grę lub użytkownika | stan i baza przy kodzie, bez osobnego serwera bazy | **nie** – brak backendu `Storage` dla platformy | **tak** – jednowątkowy obiekt ze stanem w pamięci odpowiada modelowi Flat-SQL (todo_future 7.3) |

### Co jest niedojrzałością, a co granicą koncepcji

| Brak | Rodzaj | Czy dojrzały Flat-SQL go usuwa |
|---|---|---|
| Replikacja częściowa, rozdzielenie użytkowników | niedojrzałość | tak ([todo_future.md §3](../todo_future.md)) |
| Uwierzytelnianie, reguły serwera, tabele tylko do odczytu | niedojrzałość | tak ([todo_future.md §2, §4](../todo_future.md)) |
| Obsługa wielu kart | niedojrzałość | tak ([todo_future.md 1.1](../todo_future.md)) |
| Zapytania reaktywne | niedojrzałość | tak ([todo_future.md 5.1](../todo_future.md)) |
| Łącznik z PostgreSQL, interfejs C | niedojrzałość (integracja) | tak ([todo_future.md §9, §11](../todo_future.md)) |
| Węzły edge z bazą główną | niedojrzałość | tak ([todo_future.md §7](../todo_future.md)) |
| Wyszukiwanie pełnotekstowe, typ wektorowy | niedojrzałość (rozszerzenie zgodne z koncepcją) | tak, kosztem niewielkiego wzrostu modułu ([todo_future.md 5.5–5.6](../todo_future.md)) |
| Złączenia | koncepcja „bez JOIN” | złączenia tylko do odczytu nie psują pozostałych atutów (replikacja dalej per wiersz), ale zmieniają tożsamość dialektu – świadoma decyzja autora |
| Zgodność z dialektem PostgreSQL / SQLite | koncepcja (własny dialekt) | nie |
| Przetwarzanie kolumnowe dużych zbiorów | koncepcja (silnik wierszowy w pamięci, mały moduł) | nie |
| Dane większe niż pamięć urządzenia | koncepcja (dane w pamięci) | nie – stronicowanie na OPFS wymaga Web Workera i zmienia architekturę |
| Scalanie współbieżnej edycji tekstu | koncepcja (LWW na wartościach) | nie w silniku – możliwe przez zewnętrzną bibliotekę CRDT |
| Wiele jednoczesnych transakcji | koncepcja (wykonanie jednowątkowe) | nie |

## 2. Powtarzające się wzorce

### A. Lokalna pamięć podręczna danych serwera

Notion trzyma w przeglądarce kopię stron pobranych z serwera i „ściga” odczyt lokalny z zapytaniem do API. Serwer pozostaje źródłem prawdy. **Flat-SQL pasuje:** tabele prowadzone przez serwer i tylko czytane przez klienta, szybkie odczyty, mały moduł. Notion musiał ładować bibliotekę SQLite asynchronicznie, bo kilkaset KB blokowało ładowanie strony – 206 KiB Flat-SQL to tu realna przewaga, choć ładowanie asynchroniczne i tak jest wskazane. **Brakuje:** obsługi wielu kart (Notion zbudował do tego SharedWorker i Web Locks) i replikacji częściowej.

### B. Aplikacje local-first z własnym serwerem synchronizacji

Actual Budget (SQLite w przeglądarce, zegary HLC, serwer kontrolowany przez użytkownika), Linear (IndexedDB, zmiany najpierw lokalnie), aplikacje na Dexie Cloud. **To najbliższy odpowiednik architektury Flat-SQL – i dowód, że ma ona sens produkcyjnie.** **Brakuje:** zapytań reaktywnych, uprawnień, replikacji częściowej. Doświadczenie Productlane pokazuje, że replikacja całej bazy przestaje działać przy dużych przestrzeniach roboczych.

### C. Zbieranie danych w terenie przy słabym zasięgu

Community Health Toolkit (skala krajowa), DHIS2, KoboToolbox, aplikacje przemysłowe na PowerSync. **Najsilniejsze dowody spośród wszystkich wzorców** – działające wdrożenia z prawdziwą potrzebą pracy offline. **Flat-SQL pasuje** w rdzeniu (offline, trwały outbox, synchronizacja). **Brakuje:** replikacji częściowej (pracownik widzi tylko swoje dane), uwierzytelniania, reguł serwera i przekazywania danych do PostgreSQL na potrzeby analityki – CHT robi to osobnym narzędziem.

### D. Kasy i sprzedaż offline

WCPOS, OpenPOS, rozszerzenie Odoo POS; Turso wymienia kasy wśród zastosowań. Wspólny wzorzec: katalog lokalnie, koszyk offline, zamówienia w kolejce, **płatność wymaga serwera**. **Flat-SQL pasuje** przy podziale: katalog jako tabele serwera, zamówienia jako „zamiary” zatwierdzane przez serwer ([todo_future.md 2.1, 2.4](../todo_future.md)). Zgodne z replikacją dwuwarstwową Graya i in.: urządzenie offline wykonuje transakcje wstępne, a ostateczne zatwierdza właściciel kopii głównej [C9].

### E. Repliki danych blisko użytkownika (edge)

LiteFS (dane aplikacji, konfiguracja, geolokalizacja w wielu regionach), Turso (lokalne odczyty w µs), Durable Objects (baza w obiekcie na użytkownika lub pokój). **Potwierdza założenie z prezentacji** o węzłach edge synchronizowanych z bazą główną i model „baza na użytkownika/pokój”. **Różnica:** LiteFS przekazuje zapisy do jednego węzła głównego, a Flat-SQL rozstrzygałby konflikty LWW na węzłach – prostsze dla danych tylko do odczytu (flagi, konfiguracja), ryzykowniejsze dla danych zmienianych w wielu miejscach.

### F. Zapisy gier w chmurze

js-dos sprzedaje zapis w chmurze jako płatną usługę, twórcy gier indie opisują jego samodzielne dodawanie; poradniki zalecają wersjonowanie zapisów i eksport do pliku. **Dowód, że potrzeba istnieje**, choć w niszy. **Flat-SQL pasuje** (zapis lokalny, synchronizacja, migracje, eksport).

### G. Rozszerzenia przeglądarki

Rozszerzenia do wyszukiwania w historii i biblioteki dla rozszerzeń korzystają z IndexedDB. **Flat-SQL pasuje** dzięki brakowi wymagań co do Workera, ale typowym zadaniem jest tu wyszukiwanie tekstu – potrzebne wyszukiwanie pełnotekstowe.

### H. Wyszukiwanie semantyczne i pamięć asystentów

Wyraźnie rosnąca kategoria (PGlite + pgvector w wielu projektach, Zeta-Lite). **Bez typu wektorowego Flat-SQL nie ma tu czego szukać** – to warunek konieczny, a nie dodatek.

### I. Kategorie, w których Flat-SQL nie powinien konkurować

- **Analityka i BI** – DuckDB-Wasm (przetwarzanie kolumnowe).
- **Piaskownice, testy, interaktywna dokumentacja, nauka SQL** – PGlite i sql.js; o wyborze decyduje zgodność z PostgreSQL lub SQLite, której własny dialekt z definicji nie zapewni.
- **Współpraca na żywo nad dokumentem** (tablice, edytory) – CRDT i dedykowane warstwy synchronizacji.

## 3. Wnioski projektowe z cudzych wdrożeń

1. **Obsługa wielu kart jest obowiązkowa** – Notion zbudował do tego SharedWorker i Web Locks, tldraw synchronizuje karty. W Flat-SQL to zadanie [todo_future.md 1.1](../todo_future.md).
2. **Replikacja częściowa jest obowiązkowa przy wzroście danych** – Productlane musiało zmienić silnik, bo pobieranie całej przestrzeni roboczej przestało się skalować; Linear i Zero synchronizują częściowo. W Flat-SQL: [todo_future.md 3.2](../todo_future.md).
3. **Rozmiar biblioteki ma znaczenie** – Notion ładuje SQLite asynchronicznie, bo blokował start strony. To wzmacnia argument małego modułu Flat-SQL.
4. **Na wolnych urządzeniach lokalny odczyt bywa wolniejszy od sieci** – Notion „ściga” odczyt lokalny z API. Warto to uwzględnić w benchmarku na telefonie ([todo.md 4.7](../todo.md)).
5. **Przeglądarka może usunąć dane** – kasy dodają zapis na serwerze lub kopie do pliku, gry eksport zapisu, Logseq ostrzega przed czyszczeniem pamięci podręcznej. W Flat-SQL: `navigator.storage.persist()` i synchronizacja jako kopia ([todo.md 1.5](../todo.md)).
6. **Analityka trafia do bazy serwerowej** – CHT przenosi dane do PostgreSQL. Łącznik z PostgreSQL ([todo_future.md §11](../todo_future.md)) nie jest dodatkiem, tylko warunkiem dla zastosowań C i D.
7. **Wybór zegarów HLC jest zgodny z praktyką** – stosuje je Actual Budget.
8. **Płatności zawsze przechodzą przez serwer** – we wszystkich znalezionych systemach kasowych.

## 4. Zastosowania Flat-SQL potwierdzone analizą

| # | Zastosowanie | Dowody | Dopasowanie dojrzałego Flat-SQL | Warunki |
|---|---|---|---|---|
| 1 | Zbieranie danych w terenie offline | **mocne** (CHT, DHIS2, KoboToolbox, PowerSync) | wysokie | replikacja częściowa, uwierzytelnianie, reguły serwera, eksport do PostgreSQL |
| 2 | Aplikacje osobiste na wielu urządzeniach (budżet, nawyki, notatki) | **mocne** (Actual Budget, Dexie Cloud) | wysokie – architektura jak w Actual Budget | szyfrowanie end-to-end dla danych wrażliwych |
| 3 | Lokalna pamięć podręczna danych serwera w aplikacjach SaaS | **mocne** (Notion, Linear) | wysokie | wiele kart, replikacja częściowa, ładowanie asynchroniczne |
| 4 | Kasy i sprzedaż offline | średnie (WCPOS, OpenPOS, Odoo, Turso) | średnie | serwer zatwierdza zamówienia i płatności |
| 5 | Repliki konfiguracji i danych pomocniczych na edge | średnie (LiteFS, Turso, Durable Objects) | średnie | węzły edge z bazą główną ([todo_future.md §7](../todo_future.md)) |
| 6 | Zapisy gier w chmurze | słabe–średnie (js-dos, twórcy indie) | wysokie | nisza; potrzebne migracje i eksport |
| 7 | Rozszerzenia przeglądarki | słabe (pojedyncze projekty) | średnie | wyszukiwanie pełnotekstowe |
| 8 | Wyszukiwanie semantyczne i pamięć asystentów | mocne dla kategorii, ale nie dla Flat-SQL | średnie–wysokie po dodaniu typu wektorowego (zgodny z koncepcją) | typ wektorowy i wyszukiwanie podobieństwa – poza obecnymi planami |

**Nie celować:** analityka i BI, piaskownice i testy wymagające zgodności z PostgreSQL/SQLite, nauka SQL, współpraca na żywo nad dokumentami.

### Porównanie z wcześniejszą listą ([use-cases.md](../use-cases.md))

| Wcześniejsze zastosowanie | Wynik analizy |
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
| Kasy offline | **nowe** – wcześniej traktowane tylko jako przykład ograniczeń |
| Pamięć podręczna danych serwera (SaaS) | **nowe** – najmocniejszy wzorzec obok aplikacji terenowych |

## 5. Źródła

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
