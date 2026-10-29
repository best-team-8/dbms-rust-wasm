# Metodyka przeglądu literatury i jej ograniczenia

Opis, jak zebrano [bazę wiedzy](README.md), czy wyszukiwanie objęło obszary, w których leżą atuty i plany projektu, jak wybrano rdzeń literatury i czego nie udało się sprawdzić.

## 1. Przebieg

| Etap | Opis |
|---|---|
| Runda 1 (18.10.2026) | wyszukiwanie według obszarów technologicznych (§2), pobranie 77 PDF-ów |
| Runda 2 (20.10.2026) | audyt pokrycia atutów i planów projektu (§3), wyszukiwanie uzupełniające w lukach, ponowna ocena wszystkich pozycji i wybór rdzenia (§4) |
| Rewizja 3 (22.10.2026) | bez nowego wyszukiwania: ocena każdej pozycji według podrozdziału pracy dyplomowej, w którym będzie cytowana (§4.1) |
| Punkt wyjścia | dokumenty projektu ([flatsql-core/README.md](../../crates/flatsql-core/README.md), [flatsql-core/src/sql/README.md](../../crates/flatsql-core/src/sql/README.md), [README](../../README.md), [docs/research](../research/)), dwie pozycje polecone przez promotora, w rundzie 2 także [analiza rzeczywistych wdrożeń](../research/use-cases.md) i obie listy zadań |
| Narzędzia | ogólnodostępna wyszukiwarka internetowa (indeks anglojęzyczny), strony wydawców (ACM DL, USENIX, VLDB, CIDR), arXiv, repozytoria uczelniane, strony autorów i kursów |
| Pobieranie | z weryfikacją tytułu i autorów na pierwszej stronie; w rundzie 2 tezy z każdej nowej pracy sprawdzono w pełnym tekście przed dodaniem (wyjątek: [M3]) |
| Weryfikacja treści | tezy i liczby użyte w ówczesnych analizach literatury (dziś [literature.md](literature.md)) i listach zadań sprawdzono w pełnym tekście (w rundzie 2 ponownie dla prac rdzenia, na których opierają się zadania: [C2], [C8], [D1], [D4], [D5], [L8], [L20], [L24], [L28], [M1], [S1], [S6], [S8], [E1]); [M3] oceniono na podstawie streszczenia i źródeł wtórnych |
| Śnieżna kula | bibliografie [P1], [L1], [D4], [P2]; lista prac cytujących [L1]; program PLF / PLF+PLAID 2022–2025; w rundzie 2 bibliografie [C9], [C10], [L27] |

## 2. Obszary wyszukiwania

**Runda 1** – przykładowe zapytania (po angielsku):

1. WebAssembly – projekt i wydajność: `"Bringing the Web up to Speed with WebAssembly"`, `WebAssembly vs native performance`, `WebAssembly runtimes survey`, `memory64 bounds checks`;
2. bezpieczeństwo WASM: `binary security of WebAssembly`, `WebAssembly web vulnerabilities 2026`;
3. SQL i bazy w WASM: `WebAssembly SQL database engine browser`, `DuckDB-Wasm`, `SQL queries compiled to WebAssembly`;
4. local-first: `local-first software research 2025 2026`, `Programming Local-First Software workshop`;
5. CRDT i bazy relacyjne: `conflict-free replicated relations`, `CRDT relational database SQL`, `Antidote SQL`, `encrypted CRDT`;
6. zegary: `hybrid logical clocks`, `last-writer-wins clock skew`;
7. schemat: `schema evolution local-first`, `Cambria edit lenses`;
8. składowanie i awarie: `main memory database recovery`, `group commit`, `crash-consistent applications`;
9. przeglądarka: `IndexedDB durability relaxed`, `Safari 7-day cap`, `storage persist eviction`;
10. edge: `edge computing vision challenges`, `WebAssembly serverless edge`;
11. metodyka: `fair benchmarking database`, `statistically rigorous performance evaluation`, `YCSB`;
12. model danych: `What Goes Around Comes Around 2024`, `Life beyond Distributed Transactions`.

**Runda 2** – zapytania w lukach z §3:

1. `"The Dangers of Replication and a Solution" two-tier replication tentative transactions mobile`;
2. `disconnected operation Coda reintegration log`, `client modify log optimization`;
3. `content-based partial replication filter`, `partial replication CRDT local-first 2024 2025 2026`;
4. `data sync mobile apps reliability consistency` (Simba);
5. `scale independence bounded query key-value PIQL`;
6. `checkpoint recovery main memory frequently consistent applications games`;
7. `materialized view maintenance incremental aggregates`;
8. `Approaches to Conflict-free Replicated Data Types survey 2024`;
9. `WebAssembly JavaScript interoperability overhead boundary empirical 2023 2024 2025`;
10. `IndexedDB OPFS performance evaluation empirical study`;
11. dokumentacja: dostępność `SharedWorker` (2025–2026), zastosowanie JS String Builtins dla języków z pamięcią liniową.

Kryteria włączenia do bazy: (1) związek z modułem projektu, hipotezą H1–H6 lub zadaniem; (2) praca recenzowana, uznany preprint lub praca dyplomowa uczelni; (3) pierwszeństwo dla wersji otwartych. Źródła branżowe przyjęto tylko tam, gdzie opisują zachowanie platformy, od którego zależy poprawność projektu.

## 3. Audyt pokrycia atutów projektu

Runda 1 była zorganizowana według **technologii** (WebAssembly, CRDT, edge). Dała dobre pokrycie tła technologicznego, ale nadreprezentowała tematy ogólne (środowiska uruchomieniowe WASM, formalna weryfikacja CRDT, serverless na edge), a pominęła **problemy systemowe**, którym odpowiadają założenia Flat-SQL: klient pracujący offline z trwałym logiem zmian, replikacja częściowa, synchronizacja tabel w aplikacjach, przewidywalność ograniczonego języka zapytań, checkpointy stanu w pamięci. Audyt porównał każdy atut i każdy plan z zawartością zbioru.

| Atut lub plan Flat-SQL | Co było po rundzie 1 | Luka | Uzupełnienie w rundzie 2 |
|---|---|---|---|
| Płaski model, brak JOIN, „przewidywalne wykonanie” | [C7], [C6], [C4], [C5], [M1], [L8] | uzasadnienie opierało się na replikacji; brak pracy o językach zapytań ograniczonych dla przewidywalnego kosztu | [M2] PIQL |
| Dane w pamięci, wykonanie jednowątkowe | [S1], [S4], [S5], [P1] | – | – |
| WAL + snapshot, checkpoint | [S1], [S2], [S3], [S6], [S7] | kierunki dla checkpointu wymagały wątków (SiloR) albo zmiany struktury (LSM); brak prac o checkpointach stanu w pamięci przy częstych zmianach (gry) | [S10] Cao i in. |
| Odporność na awarie zapisu | [S8], [S9] | – | – |
| Trwały outbox, praca offline klienta | [C3], [L1] | brak klasyki pracy bez połączenia i brak badań niezawodności synchronizacji w aplikacjach – dokładnie tam leży atut „outbox przetrwa zamknięcie aplikacji offline” | [C10] Coda, [L27] Simba |
| LWW na HLC | [C1], [C2], [L2]–[L7] | koszt LWW (utracone aktualizacje) i alternatywa przemienna opisane tylko w [P1] | [C9] Gray i in. |
| Atomowość per wiersz między replikami | [C6], [C7], [C8] | brak przykładu systemu synchronizacji, który świadomie wybiera tę gwarancję | [L27] Simba |
| Hybryda z serwerem, reguły serwera, zamiary zatwierdzane przez serwer | [C3], [L20], [L22] | brak modelu kopii głównej i transakcji wstępnych | [C9] Gray i in. |
| Replikacja częściowa (wg analizy wdrożeń – obowiązkowa) | brak | **całkowity brak** | [C11] Cimbiosys |
| Zapytania reaktywne (plan) | [D4] | brak techniki przyrostowego utrzymania wyników | [M3] Gupta i Mumick |
| Tombstone'y, delta-state | [L6], [L7], [L3] | trzy prace na jeden wątek | [L28] Almeida (aktualny przegląd w miejsce trzech) |
| Mały moduł, praca w wątku głównym | [P2], [W1], [W11] | brak recenzowanych badań wpływu rozmiaru modułu na start; są dowody branżowe (Notion) | luka pozostaje |
| Granica JS↔WASM (H3) | brak | brak recenzowanych pomiarów | luka pozostaje (§5) – H3 to wkład własny |
| Wydajność IndexedDB i OPFS | [Q8] (producenci) | brak badań recenzowanych | luka pozostaje |
| Wiele kart | brak | brak badań; jest dokumentacja | [Q12] (dokumentacja) |
| Edge – ten sam rdzeń | [E1]–[E8] | **nadmiar**: osiem prac o serverless i środowiskach WASM, żadna o bazach | ograniczono do [E1]; architekturę węzłów opisuje [C9] |
| Bezpieczeństwo WASM | [W2]–[W10] | **nadmiar** | ograniczono do [W2] |
| Teoria i weryfikacja CRDT | [L2]–[L7], [L12]–[L19] | **nadmiar** (formalna weryfikacja poza zakresem pracy) | ograniczono do [L2], [L28] |
| Metodyka benchmarków | [B1]–[B6] | nadmiar | ograniczono do [B1], [B3], [B5] |

## 4. Dobór rdzenia

Każdą z 77 pozycji rundy 1 oceniono ponownie pod kątem trzech pytań: czy uzasadnia decyzję projektową lub atut prototypu; czy jest punktem odniesienia dla hipotezy H1–H6 lub porównywanego systemu; czy jest podstawą zadania z [todo.md](../todo.md) lub [future/todo.md](../future/todo.md). Pozycja zostaje, jeśli odpowiada „tak” na co najmniej jedno pytanie **i** nie dubluje innej pozycji rdzenia.

Kryteria wyłączenia:

1. **duplikat** – inna pozycja rdzenia daje tę samą tezę (np. trzy prace o delta-state i stabilności przyczynowej → jeden przegląd [L28]);
2. **kierunek sprzeczny z atutami** – praca uzasadnia rozwiązanie, które odbiera atut projektu (równoległe odtwarzanie wymaga wątków i COOP/COEP; obraz pamięci modułu łamie wspólny format plików);
3. **poza zakresem pracy** – formalna weryfikacja, synchronizacja bez serwera, edycja tekstu;
4. **kontekst bez wpływu** – przeglądy ogólne i badania ekosystemu, które nie zmieniają żadnej decyzji;
5. **słabe źródło** – preprint lub praca licencjacka, gdy ta sama teza ma źródło recenzowane.

Wynik: **38 pozycji rdzenia** – 30 z rundy 1 i 8 nowych ([C9], [C10], [C11], [L27], [L28], [M2], [M3], [S10]); dwie pozycje rundy 1 dołączono do wpisów prac promotora (wersja CACM do [P2], notatki do wykładu do [P1]); **46 pozycji poza rdzeniem**, pliki w `papers/_poza-rdzeniem/` (zachowane). Liczba 30 nie była limitem – nowe prace dodano, bo wypełniają luki w obszarach, w których leżą atuty i plany projektu.

### 4.1 Rewizja 3 – cytowania w pracy dyplomowej

Kryterium z rundy 2 przyjmowało każdą podstawę zadania, także z [future/todo.md](../future/todo.md), choć tylko część rozszerzeń trafi do pracy (kierunki rozwoju w podrozdziale 9.5). Przed pisaniem pracy każdą pozycję – z rdzenia i spoza niego – oceniono pytaniem: w którym podrozdziale pracy będzie cytowana i czy inna pozycja rdzenia nie daje tam tej samej tezy ([ADR-0031](../adr/0031-rdzen-literatury-wedlug-cytowan-w-pracy.md)). Wynik zapisano w [mapie rozdziałów](README.md#mapa-rozdziały-pracy-dyplomowej--literatura).

| Pozycja | Zmiana | Powód |
|---|---|---|
| [W6] Döllerer i Engelke, VMIL 2024 | do rdzenia | jedyne recenzowane źródło dla [ADR-0007](../adr/0007-wasm32-bez-memory64.md), który trafia do tabeli 3.2 pracy; [Q6] to wpisy z blogów, które nie liczą się do 15 publikacji |
| [L11] Kuessner i in., PLF 2022 | poza rdzeń | szyfrowanie end-to-end jest w pracy tylko kierunkiem rozwoju; tę samą tezę daje recenzowane [L1] (ideał 6) – kryteria 1 i 5 |
| [L26] Cambria, PaPoC 2021 | poza rdzeń | problem opisują [L1] i [P1] rozdz. 4; soczewki są odrzuconą alternatywą w [ADR-0027](../adr/0027-struktura-w-kodzie-aplikacji-tylko-dokladanie.md) (etap po pracy); todo 1.7 tylko zgłasza konflikt |
| [M3] Gupta i Mumick, 1995 | poza rdzeń | podstawa tylko future 5.1; zapytania reaktywne w kierunkach rozwoju uzasadnia [D4]; oceniona ze streszczenia |
| [C8] Bailis i in., PVLDB 2013 | poza rdzeń | w pracy tylko jedna uwaga w podrozdziale 8.5 (MAV jako opcja na później); atomowość wiersza uzasadniają [C7] i [L27], potrzebę koordynacji – [C9] i [P1] rozdz. 9 |
| [L8] Antidote SQL, 2019 | poza rdzeń | w pracy tylko jedna uwaga w podrozdziale 3.3; dialekt uzasadniają [C7], [M2], [M1], aktualizacje przemienne – [C9] i [L2]; preprint |
| [L20] Niinikoski, 2025 | poza rdzeń | model hybrydowy uzasadniają recenzowane [C9] i [C3]; praca magisterska – kryterium 5 |

Ostatnie trzy pozycje miały w mapie po jednym zdaniu, które da się oprzeć na innych pracach rdzenia, więc przy pierwszym cięciu objętości i tak wypadłyby z tekstu.

Wynik: **33 pozycje rdzenia** (34 pliki PDF – bez pliku [P1], z wersją CACM [P2] i notatkami do wykładu [P1]) i **51 pozycji poza rdzeniem** (50 plików; [M3] nie ma pliku). Spośród pozostałych pozycji spoza rdzenia żadna nie ma podrozdziału, w którym byłaby potrzebna: [C1] Lamport – HLC w podrozdziale 2.4 wystarczy opisać przez [C2] i [P1] rozdz. 8–9 (do przywrócenia, jeśli tekst wyprowadzi HLC z zegarów logicznych); [S7] SiloR i [E5] Faasm – odrzucone kierunki z [todo.md §5](../todo.md) opisuje się w podrozdziale 7.7 przez powód odrzucenia, bez omawiania tych systemów; [S5] Harizopoulos i in. – koszt logowania w podrozdziale 7.4 uzasadniają [S4] i [S6].

Znalezione w rundzie 2 i **nie** włączone:

| Pozycja | Powód |
|---|---|
| Faerber F. i in., *Main Memory Database Systems*, Foundations and Trends in Databases 8(1–2), 2017 | dostęp płatny; tematykę pokrywają [P1] rozdz. 3 i [S4] |
| Briquemont I. i in., *Conflict-free Partially Replicated Data Types*, CloudCom 2015 | odnaleziono tylko slajdy; replikację częściową pokrywa [C11] |
| Thalakottur M. i in., *An Empirical Study of WebAssembly Usage in Node.js*, ICSE 2026 | bada wzorce współpracy JS i WASM, nie koszt granicy |
| Stepanov O., Klym H., *A Quantitative Analysis of WebAssembly Integration*, Electronics and Information Technologies 32, 2025 | jedyny znaleziony pomiar wywołań `wasm-bindgen`, ale czasopismo uczelniane o małym zasięgu, dostępne tylko streszczenie, a podany koszt kopiowania 1 MB (1–3 ms) budzi wątpliwości |
| Gupta A., Mumick I. S., Subrahmanian V. S., *Maintaining Views Incrementally*, SIGMOD 1993 | algorytm zliczania opisany w przeglądzie [M3] |

### 4.2 Rewizja 4 – zastosowanie i wynik weryfikacji

Po Benchmarku 2 i testach poprawności tezy prac rdzenia zestawiono z wynikami ([literature.md](literature.md)). Rdzeń obejmuje od 28.10.2026 tylko prace, które projekt zastosował albo których tezy sprawdził benchmark lub test; poza rdzeń przeszły [L24], [C11], [M1], [E1] i [B5]. Pozostałe 28 prac podzielono na katalogi `papers/potwierdzone/` (19), `papers/niepotwierdzone/` (2: [W1], [W11]) i `papers/zastosowane/` (7). Przegląd literatury przed pomiarem dał więc dwa przewidywania, które pomiar obalił: narzut WebAssembly rzędu 1,5× w operacjach wykonywanych w silniku i moduł optymalizowany pod rozmiar bez straty czasu.

Po czterech rewizjach rdzeń liczy 28 prac, a poza nim jest 56 pozycji (55 plików w `papers/_poza-rdzeniem/`; [M3] nie ma pliku). Każdą wyłączono według jednego z kryteriów z początku tej części albo, od rewizji 3 i 4, dlatego, że nie ma podrozdziału pracy, w którym byłaby cytowana, albo projekt jej nie zastosował, a pomiar nie sprawdził jej tezy. Identyfikatory tych prac nie są używane ponownie; tam, gdzie dokumenty projektu się na nie powołują, podają autora i rok z dopiskiem „praca poza rdzeniem”.

## 5. Ograniczenia

1. **Brak dostępu do płatnych baz bibliograficznych.** Nie przeszukano Scopus, Web of Science, pełnych tekstów IEEE Xplore i SpringerLink. Nie użyto bezpośrednio Google Scholar, więc nie ma liczby cytowań ani pełnych list „cited by”. Przegląd jest **ukierunkowany, a nie systematyczny** (w rozumieniu wytycznych Kitchenham) – nie spełnia wymogów PRISMA (zapisane zapytania z liczbą wyników na każdym etapie).
2. **Blokady automatycznego pobierania.** Część serwisów odrzucała pobranie (ACM DL, HAL, MIT DSpace, O'Reilly, część serwisów kursów); zabezpieczeń nie obchodzono. Kopie nowych prac pochodzą z otwartego dostępu (USENIX, arXiv, Microsoft Research) i z serwisów kursów (Gray i in., Coda, Cao i in.) – w pracy cytuje się wersje wydawnicze.
3. **Ograniczenia wyszukiwarki.** Wyniki z indeksu anglojęzycznego, z rankingiem faworyzującym popularne strony. W trzech obszarach nawet po rundzie 2 nie znaleziono recenzowanych badań: wydajność IndexedDB i OPFS, koszt granicy JS↔WASM w silnikach baz, koordynacja wielu kart. Brak wyników nie dowodzi braku publikacji.
4. **Najnowsze prace.** Pozycje z 2026 r. to w większości preprinty; materiały konferencji z drugiej połowy 2026 r. mogły nie być jeszcze zaindeksowane. Nie zweryfikowano spisu treści 2. wydania DDIA.
5. **Język.** Przeszukiwano literaturę anglojęzyczną; z polskiej uwzględniono tylko tłumaczenie DDIA.
6. **Źródła branżowe** są obarczone konfliktem interesów i zmieniają się bez wersjonowania; dotyczy to także źródeł analizy wdrożeń.
7. **Obszary pominięte świadomie:** wyszukiwanie pełnotekstowe i wektorowe (zadania future 5.5–5.6 – literatura do zebrania przy ich rozpoczęciu), interfejs FFI dla panelu WinUI (kolejny etap), zużycie energii WASM na telefonach, latencja sieci komórkowych.

## 6. Jak uzupełnić przegląd

Zapytania do Scopus / Web of Science / IEEE Xplore / ACM DL (filtr: 2019–2026, dla klasyki bez filtra):

```text
("WebAssembly" OR "Wasm") AND ("database" OR "SQL" OR "query engine" OR "storage")
("local-first" OR "offline-first" OR "disconnected operation") AND ("synchronization" OR "replication" OR "CRDT")
("partial replication" OR "selective replication") AND ("mobile" OR "client" OR "CRDT")
("IndexedDB" OR "Origin Private File System" OR "browser storage") AND ("performance" OR "durability" OR "persistence")
("hybrid logical clock" OR "last-writer-wins" OR "LWW") AND ("replication" OR "conflict resolution")
("conflict-free replicated" OR "CRDT") AND ("relational" OR "SQL" OR "SQLite")
("in-memory database" OR "main memory database") AND ("checkpoint" OR "recovery" OR "write-ahead log")
("WebAssembly") AND ("JavaScript interoperability" OR "boundary" OR "data exchange")
```

Lista kontrolna:

- [ ] Przeszukać bazy powyższymi zapytaniami i zapisać liczby wyników na każdym etapie (jeśli promotor wymaga przeglądu systematycznego).
- [ ] Wypożyczyć DDIA (wydanie polskie: Helion 2018) i sprawdzić z tekstem książki tezy [P1] cytowane w pracy.
- [ ] Ustawić alerty Google Scholar („cytowane przez”) dla [P2], [W1], [L1], [D4], [L24], [C11].
- [ ] Zapytać autorów Teilen-SQL o tekst [X5]; sprawdzić publikację artykułu Niinikoski i Fagerholm [X6].
- [ ] Przejrzeć programy PaPoC, PLF, Local-First Conf i VLDB z 2026 r.
- [ ] Przed rozpoczęciem future 5.5–5.6 i 9.6 zebrać literaturę tych obszarów.
- [ ] W pracy cytować wersje wydawnicze (DOI), nie lokalne kopie.
