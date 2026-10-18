# Metodyka przeglądu literatury i jej ograniczenia

Opis, jak zebrano [bazę wiedzy](README.md), czego nie udało się sprawdzić i jak uzupełnić przegląd przed oddaniem pracy.

## 1. Przebieg

| Etap | Opis |
|---|---|
| Data | 18 października 2026 (stan źródeł internetowych z tego dnia) |
| Punkt wyjścia | dokumenty projektu ([architecture.md](../architecture.md), [flat-sql.md](../flat-sql.md), [README](../../README.md), [docs/research](../research/)), dwie pozycje polecone przez promotora, lista źródeł z [wasm-engines-comparison.md](../research/wasm-engines-comparison.md) |
| Narzędzia | ogólnodostępna wyszukiwarka internetowa (indeks anglojęzyczny), strony wydawców (ACM DL, USENIX, VLDB, SIGMOD Record, CIDR), arXiv, repozytoria uczelniane (HAL, MIT DSpace, Aalto, CWI, UiT Munin), Zenodo, strony autorów |
| Pobieranie | automatyczne, z weryfikacją sygnatury pliku PDF oraz tytułu i autorów na pierwszej stronie; pliki, które okazały się innymi dokumentami, odrzucano (jeden przypadek) |
| Weryfikacja treści | tezy i liczby użyte w [literature-comparison.md](literature-comparison.md) i notatkach sprawdzono w pełnym tekście; pozostałe pozycje oceniono na podstawie streszczenia i wstępu |
| Śnieżna kula | bibliografie kluczowych prac ([P1], [L1], [D4], [P2]), lista prac cytujących [L1] w ACM DL, program warsztatów PLF / PLF+PLAID (2022–2025) |

Obszary wyszukiwania (przykładowe zapytania, po angielsku):

1. WebAssembly – projekt i wydajność: `"Bringing the Web up to Speed with WebAssembly"`, `WebAssembly vs native performance`, `WebAssembly runtimes survey`, `memory64 bounds checks`, `Wasm 3.0`;
2. bezpieczeństwo WASM: `binary security of WebAssembly`, `WebAssembly web vulnerabilities 2026`;
3. SQL i bazy w WASM: `WebAssembly SQL database engine browser performance evaluation`, `DuckDB-Wasm`, `SQL queries compiled to WebAssembly`;
4. local-first: `local-first software research 2025 2026`, `local-first architecture browser applications`, `Programming Local-First Software workshop`;
5. CRDT i relacyjne bazy: `conflict-free replicated relations`, `CRDT relational database SQL`, `Antidote SQL`, `encrypted CRDT`;
6. zegary: `hybrid logical clocks`, `last-writer-wins clock skew`;
7. schemat: `schema evolution local-first`, `Cambria edit lenses`;
8. składowanie i awarie: `main memory database recovery`, `group commit`, `crash-consistent applications`, `power fault database testing`;
9. przeglądarka: `IndexedDB durability relaxed`, `Safari 7-day cap script-writable storage`, `storage persist eviction`, `IndexedDB OPFS performance study`;
10. edge: `edge computing vision challenges`, `WebAssembly serverless edge`, `WebAssembly browser edge cloud comparison`;
11. metodyka: `fair benchmarking database`, `statistically rigorous performance evaluation`, `VM warmup`, `YCSB`, `JavaScript timer resolution`;
12. model danych: `What Goes Around Comes Around 2024`, `Life beyond Distributed Transactions`.

Kryteria doboru: (1) związek z co najmniej jednym modułem projektu lub hipotezą H1–H6; (2) praca recenzowana, uznany preprint lub praca dyplomowa uczelni; (3) pierwszeństwo dla wersji otwartych. Źródła branżowe (blogi przeglądarek, dokumentacja, pomiary producentów) przyjęto tylko tam, gdzie opisują zachowanie platformy, od którego zależy poprawność projektu.

## 2. Wynik

- 77 plików PDF: 14 prac do 2010 r., 29 z lat 2011–2020, 19 z lat 2021–2024, 13 z lat 2025–2026, oraz specyfikacja Wasm 3.0 i notatki wykładowe Kleppmanna. Cztery z nich ([L24], [L25], [L26], [W11]) pobrano w drugim podejściu przez zwykłą przeglądarkę, bo serwisy blokowały pobieranie automatyczne; ponowne wyszukiwanie dodało też preprint Zeta-Lite [D5] z września 2026 r.
- 3 ważne pozycje bez pliku ([README §11](README.md#11-źródła-ważne-których-nie-pobrano--do-pobrania-ręcznie)) i 11 grup źródeł dokumentacyjno-branżowych ([README §12](README.md#12-dokumentacja-platform-i-źródła-branżowe-nierecenzowane)).
- Bibliografie 12 rozdziałów DDIA (wyd. 1) z repozytorium autora.

## 3. Ograniczenia

1. **Brak dostępu do płatnych baz bibliograficznych.** Nie przeszukano Scopus, Web of Science, pełnych tekstów IEEE Xplore i SpringerLink ani platformy O'Reilly. Nie użyto bezpośrednio Google Scholar, więc nie ma liczby cytowań ani pełnych list „cited by”. Przegląd jest **ukierunkowany, a nie systematyczny** (w rozumieniu wytycznych Kitchenham).
2. **Blokady automatycznego pobierania.** Część serwisów odrzucała pobranie: ACM DL dla niektórych artykułów (ochrona przed botami), HAL (strona weryfikacyjna zamiast PDF), MIT DSpace (HTTP 429), O'Reilly (HTTP 403), UiT Munin. Zabezpieczeń nie obchodzono. Trzy darmowe prace z HAL, MIT i UiT pobrano później ręcznie przez zwykłą przeglądarkę; pozostałe są na liście do pobrania ręcznego (np. przez bibliotekę uczelni). Prace z IEEE (np. [E1]) pochodzą z kopii udostępnionych na stronach kursów lub autorów.
3. **Ograniczenia wyszukiwarki.** Wyniki pochodziły z indeksu anglojęzycznego (USA), a ranking faworyzuje popularne strony i blogi. W kilku obszarach nie znaleziono recenzowanych badań: wydajność IndexedDB i OPFS (dostępne są głównie pomiary producentów – RxDB, PGlite), koszt granicy JS↔WASM w silnikach baz danych, koordynacja wielu kart (Web Locks, `SharedWorker`). Brak wyników nie dowodzi braku publikacji.
4. **Najnowsze prace.** Pozycje z 2026 r. to w większości preprinty arXiv bez recenzji. Materiały konferencji z drugiej połowy 2026 r. (np. VLDB, SRDS, OOPSLA/SPLASH, UIST) mogły nie być jeszcze zaindeksowane. Wystąpienia z PLF+PLAID 2025 (np. Teilen-SQL) mają tylko streszczenia; artykułu Niinikoski i Fagerholm (2026) nie odnaleziono – są tylko materiały uzupełniające. Nie udało się zweryfikować spisu treści 2. wydania DDIA (2026).
5. **Język.** Przeszukiwano literaturę anglojęzyczną; z polskiej uwzględniono tylko tłumaczenie DDIA. Nie przeszukiwano literatury chińskiej, choć znaczna część badań nad środowiskami WASM pochodzi z Chin ([W5]).
6. **Źródła branżowe** są obarczone konfliktem interesów (producenci porównujący własne produkty) i zmieniają się bez wersjonowania; adresy URL mogą wygasnąć.
7. **Głębokość lektury.** Szczegółowo przeczytano fragmenty prac cytowanych w porównaniu; pozostałe oceniono na podstawie streszczeń i wstępów.
8. **Obszary pominięte** (poza zakresem tego etapu lub bez trafień): pomiary latencji sieci komórkowych (Cloud-First na urządzeniach mobilnych), zużycie energii WASM na telefonach (jedna praca w [W5]), interfejs FFI dla panelu WinUI (kolejny etap projektu), formalna weryfikacja protokołów odtwarzania WAL, badania bezpieczeństwa magazynów przeglądarki (XSS a IndexedDB).

## 4. Czy przeszukano wszystkie najnowsze prace?

Nie – i przy dostępnych narzędziach nie było to możliwe. Zebrany zbiór dobrze pokrywa prace fundamentalne (zegary, CRDT, WAL, WebAssembly, local-first) i najważniejsze nowe pozycje z lat 2025–2026, ale nie spełnia wymogów przeglądu systematycznego: brakuje przeszukania baz bibliograficznych z zapisanymi zapytaniami, liczbą wyników na każdym etapie i kryteriami wykluczenia (schemat PRISMA).

## 5. Jak uzupełnić przegląd (biblioteka uczelni)

Zapytania do Scopus / Web of Science / IEEE Xplore / ACM DL (filtr: 2019–2026, dla klasyki bez filtra):

```text
("WebAssembly" OR "Wasm") AND ("database" OR "SQL" OR "query engine" OR "storage")
("local-first" OR "offline-first") AND ("synchronization" OR "replication" OR "CRDT")
("IndexedDB" OR "Origin Private File System" OR "browser storage") AND ("performance" OR "durability" OR "persistence")
("hybrid logical clock" OR "last-writer-wins" OR "LWW") AND ("replication" OR "conflict resolution")
("conflict-free replicated" OR "CRDT") AND ("relational" OR "SQL" OR "SQLite")
("in-memory database" OR "main memory database") AND ("write-ahead log" OR "checkpoint" OR "recovery")
("edge computing" OR "cloud-edge continuum") AND "WebAssembly"
("WebAssembly") AND ("JavaScript interoperability" OR "boundary" OR "data exchange")
```

Dodatkowo: alerty Google Scholar „cytowane przez” dla [P2], [W1], [L1] i [D4]; przegląd programów PaPoC, PLF+PLAID, Local-First Conf i VLDB z 2026 r.; prośba do biblioteki o dostęp do 2. wydania DDIA.
