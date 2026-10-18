# Lista zadań – dojrzewanie projektu Flat-SQL

Zadania wynikające z analizy prototypu i przeglądu literatury. Uzasadnienia, pozycja projektu na tle innych rozwiązań i zastosowania: **[references/conclusions.md](references/conclusions.md)**. Szczegółowe porównanie z badaniami: [references/literature-comparison.md](references/literature-comparison.md). Identyfikatory w nawiasach (np. [C2]) odsyłają do [indeksu literatury](references/README.md).

Nakład: **mały** – pojedyncze funkcje i testy; **średni** – kilka modułów, nowe testy i pomiary; **duży** – nowy podsystem.

Zadania wykraczające poza obecny etap (Flat-SQL obok backendu, np. FastAPI: dane per użytkownik, reguły serwera, Docker, wiązania dla Pythona): [todo_future.md](todo_future.md). Zastosowania, którym służą te zadania: [use-cases.md](use-cases.md).

Proponowana kolejność dla pracy inżynierskiej: sekcje 1 → 2 → z sekcji 3 punkty LWW per kolumna, `MERGE ADD` i granice transakcji → zapytania reaktywne z sekcji 4 → sekcja 5. Jeśli scenariusze edge z prezentacji mają zostać pokazane na obronie, zadanie 4.10 (węzły edge synchronizowane z bazą główną) warto zrobić zaraz po sekcji 1 – bez niego pozostają zapowiedzią. Zakres warto uzgodnić z promotorem.

## 0. Porządki w repozytorium i dokumentacji

- [ ] Zdecydować, czy wykluczyć `docs/references/papers/` z repozytorium – część plików to wersje autorskie „nie do redystrybucji”, katalog ma ok. 94 MB.
- [ ] Dodać do README odnośniki do `docs/references/` i do tej listy (sekcja „Dokumentacja”).
- [ ] Zaktualizować zastosowania w README i prezentacji według [conclusions.md §5](references/conclusions.md#5-zastosowania): scenariusz „CSV 500 MB” przypisać DuckDB-Wasm [D1], przy portfelach kryptowalut wykluczyć przechowywanie kluczy, przy sesjach na edge opisać ryzyko spójności ostatecznej, dodać nowe zastosowania.
- [ ] Doprecyzować tezy w dokumentacji i pracy:
  - „Privacy by Design” – dane domyślnie lokalne, bez szyfrowania (do czasu zadań z sekcji 4);
  - „szybszy niż sql.js”, a nie „szybszy niż SQLite”;
  - transakcje atomowe lokalnie, między replikami – atomowa zmiana pojedynczego wiersza;
  - trwałość po `await flush()` – odporna na awarię karty i przeglądarki, nie systemu (zob. 1.4);
  - „tryb edge” – obecny serwer w [architecture.md §5.4](architecture.md) i README opisywać jako serwer synchronizacji z tym samym silnikiem (hub i baza główna naraz), a węzły brzegowe synchronizowane z bazą główną jako planowane (zob. 4.10).

## 1. Poprawność (priorytet 1)

- [ ] **1.1 Ograniczenie dryfu zegara HLC** (mały) – stała `MAX_DRIFT`; `Hlc::observe` nie przyjmuje znaczników z przyszłości, `apply_changes` liczy takie zmiany jako odrzucone, serwer zwraca je w odpowiedzi `/sync/push`; testy. Pliki: `crates/flatsql-core/src/sync.rs`, `crates/flatsql-core/src/db.rs`, `server/server.js`. [C2], [P1] rozdz. 8.
- [ ] **1.2 Kolejność utrwalania przy checkpoincie** (mały) – po `rename` snapshotu `fsync` katalogu (Windows: odpowiednik write-through), dopiero potem obcięcie WAL. Plik: `crates/flatsql-core/src/storage/file.rs`. [S8].
- [ ] **1.3 `fsync` w `FsStore`** (mały) – utrwalanie WAL, snapshotu i `meta.json`, z opcją wyłączenia; poprawić tabelę backendów w [architecture.md §4.2](architecture.md). Plik: `bridge/stores/fs.js`. [S8].
- [ ] **1.4 Trwałość IndexedDB** (mały) – opisać w [architecture.md §4.3](architecture.md) domyślny tryb `relaxed` (Chrome 121+) i opcję `durability: 'strict'`; zmierzyć jej koszt w H5. [Q1].
- [ ] **1.5 Trwały magazyn przeglądarki** (mały) – `navigator.storage.persist()` przy otwarciu bazy, stan widoczny w demo. Pliki: `bridge/flatdb.js`, `web/app.js`. [Q2]–[Q4], [L1].
- [ ] **1.6 Test protokołu checkpointu między plikami** (średni) – symulacja stanów pośrednich (rename utracony, obcięcie WAL utrwalone itp.) w stylu ALICE. [S8], [S9].
- [ ] **1.7 Konflikty schematu** (mały) – `CREATE TABLE` z innej repliki o innej definicji zgłaszany jawnie zamiast cichego odrzucania wierszy. Plik: `crates/flatsql-core/src/db.rs`. [L9], [L26].
- [ ] **1.8 Klucze dla wstawień offline** (mały) – w [flat-sql.md](flat-sql.md) zalecić klucze bezkolizyjne (UUID/ULID lub prefiks węzła). [P1] rozdz. 5 i 9.

## 2. Wydajność – szybkie wygrane

- [ ] **2.1 `COUNT(*)` po indeksie bez czytania wierszy** (mały) – długość zbioru kluczy w indeksie; widoczne w `EXPLAIN`. Odwraca przegraną z SQLite (573 µs vs 100 µs). [D2], [P1] rozdz. 3.
- [ ] **2.2 Pomijanie zbędnego filtra** (mały) – nie sprawdzać ponownie warunku, który ścieżka dostępu już gwarantuje. Plik: `crates/flatsql-core/src/plan.rs`, `exec.rs`.
- [ ] **2.3 Profil granicy JS↔WASM** (mały) – rozbić 9,7 µs odczytu po kluczu na parametry, wykonanie, serializację i dekodowanie; silnik natywnie potrzebuje 2,3 µs.
- [ ] **2.4 Binarny format wyników** (średni) – wynik w buforze w pamięci WASM czytany wprost z JS, parametry w tablicach typowanych; cel: odczyt po kluczu < 3 µs. [D1], [D5].
- [ ] **2.5 JS String Builtins** (średni) – tańsze przekazywanie tekstu w przeglądarkach z Wasm 3.0. [W10].
- [ ] **2.6 Poziomy optymalizacji** (mały) – porównać `opt-level = 3 / 2 / "s" / "z"` i flagi `wasm-opt` (rozmiar i czas). [W11].
- [ ] **2.7 `CASE` w wyrażeniach** (mały) – najczęściej brakująca konstrukcja w obrębie jednej tabeli.

## 3. Synchronizacja (priorytet 2)

- [ ] **3.1 LWW per kolumna** (średni) – znacznik dla kolumny tylko wtedy, gdy różni się od znacznika wiersza; współbieżne zmiany różnych kolumn nie giną. [L24], [L25], [X5], [Q10].
- [ ] **3.2 Scalanie deklarowane w SQL** (średni) – np. `qty INT MERGE ADD` (licznik sumowany zamiast nadpisywany). [L8], [L2], [L24].
- [ ] **3.3 Granice transakcji w paczkach** (mały–średni) – outbox nie dzieli transakcji między paczki (Monotonic Atomic View). Pliki: `crates/flatsql-core/src/db.rs`, `bridge/sync.js`. [C8].
- [ ] **3.4 Raport konfliktów klucza głównego** (średni) – wykrywanie dwóch wstawień tego samego klucza i udostępnienie konfliktu aplikacji. [L24].
- [ ] **3.5 Wersjonowanie protokołu i schematu** (średni) – wersja formatu paczek, wersje schematu tabel, migracje. [P1] rozdz. 4, [L26], [L9].
- [ ] **3.6 Powiadomienia push** (średni) – WebSocket lub SSE zamiast samego odpytywania. [P1] rozdz. 12.
- [ ] **3.7 Uzgadnianie bez pełnego stanu** (duży) – po utracie kursora zamiast `exportState`. [L14].
- [ ] **3.8 Usuwanie tombstone'ów** (średni) – po potwierdzeniu przez wszystkich aktywnych klientów albo licznik „długości przyczynowej”. [L7], [L24].
- [ ] **3.9 Testy synchronizacji** (średni) – losowe sesje z rozjechanymi zegarami i podziałami sieci; opcjonalnie formalna specyfikacja reguł scalania. [L4], [L19].

## 4. Nowe możliwości

- [ ] **4.1 Zapytania reaktywne** (średni) – subskrypcja zapytania jednotabelowego; po zatwierdzeniu sprawdzane tylko zmienione klucze, agregaty aktualizowane przyrostowo. [D4].
- [ ] **4.2 Szyfrowanie end-to-end** (duży) – paczki zmian szyfrowane kluczem użytkownika; serwer jako przekaźnik nieprzezroczystych zmian. [L11], [L1].
- [ ] **4.3 Szyfrowanie danych lokalnych** (średni) – WAL i snapshot szyfrowane przez WebCrypto. [V1].
- [ ] **4.4 Uwierzytelnianie i autoryzacja** (średni) – klienci serwera synchronizacji. [L18], [L12].
- [ ] **4.5 Złączenia tylko do odczytu** (duży) – równościowe, po kluczu lub indeksie, bez `FOREIGN KEY`. [M1], [D4], [L8].
- [ ] **4.6 Wiele kart** (średni) – jeden silnik we wspólnym `SharedWorker` lub wybór lidera. [architecture.md §4.4](architecture.md).
- [ ] **4.7 Zwarta reprezentacja wierszy** (duży) – mniej pamięci i szybszy snapshot/odtworzenie. [W11], [S7], [S3].
- [ ] **4.8 Eksperyment: snapshot jako obraz pamięci modułu** (duży, ryzykowny) – odtwarzanie z szybkością kopiowania bajtów, ze zwykłym snapshotem jako zapasem przy zmianie wersji modułu. [E5].
- [ ] **4.9 Backend OPFS** (średni) – opcja dla większych baz (w Web Workerze). [Q7].
- [ ] **4.10 Węzły edge synchronizowane z bazą główną** (średni–duży) – założenie z prezentacji: przeglądarki ↔ węzły edge blisko użytkowników ↔ baza główna; dziś jest jeden serwer, który jest hubem i bazą główną naraz ([conclusions.md §3.6](references/conclusions.md#36-edge-computing)). Pliki: `server/server.js`, `test/`, [architecture.md §5.4](architecture.md). [P1] rozdz. 5, [L24], [L2], [E2], [C3].
  - [ ] opcja `--upstream <url>`: węzeł edge zachowuje własną replikę i log dla swoich klientów, a z bazą główną rozmawia tym samym protokołem `/sync/push` i `/sync/pull`;
  - [ ] przekazywanie w górę zmian przyjętych od klientów i zapisów z `/api/query` (dziś zmiany od klientów celowo nie trafiają do outboksu serwera), z kursorem przesuwanym dopiero po odpowiedzi 200;
  - [ ] pobieranie zmian z bazy głównej z kursorem i epoką, dopisywanie ich do logu węzła dla jego klientów bez odsyłania z powrotem w górę; obsługa pełnego stanu przy nieznanym kursorze;
  - [ ] trwałe kursory i log (dziś log jest tylko w pamięci, a restart oznacza nową epokę i pełne pobranie stanu przez klientów);
  - [ ] praca węzła przy niedostępnej bazie głównej (obsługa klientów, ponawianie z wykładniczym odstępem);
  - [ ] uwierzytelnianie między węzłem edge a bazą główną (zob. 4.4);
  - [ ] testy end-to-end: 2 klientów ↔ 2 węzły edge ↔ baza główna, odcięcie węzła od centrali, konflikt edycji tego samego wiersza przez klientów różnych węzłów, restart węzła – identyczny stan wszędzie;
  - [ ] pomiar latencji klient → węzeł edge vs klient → baza główna z symulowanym RTT (rozszerzenie H1), który da liczby scenariuszom edge z prezentacji.
- [ ] **4.11 Wdrożenie na platformie edge** (średni) – najprościej: serwer Node.js z 4.10 uruchomiony w kilku regionach; docelowo platforma typu Cloudflare Workers, gdzie jednowątkowy obiekt ze stanem i trwałym magazynem (np. Durable Object) odpowiada modelowi Flat-SQL – potrzebny backend `Storage` dla magazynu platformy i synchronizacja wyzwalana żądaniami lub harmonogramem. [E3]–[E8].
- [ ] **4.12 Kanonizacja NaN przy zapisie** (mały) – deterministyczne bajty snapshotu i paczek. [P2].

## 5. Benchmarki i dowody

- [ ] **5.1 Nowe punkty odniesienia** (średni) – SQLite WASM z OPFS (`opfs-sahpool`), PGlite, Zeta-Lite w `bench/engines.js`. [Q7], [D5], [B3].
- [ ] **5.2 Statystyka** (mały) – ≥ 10 uruchomień procesu, przedziały ufności lub mediana z IQR. [B1], [B2].
- [ ] **5.3 Zimny start** (mały) – pobranie, kompilacja, otwarcie bazy, pierwsze zapytanie – osobna tabela. [B3].
- [ ] **5.4 Zużycie pamięci** (mały). [W11].
- [ ] **5.5 Realistyczne obciążenie** (średni) – YCSB z rozkładem Zipfa (np. 95% odczytów) i scenariusz z aplikacji demo. [B5], [W7], [B4].
- [ ] **5.6 Kontrola wyników** (mały) – suma kontrolna wyników zamiast samej liczby wierszy. [B3].
- [ ] **5.7 Pomiar na telefonie** (mały).
- [ ] **5.8 Dokładniejszy zegar w przeglądarce** (mały) – strona benchmarku z nagłówkami COOP/COEP (5 µs zamiast 100 µs). [Q5], [B6].
- [ ] **5.9 Koszt trwałości `strict`** (mały) – dodać do H5 (zob. 1.4).

## 6. Literatura i praca dyplomowa

- [ ] Przeszukać Scopus, Web of Science, IEEE Xplore i ACM DL zapytaniami z [search-methodology.md §5](references/search-methodology.md#5-jak-uzupełnić-przegląd-biblioteka-uczelni) i zapisać liczby wyników na każdym etapie (jeśli promotor wymaga przeglądu systematycznego).
- [ ] Wypożyczyć DDIA (wydanie polskie: Helion 2018) i zweryfikować [notatkę](references/notes/ddia-kleppmann.md) z tekstem książki.
- [ ] Ustawić alerty Google Scholar („cytowane przez”) dla [P2], [W1], [L1], [D4].
- [ ] Zapytać autorów Teilen-SQL o tekst [X5]; sprawdzić publikację artykułu Niinikoski i Fagerholm [X6].
- [ ] Dokładnie przeczytać prace ocenione dotąd po streszczeniu – w obszarach wybranych do realizacji.
- [ ] W pracy cytować wersje wydawnicze (DOI), nie lokalne kopie.

## 7. Kolejny etap projektu

- [ ] Panel administracyjny WinUI 3 – komunikacja z silnikiem przez FFI (biblioteka `cdylib` z interfejsem C) lub named pipes (README, prezentacja projektu). Literatury dla tego etapu jeszcze nie zebrano ([search-methodology.md §3](references/search-methodology.md#3-ograniczenia)).
