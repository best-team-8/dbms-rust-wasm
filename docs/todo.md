# Lista zadań – optymalizacja prototypu Flat-SQL

Zakres pracy inżynierskiej ([ADR-0029](adr/0029-listy-zadan-wedlug-zakresu-pracy.md)): zadania, które poprawiają wynik Flat-SQL w metrykach jego niszy, czynią porównanie z innymi silnikami uczciwym i wiarygodnym albo realizują punkty karty pracy – warstwę lokalnego składowania, benchmarki i walidację Local-First w warunkach offline. Zadania bez dopisku są wymagane; dopisek „dobrze mieć” oznacza zadania wykonywane, jeśli starczy czasu. Wszystko inne – nowe możliwości i optymalizacje odłożone na później (§6) – jest w **[todo_future.md](todo_future.md)**.

Uzasadnienia: [references/conclusions.md](references/conclusions.md), szczegóły: [references/literature-comparison.md](references/literature-comparison.md), wdrożenia: [research/real-world-usage.md](research/real-world-usage.md). Identyfikatory w nawiasach (np. [C2]) odsyłają do [indeksu literatury](references/README.md).

Nakład: **mały** – pojedyncze funkcje i testy; **średni** – kilka modułów, nowe testy i pomiary; **duży** – przebudowa podsystemu.

**Kolejność:** sekcja 1 → 2.1, 2.2, 2.8 → 2.3–2.5 → sekcja 4 → zadania „dobrze mieć”. Sekcja 0 może iść równolegle. Zakres warto uzgodnić z promotorem.

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

## 1. Poprawność trwałości i synchronizacji (priorytet 1)

Każde zadanie kończy się testem zaprojektowanym i opisanym według [metodyki testów](research/tests-methodology.md) ([ADR-0032](adr/0032-metodyka-testow-poprawnosci.md)).

- [ ] **1.1 Ograniczenie dryfu zegara HLC** (mały) – stała `MAX_DRIFT` (np. 60 s); `Hlc::observe` nie wchłania znaczników spoza granicy, `apply_changes` liczy takie zmiany jako odrzucone, serwer zwraca je w odpowiedzi `/sync/push`, klient sygnalizuje problem z zegarem. Bez tego urządzenie z zegarem ustawionym w przyszłość wygrywa wszystkie konflikty, a jego czas rozlewa się na wszystkie repliki. Pliki: `crates/flatsql-core/src/sync.rs`, `db.rs`, `server/server.js`. [C2] §4, [P1] rozdz. 8.
- [x] **1.2 Kolejność utrwalania przy checkpoincie** (mały) – po `rename` snapshotu `fsync` katalogu (Windows: odpowiednik write-through), dopiero potem obcięcie WAL; alternatywnie nowy plik WAL zamiast obcinania starego. Plik: `crates/flatsql-core/src/storage/file.rs`. [S8]. Wykonane: `fsync` katalogu (w Windows uchwyt z prawem zapisu i `FILE_FLAG_BACKUP_SEMANTICS`), nowy plik WAL odrzucony ([ADR-0033](adr/0033-checkpoint-plikowy-fsync-katalogu.md), [tests-results.md §1.1–1.2](research/tests-results.md#1-trwałość-i-odtwarzanie-po-awarii)).
- [x] **1.3 `fsync` w `FsStore`** (mały) – utrwalanie WAL, snapshotu i `meta.json` z opcją wyłączenia; poprawić tabelę backendów w [architecture.md §4.2](architecture.md), która dziś przypisuje mu „opcjonalnie `fsync`”. Plik: `bridge/stores/fs.js`. [S8]. Wykonane: protokół jak w `FileStorage` ([ADR-0033](adr/0033-checkpoint-plikowy-fsync-katalogu.md)), `new FsStore(dir, { fsync: false })` wyłącza tylko synchronizację zatwierdzeń i `meta.json` ([tests-results.md §1.3](research/tests-results.md#13-utrwalanie-zapisów-w-magazynie-plikowym-nodejs)).
- [ ] **1.4 Trwałość IndexedDB** (mały) – `IndexedDbStore` już przyjmuje `durability`; udostępnić opcję w `FlatDB.open`, opisać w [architecture.md §4.3](architecture.md) domyślny tryb `relaxed` (Chrome 121+) i tryb `strict`; koszt zmierzyć w 4.9. [Q1].
- [ ] **1.5 Trwały magazyn przeglądarki** (mały) – `navigator.storage.persist()` przy otwarciu bazy, stan widoczny w demo; bez tego niewysłany outbox może zostać usunięty przez przeglądarkę (Safari po 7 dniach bez interakcji). Wdrożenia kas i gier dodają z tego powodu kopię na serwerze ([real-world-usage.md §3](research/real-world-usage.md#3-wnioski-projektowe-z-cudzych-wdrożeń)). Pliki: `bridge/flatdb.js`, `web/app.js`. [Q2]–[Q4], [L1].
- [x] **1.6 Test protokołu checkpointu między plikami** (średni) – symulacja stanów pośrednich w stylu ALICE (rename utracony, obcięcie WAL utrwalone itp.); test „każdy prefiks WAL” obejmuje tylko jeden plik. [S8]. Wykonane dla `FileStorage` i `FsStore`, bez zmian w kodzie produkcyjnym ([tests-results.md §1.4](research/tests-results.md#14-wszystkie-stany-po-awarii-dopuszczone-przez-model)).
- [ ] **1.7 Jawne konflikty schematu** (mały) – `CREATE TABLE` z innej repliki o tej samej nazwie i innej definicji zgłaszany jako konflikt zamiast cichego odrzucania wszystkich jej wierszy. Plik: `crates/flatsql-core/src/db.rs`. [L1].
- [x] **1.8 Kanonizacja NaN przy zapisie** (mały) – deterministyczne bajty snapshotu i paczek (warunek porównywania replik po skrócie). [P2] §2.5. Wykonane w kodeku (jeden wzorzec bitów dla WAL, snapshotu i paczek), `-0.0` bez zmian ([architecture.md §6](architecture.md#6-formaty-binarne), [tests-results.md §3.1](research/tests-results.md#31-ten-sam-zapis-każdego-nan)).
- [ ] **1.9 Testy synchronizacji w trudnych warunkach** (średni) – rozszerzyć `randomized_offline_sessions_converge` o rozjechane zegary (w tym urządzenie z przyszłości – 1.1), podziały sieci i powtórzone rundy; sprawdzać zbieżność i brak „wskrzeszeń” usuniętych wierszy. Minimum na pracę: urządzenie z zegarem w przyszłości, podział sieci i usunięcie wiersza w trakcie przerwy. [C2], [L2].
- [ ] **1.10 Domyślny magazyn w przeglądarce** (mały) – `FlatDB.open` bez opcji `store` używa `MemoryStore`, więc dane znikają po odświeżeniu strony, choć [ADR-0012](adr/0012-indexeddb-domyslnie-opfs-jako-opcja.md) zakłada IndexedDB jako magazyn domyślny; w przeglądarce domyślnie `IndexedDbStore`, w Node.js jawny wybór magazynu albo ostrzeżenie. Plik: `bridge/flatdb.js`.

## 2. Wydajność silnika i granicy JS↔WASM

- [ ] **2.1 `COUNT(*)` po indeksie bez czytania wierszy** (mały) – długość zbioru kluczy w indeksie; widoczne w `EXPLAIN`. Odwraca przegraną z SQLite (573 µs vs 100 µs). [D2], [P1] rozdz. 3.
- [ ] **2.2 Pomijanie zbędnego filtra** (mały) – nie sprawdzać ponownie warunku, który ścieżka dostępu już gwarantuje (np. `category = ?` przy lookupie po indeksie). Pliki: `crates/flatsql-core/src/plan.rs`, `exec.rs`.
- [ ] **2.3 Profil granicy JS↔WASM** (mały) – rozbić 9,7 µs odczytu po kluczu na przekazanie parametrów, wykonanie, serializację i dekodowanie (natywnie silnik potrzebuje 2,3 µs). Recenzowanych pomiarów tej granicy dla silników baz nie ma – to wkład własny (H3).
- [ ] **2.4 Binarny format wyników** (średni) – wynik w buforze w pamięci WASM czytany z JS bez pośrednich obiektów, parametry w tablicach typowanych; wzór: Arrow IPC w DuckDB-Wasm [D1]. Cel: odczyt po kluczu < 3,5 µs (Zeta-Lite deklaruje 3,2–3,7 µs [D5]).
- [ ] **2.5 Poziomy optymalizacji** (mały) – porównać `opt-level = 3 / 2 / "s" / "z"` i flagi `wasm-opt` (rozmiar modułu i czas); optymalizacje LLVM nie są dostrojone do WASM, a mniejszy moduł nie musi być wolniejszy. [W11].
- [ ] **2.6 Checkpoint bez przestoju** (średni, dobrze mieć) – dziś checkpoint to „naiwny snapshot”: synchroniczne kodowanie całej bazy (35 ms dla 3 MB w przeglądarce, 49 ms dla 100 tys. wierszy), czyli 2–3 klatki przerwy w grze. Kierunki z [S10]: kodowanie porcjami między zadaniami pętli zdarzeń z kopiowaniem wierszy zmienianych w trakcie (copy-on-update) albo przyrostowy snapshot tylko zmienionych wierszy. Bez wątków – zachowuje brak wymagań COOP/COEP. Metryka: najdłuższa pauza (4.10). Pliki: `crates/flatsql-core/src/db.rs`, `storage/snapshot.rs`, `bridge/flatdb.js`.
- [ ] **2.7 Zwarta reprezentacja wierszy** (duży, dobrze mieć) – wiersze w ciągłych blokach zamiast osobnych alokacji `BTreeMap`; mniej pamięci, szybszy snapshot i odtworzenie (dziś 200 ms dla 100 tys. wierszy, SQLite kopiuje plik). WASM zużywa więcej pamięci niż JS, a pamięć liniowa nie jest zwracana [W11]; przejście na memory64 kosztuje wydajność [W6], [Q6].
- [ ] **2.8 Szybszy `UPDATE`** (mały–średni) – dziś każdy zmieniany wiersz jest kopiowany w całości (także kolumny tekstowe), drugi raz wyszukiwany w drzewie przy zapisie, a jego klucz jest usuwany i wstawiany ponownie we wszystkich indeksach, także gdy kolumna indeksu się nie zmieniła; znacznik HLC odczytuje zegar JS (`Date.now()`) osobno dla każdego wiersza. Kierunki – hipotezy z analizy kodu, nie wyniki z literatury, do potwierdzenia pomiarem: pomijanie indeksów niezmienionych kolumn (tak robią dojrzałe silniki: SQLite aktualizuje tylko indeksy zmienionych kolumn, PostgreSQL ma HOT updates), zmiana wiersza w miejscu zamiast kopii i drugiego wyszukiwania, jeden odczyt zegara na instrukcję. Każdą zmianę mierzyć osobno, przed i po, razem z profilem z 2.3. Koszt logowania (undo log, WAL, outbox) zostaje – to cena trwałości i synchronizacji ([S4], [S6]). Dziś `UPDATE` 10% ze 100 tys. wierszy: 57,8 ms wobec 18,0 ms w sql.js. Pliki: `crates/flatsql-core/src/exec.rs`, `db.rs`, `table.rs`, `crates/flatsql-wasm/src/lib.rs`.

## 3. Semantyka istniejącej synchronizacji

Wszystkie zadania tej sekcji przeniesiono do [todo_future.md §12](todo_future.md) (§6) – obecna synchronizacja wystarcza do walidacji pracy offline.

## 4. Badania i benchmarki

- [ ] **4.1 Szeroki zestaw punktów odniesienia** (duży) – scenariusze H1–H5 dla jak najszerszego zestawu silników w `bench/engines.js` i na stronie benchmarku. Wyniki raportowane osobno w grupach o tych samych warunkach uruchomienia, żeby nie porównywać „jabłek z pomarańczami” [B3]; bez SQLite na OPFS teza o trwałości dotyczy tylko sql.js:
  - [ ] ten sam tryb co Flat-SQL (wątek główny, bez Workera i nagłówków): sql.js i `Map` (już są), AlaSQL (SQL w czystym JavaScripcie), Dexie.js (nakładka na IndexedDB – trwały zapis bez WASM);
  - [ ] silniki SQL z Workerem lub OPFS: oficjalny SQLite WASM (`opfs-sahpool`), wa-sqlite, PGlite, Zeta-Lite [Q7], [D5];
  - [ ] DuckDB-Wasm – silnik analityczny (kolumnowy): spodziewana przewaga w skanach i `GROUP BY`, słabość w odczytach po kluczu i pojedynczych zapisach; wynik wyznacza granicę zastosowań Flat-SQL [D1];
  - [ ] bazy local-first z synchronizacją: PouchDB i RxDB – konkurencja w aplikacjach offline ([use-cases.md](use-cases.md)); bez SQL, więc zapytania odwzorowane na ich API, z adnotacją w wynikach;
  - [ ] dla każdego silnika w jednej tabeli: rozmiar modułu (gzip), zimny start (4.3), pamięć (4.4) i wymagania środowiska (Worker, COOP/COEP, OPFS).
- [ ] **4.2 Statystyka** (mały) – ≥ 10 uruchomień procesu, przedziały ufności lub mediana z IQR. [B1].
- [ ] **4.3 Zimny start** (mały) – pobranie, kompilacja, otwarcie bazy, pierwsze zapytanie – osobna tabela; rozmiar modułu ma znaczenie przy starcie strony (Notion ładuje SQLite asynchronicznie). [B3].
- [ ] **4.4 Zużycie pamięci** (mały) – pamięć liniowa WASM na 100 tys. wierszy wobec sql.js i `Map`. [W11].
- [ ] **4.6 Kontrola wyników** (mały) – suma kontrolna wyników zamiast samej liczby wierszy. [B3].
- [ ] **4.7 Pomiar na telefonie** (mały, dobrze mieć) – wszystkie pomiary H1–H5 na urządzeniu mobilnym; osobno czas otwarcia bazy z IndexedDB, bo na wolnych urządzeniach lokalny odczyt bywa wolniejszy od sieci ([real-world-usage.md §3](research/real-world-usage.md#3-wnioski-projektowe-z-cudzych-wdrożeń)).
- [ ] **4.9 Koszt trwałości `strict`** (mały, dobrze mieć) – dodać do H5 (zob. 1.4). [Q1].
- [ ] **4.10 Pauza checkpointu** (mały) – najdłuższa przerwa wątku głównego podczas checkpointu dla 10, 50 i 100 tys. wierszy (i po 2.6, jeśli zostanie wykonane), na tle budżetu 16 ms na klatkę; checkpoint to jedyny wyjątek od pracy w wątku głównym bez przestojów [D4], [S10].
- [ ] **4.12 Koszt trybu Worker** (mały, dobrze mieć) – czas przesłania zapytania i wyniku do Web Workera i z powrotem (`postMessage`) dla odczytu po kluczu i dla 10 000 wierszy, na tle 9,7 µs w wątku głównym; rozstrzyga, od jakiej skali tryb Worker się opłaca ([ADR-0013](adr/0013-watek-glowny-domyslnie-worker-jako-opcja.md)).
- [ ] **4.13 Koszt utrwalania plików** (mały, dobrze mieć) – czas zatwierdzenia z `fsync` i bez (`--no-fsync`, `fsync: false`) oraz czas checkpointu z `fsync` katalogu w `FileStorage` (CLI) i `FsStore` (serwer: czas `/sync/push`), w Windows i Linuksie; koszt poprawek trwałości ([ADR-0033](adr/0033-checkpoint-plikowy-fsync-katalogu.md)). [S8].

## 5. Zadania usunięte po weryfikacji z literaturą

| Dawne zadanie | Dlaczego usunięte | Gdzie teraz |
|---|---|---|
| JS String Builtins (dawne 2.5) | mechanizm dla języków używających łańcuchów JS jako własnych (GC); Rust trzyma tekst w pamięci liniowej i kopiowania nie uniknie [Q13] | zastąpione przez 2.4 |
| Równoległe odtwarzanie (SiloR) | wymaga wątków WASM, czyli `SharedArrayBuffer` i nagłówków COOP/COEP – sprzeczne z atutem „zero konfiguracji” | 2.6, 2.7 |
| Snapshot jako obraz pamięci modułu (dawne 4.8) | obraz pamięci zależy od wersji modułu i układu wskaźników, więc łamie wspólny format plików Rust/WASM/Node | 2.7 |
| Formalna specyfikacja reguł scalania (część dawnego 3.9) | poza zakresem pracy inżynierskiej; zbieżność LWW wynika z teorii CRDT [L2], [L28] | 1.9 – testy losowe |
| Uzgadnianie zbiorów po utracie kursora (dawne 3.7) | przyczyną pełnych pobrań jest log serwera w pamięci – najpierw [todo_future.md 12.4](todo_future.md) | [todo_future.md 3.7](todo_future.md), warunkowo |
| Decyzja o wykluczeniu `docs/references/papers/` | katalog jest już wykluczony lokalnie (`.git/info/exclude`) | – |
| Nowe możliwości z dawnych sekcji 4, 7, 8 i część sekcji 3 | to rozszerzenia, nie optymalizacja istniejącego prototypu | [todo_future.md](todo_future.md) |
| Zadania literaturowe (dawna sekcja 6) | dotyczą pracy dyplomowej, nie prototypu | [search-methodology.md §6](references/search-methodology.md#6-jak-uzupełnić-przegląd) |

## 6. Zadania przeniesione do todo_future.md

Optymalizacje niepotrzebne w pracy inżynierskiej ([ADR-0029](adr/0029-listy-zadan-wedlug-zakresu-pracy.md)). Numery nie są używane ponownie.

| Dawne zadanie | Teraz |
|---|---|
| 0.3 Klucze dla wstawień offline | [todo_future.md 12.1](todo_future.md) |
| 0.4 Odnośniki w README | [todo_future.md 12.2](todo_future.md) |
| 3.1 LWW per kolumna | [todo_future.md 12.3](todo_future.md) |
| 3.2 Trwały log synchronizacji serwera | [todo_future.md 12.4](todo_future.md) |
| 3.3 Raport konfliktów klucza głównego | [todo_future.md 12.5](todo_future.md) |
| 3.4 Usuwanie tombstone'ów | [todo_future.md 12.6](todo_future.md) |
| 3.5 Granice transakcji w paczkach | [todo_future.md 12.7](todo_future.md) |
| 4.5 Realistyczne obciążenie | [todo_future.md 12.8](todo_future.md) |
| 4.8 Dokładniejszy zegar w przeglądarce | [todo_future.md 12.9](todo_future.md) |
| 4.11 Koszt LWW per kolumna | [todo_future.md 12.10](todo_future.md) |
