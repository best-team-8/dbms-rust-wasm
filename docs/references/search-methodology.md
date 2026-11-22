# Metodyka przeglądu literatury

Dokument opisuje, jak zebrano [spis literatury](README.md): gdzie i czego szukano, jak sprawdzono, czy wyszukiwanie objęło obszary, w których leżą atuty i plany projektu, jak wybrano rdzeń literatury i czego nie udało się sprawdzić.

## Wyszukiwanie

Szukano w ogólnodostępnej wyszukiwarce (indeks anglojęzyczny), na stronach wydawców (ACM DL, USENIX, VLDB, CIDR), w arXiv, w repozytoriach uczelni i na stronach autorów. Punktem wyjścia były dwie prace polecone przez promotora. Dalsze prace znaleziono metodą kuli śnieżnej: w bibliografiach Kleppmanna (2017), Kleppmanna i in. (2019), Haasa i in. (2017), Litta i in. (2023), Graya i in. (1996), Kistlera i Satyanarayanana (1992) i Go i in. (2015), w liście prac cytujących Kleppmanna i in. (2019) oraz w programach warsztatów PLF z lat 2022–2025.

Wyszukiwanie miało dwa przejścia. Pierwsze szło według obszarów technologicznych i dało 77 prac. Drugie uzupełniało luki wskazane przez audyt pokrycia (niżej). Przykładowe zapytania według obszarów:

| Obszar | Przykładowe zapytania |
|---|---|
| WebAssembly: wydajność i bezpieczeństwo | `WebAssembly vs native performance`, `memory64 bounds checks`, `binary security of WebAssembly` |
| Granica JavaScript–WebAssembly | `WebAssembly JavaScript interoperability overhead boundary empirical` |
| Bazy SQL w WebAssembly | `WebAssembly SQL database engine browser`, `DuckDB-Wasm`, `SQL queries compiled to WebAssembly` |
| Magazyny przeglądarki | `IndexedDB durability relaxed`, `IndexedDB OPFS performance evaluation`, `storage persist eviction` |
| Local-first i CRDT | `local-first software`, `conflict-free replicated relations`, `Approaches to Conflict-free Replicated Data Types survey` |
| Praca bez połączenia | `disconnected operation Coda reintegration log`, `data sync mobile apps reliability consistency` |
| Replikacja i koordynacja | `The Dangers of Replication and a Solution`, `content-based partial replication filter` |
| Zegary | `hybrid logical clocks`, `last-writer-wins clock skew` |
| Zmiany schematu | `schema evolution local-first`, `Cambria edit lenses` |
| Składowanie i awarie | `main memory database recovery`, `group commit`, `crash-consistent applications`, `checkpoint recovery main memory games` |
| Model danych i zapytania | `scale independence bounded query key-value`, `What Goes Around Comes Around`, `Life beyond Distributed Transactions`, `incremental view maintenance` |
| Edge | `edge computing vision challenges`, `WebAssembly serverless edge` |
| Metodyka pomiarów | `fair benchmarking database`, `statistically rigorous performance evaluation`, `YCSB` |

Do spisu trafiały prace związane z modułem projektu, pytaniem badawczym albo zadaniem; recenzowane, uznane preprinty albo prace dyplomowe uczelni, z pierwszeństwem dla wersji otwartych. Dokumentację przeglądarek przyjęto tylko tam, gdzie opisuje zachowanie platformy, od którego zależy poprawność projektu.

## Audyt pokrycia

Pierwsze przejście, zorganizowane według technologii, dobrze pokryło tło technologiczne, ale za dużo w nim było tematów ogólnych (środowiska WebAssembly, formalna weryfikacja CRDT, serverless na edge), a za mało problemów, które odpowiadają założeniom Flat-SQL. Audyt porównał każdy atut i plan projektu z zebranymi pracami.

| Atut albo plan Flat-SQL | Luka po pierwszym przejściu | Uzupełnienie |
|---|---|---|
| Płaski model, przewidywalne wykonanie | brak pracy o językach zapytań z ograniczonym kosztem | Armbrust i in. (2011) |
| Checkpoint | kierunki wymagały wątków albo zmiany struktury; brak prac o checkpointach stanu w pamięci przy częstych zmianach | Cao i in. (2011) |
| Trwała kolejka zmian, praca offline | brak klasyki pracy bez połączenia i badań niezawodności synchronizacji | Kistler i Satyanarayanan (1992), Go i in. (2015) |
| Reguła „ostatni zapis wygrywa” | koszt reguły (utracone zmiany) i alternatywa przemienna tylko w podręczniku | Gray i in. (1996) |
| Atomowość wiersza między urządzeniami | brak systemu, który świadomie wybiera tę gwarancję | Go i in. (2015) |
| Serwer i zamiary zatwierdzane przez serwer | brak modelu kopii głównej i transakcji wstępnych | Gray i in. (1996) |
| Replikacja częściowa | całkowity brak | Ramasubramanian i in. (2009) |
| Zapytania reaktywne | brak techniki przyrostowego utrzymania wyników | Gupta i Mumick (1995) |
| Znaczniki usunięcia, wysyłanie stanu zmian | trzy prace na jeden temat | Almeida (2024) zamiast trzech |
| Mały moduł, start strony | brak recenzowanych badań | luka pozostaje; są dane branżowe (Notion) |
| Granica JavaScript–WebAssembly | brak recenzowanych pomiarów | luka pozostaje; pomiar jest wkładem projektu |
| Wydajność IndexedDB i OPFS | tylko pomiary producentów | luka pozostaje |
| Edge, bezpieczeństwo WebAssembly, teoria CRDT, metodyka | nadmiar prac bez wpływu na decyzje | ograniczono do kilku prac |

## Dobór rdzenia

Rdzeń literatury to prace, które projekt zastosował (przejął z nich mechanizm, metodę pomiaru albo uzasadnienie decyzji) albo których tezy sprawdził benchmark lub test. Praca nie wchodzi do rdzenia, jeśli:

1. inna praca rdzenia daje tę samą tezę;
2. uzasadnia rozwiązanie, które odbiera atut projektu, na przykład równoległe odtwarzanie wymagające wątków;
3. dotyczy obszaru poza zakresem: formalnej weryfikacji, synchronizacji bez serwera, edycji tekstu;
4. jest przeglądem ogólnym, który nie zmienia żadnej decyzji;
5. jest preprintem albo pracą licencjacką, gdy tę samą tezę ma źródło recenzowane.

Rdzeń liczy 28 prac. Prace spoza rdzenia, które przywołują dokumenty projektu, są w spisie jako „prace pomocnicze”. Liczba prac nie była limitem: prace z drugiego przejścia weszły do rdzenia, bo wypełniły luki w obszarach, w których leżą atuty i plany projektu.

## Rewizja pod kątem pracy dyplomowej

Przed pisaniem pracy każdą pracę oceniono pytaniem: w którym podrozdziale będzie cytowana i czy inna praca rdzenia nie daje tam tej samej tezy. Rewizja zmieniła skład rdzenia:

| Praca | Zmiana | Powód |
|---|---|---|
| Döllerer i Engelke (2024) | do rdzenia | jedyne recenzowane źródło dla decyzji o module 32-bitowym; pozostałe to wpisy z blogów |
| Kuessner i in. (2022) | poza rdzeń | szyfrowanie end-to-end jest w pracy tylko kierunkiem rozwoju, a tę samą tezę daje recenzowana praca Kleppmanna i in. (2019) |
| Litt i in. (2021) | poza rdzeń | problem zmian schematu opisują Kleppmann i in. (2019) i Kleppmann (2017); tłumaczenie danych to odrzucony wariant |
| Gupta i Mumick (1995) | poza rdzeń | podstawa tylko jednego zadania na później; oceniona ze streszczenia |
| Bailis i in. (2013) | poza rdzeń | w pracy tylko jedna uwaga; atomowość wiersza uzasadniają Helland (2007) i Go i in. (2015) |
| Lopes i in. (2019) | poza rdzeń | preprint z jedną uwagą w pracy; dialekt uzasadniają inne prace rdzenia |
| Niinikoski (2025) | poza rdzeń | praca magisterska; model hybrydowy uzasadniają recenzowane prace Graya i in. (1996) i Terry'ego i in. (1995) |

Znalezione, ale niewłączone: monografia o bazach w pamięci (dostęp płatny, temat pokrywa Kleppmann, 2017), praca o częściowo replikowanych typach CRDT (dostępne tylko slajdy), badanie użycia WebAssembly w Node.js (bada wzorce współpracy, a nie koszt granicy) i pomiar wywołań `wasm-bindgen` w czasopiśmie uczelnianym (dostępne tylko streszczenie, wątpliwe liczby).

## Ograniczenia

1. Nie przeszukano płatnych baz bibliograficznych (Scopus, Web of Science, pełne teksty IEEE Xplore i SpringerLink) ani bezpośrednio Google Scholar. Przegląd jest ukierunkowany, a nie systematyczny w rozumieniu wytycznych Kitchenham, i nie spełnia wymogów PRISMA.
2. Część serwisów odrzucała automatyczne pobieranie; zabezpieczeń nie obchodzono. Kopie części prac pochodzą z serwisów kursów, a w pracy cytuje się wersje wydawnicze.
3. W trzech obszarach nie znaleziono recenzowanych badań: wydajność IndexedDB i OPFS, koszt granicy JavaScript–WebAssembly w silnikach baz, koordynacja wielu kart. Brak wyników nie dowodzi braku publikacji.
4. Prace z 2026 roku to w większości preprinty, a materiały konferencji z drugiej połowy 2026 roku mogły nie być jeszcze zaindeksowane.
5. Przeszukiwano literaturę anglojęzyczną; z polskiej uwzględniono tylko tłumaczenie książki Kleppmanna.
6. Źródła branżowe są obarczone konfliktem interesów i zmieniają się bez wersji.
7. Świadomie pominięto wyszukiwanie pełnotekstowe i wektorowe, interfejs C dla aplikacji natywnych, zużycie energii na telefonach i opóźnienia sieci komórkowych. Literaturę tych obszarów trzeba zebrać przy rozpoczęciu odpowiednich zadań.
