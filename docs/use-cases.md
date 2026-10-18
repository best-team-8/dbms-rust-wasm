# Zastosowania Flat-SQL

Katalog zastosowań wynikający z analizy prototypu i przeglądu literatury (stan na 18 października 2026). Pierwotne zastosowania: [prezentacja projektu](ProductPresentation.pdf). Uzasadnienia i źródła: [references/conclusions.md](references/conclusions.md). Zadania: [todo.md](todo.md) i [todo_future.md](todo_future.md). Identyfikatory w nawiasach (np. [L24]) odsyłają do [indeksu literatury](references/README.md).

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

## 2. Mocne strony i wynikające z nich zastosowania

| Przewaga | Liczba (pomiary projektu) | Co z niej wynika |
|---|---|---|
| Szybki odczyt | ok. 10 µs (sql.js: 20 µs) | interfejs nie czeka na dane |
| Tani trwały zapis | 0,34 ms; group commit 5,6 µs na zatwierdzenie | zapis przy każdej zmianie, także w każdej klatce gry |
| Mały moduł | 206 KiB gzip (PGlite, DuckDB-Wasm, Zeta-Lite: 2,5–3 MB) | szybkie pierwsze uruchomienie, także na telefonie |
| Zero konfiguracji | wątek główny, bez Web Workera i nagłówków COOP/COEP | osadzanie wszędzie, np. w rozszerzeniu przeglądarki |
| Wbudowana synchronizacja | trwały outbox przetrwa zamknięcie aplikacji offline | aplikacje dla miejsc ze słabym internetem |
| Ten sam silnik wszędzie | przeglądarka, serwer, natywnie | edge, CLI, panel administracyjny, backendy |
| Szybkie grupowanie | `GROUP BY` ok. 5× szybciej niż sql.js | podsumowania i statystyki lokalne |

## 3. Ocena zastosowań z prezentacji

| Zastosowanie | Ocena | Uzasadnienie i warunki |
|---|---|---|
| Gry przeglądarkowe (ekwipunek, statystyki) | bardzo dobre | szybkie odczyty i tani zapis; liczniki na wielu urządzeniach wymagają scalania sumującego (`MERGE ADD`); w grach wieloosobowych serwer musi być autorytetem (zob. §5) |
| Feature flags / testy A/B | bardzo dobre | małe dane zmieniane przez jednego autora, LWW wystarcza; potrzebne węzły edge zsynchronizowane z bazą główną (§7) |
| Geolokalizacja i personalizacja | bardzo dobre | tabela tylko do odczytu, odczyt po kluczu w µs; jak wyżej |
| Edytory graficzne – katalog assetów | dobre | metadane, filtrowanie po indeksach, praca offline, synchronizacja; pliki binarne poza tabelą (OPFS/IndexedDB); relacje wiele-do-wielu (tagi) trzeba modelować bez JOIN |
| Agregacja metryk i telemetria | dobre, z warunkiem | paczkowanie zdarzeń to wzorzec outboksu i group commitu [S1]; wymaga węzłów edge przekazujących paczki do bazy głównej (§7) |
| Portfele kryptowalut | częściowe | historia transakcji – tak; klucze prywatne – nigdy w bazie (§6) |
| Sesje i autoryzacja na edge | ryzykowne | spójność ostateczna: unieważniona sesja może być jeszcze ważna na innym węźle (§7) |
| Analityka CSV 500 MB | słabe | dane w pamięci w reprezentacji wierszowej, limit 4 GiB `wasm32`; właściwy jest DuckDB-Wasm [D1] (§12) |

## 4. Zastosowania pasujące już dziś

- **Aplikacje terenowe offline** – inspekcje techniczne, inwentaryzacja, serwis, ankiety, dostawy; trwały outbox przetrwa zamknięcie aplikacji bez sieci.
- **Aplikacje osobiste na wielu urządzeniach** – notatki, zadania, nawyki, dziennik treningów; szybki start, praca offline, synchronizacja.
- **Autozapis i szkice** – formularze wieloetapowe, edytory; zapis przy każdej zmianie bez spowalniania interfejsu.
- **Lokalna kopia danych referencyjnych** – katalog produktów, cenniki, słowniki, rozkłady jazdy; głównie odczyty po kluczu, przyrostowa synchronizacja.
- **Rozszerzenia przeglądarki i widżety osadzane na cudzych stronach** – działa w wątku głównym bez Workera i nagłówków, a SQLite z trwałością na OPFS wymaga Web Workera.
- **Edukacja przy słabym internecie** – materiały i postępy offline, synchronizacja po powrocie sieci.
- **Bufor telemetrii po stronie klienta** – zdarzenia zbierane offline i wysyłane paczkami.
- **Narzędzia administracyjne** – wspólny format plików pozwala otworzyć bazę serwera w CLI, a w przyszłości w panelu WinUI.

## 5. Gry przeglądarkowe

**Nadaje się:**

- **gry jednoosobowe z zapisem w chmurze** – gry „idle”, RPG, roguelike, strategie: postęp zapisywany lokalnie przy każdej akcji, gra działa offline, ten sam zapis na telefonie i komputerze;
- **zabezpieczenie postępu** – kopia na serwerze chroni przed usunięciem danych przez przeglądarkę (Safari po 7 dniach bez interakcji – [Q2]);
- **gry turowe i asynchroniczne** – ruch raz na jakiś czas (np. szachy korespondencyjne); local-first sprawdza się przy współpracy asynchronicznej [L22];
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

## 6. Portfele kryptowalut

W portfelu są dwie różne rzeczy: **klucze** (czyli środki) i **dane** (historia, saldo, książka adresowa). Flat-SQL nadaje się do danych, **nigdy do kluczy**: dane nie są szyfrowane, IndexedDB może odczytać każdy skrypt strony (np. przy XSS lub złośliwej bibliotece), izolacja WebAssembly nie chroni danych wewnątrz modułu ([W2], [W9]), a przeglądarka może dane usunąć – utrata klucza to utrata środków.

Przepis:

1. **Klucze poza bazą** – portfel sprzętowy albo zaszyfrowany sejf (hasło → klucz przez PBKDF2 → AES-GCM w WebCrypto), odszyfrowywany tylko na czas podpisu. WebCrypto nie obsługuje krzywej secp256k1 (Bitcoin, Ethereum), więc podpis wykonuje osobna biblioteka, a WebCrypto szyfruje sejf.
2. **Blockchain jako źródło prawdy, Flat-SQL jako lokalny indeks** – tabela `tx` z kluczem = hash transakcji (niezmienny, więc bez konfliktów), indeksy po tokenie, dacie i adresie, podsumowania portfela przez `GROUP BY`.
3. **Transakcje oczekujące** – status „oczekująca → potwierdzona/odrzucona”, jak zapisy wstępne w Bayou [C3].
4. **Synchronizacja tylko metadanych użytkownika** (etykiety adresów, notatki, książka adresowa, ustawienia) – z szyfrowaniem end-to-end, bo etykiety ujawniają finanse; historię zawsze można pobrać ponownie z blockchaina.
5. **Szyfrowanie lokalne i `navigator.storage.persist()`** ([todo.md 4.3 i 1.5](todo.md)).
6. **„Weryfikacja na edge” nie jest dowodem** – wymaga zaufania do węzła; dane z węzła RPC warto sprawdzać w kilku źródłach.
7. **Forma: rozszerzenie przeglądarki** – tu mały moduł działający bez Workera i nagłówków to realna przewaga.

Proponowane sformułowanie: *„Portfele kryptowalut: szybki lokalny indeks historii transakcji i prywatne metadane synchronizowane end-to-end. Klucze w portfelu sprzętowym lub zaszyfrowanym sejfie, blockchain jako źródło prawdy.”*

## 7. Węzły edge zsynchronizowane z bazą główną

Założenie z prezentacji: baza działa na serwerach brzegowych blisko użytkowników i synchronizuje się z bazą główną.

```text
przeglądarki ─► węzeł edge (np. Warszawa) ─┐
przeglądarki ─► węzeł edge (np. Berlin)  ──┴─► baza główna
```

**Stan prototypu:** jest jeden serwer ([server/server.js](../server/server.js)), który jest jednocześnie hubem synchronizacji, API do porównań w H1 i bazą główną; węzłów edge zsynchronizowanych z bazą główną jeszcze nie ma ([todo.md 4.10–4.11](todo.md)). W literaturze to replikacja multi-leader w topologii drzewa ([P1] rozdz. 5) i scenariusz CRR [L24]; węzeł edge może obsługiwać użytkowników przy niedostępnej centrali [E2].

| Scenariusz | Ocena | Uwagi |
|---|---|---|
| Feature flags, testy A/B | bardzo dobre | zmienia centrala, węzły tylko czytają |
| Personalizacja wg regionu | bardzo dobre | jak wyżej |
| Telemetria | bardzo dobre | węzeł zbiera zdarzenia i wysyła paczkami w górę |
| Sesje i autoryzacja | ryzykowne | unieważnienie dociera z opóźnieniem; bezpieczniej: krótki czas życia sesji, natychmiastowe rozsyłanie unieważnień, węzeł jako replika tylko do odczytu |

## 8. Flat-SQL obok backendu (np. FastAPI w Pythonie)

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
| B. Silnik w procesie backendu | rdzeń jako moduł Pythona przez interfejs C, synchronizacja w FastAPI | wymaga wiązań ([todo_future.md §4](todo_future.md)) |

Już działa: nowy klient bez kursora dostaje pełny stan bazy, a potem `SyncClient` synchronizuje zmiany w obie strony; zapisy backendu przez `/api/query` docierają do klientów.

**Jak dzielić dane między użytkowników:**

| Sposób | Kiedy | Plusy | Minusy |
|---|---|---|---|
| Jedna baza główna + reguły, kto dostaje które wiersze (domyślnie) | dane wspólne: rankingi, rynek, zespoły | zapytania przekrojowe, jedno źródło prawdy | wymaga replikacji częściowej i reguł ([todo_future.md 1.2](todo_future.md)) |
| Osobna baza na użytkownika | dane niezależne: zapisy gry, notatki | izolacja, łatwe usunięcie konta i eksport danych | brak zapytań między użytkownikami (do tego osobna baza, np. PostgreSQL) |

Baza Flat-SQL to struktura w pamięci i dwa pliki, a nie osobny serwer ani proces. Szacunek: ekwipunek 200 wierszy to kilkadziesiąt KB, 100 tys. graczy – ok. 2 GB na dysku, a w pamięci tylko aktywni użytkownicy. Model „baza na użytkownika” stosują m.in. Cloudflare Durable Objects, Turso i Apple iCloud (praca o FoundationDB Record Layer, SIGMOD 2019 – do doczytania).

Czego brakuje do realnego użycia: rozdzielenia danych użytkowników, uwierzytelniania, reguł po stronie serwera, obrazu Dockera ([todo_future.md](todo_future.md)). `/api/query` nie wolno wystawiać publicznie bez logowania.

## 9. Synchronizacja z innymi bazami (PostgreSQL, SQLite)

| Sytuacja | Czy potrzebna synchronizacja z inną bazą | Jak |
|---|---|---|
| Zapis w chmurze dla jednego użytkownika, aplikacje osobiste, aplikacje terenowe z własnym serwerem | nie – baza główna może być Flat-SQL | obecny protokół synchronizacji |
| Dane wspólne (rankingi, rynek), analityka wszystkich użytkowników | tak – zwykle PostgreSQL jako baza główna | łącznik przechwytujący zmiany PostgreSQL (CDC, [P1] rozdz. 11) i zapisujący do niego zmiany klientów po walidacji ([todo_future.md §6](todo_future.md)) |
| Istniejący backend z PostgreSQL | tak | jak wyżej; to dokładnie zakres PowerSync i ElectricSQL [Q11] |
| Aplikacje natywne (mobilne, desktopowe) używające SQLite | zależy | albo łącznik SQLite ↔ Flat-SQL, albo natywny Flat-SQL zamiast SQLite przez interfejs C ([todo_future.md §4](todo_future.md)) – wtedy synchronizacja działa bez tłumaczenia |
| Serwer lub edge na SQLite (np. Durable Objects, Turso) | zależy | backend `Storage` dla magazynu platformy albo łącznik jak dla PostgreSQL |

## 10. Lokalna pamięć asystentów i agentów programowych

Trend opisany m.in. w pracy o Zeta-Lite [D5] (2026): lokalna baza w przeglądarce jako trwała pamięć asystentów – ustawienia, fakty, notatki, stan zadań i wyniki narzędzi. Asystent działa zwykle na serwerze dostawcy, a aplikacja przekazuje mu tylko potrzebne fragmenty danych.

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
| Rozmiar modułu | 206 KiB | 2,87 MB |
| Rozgałęzianie bazy | tylko `ROLLBACK` | jest |

Możliwa unikalna propozycja: **pamięć trzymana na urządzeniu, trwała przy każdym zapisie, synchronizowana między urządzeniami z szyfrowaniem end-to-end (serwer jej nie widzi), w module o rozmiarze ok. 200 KiB.** Wymaga: typu wektorowego i wyszukiwania podobieństwa, prostego wyszukiwania pełnotekstowego, szyfrowania end-to-end, `navigator.storage.persist()`, zapytań reaktywnych i punktów przywracania. Zasada bezpieczeństwa: zapisywać źródło każdego wpisu i dać użytkownikowi podgląd oraz usuwanie, bo treść z obcej strony zapisana jako wspomnienie może później wpływać na działanie asystenta (przejrzystość – [V1]).

## 11. Zastosowania odblokowane przez planowane funkcje

| Funkcja ([todo.md](todo.md)) | Nowe zastosowania |
|---|---|
| LWW per kolumna i liczniki `MERGE ADD` (3.1, 3.2) | wspólne listy i tablice zadań, stany magazynowe liczone z wielu skanerów, wspólna skrzynia w grze, ekonomia gry na wielu urządzeniach |
| Zapytania reaktywne (4.1) | interfejsy i pulpity aktualizowane bez ręcznego odświeżania |
| Szyfrowanie end-to-end i lokalne (4.2, 4.3) | zdrowie, dzienniki, śledzenie wydatków, dokumenty firmowe – dopiero wtedy „Privacy by Design” jest w pełni prawdziwe |
| Złączenia tylko do odczytu (4.5) | proste CRM, katalogi z relacjami |
| Węzły edge z bazą główną (4.10, 4.11) | scenariusze edge z prezentacji w rzeczywistym układzie |
| Wyszukiwanie wektorowe i pełnotekstowe | pamięć asystentów (§10), wyszukiwanie w notatkach |
| Interfejs C i wiązania dla Pythona ([todo_future.md §4](todo_future.md)) | backendy w innych językach, aplikacje natywne zamiast SQLite |

## 12. Czego nie polecać i dlaczego

| Ograniczenie | Źródło | Dlaczego | Obejście |
|---|---|---|---|
| Duża analityka (np. CSV 500 MB) | implementacja i platforma (nie język – plik CSV to jedna tabela) | każdy wiersz to osobny obiekt w pamięci, przetwarzanie wiersz po wierszu zamiast kolumnami, limit 4 GiB `wasm32` | DuckDB-Wasm [D1]; tabele kolumnowe w Flat-SQL byłyby budową drugiego silnika |
| Płatności jako jedyne źródło prawdy | założenie Local-First (każdy system offline) | urządzenie offline nie wie, czy środki nie zostały wydane na innym urządzeniu; zapobieganie podwójnemu wydaniu wymaga koordynacji ([C8], [P1] rozdz. 9, [L22]) | serwer lub blockchain jako źródło prawdy, lokalne operacje jako wstępne do potwierdzenia [C3]; ewentualnie limit do wydania offline przydzielony z góry (metoda „escrow”, O'Neil 1986 – do doczytania) |
| Wspólna edycja tekstu na żywo | model danych (pole jako całość + LWW) i implementacja (pobieranie zmian co 5 s) | edytory współdzielone śledzą każdy znak osobno (CRDT tekstu, np. [L16]) | każda edycja jako osobny niezmienny wiersz, tekst składany biblioteką CRDT (np. Yjs) w aplikacji, powiadomienia push |
| Dane większe niż pamięć urządzenia | implementacja (decyzja „dane w pamięci”) i platforma | cała baza musi zmieścić się w pamięci modułu | magazyn stronicowany na OPFS z buforem stron (duży nakład); 64-bitowa pamięć WASM kosztuje wydajność [W6] |
| Rozgrywka w czasie rzeczywistym, rankingi i rynek bez serwera | założenie Local-First | dane wspólne i podatne na oszustwa | serwer gry i baza serwerowa (§5) |
| Przechowywanie kluczy prywatnych | bezpieczeństwo przeglądarki | brak szyfrowania, dostęp skryptów strony, możliwe usunięcie danych | portfel sprzętowy lub zaszyfrowany sejf (§6) |
