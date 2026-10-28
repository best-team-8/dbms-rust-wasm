# Rozszerzenia – droga do dojrzałego Flat-SQL

Nowe możliwości, których prototyp nie ma, a bez których nie obsłuży zastosowań potwierdzonych analizą rzeczywistych wdrożeń ([research/real-world-usage.md](research/real-world-usage.md)). Lista obejmuje też optymalizacje istniejącego prototypu odłożone na po pracy inżynierskiej (§12); **[todo.md](todo.md)** zawiera tylko zakres pracy ([ADR-0029](adr/0029-listy-zadan-wedlug-zakresu-pracy.md)) – część rozszerzeń zakłada wykonanie zadań stamtąd (podano przy zadaniu). Wnioski: [references/conclusions.md](references/conclusions.md). Identyfikatory w nawiasach (np. [C9]) odsyłają do [indeksu literatury](references/README.md).

Projekt docelowego serwera synchronizacji zebrany w jednym miejscu: [future/components/sync-server.md](future/components/sync-server.md).

Nakład: **mały** – pojedyncze funkcje i testy; **średni** – kilka modułów, nowe testy; **duży** – nowy podsystem.

**Warunek każdego rozszerzenia:** nie odbiera atutów Flat-SQL – mały moduł (setki KiB), praca w wątku głównym bez Web Workera i nagłówków COOP/COEP, zapytania lokalne w mikrosekundach, przyrostowa trwałość (WAL), wbudowana synchronizacja, płaski model tabel adresowanych kluczem, jeden rdzeń bez I/O działający wszędzie. Rozszerzenie, które wymaga Workera lub dużej biblioteki, jest **opcją**, nie domyślnym trybem.

## Priorytety według zastosowań

Najmocniej potwierdzone zastosowania ([real-world-usage.md §4](research/real-world-usage.md#4-zastosowania-flat-sql-potwierdzone-analizą)) i co je blokuje:

| Zastosowanie | Dowody | Potrzebne rozszerzenia |
|---|---|---|
| Zbieranie danych w terenie offline | mocne (CHT, DHIS2, KoboToolbox, PowerSync) | 4.1, 3.1 lub 3.2, 2.1–2.3 |
| Aplikacje osobiste na wielu urządzeniach | mocne (Actual Budget, Dexie Cloud) | 4.1, 3.1, 4.3 |
| Pamięć podręczna danych serwera w aplikacjach SaaS | mocne (Notion, Linear) | 1.1, 1.2, 2.1, 3.2 |
| Kasy i sprzedaż offline | średnie | 2.1, 2.4, 4.1 |
| Repliki konfiguracji na edge (flagi, personalizacja) | średnie | 7.1–7.3, 2.1 |
| Zapisy gier w chmurze | słabe–średnie | 3.1, 4.1, 1.3, 2.5, 5.4 |
| Pamięć asystentów i agentów programowych | średnie (PGlite + pgvector, Zeta-Lite [D5]) | 5.6, 5.5, 5.7, 4.3, 4.4, 12.11, 12.15 |

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
| Interfejs C, aplikacje natywne | PowerSync, Turso (aplikacje mobilne) | 9.1, 9.5 |
| Eksport bazy do pliku, punkty przywracania | gry, kasy, Logseq | 1.3, 5.4 |
| Wyszukiwanie pełnotekstowe, wektory | rozszerzenia przeglądarki, notatki, pamięć asystentów | 5.5, 5.6 |
| Rozgałęzianie bazy i scalanie gałęzi | Zeta-Lite, pamięć agentów, sloty zapisu w grach | 5.4, 5.7, 12.13, 12.14 |

## 1. Platforma przeglądarki

- [ ] **1.1 Wiele kart** (średni) – dziś demo trzyma blokadę Web Locks: pierwsza karta zapisuje do IndexedDB, kolejne pracują tylko w pamięci, więc ich zapisy giną po zamknięciu ([architecture.md §4.5](architecture.md)). Docelowo jedna instancja silnika obsługuje wszystkie karty – wzór z wdrożenia Notion, gdzie współbieżne zapisy z kilku kart uszkadzały dane [Q12]:
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

Reguły biznesowe należą do backendu; serwer synchronizacji daje mu tylko mechanizmy, w trybie z silnikiem ([ADR-0026](adr/0026-reguly-w-backendzie-mechanizmy-w-serwerze.md)). Wspólna podstawa: replikacja dwuwarstwowa Graya i in. [C9] – każdy obiekt ma właściciela (kopię główną), klient wykonuje zmiany wstępne na lokalnej kopii, a właściciel zatwierdza je lub odrzuca według kryterium akceptacji – oraz zapisy wstępne i zatwierdzone w Bayou [C3].

- [ ] **2.1 Tabele tylko do odczytu dla klientów** (mały–średni) – lista tabel w konfiguracji przestrzeni (np. `"clientReadOnly": ["prices"]`); serwer odrzuca paczki klientów zmieniające te tabele, zapisuje je tylko backend przez `/api/query`; lokalny zapis do takiej tabeli kończy się błędem, zmiany przychodzą tylko z synchronizacji ([ADR-0026](adr/0026-reguly-w-backendzie-mechanizmy-w-serwerze.md)). Podstawa dla pamięci podręcznej danych serwera, katalogów w kasach, flag i konfiguracji, rankingów i danych gry wrażliwych na oszustwa. Pliki: `crates/flatsql-core/src/sql/parser.rs`, `table.rs`, `db.rs`, [flat-sql.md](flat-sql.md).
- [ ] **2.2 Webhook do backendu** (średni) – opcjonalny adres backendu w konfiguracji przestrzeni; przed przyjęciem paczki serwer wysyła zmiany jako JSON (tabela, klucz, stan przed i po), jedno żądanie na paczkę, a backend odpowiada listą odrzuconych; odrzucone zmiany raportowane jak dziś `rejected`. Reguły pisze programista w backendzie, nie w serwerze ([ADR-0026](adr/0026-reguly-w-backendzie-mechanizmy-w-serwerze.md)). [C9].
- [ ] **2.3 Kompensacja odrzuconych zmian** (średni) – serwer odsyła poprawną wartość z nowszym znacznikiem, żeby replika klienta wróciła do dozwolonego stanu (wzór: cofanie zmian naruszających więzy w CRR [L24]).
- [ ] **2.4 Zamiary zatwierdzane przez backend** (średni) – klient dopisuje wiersz ze statusem „oczekujący” (zamówienie w kasie, ruch w grze turowej, zakup), backend sprawdza go kryterium akceptacji (webhook z 2.2 albo własny odczyt przez `/api/query`) i zmienia status na „zatwierdzony” albo „odrzucony” z powodem, a wynik zapisuje w tabeli z 2.1; interfejs pokazuje stan wstępny do czasu zatwierdzenia. To transakcje wstępne i bazowe z [C9]. Płatności zawsze zatwierdza serwer – tak działają wszystkie znalezione systemy kasowe.
- [ ] **2.5 Scalanie deklarowane w SQL** (średni) – np. `qty INT MERGE ADD`: licznik sumowany zamiast nadpisywanego. Aktualizacje przemienne nie gubią zmian, w odróżnieniu od znaczników czasu [C9]; licznik CRDT [L2]; kolumny addytywne w Antidote SQL (Lopes i in. 2019 – praca poza rdzeniem). Rozszerzenie: licznik z dolną granicą (np. saldo ≥ 0) przez przydział „do wydania offline” z góry – metoda escrow z licznikami ograniczonymi w Antidote SQL. Wymaga 12.3.
- [ ] **2.6 Blokada zmian struktury z synchronizacji** (mały) – serwer odrzuca w paczkach klientów polecenia `CREATE/DROP TABLE` i `INDEX`; strukturę buduje kod aplikacji ([ADR-0027](adr/0027-struktura-w-kodzie-aplikacji-tylko-dokladanie.md)); powiązane z [todo.md 1.7](todo.md) i 6.1.

**Propozycja konfiguracji przestrzeni i webhooka** (do ustalenia przy 2.1–2.2 i 4.1; format przykładowy):

```json
{
  "gatewayKey": "env:SYNC_KEY",
  "default": { "mode": "packets" },
  "spaces": {
    "user-*":   { "mode": "packets" },
    "shop":     { "mode": "engine", "clientReadOnly": ["prices"] },
    "player-*": { "mode": "engine", "validate": "http://backend:3000/flatsql/validate" }
  }
}
```

Serwer dopasowuje nazwę przestrzeni podaną przez bramkę do wzorca, a bez dopasowania używa `default`; tryb jest zapisywany w `space.json` przy tworzeniu przestrzeni. Webhook – jedno żądanie na paczkę, z kluczem bramki w nagłówku `x-flatsql-key`:

```json
{
  "space": "player-42",
  "batch": "7f3c9a1e",
  "changes": [
    { "index": 0, "op": "update", "table": "player", "key": 42,
      "before": { "id": 42, "gold": 120 }, "after": { "id": 42, "gold": 999999 } },
    { "index": 1, "op": "insert", "table": "inventory", "key": 1001,
      "before": null, "after": { "id": 1001, "player": 42, "item": "sword" } }
  ]
}
```

Odpowiedź backendu: `{ "reject": [ { "index": 0, "reason": "…" } ] }` (pusta lista – wszystko przyjęte). Odrzucenie dotyczy pojedynczych wierszy (ADR-0018); brak odpowiedzi w limicie czasu albo błąd backendu – paczka nie jest przyjęta i zostaje w outboksie urządzenia; `batch` pozwala backendowi zwrócić tę samą decyzję przy ponowieniu.

## 3. Rozdzielenie danych i replikacja częściowa

Dziś serwer ma jedną bazę dla wszystkich: każdy klient wysyła do niej swoje zmiany i pobiera zmiany wszystkich pozostałych. Analiza wdrożeń pokazała, że przy wzroście danych pobieranie całości przestaje działać (Productlane musiało zmienić silnik).

- [ ] **3.1 Osobna baza na użytkownika** (średni) – mapa `użytkownik → baza + log + epoka`, ładowanie z dysku przy pierwszym żądaniu, zwalnianie nieużywanych baz z pamięci, osobny katalog danych; model stosowany m.in. przez Cloudflare Durable Objects i Turso. Najprostsza droga dla danych niezależnych (zapisy gier, notatki). Według [ADR-0023](adr/0023-konfigurowalne-tryby-serwera-synchronizacji.md) to tryb z silnikiem; domyślnym trybem przestrzeni jest magazyn paczek bez silnika.
- [ ] **3.2 Replikacja częściowa z filtrami** (duży) – dla danych współdzielonych: klient deklaruje swój zakres (np. przestrzeń roboczą, region, listę rodzin pacjentów), serwer filtruje `/sync/pull` i odrzuca w `/sync/push` zmiany spoza uprawnień. Wzór z Cimbiosys [C11]: gwarancja, że klient w końcu ma dokładnie wiersze pasujące do filtra; wiersz, który przestał pasować (zmiana danych albo zakresu), serwer zgłasza komunikatem „wyjścia z zakresu”, a klient usuwa go **tylko lokalnie**, bez wysyłania `DELETE`. Hybryda „lokalny silnik + serwer koordynujący” [C9].
- [ ] **3.3 Przestrzenie współdzielone** (średni) – baza zespołu, pokoju gry lub dokumentu z listą członków i ich uprawnieniami (na 3.1 lub 3.2).
- [ ] **3.4 Stopniowe pobieranie stanu początkowego** (średni) – pierwszy pełny stan w częściach (najpierw dane pierwszego ekranu, reszta w tle), z wznowieniem po przerwaniu; dziś `exportState` to jedna paczka całej bazy. Wzór: start aplikacji Linear; przy filtrach z 3.2 – pobranie zakresu [C11].
- [ ] **3.5 Testy izolacji** (mały) – klient użytkownika A nigdy nie otrzymuje wierszy użytkownika B, także po kompaktowaniu logu, przy pełnym stanie i po zmianie zakresu.
- [ ] **3.6 Pomiar przy wielu użytkownikach** (mały) – pamięć serwera, czas ładowania bazy użytkownika, czas pierwszej synchronizacji.
- [ ] **3.7 Uzgadnianie zbiorów po utracie kursora** (duży, warunkowo) – zamiast pełnego `exportState` porównywanie odcisków zakresów kluczy (uzgadnianie zakresów, Meyer 2023 – praca poza rdzeniem bazy wiedzy). Tylko jeśli po trwałym logu (12.4) pełne pobrania nadal są problemem.

## 4. Bezpieczeństwo i prywatność

- [ ] **4.1 Uwierzytelnianie przez bramkę backendu** (średni) – według [ADR-0025](adr/0025-serwer-synchronizacji-za-bramka-backendu.md): serwer synchronizacji nasłuchuje tylko w sieci wewnętrznej, przestrzeń bierze z nagłówka ustawionego przez backend i odrzuca żądania bez wspólnego klucza bramki; konta, hasła i logowanie zostają w backendzie aplikacji, który przekazuje `/sync/*` przez HTTP bez czytania paczek. Weryfikacja przepustek JWT przez sam serwer – zob. [Kierunki na później](#kierunki-na-później-bez-zadań). Uwierzytelnianie między węzłami edge a bazą główną (7.1).
- [ ] **4.2 `/api/query` tylko dla backendu** (mały) – dostęp tylko z bramką i jej kluczem (4.1), wyłączanie endpointu w konfiguracji; dziś wykonuje dowolny SQL bez logowania i nie wolno go wystawiać publicznie.
- [ ] **4.3 Szyfrowanie end-to-end** (duży) – paczki zmian szyfrowane kluczem użytkownika; serwer jako przekaźnik nieprzezroczystych zmian – do rozstrzygnięcia LWW wystarczy jawny znacznik i skrót klucza, wartości mogą być zaszyfrowane [L1]; wzór: szyfrowane typy replikowane (Kuessner i in. 2022 – praca poza rdzeniem). Koszt: utrata `/api/query` i walidacji po stronie serwera (2.2) dla zaszyfrowanych tabel – do decyzji per tabela.
- [ ] **4.4 Szyfrowanie danych lokalnych** (średni) – WAL i snapshot szyfrowane przez WebCrypto (AES-GCM, klucz z hasła); dopiero 4.3 i 4.4 razem czynią „Privacy by Design” prawdziwym w sensie [V1].

## 5. Zapytania

- [ ] **5.1 Zapytania reaktywne** (średni) – subskrypcja zapytania jednotabelowego; po zatwierdzeniu sprawdzane tylko zmienione klucze. `COUNT` i `SUM` aktualizowane przyrostowo, `MIN`/`MAX` po usunięciu bieżącego ekstremum wymagają ponownego przeliczenia (lub indeksu) – przyrostowe utrzymanie widoków, Gupta i Mumick 1995 (praca poza rdzeniem); cel: aktualizacja w mikrosekundach, w budżecie 16 ms [D4]. Wymaga 1.1 (powiadomienia między kartami).
- [ ] **5.3 `CASE` w wyrażeniach** (mały) – najczęściej brakująca konstrukcja w obrębie jednej tabeli.
- [ ] **5.4 Punkty przywracania i sloty** (średni) – nazwany punkt przywracania zapamiętujący LSN; przywrócenie stanu (snapshot + odtworzenie WAL do LSN) albo kopia bazy do osobnego slotu; zastosowania: sloty zapisu w grach, „spróbuj i cofnij”. Pliki: `crates/flatsql-core/src/db.rs`, `storage/`, `bridge/flatdb.js`.
- [ ] **5.5 Wyszukiwanie pełnotekstowe** (średni) – indeks słów dla kolumny `TEXT`, operator wyszukiwania, podział na słowa z obsługą Unicode i wielkości liter, proste ustalanie trafności; limit wzrostu modułu: kilkadziesiąt KiB. Literatury nie zebrano – przy rozpoczęciu zadania: indeks odwrócony (np. Zobel i Moffat, *Inverted files for text search engines*, ACM CSUR 2006).
- [ ] **5.6 Typ wektorowy i wyszukiwanie podobieństwa** (średni–duży) – `VECTOR(n)` przechowywany zwarcie (`f32`), funkcje odległości, `ORDER BY odległość LIMIT k` przez istniejące top-k; najpierw porównanie ze wszystkimi wierszami (do kilkudziesięciu tysięcy wpisów), później indeks przybliżony (np. HNSW – Malkov i Yashunin, IEEE TPAMI 2020; literatury nie zebrano). Zasada przejrzystości: źródło każdego wpisu zapisywane i widoczne dla użytkownika [V1].
- [ ] **5.7 Gałęzie bazy jako repliki** (średni) – kopia bazy, na której agent albo użytkownik pracuje osobno, a potem ją scala albo porzuca; izolacja pracy bez wielu wersji wierszy ([ADR-0053](adr/0053-galezie-bazy-zamiast-mvcc.md)). Rozwija 5.4. Pliki: `crates/flatsql-core/src/db.rs`, `sync.rs`, `bridge/flatdb.js`.
  - [ ] utworzenie gałęzi: snapshot i odtworzenie do drugiej instancji z własnym identyfikatorem węzła, żeby znaczniki HLC były unikalne; koszt to kopia bazy (przy 100 000 wierszy około 70 ms i podwójna pamięć w Benchmarku 2);
  - [ ] gałąź zbiera klucze zmienione od chwili utworzenia osobno od outboxa, bo włączenie synchronizacji oznacza cały stan jako oczekujący (ADR-0043);
  - [ ] scalanie: zmiany gałęzi nakładane na bazę główną tą samą ścieżką co zmiany z serwera (`apply_changes`, LWW per wiersz);
  - [ ] scalanie warunkowe: gałąź zapamiętuje znaczniki wierszy przeczytanych i zmienionych, baza główna sprawdza przed scaleniem, czy się nie zmieniły; przy zmianie odrzuca całe scalenie, w innym wypadku scala w jednej lokalnej transakcji (wszystko albo nic) – sterowanie optymistyczne wykonywane raz, przy scaleniu;
  - [ ] odświeżenie gałęzi zmianami z bazy głównej;
  - [ ] testy: dwie gałęzie zmieniające ten sam wiersz, odrzucone scalenie warunkowe, porzucenie gałęzi bez śladu w WAL bazy głównej, losowe przeploty jak w testach synchronizacji.
  Gwarancje scalania warunkowego dotyczą jednego urządzenia; między replikami atomowy pozostaje wiersz (ADR-0018).
- [ ] **5.8 Przykład pamięci asystenta** (średni) – aplikacja przykładowa według [use-cases.md §10](use-cases.md): tabela wpisów pamięci ze źródłem każdego wpisu, podgląd i usuwanie przez użytkownika (treść z obcej strony zapisana jako wspomnienie może później sterować asystentem), wyszukiwanie z 5.5 i 5.6, gałęzie z 5.7, synchronizacja z 4.3. Pomiar czasu otwarcia bazy z wektorami: 10 000 wpisów z wektorem 384 liczb `f32` to około 15 MB snapshotu więcej (zob. 12.11).

## 6. Ewolucja schematu, protokół i współpraca

- [ ] **6.1 Wersjonowanie protokołu i struktury** (średni) – według [ADR-0027](adr/0027-struktura-w-kodzie-aplikacji-tylko-dokladanie.md): kroki budowy bazy z numerami wersji w `FlatDB.open` (lokalna baza pamięta wersję i wykonuje tylko brakujące kroki), `ALTER TABLE … ADD COLUMN`, przechowywanie wartości nieznanych kolumn w wierszu, polecenia struktury nierozsyłane do innych replik; ustalić, skąd serwer w trybie z silnikiem zna strukturę przestrzeni (deklaracje scalane jako suma tabel i kolumn albo szablon w konfiguracji). Do tego wersja formatu paczek – klient offline może wrócić po miesiącach ze starszą wersją aplikacji ([P1] rozdz. 4). Na bazie [todo.md 1.7](todo.md).
- [ ] **6.2 Powiadomienia push** (średni) – WebSocket lub SSE zamiast samego odpytywania co kilka sekund ([P1] rozdz. 12); przydatne, gdy ktoś czeka na cudzą zmianę (np. ruch przeciwnika w grze turowej), i do szybkiego rozsyłania zmian z centrali (7.1).

## 7. Edge – węzły synchronizowane z bazą główną

Założenie z prezentacji: przeglądarki ↔ węzły edge blisko użytkowników ↔ baza główna. Dziś jest jeden serwer, który jest hubem i bazą główną naraz ([conclusions.md §3.6](references/conclusions.md#36-edge-computing)). W literaturze to replikacja multi-leader w topologii drzewa ([P1] rozdz. 5), scenariusz CRR [L24] i warstwa bazowa z replikacji dwuwarstwowej [C9]. Gray i in. ostrzegają, że replikacja „zapis wszędzie” traci stabilność z liczbą węzłów – przy LWW nie rosną uzgodnienia, ale rośnie liczba utraconych aktualizacji, dlatego dla danych ryzykownych potrzebny jest tryb jednego zapisującego (7.2).

- [ ] **7.1 Węzły edge** (średni–duży) – pliki: `server/server.js`, `test/`, [architecture.md §5.4](architecture.md). Wymaga 12.4 (trwały log):
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
- [ ] **8.4 Przykład bramki i `docker-compose`** (średni) – przykładowy backend jako bramka ([ADR-0025](adr/0025-serwer-synchronizacji-za-bramka-backendu.md)): logowanie, wybór przestrzeni z sesji i przekazywanie `/sync/*` do kontenera bez publicznego portu, logika biznesowa przez `/api/query` w sieci wewnętrznej; w demo – mała bramka w Node.js z udawanym logowaniem dwóch użytkowników, z których każdy widzi tylko swoje dane. Wymaga przestrzeni (4.1, [ADR-0023](adr/0023-konfigurowalne-tryby-serwera-synchronizacji.md)).
- [ ] **8.5 Instrukcja wdrożenia** (mały) – po polsku: uruchomienie, parametry, kopie zapasowe katalogu danych (`snapshot.fsql`, `wal.fsql`), aktualizacja obrazu.

## 9. Interfejs C, wiązania i panel administracyjny

Pakiet Pythona i synchronizacja w FastAPI (dawne 9.2–9.4) wypadły z planów – [ADR-0024](adr/0024-bez-wiazan-dla-pythona.md).

- [ ] **9.1 Interfejs C nad `flatsql-core`** (średni) – biblioteka `cdylib`: otwarcie bazy z katalogu, `execute`, przygotowane instrukcje, `export_outbox` / `ack_outbox` / `apply_changes` / `export_state`, zwalnianie pamięci.
- [ ] **9.5 Inne języki i aplikacje natywne** (średni) – interfejs C dla Go, C# i aplikacji mobilnych (Swift, Kotlin, React Native): natywny Flat-SQL zamiast SQLite, z tym samym formatem plików i synchronizacją co w przeglądarce (aplikacje mobilne to jedno z głównych zastosowań PowerSync i Turso).
- [ ] **9.6 Panel administracyjny WinUI 3** (duży) – kolejny etap projektu z README i prezentacji: komunikacja z silnikiem przez interfejs C z 9.1 lub named pipes; wspólny format plików pozwala otworzyć bazę serwera. Literatury dla tego etapu nie zebrano.
- [ ] **9.7 Serwer synchronizacji jako biblioteka w backendzie** (duży, rozszerzenie bez terminu) – logika serwera (przestrzenie, log, kursory, tryby z ADR-0023) przeniesiona do Rust i udostępniona przez interfejs C (9.1) z nakładkami dla języków backendu, tak żeby backend obsługiwał `/sync/*` w swoim procesie bez osobnej usługi; przy wielu instancjach backendu wymaga blokad na plikach przestrzeni albo wspólnego magazynu. Do tego czasu integracja tylko przez HTTP ([ADR-0025](adr/0025-serwer-synchronizacji-za-bramka-backendu.md)).

## 10. Skala serwera

- [ ] **10.1 Zarządzanie pamięcią** (średni) – limit baz w pamięci, zrzucanie nieużywanych (z 3.1).
- [ ] **10.2 Wiele instancji serwera** (duży) – przypisanie użytkowników do instancji (np. po identyfikatorze) albo węzły z sekcji 7.
- [ ] **10.3 Monitoring** (mały) – liczba klientów, rozmiar logu, czas `push`/`pull`, liczba odrzuconych zmian i konfliktów (rozszerzenie `/api/status`).

## 12. Optymalizacje prototypu po pracy inżynierskiej

Zadania przeniesione z [todo.md](todo.md) ([ADR-0029](adr/0029-listy-zadan-wedlug-zakresu-pracy.md)): ulepszają to, co prototyp już ma, ale nie są potrzebne w pracy – obecna synchronizacja wystarcza do walidacji pracy offline, a obecny zestaw operacji do porównania w niszy. W nawiasach dawne numery z todo.md, nieużywane ponownie.

- [ ] **12.1 Klucze dla wstawień offline** (mały; dawne todo 0.3) – w [flat-sql.md](flat-sql.md) zalecić klucze bez kolizji (UUID/ULID lub prefiks identyfikatora węzła) dla tabel zapisywanych offline; unikalność klucza głównego między replikami wymaga koordynacji ([P1] rozdz. 5 i 9), a Coda rozwiązywała to tymczasowymi identyfikatorami zastępowanymi przy reintegracji [C10].
- [ ] **12.2 Odnośniki w README** (mały; dawne todo 0.4) – sekcja „Dokumentacja”: `docs/references/`, obie listy zadań, analiza wdrożeń.
- [ ] **12.3 LWW per kolumna** (średni; dawne todo 3.1) – znacznik dla kolumny przechowywany tylko wtedy, gdy różni się od znacznika wiersza; współbieżne zmiany różnych kolumn jednego wiersza przestają ginąć. Schemat ze znacznikami gubi aktualizacje [C9], CRR stosuje rejestr LWW per atrybut [L24]. Protokół się nie zmienia (paczka niesie wiersz z wektorem znaczników); koszt rozmiaru zmierzyć w 12.10.
- [ ] **12.4 Trwały log synchronizacji serwera** (średni; dawne todo 3.2) – dziś log jest tylko w pamięci, a epoka losowana przy starcie, więc **każdy restart serwera** wysyła wszystkim klientom pełny stan bazy. Zapis logu i kursorów w katalogu danych, epoka zmieniana tylko po utracie logu. Wzorzec: trwały log zmian z offsetami konsumentów ([P1] rozdz. 11), trwały log reintegracji w Coda [C10]. Kursor to numer przyjęcia nadany przez serwer, nie znacznik czasu ([ADR-0028](adr/0028-kursor-jako-numer-przyjecia.md)). Log prowadzony per przestrzeń – wspólny fundament obu trybów serwera ([ADR-0023](adr/0023-konfigurowalne-tryby-serwera-synchronizacji.md)). Plik: `server/server.js`.
- [ ] **12.5 Raport konfliktów klucza głównego** (średni; dawne todo 3.3) – wykrywanie dwóch wstawień tego samego klucza z różnych replik i zwracanie ich w raporcie `apply_changes` / `/sync/push` zamiast cichego nadpisania. CRR wykrywa takie naruszenia przy scalaniu [L24], Simba zgłasza konflikty aplikacji [L27].
- [ ] **12.6 Usuwanie tombstone'ów** (średni; dawne todo 3.4) – tombstone można usunąć po stabilności przyczynowej, gdy wszystkie repliki potwierdziły jego znacznik [L28]; przy serwerze centralnym wystarczy minimalny potwierdzony kursor aktywnych klientów, a klienci nieaktywni dłużej niż próg dostają pełny stan (już obsłużone). Wymaga 12.4.
- [ ] **12.7 Granice transakcji w paczkach** (mały–średni; dawne todo 3.5) – najpierw udokumentować gwarancję „między replikami atomowy jest wiersz” (todo 0.1) – Simba świadomie poprzestaje na niej, uznając atomową synchronizację wielu wierszy za zbędną dla większości aplikacji [L27]. Opcjonalnie: outbox nie dzieli transakcji między paczki (Monotonic Atomic View, osiągalne bez koordynacji – Bailis i in. 2013, praca poza rdzeniem) – dopiero gdy demo pokaże potrzebę (np. wymiana przedmiotów między graczami). Pliki: `crates/flatsql-core/src/db.rs`, `bridge/sync.js`.
- [ ] **12.8 Realistyczne obciążenie** (średni; dawne todo 4.5) – YCSB z rozkładem Zipfa (np. 95% odczytów) i scenariusz z aplikacji demo. [B5].
- [ ] **12.9 Dokładniejszy zegar w przeglądarce** (mały; dawne todo 4.8) – strona benchmarku z nagłówkami COOP/COEP (5 µs zamiast 100 µs); dotyczy tylko strony pomiarowej, nie biblioteki. [Q5].
- [ ] **12.10 Koszt LWW per kolumna** (mały; dawne todo 4.11) – rozmiar snapshotu, paczek i czas insertów przed i po 12.3.
- [ ] **12.11 Odtwarzanie bazy krokami** (średni) – `Database.restore` w `FlatDB.open` to jedno synchroniczne wywołanie: 46 ms w wątku głównym przy 100 000 wierszy (Benchmark 2), czyli więcej niż budżet jednej klatki. Dekodowanie snapshotu, budowa indeksów i odtworzenie WAL porcjami z obrotem pętli zdarzeń między nimi, jak checkpoint krokami ([optimization-results.md §15](research/optimization-results.md#15-checkpoint-krokami)); baza dostępna dopiero po ostatnim kroku. Pomiar: najdłuższa pauza i cały czas otwarcia przy 10, 50 i 100 tys. wierszy. Zyskuje na znaczeniu przy 5.6 (większy snapshot).
- [ ] **12.12 Szybsza suma CRC32** (mały) – `crc32_update` w `storage/codec.rs` liczy bajt po bajcie z jedną tablicą; przy 100 000 wierszy to około 9 ms snapshotu (4,5 MB, [optimization-results.md §15](research/optimization-results.md#15-checkpoint-krokami)) i drugie tyle na początku odtworzenia. Wariant slicing-by-8 (osiem tablic, 8 bajtów na krok) daje te same wartości i nie zmienia formatu. Pomiar snapshotu, odtworzenia i ponownego otwarcia przed i po.
- [ ] **12.13 Port warstwy wierszy** (średni) – cecha `Storage` w rdzeniu (odczyt po kluczu, zakres, skan, indeksy, zapis, zatwierdzenie) z dzisiejszą implementacją jako jedynym adapterem i silnikiem generycznym po niej, więc kompilator usuwa abstrakcję. Odczyty przechodzą dziś przez ścieżki dostępu w `plan.rs`, zapisy przez `Table` (`table.rs`) i `db.rs`. Warunek: Benchmark 2 bez pogorszenia. Podstawa dla 12.14 i dla wariantu z wieloma wersjami wierszy (kierunki na później).
- [ ] **12.14 Drzewo trwałe jako adapter** (duży, warunkowo) – drzewo współdzielące niezmienione węzły, w którym gałąź z 5.7 powstaje bez kopii bazy. Tylko jako adapter z 12.13 wybierany przy budowie, bo kopiowanie ścieżek spowalnia zapisy; tylko jeśli kopia przez snapshot okaże się za droga w 5.8.
- [ ] **12.15 Warianty modułu** (mały–średni) – flagi kompilacji (`cargo` features) dla 5.5, 5.6 i 5.7, np. moduł podstawowy i moduł z dodatkami dla pamięci asystentów, żeby moduł podstawowy nie rósł; most JavaScript wczytuje wskazany wariant. Pomiar rozmiaru i zimnego startu obu wariantów.

## Kierunki na później (bez zadań)

Możliwości zgodne z koncepcją, na które nie ma jeszcze zadań ani terminu.

- **Wiele wersji wierszy (MVCC) jako wariant kompilacji** – adapter z 12.13 z łańcuchami wersji, wykrywaniem konfliktów przy zatwierdzeniu i usuwaniem starych wersji, dla wielu długich transakcji otwartych naraz (np. agenci czekający na odpowiedź modelu wewnątrz transakcji). Format WAL i snapshotu oraz protokół synchronizacji zostają, bo zawierają tylko zatwierdzony stan; API dostaje obiekt transakcji, a zatwierdzenie może zostać odrzucone. Zmiana objęłaby około 25–35% rdzenia (szacunek z kodu: `table.rs`, `db.rs`, część `plan.rs`, `exec.rs`, `snapshot.rs`). Tylko przy potrzebie, której nie pokrywa 5.7; silnik domyślny zostaje szeregowy ([ADR-0008](adr/0008-transakcje-undo-log-wykonanie-szeregowe.md), [ADR-0053](adr/0053-galezie-bazy-zamiast-mvcc.md)).

- **Weryfikacja przepustek JWT przez serwer synchronizacji** – dla aplikacji bez backendu (aplikacje osobiste, gry indie), w których nie ma kto postawić bramki z [ADR-0025](adr/0025-serwer-synchronizacji-za-bramka-backendu.md). Serwer sprawdza podpis przepustki kluczem publicznym wydawcy (np. usługi logowania) i bierze przestrzeń z jej treści (`sub`); w serwerze to drugie źródło tego samego identyfikatora przestrzeni, więc nie wymaga przebudowy. Wymaga publicznego wystawienia serwera (HTTPS, CORS, limity żądań).

## Poza zakresem (decyzja autora)

Zadania usunięte z planu ([ADR-0030](adr/0030-zlaczenia-postgresql-i-edycja-na-zywo-poza-zakresem.md)). Numery usuniętych zadań nie są używane ponownie, żeby odnośniki w innych dokumentach pozostały jednoznaczne.

| Dawne zadanie | Dlaczego poza zakresem |
|---|---|
| Złączenia tylko do odczytu po kluczu głównym (dawne 5.2) | Flat-SQL pozostaje dialektem jednotabelowym – to część definicji języka ([flat-sql.md](flat-sql.md), [ADR-0004](adr/0004-dialekt-flat-sql-bez-zlaczen.md)); dane z kilku tabel składa aplikacja. Krytykę modelu bez złączeń [M1], [D4] omawia [literature-comparison.md §5](references/literature-comparison.md#5-model-danych-flat-sql-bez-join). |
| Współdzielona treść edytowana na żywo przez bibliotekę CRDT (dawne 6.3) | wspólna edycja dokumentów na żywo nie jest docelowym zastosowaniem ([real-world-usage.md §4](research/real-world-usage.md#4-zastosowania-flat-sql-potwierdzone-analizą)); takie edytory opierają się na CRDT tekstu (Yjs, Automerge) |
| Integracja z PostgreSQL – eksport do analityki, łącznik CDC, mapowanie typów, konflikty między źródłami (dawna sekcja 11) | serwer Flat-SQL jest własną bazą główną; synchronizacja z istniejącą bazą serwerową to zakres osobnych produktów (PowerSync, ElectricSQL [Q11]); paczki można trzymać w bazie SQL backendu przez adapter magazynu ([sync-server.md §6](future/components/sync-server.md#6-dane-na-serwerze)) |
