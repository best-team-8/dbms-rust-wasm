# Rozszerzenia – droga do dojrzałego Flat-SQL

Nowe możliwości, których prototyp nie ma, a bez których nie obsłuży zastosowań potwierdzonych analizą rzeczywistych wdrożeń ([research/real-world-usage.md](research/real-world-usage.md)). Optymalizacja tego, co już istnieje, jest w **[todo.md](todo.md)** – część rozszerzeń zakłada wykonanie zadań stamtąd (podano przy zadaniu). Wnioski: [references/conclusions.md](references/conclusions.md). Identyfikatory w nawiasach (np. [C9]) odsyłają do [indeksu literatury](references/README.md).

Nakład: **mały** – pojedyncze funkcje i testy; **średni** – kilka modułów, nowe testy; **duży** – nowy podsystem.

**Warunek każdego rozszerzenia:** nie odbiera atutów Flat-SQL – mały moduł (setki KiB), praca w wątku głównym bez Web Workera i nagłówków COOP/COEP, zapytania lokalne w mikrosekundach, przyrostowa trwałość (WAL), wbudowana synchronizacja, płaski model tabel adresowanych kluczem, jeden rdzeń bez I/O działający wszędzie. Rozszerzenie, które wymaga Workera lub dużej biblioteki, jest **opcją**, nie domyślnym trybem.

## Priorytety według zastosowań

Najmocniej potwierdzone zastosowania ([real-world-usage.md §4](research/real-world-usage.md#4-zastosowania-flat-sql-potwierdzone-analizą)) i co je blokuje:

| Zastosowanie | Dowody | Potrzebne rozszerzenia |
|---|---|---|
| Zbieranie danych w terenie offline | mocne (CHT, DHIS2, KoboToolbox, PowerSync) | 4.1, 3.1 lub 3.2, 2.1–2.3, 11.1 |
| Aplikacje osobiste na wielu urządzeniach | mocne (Actual Budget, Dexie Cloud) | 4.1, 3.1, 4.3 |
| Pamięć podręczna danych serwera w aplikacjach SaaS | mocne (Notion, Linear) | 1.1, 1.2, 2.1, 3.2 |
| Kasy i sprzedaż offline | średnie | 2.1, 2.4, 4.1 |
| Repliki konfiguracji na edge (flagi, personalizacja) | średnie | 7.1–7.3, 2.1 |
| Zapisy gier w chmurze | słabe–średnie | 3.1, 4.1, 1.3, 2.5, 5.4 |

Jeśli scenariusze edge z prezentacji mają zostać pokazane na obronie, 7.1 warto zrobić zaraz po sekcji 1 z [todo.md](todo.md) – bez niego pozostają zapowiedzią.

## Braki dojrzałości i zadania, które je usuwają

| Brak (wg analizy wdrożeń) | Gdzie wystąpił | Zadania |
|---|---|---|
| Obsługa wielu kart | Notion, tldraw | 1.1 |
| Tryb pamięci podręcznej danych serwera | Notion | 1.2 |
| Tabele prowadzone przez serwer, reguły serwera | Notion, kasy, flagi na edge, rankingi | 2.1–2.3, 2.6 |
| Zamiary zatwierdzane przez serwer | kasy, gry turowe | 2.4 |
| Rozdzielenie użytkowników, replikacja częściowa | Productlane, Linear, CHT, Actual Budget, gry | 3.1–3.6 |
| Uwierzytelnianie | wszystkie aplikacje z wieloma użytkownikami | 4.1, 4.2 |
| Szyfrowanie danych wrażliwych | aplikacje osobiste, zdrowie, finanse | 4.3, 4.4 |
| Zapytania reaktywne | Linear, Riffle, LiveStore | 5.1 |
| Migracje schematu | gry, aplikacje z długim życiem danych | 6.1 |
| Węzły edge z bazą główną, platformy edge | LiteFS, Turso, Durable Objects | 7.1–7.3 |
| Łącznik z PostgreSQL, eksport do analityki | PowerSync, CHT | 11.1–11.4 |
| Interfejs C, aplikacje natywne | PowerSync, Turso (aplikacje mobilne) | 9.1, 9.5 |
| Eksport bazy do pliku, punkty przywracania | gry, kasy, Logseq | 1.3, 5.4 |
| Wyszukiwanie pełnotekstowe, wektory | rozszerzenia przeglądarki, notatki, pamięć asystentów | 5.5, 5.6 |
| Współdzielona treść edytowana na żywo | tldraw, Excalidraw | 6.3 |

## 1. Platforma przeglądarki

- [ ] **1.1 Wiele kart** (średni) – dziś demo trzyma blokadę Web Locks: pierwsza karta zapisuje do IndexedDB, kolejne pracują tylko w pamięci, więc ich zapisy giną po zamknięciu ([architecture.md §4.4](architecture.md)). Docelowo jedna instancja silnika obsługuje wszystkie karty – wzór z wdrożenia Notion, gdzie współbieżne zapisy z kilku kart uszkadzały dane [Q12]:
  - [ ] `SharedWorker` jako pośrednik – od 2026 r. dostępny we wszystkich głównych przeglądarkach (Chrome na Androidzie od wersji 148); silnik w workerze działa jak w wątku głównym, bez COOP/COEP;
  - [ ] wariant zapasowy dla starszych przeglądarek: wybór aktywnej karty przez Web Locks, pozostałe kierują do niej zapytania, przejęcie roli po zamknięciu aktywnej karty;
  - [ ] powiadamianie kart o zmianach (`BroadcastChannel`) – potrzebne też dla 5.1;
  - [ ] test: dwie karty zapisują jednocześnie, po zamknięciu aktywnej druga przejmuje zapis bez utraty danych.
- [ ] **1.2 Tryb pamięci podręcznej danych serwera** (średni) – wzór z wdrożenia Notion i z „hoardingu” w Coda [C10]: klient trzyma kopię potrzebnych danych serwera, serwer pozostaje źródłem prawdy:
  - [ ] aplikacja działa, zanim moduł się wczyta (ładowanie asynchroniczne) – nawet kilkaset KB biblioteki opóźniało start strony;
  - [ ] pomocnik w `FlatDB`: odczyt lokalny „ścigający się” z zapytaniem do API – na wolnych urządzeniach dysk bywa wolniejszy od sieci;
  - [ ] limit rozmiaru tabel podręcznych i usuwanie najdawniej używanych wierszy **tylko lokalnie** – usunięcie z pamięci podręcznej nie może trafić do synchronizacji jako `DELETE`;
  - [ ] wymaga 2.1 (tabele prowadzone przez serwer) i pomiaru z [todo.md 4.7](todo.md).
- [ ] **1.3 Eksport i import bazy do pliku** (mały) – pobranie bazy jako pliku (snapshot + WAL albo czytelny JSON przez `changes_to_json`) i wczytanie w `FlatDB`; kopia zapasowa na wypadek usunięcia danych przez przeglądarkę i kontrola użytkownika nad danymi ([L1], ideał 7).
- [ ] **1.4 Rozszerzenia przeglądarki** (mały) – przykładowe rozszerzenie (Manifest V3) z `FlatDB` w workerze tła i w stronie rozszerzenia; worker tła bywa zatrzymywany, więc test odtworzenia bazy z IndexedDB po wybudzeniu i pomiar jego czasu.
- [ ] **1.5 Backend OPFS – tylko jako opcja** (średni) – szybszy zapis dla większych baz, ale synchroniczny dostęp do OPFS istnieje wyłącznie w Web Workerze, więc nie może zastąpić IndexedDB jako trybu domyślnego. [Q7].

## 2. Własność danych i reguły serwera

Wspólna podstawa: replikacja dwuwarstwowa Graya i in. [C9] – każdy obiekt ma właściciela (kopię główną), klient wykonuje zmiany wstępne na lokalnej kopii, a właściciel zatwierdza je lub odrzuca według kryterium akceptacji – oraz zapisy wstępne i zatwierdzone w Bayou [C3].

- [ ] **2.1 Tabele prowadzone przez serwer** (mały–średni) – deklaracja w schemacie (opcja tabeli wskazująca serwer jako jedynego autora); lokalny zapis do takiej tabeli kończy się błędem, zmiany przychodzą tylko z synchronizacji. Podstawa dla pamięci podręcznej danych serwera, katalogów w kasach, flag i konfiguracji, rankingów i danych gry wrażliwych na oszustwa. Pliki: `crates/flatsql-core/src/sql/parser.rs`, `table.rs`, `db.rs`, [flat-sql.md](flat-sql.md).
- [ ] **2.2 Walidacja zmian w `/sync/push`** (średni) – kolumny tylko do odczytu dla klientów (np. ceny), sprawdzanie właściciela wiersza, zakresy wartości; odrzucone zmiany raportowane jak dziś `rejected`. [C9], [L8].
- [ ] **2.3 Kompensacja odrzuconych zmian** (średni) – serwer odsyła poprawną wartość z nowszym znacznikiem, żeby replika klienta wróciła do dozwolonego stanu (wzór: cofanie zmian naruszających więzy w CRR [L24]).
- [ ] **2.4 Zamiary zatwierdzane przez serwer** (średni) – klient dopisuje wiersz ze statusem „oczekujący” (zamówienie w kasie, ruch w grze turowej, zakup), serwer sprawdza go kryterium akceptacji i zmienia status na „zatwierdzony” albo „odrzucony” z powodem, a wynik zapisuje w tabeli z 2.1; interfejs pokazuje stan wstępny do czasu zatwierdzenia. To transakcje wstępne i bazowe z [C9]. Płatności zawsze zatwierdza serwer – tak działają wszystkie znalezione systemy kasowe.
- [ ] **2.5 Scalanie deklarowane w SQL** (średni) – np. `qty INT MERGE ADD`: licznik sumowany zamiast nadpisywanego. Aktualizacje przemienne nie gubią zmian, w odróżnieniu od znaczników czasu [C9]; licznik CRDT [L2]; kolumny addytywne w Antidote SQL [L8]. Rozszerzenie: licznik z dolną granicą (np. saldo ≥ 0) przez przydział „do wydania offline” z góry – metoda escrow z licznikami ograniczonymi w [L8]. Wymaga [todo.md 3.1](todo.md).
- [ ] **2.6 Schemat zarządzany przez backend** (mały) – klienci bez prawa do `CREATE/DROP TABLE` i `INDEX` w synchronizacji; powiązane z [todo.md 1.7](todo.md) i 6.1.

## 3. Rozdzielenie danych i replikacja częściowa

Dziś serwer ma jedną bazę dla wszystkich: każdy klient wysyła do niej swoje zmiany i pobiera zmiany wszystkich pozostałych. Analiza wdrożeń pokazała, że przy wzroście danych pobieranie całości przestaje działać (Productlane musiało zmienić silnik).

- [ ] **3.1 Osobna baza na użytkownika** (średni) – mapa `użytkownik → baza + log + epoka`, ładowanie z dysku przy pierwszym żądaniu, zwalnianie nieużywanych baz z pamięci, osobny katalog danych; model stosowany m.in. przez Cloudflare Durable Objects i Turso. Najprostsza droga dla danych niezależnych (zapisy gier, notatki). Według [ADR-0023](adr/0023-konfigurowalne-tryby-serwera-synchronizacji.md) to tryb z silnikiem; domyślnym trybem przestrzeni jest magazyn paczek bez silnika.
- [ ] **3.2 Replikacja częściowa z filtrami** (duży) – dla danych współdzielonych: klient deklaruje swój zakres (np. przestrzeń roboczą, region, listę rodzin pacjentów), serwer filtruje `/sync/pull` i odrzuca w `/sync/push` zmiany spoza uprawnień. Wzór z Cimbiosys [C11]: gwarancja, że klient w końcu ma dokładnie wiersze pasujące do filtra; wiersz, który przestał pasować (zmiana danych albo zakresu), serwer zgłasza komunikatem „wyjścia z zakresu”, a klient usuwa go **tylko lokalnie**, bez wysyłania `DELETE`. Hybryda „lokalny silnik + serwer koordynujący” [L20].
- [ ] **3.3 Przestrzenie współdzielone** (średni) – baza zespołu, pokoju gry lub dokumentu z listą członków i ich uprawnieniami (na 3.1 lub 3.2).
- [ ] **3.4 Stopniowe pobieranie stanu początkowego** (średni) – pierwszy pełny stan w częściach (najpierw dane pierwszego ekranu, reszta w tle), z wznowieniem po przerwaniu; dziś `exportState` to jedna paczka całej bazy. Wzór: start aplikacji Linear; przy filtrach z 3.2 – pobranie zakresu [C11].
- [ ] **3.5 Testy izolacji** (mały) – klient użytkownika A nigdy nie otrzymuje wierszy użytkownika B, także po kompaktowaniu logu, przy pełnym stanie i po zmianie zakresu.
- [ ] **3.6 Pomiar przy wielu użytkownikach** (mały) – pamięć serwera, czas ładowania bazy użytkownika, czas pierwszej synchronizacji.
- [ ] **3.7 Uzgadnianie zbiorów po utracie kursora** (duży, warunkowo) – zamiast pełnego `exportState` porównywanie odcisków zakresów kluczy (uzgadnianie zakresów, Meyer 2023 – praca poza rdzeniem bazy wiedzy). Tylko jeśli po trwałym logu ([todo.md 3.2](todo.md)) pełne pobrania nadal są problemem.

## 4. Bezpieczeństwo i prywatność

- [ ] **4.1 Uwierzytelnianie** (średni) – backend wydaje i sprawdza token; serwer synchronizacji przyjmuje tylko żądania z poprawnym tokenem albo jest dostępny wyłącznie przez bramkę backendu. `SyncClient` przyjmuje własną funkcję `fetch`, więc nagłówek z tokenem można dodać bez zmian w bibliotece. Uwierzytelnianie między węzłami edge a bazą główną (7.1).
- [ ] **4.2 `/api/query` tylko dla backendu** (mały) – osobny port lub klucz API, wyłączanie endpointu w konfiguracji; dziś wykonuje dowolny SQL bez logowania i nie wolno go wystawiać publicznie.
- [ ] **4.3 Szyfrowanie end-to-end** (duży) – paczki zmian szyfrowane kluczem użytkownika; serwer jako przekaźnik nieprzezroczystych zmian – do rozstrzygnięcia LWW wystarczy jawny znacznik i skrót klucza, wartości mogą być zaszyfrowane [L11], [L1]. Koszt: utrata `/api/query` i walidacji po stronie serwera (2.2) dla zaszyfrowanych tabel – do decyzji per tabela.
- [ ] **4.4 Szyfrowanie danych lokalnych** (średni) – WAL i snapshot szyfrowane przez WebCrypto (AES-GCM, klucz z hasła); dopiero 4.3 i 4.4 razem czynią „Privacy by Design” prawdziwym w sensie [V1].

## 5. Zapytania

- [ ] **5.1 Zapytania reaktywne** (średni) – subskrypcja zapytania jednotabelowego; po zatwierdzeniu sprawdzane tylko zmienione klucze. `COUNT` i `SUM` aktualizowane przyrostowo, `MIN`/`MAX` po usunięciu bieżącego ekstremum wymagają ponownego przeliczenia (lub indeksu) [M3]; cel: aktualizacja w mikrosekundach, w budżecie 16 ms [D4]. Wymaga 1.1 (powiadomienia między kartami).
- [ ] **5.2 Złączenia tylko do odczytu po kluczu głównym** (duży) – równościowe, bez `FOREIGN KEY`: złączenie po unikalnym kluczu zachowuje ograniczony koszt zapytania [M2] i nie zmienia replikacji (dalej per wiersz – trudność leży w więzach, nie w złączeniach [L8]). Brak złączeń jest najczęstszym zarzutem wobec modeli NoSQL [M1], a w aplikacjach local-first złączenia stanowią dużą część kodu UI [D4]. **Decyzja autora:** zmienia tożsamość dialektu „bez JOIN”.
- [ ] **5.3 `CASE` w wyrażeniach** (mały) – najczęściej brakująca konstrukcja w obrębie jednej tabeli.
- [ ] **5.4 Punkty przywracania i sloty** (średni) – nazwany punkt przywracania zapamiętujący LSN; przywrócenie stanu (snapshot + odtworzenie WAL do LSN) albo kopia bazy do osobnego slotu; zastosowania: sloty zapisu w grach, „spróbuj i cofnij”. Pliki: `crates/flatsql-core/src/db.rs`, `storage/`, `bridge/flatdb.js`.
- [ ] **5.5 Wyszukiwanie pełnotekstowe** (średni) – indeks słów dla kolumny `TEXT`, operator wyszukiwania, podział na słowa z obsługą Unicode i wielkości liter, proste ustalanie trafności; limit wzrostu modułu: kilkadziesiąt KiB. Literatury nie zebrano – przy rozpoczęciu zadania: indeks odwrócony (np. Zobel i Moffat, *Inverted files for text search engines*, ACM CSUR 2006).
- [ ] **5.6 Typ wektorowy i wyszukiwanie podobieństwa** (średni–duży) – `VECTOR(n)` przechowywany zwarcie (`f32`), funkcje odległości, `ORDER BY odległość LIMIT k` przez istniejące top-k; najpierw porównanie ze wszystkimi wierszami (do kilkudziesięciu tysięcy wpisów), później indeks przybliżony (np. HNSW – Malkov i Yashunin, IEEE TPAMI 2020; literatury nie zebrano). Zasada przejrzystości: źródło każdego wpisu zapisywane i widoczne dla użytkownika [V1].

## 6. Ewolucja schematu, protokół i współpraca

- [ ] **6.1 Wersjonowanie protokołu i schematu** (średni) – wersja formatu paczek, wersje schematu tabel, migracje deklarowane przez aplikację; klient offline może wrócić po miesiącach ze starszym formatem ([P1] rozdz. 4). Wzór tłumaczenia danych między wersjami schematu: Cambria [L26]. Na bazie [todo.md 1.7](todo.md).
- [ ] **6.2 Powiadomienia push** (średni) – WebSocket lub SSE zamiast samego odpytywania co kilka sekund ([P1] rozdz. 12); potrzebne do współpracy na żywo (6.3) i szybkiego rozsyłania zmian z centrali (7.1).
- [ ] **6.3 Współdzielona treść edytowana na żywo przez bibliotekę CRDT** (średni) – każda aktualizacja dokumentu z biblioteki CRDT (np. Yjs) zapisywana jako osobny, niezmienny wiersz (unikalny klucz, więc bez konfliktów LWW), a dokument składany przez bibliotekę w aplikacji; Flat-SQL zapewnia trwałość, pracę offline i transport [L1], [L28]. Wymaga 6.2.

## 7. Edge – węzły synchronizowane z bazą główną

Założenie z prezentacji: przeglądarki ↔ węzły edge blisko użytkowników ↔ baza główna. Dziś jest jeden serwer, który jest hubem i bazą główną naraz ([conclusions.md §3.6](references/conclusions.md#36-edge-computing)). W literaturze to replikacja multi-leader w topologii drzewa ([P1] rozdz. 5), scenariusz CRR [L24] i warstwa bazowa z replikacji dwuwarstwowej [C9]. Gray i in. ostrzegają, że replikacja „zapis wszędzie” traci stabilność z liczbą węzłów – przy LWW nie rosną uzgodnienia, ale rośnie liczba utraconych aktualizacji, dlatego dla danych ryzykownych potrzebny jest tryb jednego zapisującego (7.2).

- [ ] **7.1 Węzły edge** (średni–duży) – pliki: `server/server.js`, `test/`, [architecture.md §5.4](architecture.md). Wymaga [todo.md 3.2](todo.md) (trwały log):
  - [ ] opcja `--upstream <url>`: węzeł zachowuje własną replikę i log dla swoich klientów, a z bazą główną rozmawia tym samym protokołem `/sync/push` i `/sync/pull`;
  - [ ] przekazywanie w górę zmian przyjętych od klientów i zapisów z `/api/query` (dziś zmiany od klientów celowo nie trafiają do outboksu serwera), z kursorem przesuwanym dopiero po odpowiedzi 200;
  - [ ] pobieranie zmian z bazy głównej z kursorem i epoką, dopisywanie ich do logu węzła bez odsyłania w górę; obsługa pełnego stanu przy nieznanym kursorze;
  - [ ] praca węzła przy niedostępnej bazie głównej (obsługa klientów, ponawianie z wykładniczym odstępem) – praca bez połączenia jak w [C10], na poziomie węzła;
  - [ ] uwierzytelnianie między węzłem a bazą główną (4.1);
  - [ ] testy end-to-end: 2 klientów ↔ 2 węzły ↔ baza główna, odcięcie węzła, konflikt edycji tego samego wiersza przez klientów różnych węzłów, restart węzła – identyczny stan wszędzie;
  - [ ] pomiar latencji klient → węzeł vs klient → baza główna z symulowanym RTT (rozszerzenie H1).
- [ ] **7.2 Tryb jednego zapisującego dla wybranych tabel** (średni) – zapis na węźle przekazywany do bazy głównej i potwierdzany przez nią zamiast rozstrzygania LWW na węźle (replikacja „lazy-master” w [C9], tak działa LiteFS) – dla danych, przy których spójność ostateczna jest ryzykowna, np. unieważniania sesji. Właściwość tabeli w trybie z silnikiem ([ADR-0023](adr/0023-konfigurowalne-tryby-serwera-synchronizacji.md)).
- [ ] **7.3 Wdrożenie na platformie edge** (średni) – najprościej: serwer Node.js z 7.1 w kilku regionach; docelowo platforma typu Cloudflare Workers, gdzie jednowątkowy obiekt ze stanem i trwałym magazynem (Durable Object) odpowiada modelowi wykonania Flat-SQL – potrzebny backend `Storage` dla magazynu platformy i synchronizacja wyzwalana żądaniami lub harmonogramem. [E1].

## 8. Wdrożenie obok backendu (np. FastAPI)

```text
przeglądarka (FlatDB + SyncClient)
      │ synchronizacja + token
      ▼
backend, np. FastAPI (logowanie, uprawnienia) ──przekazuje──► serwer Flat-SQL (kontener)
      │                                                             ▲
      └── logika biznesowa: odczyty i zapisy przez API ─────────────┘
```

Już działa: nowy klient bez kursora dostaje pełny stan bazy, a potem `SyncClient` synchronizuje zmiany w obie strony; zapisy przez `/api/query` trafiają do logu serwera, więc dostają je klienci. Serwer działa w osobnym kontenerze, backend jako bramka; synchronizacja w procesie backendu przez wiązania dla Pythona wypadła z planów ([ADR-0024](adr/0024-bez-wiazan-dla-pythona.md)).

- [ ] **8.1 Dockerfile** (mały) – obraz Node.js 22; moduł `pkg/` budowany w osobnym etapie (Rust + `wasm-pack`) albo kopiowany gotowy; `node server/server.js --host 0.0.0.0 --port 8080 --data /data` (serwer domyślnie słucha na `127.0.0.1`), wolumen `/data`, healthcheck przez `/api/status`.
- [ ] **8.2 Tryb bez dema** (mały) – opcja wyłączająca hosting `web/`, żeby kontener udostępniał tylko synchronizację i API.
- [ ] **8.3 Konfigurowalny CORS** (mały) – dziś `access-control-allow-origin: *`.
- [ ] **8.4 Przykład `docker-compose` z FastAPI** (średni) – backend jako bramka: logowanie, przekazywanie `/sync/*` do kontenera, logika biznesowa przez `/api/query` w sieci wewnętrznej.
- [ ] **8.5 Instrukcja wdrożenia** (mały) – po polsku: uruchomienie, parametry, kopie zapasowe katalogu danych (`snapshot.fsql`, `wal.fsql`), aktualizacja obrazu.

## 9. Interfejs C, wiązania i panel administracyjny

Pakiet Pythona i synchronizacja w FastAPI (dawne 9.2–9.4) wypadły z planów – [ADR-0024](adr/0024-bez-wiazan-dla-pythona.md).

- [ ] **9.1 Interfejs C nad `flatsql-core`** (średni) – biblioteka `cdylib`: otwarcie bazy z katalogu, `execute`, przygotowane instrukcje, `export_outbox` / `ack_outbox` / `apply_changes` / `export_state`, zwalnianie pamięci.
- [ ] **9.5 Inne języki i aplikacje natywne** (średni) – interfejs C dla Go, C# i aplikacji mobilnych (Swift, Kotlin, React Native): natywny Flat-SQL zamiast SQLite, z tym samym formatem plików i synchronizacją co w przeglądarce (aplikacje mobilne to jedno z głównych zastosowań PowerSync i Turso).
- [ ] **9.6 Panel administracyjny WinUI 3** (duży) – kolejny etap projektu z README i prezentacji: komunikacja z silnikiem przez interfejs C z 9.1 lub named pipes; wspólny format plików pozwala otworzyć bazę serwera. Literatury dla tego etapu nie zebrano.

## 10. Skala serwera

- [ ] **10.1 Zarządzanie pamięcią** (średni) – limit baz w pamięci, zrzucanie nieużywanych (z 3.1).
- [ ] **10.2 Wiele instancji serwera** (duży) – przypisanie użytkowników do instancji (np. po identyfikatorze) albo węzły z sekcji 7.
- [ ] **10.3 Monitoring** (mały) – liczba klientów, rozmiar logu, czas `push`/`pull`, liczba odrzuconych zmian i konfliktów (rozszerzenie `/api/status`).

## 11. Integracja z istniejącą bazą główną (np. PostgreSQL)

Jeśli backend ma już własną bazę główną, Flat-SQL staje się warstwą synchronizacji obok niej – tak działają PowerSync i ElectricSQL [Q11]. Analityka wszystkich użytkowników i tak trafia do bazy serwerowej (Community Health Toolkit przenosi dane z CouchDB do PostgreSQL).

- [ ] **11.1 Jednokierunkowy eksport do PostgreSQL** (średni) – zmiany z logu serwera przekazywane do tabel PostgreSQL na potrzeby analityki; prostszy krok przed 11.2.
- [ ] **11.2 Łącznik PostgreSQL ↔ Flat-SQL** (duży) – przechwytywanie zmian PostgreSQL (np. replikacja logiczna) i zapis do niego zmian klientów po walidacji (2.2). Wzorzec: Change Data Capture ([P1] rozdz. 11).
- [ ] **11.3 Mapowanie typów i kluczy** (średni) – typy PostgreSQL na `INT/FLOAT/TEXT/BOOL`, klucze złożone na jednokolumnowe.
- [ ] **11.4 Konflikty między źródłami** (średni) – znaczniki HLC dla zapisów pochodzących z PostgreSQL, żeby LWW rozstrzygało je spójnie z zapisami klientów [C2]; dla tabel z 2.1 PostgreSQL jest jedynym autorem i konfliktów nie ma.
