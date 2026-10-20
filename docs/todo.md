# Lista zadań – optymalizacja prototypu Flat-SQL

Zadania ulepszające to, co prototyp **już ma** – silnik, warstwę składowania, istniejący protokół synchronizacji, bridge, serwer-hub i benchmarki – wynikające z przeglądu literatury i z analizy rzeczywistych wdrożeń. Nowe możliwości (wiele kart, replikacja częściowa, reguły serwera, szyfrowanie, zapytania reaktywne, węzły edge, integracje) są w **[todo_future.md](todo_future.md)**.

Uzasadnienia: [references/conclusions.md](references/conclusions.md), szczegóły: [references/literature-comparison.md](references/literature-comparison.md), wdrożenia: [research/real-world-usage.md](research/real-world-usage.md). Identyfikatory w nawiasach (np. [C2]) odsyłają do [indeksu literatury](references/README.md).

Nakład: **mały** – pojedyncze funkcje i testy; **średni** – kilka modułów, nowe testy i pomiary; **duży** – przebudowa podsystemu.

**Kolejność dla pracy inżynierskiej:** sekcja 1 → 2.1–2.2 → 3.1–3.2 → 2.3–2.4 → 2.6 → sekcja 4 → reszta. Sekcja 0 może iść równolegle. Zakres warto uzgodnić z promotorem.

## 0. Doprecyzowanie dokumentacji prototypu

- [ ] **0.1 Tezy w README, dokumentacji i pracy** (mały):
  - „0 ms latencji” – brak opóźnienia sieci i zależności od serwera przy zapytaniu, nie zerowy czas wykonania ([ADR-0001](adr/0001-zapytania-lokalne-bez-serwera.md));
  - „Privacy by Design” – dane domyślnie lokalne, ale nieszyfrowane; izolacja WASM chroni hosta, nie dane modułu [V1], [L1], [W2];
  - „szybszy niż sql.js”, a nie „szybszy niż SQLite” – SQLite z OPFS nie był mierzony [B3];
  - atomowość: transakcja lokalnie, wiersz między replikami – ta sama gwarancja co w Simba [L27], zgodna z modelem encji [C7];
  - trwałość po `await flush()` – odporna na zamknięcie karty i awarię przeglądarki, nie na awarię systemu (zob. 1.4) [Q1];
  - „przewidywalne wykonanie” – plan zawsze jednotabelowy i widoczny w `EXPLAIN`/`rowsScanned`, ale ograniczony koszt mają tylko odczyty po kluczu lub indeksie z limitem; pełne skany i `GROUP BY` rosną z tabelą [M2];
  - „tryb edge” – obecny serwer to serwer synchronizacji z tym samym silnikiem (hub i baza główna naraz); węzły edge synchronizowane z bazą główną są planowane ([todo_future.md §7](todo_future.md)).
- [ ] **0.2 Zastosowania w README i prezentacji** (mały) – „CSV 500 MB” przypisać DuckDB-Wasm [D1]; przy portfelach kryptowalut wykluczyć przechowywanie kluczy [W2]; sesje na edge opisać jako ryzykowne (spójność ostateczna, [P1] rozdz. 9); dodać wzorce potwierdzone analizą wdrożeń – zbieranie danych w terenie, aplikacje osobiste na wielu urządzeniach, pamięć podręczna danych serwera ([real-world-usage.md §4](research/real-world-usage.md#4-zastosowania-flat-sql-potwierdzone-analizą)).
- [ ] **0.3 Klucze dla wstawień offline** (mały) – w [flat-sql.md](flat-sql.md) zalecić klucze bez kolizji (UUID/ULID lub prefiks identyfikatora węzła) dla tabel zapisywanych offline; unikalność klucza głównego między replikami wymaga koordynacji ([P1] rozdz. 5 i 9), a Coda rozwiązywała to tymczasowymi identyfikatorami zastępowanymi przy reintegracji [C10].
- [ ] **0.4 Odnośniki w README** (mały) – sekcja „Dokumentacja”: `docs/references/`, obie listy zadań, analiza wdrożeń.

## 1. Poprawność trwałości i synchronizacji (priorytet 1)

- [ ] **1.1 Ograniczenie dryfu zegara HLC** (mały) – stała `MAX_DRIFT` (np. 60 s); `Hlc::observe` nie wchłania znaczników spoza granicy, `apply_changes` liczy takie zmiany jako odrzucone, serwer zwraca je w odpowiedzi `/sync/push`, klient sygnalizuje problem z zegarem. Bez tego urządzenie z zegarem ustawionym w przyszłość wygrywa wszystkie konflikty, a jego czas rozlewa się na wszystkie repliki. Pliki: `crates/flatsql-core/src/sync.rs`, `db.rs`, `server/server.js`. [C2] §4, [P1] rozdz. 8.
- [ ] **1.2 Kolejność utrwalania przy checkpoincie** (mały) – po `rename` snapshotu `fsync` katalogu (Windows: odpowiednik write-through), dopiero potem obcięcie WAL; alternatywnie nowy plik WAL zamiast obcinania starego. Plik: `crates/flatsql-core/src/storage/file.rs`. [S8].
- [ ] **1.3 `fsync` w `FsStore`** (mały) – utrwalanie WAL, snapshotu i `meta.json` z opcją wyłączenia; poprawić tabelę backendów w [architecture.md §4.2](architecture.md), która dziś przypisuje mu „opcjonalnie `fsync`”. Plik: `bridge/stores/fs.js`. [S8].
- [ ] **1.4 Trwałość IndexedDB** (mały) – `IndexedDbStore` już przyjmuje `durability`; udostępnić opcję w `FlatDB.open`, opisać w [architecture.md §4.3](architecture.md) domyślny tryb `relaxed` (Chrome 121+) i tryb `strict`; koszt zmierzyć w 4.9. [Q1].
- [ ] **1.5 Trwały magazyn przeglądarki** (mały) – `navigator.storage.persist()` przy otwarciu bazy, stan widoczny w demo; bez tego niewysłany outbox może zostać usunięty przez przeglądarkę (Safari po 7 dniach bez interakcji). Wdrożenia kas i gier dodają z tego powodu kopię na serwerze ([real-world-usage.md §3](research/real-world-usage.md#3-wnioski-projektowe-z-cudzych-wdrożeń)). Pliki: `bridge/flatdb.js`, `web/app.js`. [Q2]–[Q4], [L1].
- [ ] **1.6 Test protokołu checkpointu między plikami** (średni) – symulacja stanów pośrednich w stylu ALICE (rename utracony, obcięcie WAL utrwalone itp.); test „każdy prefiks WAL” obejmuje tylko jeden plik. [S8].
- [ ] **1.7 Jawne konflikty schematu** (mały) – `CREATE TABLE` z innej repliki o tej samej nazwie i innej definicji zgłaszany jako konflikt zamiast cichego odrzucania wszystkich jej wierszy. Plik: `crates/flatsql-core/src/db.rs`. [L26].
- [ ] **1.8 Kanonizacja NaN przy zapisie** (mały) – deterministyczne bajty snapshotu i paczek (warunek porównywania replik po skrócie). [P2] §2.5.
- [ ] **1.9 Testy synchronizacji w trudnych warunkach** (średni) – rozszerzyć `randomized_offline_sessions_converge` o rozjechane zegary (w tym urządzenie z przyszłości – 1.1), podziały sieci i powtórzone rundy; sprawdzać zbieżność i brak „wskrzeszeń” usuniętych wierszy. [C2], [L2].

## 2. Wydajność silnika i granicy JS↔WASM

- [ ] **2.1 `COUNT(*)` po indeksie bez czytania wierszy** (mały) – długość zbioru kluczy w indeksie; widoczne w `EXPLAIN`. Odwraca przegraną z SQLite (573 µs vs 100 µs). [D2], [P1] rozdz. 3.
- [ ] **2.2 Pomijanie zbędnego filtra** (mały) – nie sprawdzać ponownie warunku, który ścieżka dostępu już gwarantuje (np. `category = ?` przy lookupie po indeksie). Pliki: `crates/flatsql-core/src/plan.rs`, `exec.rs`.
- [ ] **2.3 Profil granicy JS↔WASM** (mały) – rozbić 9,7 µs odczytu po kluczu na przekazanie parametrów, wykonanie, serializację i dekodowanie (natywnie silnik potrzebuje 2,3 µs). Recenzowanych pomiarów tej granicy dla silników baz nie ma – to wkład własny (H3).
- [ ] **2.4 Binarny format wyników** (średni) – wynik w buforze w pamięci WASM czytany z JS bez pośrednich obiektów, parametry w tablicach typowanych; wzór: Arrow IPC w DuckDB-Wasm [D1]. Cel: odczyt po kluczu < 3,5 µs (Zeta-Lite deklaruje 3,2–3,7 µs [D5]).
- [ ] **2.5 Poziomy optymalizacji** (mały) – porównać `opt-level = 3 / 2 / "s" / "z"` i flagi `wasm-opt` (rozmiar modułu i czas); optymalizacje LLVM nie są dostrojone do WASM, a mniejszy moduł nie musi być wolniejszy. [W11].
- [ ] **2.6 Checkpoint bez przestoju** (średni) – dziś checkpoint to „naiwny snapshot”: synchroniczne kodowanie całej bazy (35 ms dla 3 MB w przeglądarce, 49 ms dla 100 tys. wierszy), czyli 2–3 klatki przerwy w grze. Kierunki z [S10]: kodowanie porcjami między zadaniami pętli zdarzeń z kopiowaniem wierszy zmienianych w trakcie (copy-on-update) albo przyrostowy snapshot tylko zmienionych wierszy. Bez wątków – zachowuje brak wymagań COOP/COEP. Metryka: najdłuższa pauza (4.10). Pliki: `crates/flatsql-core/src/db.rs`, `storage/snapshot.rs`, `bridge/flatdb.js`.
- [ ] **2.7 Zwarta reprezentacja wierszy** (duży) – wiersze w ciągłych blokach zamiast osobnych alokacji `BTreeMap`; mniej pamięci, szybszy snapshot i odtworzenie (dziś 200 ms dla 100 tys. wierszy, SQLite kopiuje plik). WASM zużywa więcej pamięci niż JS, a pamięć liniowa nie jest zwracana [W11]; przejście na memory64 kosztuje wydajność [Q6].

## 3. Semantyka istniejącej synchronizacji

- [ ] **3.1 LWW per kolumna** (średni) – znacznik dla kolumny przechowywany tylko wtedy, gdy różni się od znacznika wiersza; współbieżne zmiany różnych kolumn jednego wiersza przestają ginąć. Schemat ze znacznikami gubi aktualizacje [C9], CRR stosuje rejestr LWW per atrybut [L24]. Protokół się nie zmienia (paczka niesie wiersz z wektorem znaczników); koszt rozmiaru zmierzyć w 4.11.
- [ ] **3.2 Trwały log synchronizacji serwera** (średni) – dziś log jest tylko w pamięci, a epoka losowana przy starcie, więc **każdy restart serwera** wysyła wszystkim klientom pełny stan bazy. Zapis logu i kursorów w katalogu danych, epoka zmieniana tylko po utracie logu. Wzorzec: trwały log zmian z offsetami konsumentów ([P1] rozdz. 11), trwały log reintegracji w Coda [C10]. Log prowadzony per przestrzeń – wspólny fundament obu trybów serwera ([ADR-0023](adr/0023-konfigurowalne-tryby-serwera-synchronizacji.md)). Plik: `server/server.js`.
- [ ] **3.3 Raport konfliktów klucza głównego** (średni) – wykrywanie dwóch wstawień tego samego klucza z różnych replik i zwracanie ich w raporcie `apply_changes` / `/sync/push` zamiast cichego nadpisania. CRR wykrywa takie naruszenia przy scalaniu [L24], Simba zgłasza konflikty aplikacji [L27].
- [ ] **3.4 Usuwanie tombstone'ów** (średni) – tombstone można usunąć po stabilności przyczynowej, gdy wszystkie repliki potwierdziły jego znacznik [L28]; przy serwerze centralnym wystarczy minimalny potwierdzony kursor aktywnych klientów, a klienci nieaktywni dłużej niż próg dostają pełny stan (już obsłużone). Wymaga 3.2.
- [ ] **3.5 Granice transakcji w paczkach** (mały–średni) – najpierw udokumentować gwarancję „między replikami atomowy jest wiersz” (0.1) – Simba świadomie poprzestaje na niej, uznając atomową synchronizację wielu wierszy za zbędną dla większości aplikacji [L27]. Opcjonalnie: outbox nie dzieli transakcji między paczki (Monotonic Atomic View, osiągalne bez koordynacji [C8]) – dopiero gdy demo pokaże potrzebę (np. wymiana przedmiotów między graczami). Pliki: `crates/flatsql-core/src/db.rs`, `bridge/sync.js`.

## 4. Badania i benchmarki

- [ ] **4.1 Nowe punkty odniesienia** (średni) – SQLite WASM z OPFS (`opfs-sahpool`, w Workerze), PGlite i Zeta-Lite w `bench/engines.js`; bez SQLite na OPFS teza o trwałości dotyczy tylko sql.js. [Q7], [D5], [B3].
- [ ] **4.2 Statystyka** (mały) – ≥ 10 uruchomień procesu, przedziały ufności lub mediana z IQR. [B1].
- [ ] **4.3 Zimny start** (mały) – pobranie, kompilacja, otwarcie bazy, pierwsze zapytanie – osobna tabela; rozmiar modułu ma znaczenie przy starcie strony (Notion ładuje SQLite asynchronicznie). [B3].
- [ ] **4.4 Zużycie pamięci** (mały) – pamięć liniowa WASM na 100 tys. wierszy wobec sql.js i `Map`. [W11].
- [ ] **4.5 Realistyczne obciążenie** (średni) – YCSB z rozkładem Zipfa (np. 95% odczytów) i scenariusz z aplikacji demo. [B5].
- [ ] **4.6 Kontrola wyników** (mały) – suma kontrolna wyników zamiast samej liczby wierszy. [B3].
- [ ] **4.7 Pomiar na telefonie** (mały) – wszystkie pomiary H1–H5 na urządzeniu mobilnym; osobno czas otwarcia bazy z IndexedDB, bo na wolnych urządzeniach lokalny odczyt bywa wolniejszy od sieci ([real-world-usage.md §3](research/real-world-usage.md#3-wnioski-projektowe-z-cudzych-wdrożeń)).
- [ ] **4.8 Dokładniejszy zegar w przeglądarce** (mały) – strona benchmarku z nagłówkami COOP/COEP (5 µs zamiast 100 µs); dotyczy tylko strony pomiarowej, nie biblioteki. [Q5].
- [ ] **4.9 Koszt trwałości `strict`** (mały) – dodać do H5 (zob. 1.4). [Q1].
- [ ] **4.10 Pauza checkpointu** (mały) – najdłuższa przerwa wątku głównego podczas checkpointu przed i po 2.6, na tle budżetu 16 ms na klatkę [D4], [S10].
- [ ] **4.11 Koszt LWW per kolumna** (mały) – rozmiar snapshotu, paczek i czas insertów przed i po 3.1.
- [ ] **4.12 Koszt trybu Worker** (mały) – czas przesłania zapytania i wyniku do Web Workera i z powrotem (`postMessage`) dla odczytu po kluczu i dla 10 000 wierszy, na tle 9,7 µs w wątku głównym; rozstrzyga, od jakiej skali tryb Worker się opłaca ([ADR-0013](adr/0013-watek-glowny-domyslnie-worker-jako-opcja.md)).

## 5. Zadania usunięte po weryfikacji z literaturą

| Dawne zadanie | Dlaczego usunięte | Gdzie teraz |
|---|---|---|
| JS String Builtins (dawne 2.5) | mechanizm dla języków używających łańcuchów JS jako własnych (GC); Rust trzyma tekst w pamięci liniowej i kopiowania nie uniknie [Q13] | zastąpione przez 2.4 |
| Równoległe odtwarzanie (SiloR) | wymaga wątków WASM, czyli `SharedArrayBuffer` i nagłówków COOP/COEP – sprzeczne z atutem „zero konfiguracji” | 2.6, 2.7 |
| Snapshot jako obraz pamięci modułu (dawne 4.8) | obraz pamięci zależy od wersji modułu i układu wskaźników, więc łamie wspólny format plików Rust/WASM/Node | 2.7 |
| Formalna specyfikacja reguł scalania (część dawnego 3.9) | poza zakresem pracy inżynierskiej; zbieżność LWW wynika z teorii CRDT [L2], [L28] | 1.9 – testy losowe |
| Uzgadnianie zbiorów po utracie kursora (dawne 3.7) | przyczyną pełnych pobrań jest log serwera w pamięci – najpierw 3.2 | [todo_future.md 3.7](todo_future.md), warunkowo |
| Decyzja o wykluczeniu `docs/references/papers/` | katalog jest już wykluczony lokalnie (`.git/info/exclude`) | – |
| Nowe możliwości z dawnych sekcji 4, 7, 8 i część sekcji 3 | to rozszerzenia, nie optymalizacja istniejącego prototypu | [todo_future.md](todo_future.md) |
| Zadania literaturowe (dawna sekcja 6) | dotyczą pracy dyplomowej, nie prototypu | [search-methodology.md §6](references/search-methodology.md#6-jak-uzupełnić-przegląd) |
