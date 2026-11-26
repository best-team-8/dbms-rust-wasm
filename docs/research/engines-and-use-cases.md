# Flat-SQL wśród innych baz w przeglądarce

Flat-SQL to baza po stronie klienta z wbudowaną synchronizacją. Daje natychmiastowe działanie i pracę offline, a serwer pozostaje źródłem prawdy dla danych wspólnych. Ma najmniejszy moduł wśród silników SQL w WebAssembly i jako jedyny silnik SQL ma synchronizację wbudowaną w sam silnik. Płaci za to węższym językiem (jedna tabela w zapytaniu) i tym, że cała baza musi zmieścić się w pamięci. Do analizy dużych plików lepszy jest DuckDB-Wasm, a gdy potrzebny jest pełny PostgreSQL – PGlite.

Dokument porównuje Flat-SQL z innymi silnikami, opisuje, kto i do czego używa lokalnych baz w przeglądarce, i wyjaśnia wybór silników do benchmarków.

## Lokalnie a przez serwer

| | Każda akcja przez serwer | Flat-SQL lokalnie i synchronizacja w tle |
|---|---|---|
| Czas reakcji | 20–100 ms (opóźnienie sieci) | około 0,01 ms |
| Bez internetu | aplikacja nie działa | aplikacja działa |
| Obciążenie serwera | żądanie przy każdej akcji | paczki zmian co kilka sekund |
| Dane wspólne (rankingi, rynek) | naturalnie | wymagają serwera |
| Ochrona przed oszustwami | serwer decyduje | urządzenie może zmienić własne dane |
| Analityka wszystkich użytkowników | tak | nie |

Flat-SQL nie zastępuje bazy na serwerze, tylko ją uzupełnia. Tak działają PowerSync i ElectricSQL (PostgreSQL na serwerze, lokalna baza u klienta) oraz aplikacja Linear.

## Inne silniki

| Silnik | Czym jest | Pliki po kompresji | Zapis na dysk w przeglądarce | Synchronizacja |
|---|---|---:|---|---|
| Flat-SQL | SQL na płaskich tabelach, Rust | 287 KiB | tylko zmiany, w IndexedDB | wbudowana |
| sql.js | SQLite skompilowany z C | 331 KiB | cała baza od nowa przy każdym zapisie | brak |
| SQLite WASM | oficjalna wersja SQLite | 583 KiB | zmienione strony pliku, wymaga Web Workera | brak |
| wa-sqlite | SQLite z zapisem stronami do IndexedDB | 821 KiB | zmienione strony | brak |
| PGlite | PostgreSQL skompilowany z C | 5,6 MiB | IndexedDB albo pliki | osobne narzędzie |
| DuckDB-Wasm | silnik analityczny | 7,6 MiB | głównie odczyt plików danych | brak |
| Zeta-Lite | silnik z dialektem PostgreSQL, Rust | 2,9 MB (deklaracja autora) | kopia bazy na żądanie; zmiany po niej giną | brak |
| AlaSQL | SQL w czystym JavaScripcie | 111 KiB | opcjonalne dodatki | brak |
| Dexie.js, PouchDB, RxDB | bazy dokumentów bez SQL | 42–80 KiB | IndexedDB | PouchDB i RxDB: wbudowana |

**SQLite w przeglądarce** (sql.js, SQLite WASM, wa-sqlite) daje pełny SQL i dojrzały silnik. sql.js przy zapisie eksportuje cały plik, więc koszt zapisu rośnie z rozmiarem bazy. SQLite WASM zapisuje tylko zmienione strony, ale do prywatnego systemu plików przeglądarki (OPFS), dostępnego szybko tylko z Web Workera. Synchronizację dokładają osobne narzędzia: cr-sqlite, ElectricSQL, PowerSync.

**Duże silniki** rozwiązują inne problemy. DuckDB-Wasm analizuje duże pliki danych (Kohn i in., 2022), a PGlite przydaje się, gdy liczy się zgodność z PostgreSQL. Oba ważą kilka megabajtów i startują 0,8–1,6 s.

**Silniki w Rust** są architektonicznie najbliżej. Zeta-Lite (Zhang, 2026) uruchamia w przeglądarce ten sam kod co na serwerze, ale wybiera pełny SQL i duży moduł zamiast przyrostowego zapisu i synchronizacji. GlueSQL i Turso nie mają synchronizacji wbudowanej w silnik, a Turso w przeglądarce jest w wersji beta (stan z października 2026).

**Bez WebAssembly** działają AlaSQL oraz bazy dokumentów. PouchDB i RxDB synchronizują dane i są konkurencją w aplikacjach offline.

## Kto używa lokalnych baz i do czego

Analiza wdrożeń z 18 października 2026 objęła około 25 projektów korzystających z wymienionych silników, z warstw synchronizacji i z replik na brzegu sieci. Dla każdego ustalono, do czego służy lokalna baza, co zdecydowało o wyborze i czy sprostałby temu dojrzały Flat-SQL, czyli Flat-SQL uzupełniony o rozszerzenia z [listy na później](../management/future.md). Źródła to głównie blogi firm i dokumentacja, często z elementem marketingu, i bez danych o skali wdrożeń.

| Zastosowanie | Przykłady | Dowody | Czego Flat-SQL brakuje |
|---|---|---|---|
| Zbieranie danych w terenie przy słabym zasięgu | Community Health Toolkit, DHIS2, KoboToolbox, aplikacje na PowerSync | mocne | replikacja częściowa, uwierzytelnianie, reguły serwera |
| Aplikacje osobiste na wielu urządzeniach | Actual Budget, aplikacje na Dexie Cloud | mocne | uwierzytelnianie, szyfrowanie end-to-end |
| Kopia danych serwera w aplikacjach SaaS | Notion, Linear | mocne | wiele kart, replikacja częściowa |
| Kasy i sprzedaż offline | WCPOS, OpenPOS, Odoo | średnie | tabele tylko do odczytu, zamówienia zatwierdzane przez serwer |
| Konfiguracja i dane pomocnicze na edge | LiteFS, Turso, Durable Objects | średnie | węzły edge z bazą główną |
| Zapisy gier w chmurze | js-dos, gry niezależne | słabe–średnie | migracje wersji, eksport zapisu |
| Rozszerzenia przeglądarki | SmrutiCortex | słabe | wyszukiwanie pełnotekstowe |
| Pamięć asystentów | projekty na PGlite z pgvector, Zeta-Lite | mocne dla kategorii, nie dla Flat-SQL | wektory, wyszukiwanie pełnotekstowe, gałęzie bazy |

Najbliższym odpowiednikiem architektury Flat-SQL jest Actual Budget: lokalna baza SQLite, zegar hybrydowy i własny serwer synchronizacji. To dowód, że taka architektura działa produkcyjnie. Najmocniejsze dowody potrzeby pracy offline dają aplikacje terenowe.

Wszystkie braki w tabeli wynikają z niedojrzałości projektu, a nie z jego koncepcji. Granicą koncepcji są: dialekt bez złączeń, dane w pamięci, przetwarzanie wierszy zamiast kolumn i wykonanie w jednym wątku.

## Wnioski z cudzych wdrożeń

- **Obsługa wielu kart jest obowiązkowa.** Notion zbudował do niej `SharedWorker` i Web Locks, bo karty uszkadzały sobie dane.
- **Replikacja częściowa jest obowiązkowa przy wzroście danych.** Productlane zmieniło silnik, gdy pobieranie całej przestrzeni roboczej przestało działać.
- **Rozmiar biblioteki ma znaczenie.** Notion ładuje SQLite w tle, bo kilkaset kilobajtów opóźniało start strony.
- **Na wolnych urządzeniach dysk bywa wolniejszy od sieci.** Notion wysyła odczyt lokalny i zapytanie do serwera naraz i bierze szybszy wynik.
- **Przeglądarka może usunąć dane.** Kasy trzymają kopię na serwerze, gry eksportują zapis do pliku. Flat-SQL prosi o trwały magazyn, a synchronizacja służy jako kopia.
- **Płatności zawsze przechodzą przez serwer.** Tak działają wszystkie znalezione systemy kasowe.

## Czego nie polecać

| Zastosowanie | Dlaczego | Lepiej |
|---|---|---|
| Analityka dużych plików | każdy wiersz to osobny obiekt w pamięci, limit 4 GiB modułu | DuckDB-Wasm |
| Piaskownice, testy, nauka SQL | wymagają zgodności z PostgreSQL albo SQLite | PGlite, sql.js |
| Wspólna edycja tekstu na żywo | reguła „ostatni zapis wygrywa” traktuje pole jako całość | biblioteka CRDT tekstu (Yjs, Automerge) |
| Płatności i salda bez serwera | urządzenie offline nie wie, czy środki nie zostały wydane gdzie indziej (Gray i in., 1996) | serwer jako źródło prawdy, lokalne zapisy jako wstępne |
| Rozgrywka w czasie rzeczywistym, rankingi, rynek w grach | dane wspólne i podatne na oszustwa | serwer gry i baza serwera |
| Klucze prywatne, na przykład w portfelu kryptowalut | dane nie są szyfrowane, każdy skrypt strony ma dostęp do IndexedDB, przeglądarka może je usunąć (Lehmann i in., 2020) | portfel sprzętowy albo zaszyfrowany sejf; Flat-SQL tylko jako indeks historii |
| Dane większe niż pamięć urządzenia | cała baza musi zmieścić się w pamięci | silnik ze stronicowaniem na dysku |

## Silniki w benchmarkach

Benchmark porównuje silniki tylko w grupach o tych samych warunkach ([metodyka](benchmark-scenario.md)).

| Silnik | Po co jest w benchmarku |
|---|---|
| Flat-SQL w przeglądarce i natywnie | badany silnik; wersja natywna pokazuje koszt samego WebAssembly |
| sql.js | najpopularniejszy SQL w przeglądarce, główny punkt odniesienia |
| SQLite WASM, wa-sqlite | SQLite zapisujący tylko zmiany; bez nich porównanie trwałego zapisu dotyczyłoby tylko sql.js |
| PGlite, DuckDB-Wasm | duże silniki o innym zastosowaniu; DuckDB-Wasm wyznacza granicę, za którą lepszy jest silnik analityczny |
| AlaSQL | ile daje WebAssembly wobec SQL w JavaScripcie |
| Dexie.js, PouchDB, RxDB | zapis do IndexedDB bez SQL; PouchDB i RxDB jako konkurencja w aplikacjach offline |
| zwykła mapa w JavaScripcie | dolna granica: koszt warstwy SQL |
| serwer z tym samym silnikiem | zapytanie przez sieć zamiast lokalnie |

Przy porównaniu wersji po optymalizacjach silniki „pokrewne” to te, które rozwiązują to samo zadanie co Flat-SQL: sql.js, SQLite WASM, wa-sqlite, PouchDB i RxDB. PGlite i DuckDB-Wasm mają inne zastosowanie, a AlaSQL, Dexie.js i zwykła mapa są punktami odniesienia. Nie mierzono Zeta-Lite, bo jego kod jest zamknięty i nie ma modułu do pobrania.

## Źródła

- Silniki: [SQLite WASM i zapis danych](https://sqlite.org/wasm/doc/tip/persistence.md), [sql.js](https://github.com/sql-js/sql.js), [wa-sqlite](https://github.com/rhashimoto/wa-sqlite), [PGlite](https://pglite.dev), [DuckDB-Wasm](https://duckdb.org/2021/10/29/duckdb-wasm), [GlueSQL](https://github.com/gluesql/gluesql), [Turso w przeglądarce](https://turso.tech/blog/introducing-turso-in-the-browser), [AlaSQL](https://github.com/AlaSQL/alasql), [Dexie.js](https://dexie.org), [RxDB](https://rxdb.info), [PouchDB](https://pouchdb.com)
- Warstwy synchronizacji: [cr-sqlite](https://github.com/vlcn-io/cr-sqlite), [ElectricSQL](https://electric-sql.com), [PowerSync](https://www.powersync.com)
- Notion: [How we sped up Notion in the browser with WASM SQLite](https://www.notion.com/blog/how-we-sped-up-notion-in-the-browser-with-wasm-sqlite)
- Actual Budget: [James Long, „Actual: Using CRDTs in the Wild” (2019)](https://archive.jlongster.com/using-crdts-in-the-wild), [localfirst.fm #7 – James Long](https://www.localfirst.fm/7)
- Linear: [Reverse Linear Sync Engine Study](https://docsearch.algolia.com/mcp/docs/repo/wzhudev/reverse-linear-sync-engine)
- Productlane: [Moving to Zero](https://feedback.clerk.com/blog/zero-migration)
- Community Health Toolkit: [Overview of CHT Components](https://docs.communityhealthtoolkit.org/core/overview/), [Data Flows for Analytics](https://docs.communityhealthtoolkit.org/core/overview/data-flows-for-analytics)
- DHIS2: [Android Capture](https://developers.dhis2.org/docs/mobile/android-capture-app/overview); KoboToolbox: [formularze w przeglądarce](https://support.kobotoolbox.org/data_through_webforms.html)
- Kasy: [WCPOS – praca offline](https://docs.wcpos.com/getting-started/offline), [OpenPOS](https://github.com/tasmon/OpenPOS)
- Dexie Cloud: [dexie.org/cloud](https://dexie.org/cloud)
- Gry: [js-dos – zapis w chmurze](https://js-dos.com/cloud-overview.html), [Adventures in Lestoria](https://sigonasr2.itch.io/adventures-in-lestoria/devlog/672554/cloud-saving-web-browser-saving)
- Tablice: [tldraw – trwałość](https://tldraw.dev/docs/persistence); rozszerzenia: [SmrutiCortex](https://chromeboard.com/extension/smruticortex-ecnkiihcifbfnhjblicfbppplobiicoi)
- PGlite: [10 milionów pobrań tygodniowo](https://electric.ax/blog/2026/06/25/pglite-reaches-10-million-weekly-downloads), [wyszukiwanie semantyczne w przeglądarce](https://supabase.com/blog/in-browser-semantic-search-pglite)
- Edge: [LiteFS](https://fly.io/blog/introducing-litefs/), [Turso – repliki osadzone](https://docs.turso.tech/features/embedded-replicas), [Cloudflare Durable Objects](https://www.cloudflare.com/products/durable-objects/)
