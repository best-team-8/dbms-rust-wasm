# Flat-SQL na tle literatury

Dokument zestawia założenia i wyniki prototypu (gałąź `dbms-2-engine-prototype`, dokumenty [architecture.md](../architecture.md), [flat-sql.md](../flat-sql.md), [README](../../README.md), [docs/research](../research/)) z publikacjami zebranymi w [bazie wiedzy](README.md). Identyfikatory w nawiasach kwadratowych (np. [W1]) odsyłają do wpisów w indeksie. Stan na 18 października 2026.

Ocena w tabelach:

- **zgodne** – rozwiązanie odpowiada temu, co literatura uznaje za poprawne lub sprawdzone,
- **częściowo** – kierunek zgodny, ale z istotnym zastrzeżeniem albo brakującym elementem,
- **luka** – literatura opisuje problem, którego projekt nie rozwiązuje albo którego dokumentacja nie wymienia.

## 1. Podsumowanie

| # | Obszar | Założenie / wynik projektu | Literatura | Ocena |
|---|---|---|---|---|
| 1 | Narzut WebAssembly (H2) | 1,1–1,8× wolniej niż natywnie w operacjach wewnątrz silnika | [P2]: prawie wszystkie jądra < 2×; [W1]: średnio 1,45–1,55×, maks. 2,5× | zgodne |
| 2 | Granica JS↔WASM (H3) | JSON 1,9× szybszy od obiektów `js-sys` dla jednego wiersza | [P2] nie mierzy; [D1]: wyniki przekazywane jako Arrow IPC prosto ze sterty WASM; [E8]: wysoki narzut serializacji I/O | zgodne, **wkład własny** |
| 3 | Izolacja i Privacy by Design | dane przetwarzane „w izolowanym środowisku przeglądarki” | [P2]: izolacja chroni hosta, nie dane modułu; [W2], [W9]; [L1] i [V1]: szyfrowanie end-to-end, ochrona w całym cyklu życia | częściowo |
| 4 | Siedem ideałów local-first | szybkość, offline, synchronizacja | [L1] | 3 zgodne, 4 częściowo (§3) |
| 5 | Zegar HLC | 48 bitów ms + 16 bitów licznika, wchłanianie obcych znaczników | [C2]: ta sama postać zwarta, ale także reguła odrzucania znaczników zbyt daleko w przyszłości; [P1] rozdz. 8 | luka |
| 6 | LWW per wiersz | deterministyczna zbieżność bez koordynacji | [L2] rejestr LWW; [P1] rozdz. 5: zbieżność kosztem utraty współbieżnych zapisów; [L24], [X5], [Q10]: granulacja kolumny/komórki | częściowo |
| 7 | Transakcje a replikacja | transakcja = jedna ramka WAL, atomowa lokalnie | [C8]: Monotonic Atomic View osiągalne bez koordynacji; outbox Flat-SQL zachowuje granice transakcji tylko wtedy, gdy mieści się w jednej paczce | częściowo |
| 8 | Ewolucja schematu w replikach | „replikowana w uproszczony sposób” | [L1], [L9], [L10], [L26]: problem otwarty | luka (znana) |
| 9 | Model „bez JOIN” | upraszcza wykonanie i replikację | [C7], [C6], [C4]: encja pod kluczem jako jednostka atomowości; [M1], [D4]: brak złączeń to koszt; [L8]: trudność replikacji leży w więzach, nie w złączeniach | częściowo |
| 10 | Baza w pamięci + WAL + snapshot, group commit | trwałość przyrostowa, koszt ∝ zmianie | [S1], [S4], [S5], [S6], [P1] rozdz. 3 | zgodne |
| 11 | Trwałość w przeglądarce | „po `await flush()` dane są trwałe” | [Q1]: od Chrome 121 domyślnie `relaxed` – trwałe wobec awarii karty, nie systemu | częściowo (doprecyzować) |
| 12 | Backend plikowy | `tmp` + `fsync` + `rename` + obcięcie WAL | [S8]: bez `fsync` katalogu kolejność `rename` i obcięcia WAL nie jest gwarantowana | luka |
| 13 | Usuwanie danych przez przeglądarkę | dane w IndexedDB „na zawsze” | [L1], [Q2], [Q3], [Q4]: magazyn best-effort, 7-dniowy limit Safari | luka |
| 14 | Local-First vs Cloud-First (H1) | µs lokalnie vs RTT w chmurze | [E1], [E2], [D4] (próg 100 ms, cel 16 ms), [E7] | zgodne |
| 15 | Metodyka benchmarków | rozgrzewka, kontrola wyników, JSON | [B3]: zimne i gorące przebiegi osobno; [B1], [B2]: wiele uruchomień i przedziały ufności; [B5]: rozkład Zipfa | częściowo |

## 2. WebAssembly jako platforma silnika

### 2.1 Narzut względem kodu natywnego (H2)

Praca twórców standardu [P2] pokazała wydajność bliską natywnej na PolyBenchC (7 z 24 jąder w granicy 10%, prawie wszystkie poniżej 2×). Jangda i in. [W1] na pełnych programach SPEC CPU zmierzyli średnio 1,55× (Chrome) i 1,45× (Firefox), z maksimum 2,5×, i przypisali narzut głównie większej liczbie dostępów do pamięci (presja na rejestry), dodatkowym skokom (m.in. kontrola przepełnienia stosu przy każdym wywołaniu) i większemu kodowi.

Wynik Flat-SQL – 1,1–1,8× dla operacji wewnątrz silnika ([benchmark-results.md §6](../research/benchmark-results.md)) – mieści się w tym przedziale. Co ważniejsze, obserwacja z §5 wyników (rekurencyjny `eval` z ramką ~500 B zapisywał rejestry przy każdym wywołaniu, a obsługa liści wyrażeń bez wywołania funkcji skróciła porównanie z ~90 do ~30 ns) jest praktycznym potwierdzeniem diagnozy z [W1]: w WASM każde wywołanie funkcji jest droższe niż natywnie, więc spłaszczanie gorących ścieżek opłaca się bardziej niż w kodzie natywnym.

**Wniosek:** zgodne. W pracy warto zestawić trzy liczby: [P2], [W1] i własny pomiar, oraz zaznaczyć, że narzut 4,3× dla PK lookup wynika z granicy JS↔WASM, a nie z samego WebAssembly (to rozróżnienie jest w literaturze rzadko mierzone).

### 2.2 Granica JS↔WASM (H3)

[P2] stawia prostą współpracę z platformą WWW za cel, ale nie mierzy kosztu przekazywania danych; [W1] analizuje jakość kodu generowanego przez kompilatory JIT, a narzut własnej infrastruktury (wywołania systemowe obsługiwane przez JS) ocenia jako pomijalny (średnio 0,2%) i nie bada go dalej. Nowsze prace o WASM poza przeglądarką wskazują wysoki narzut serializacji wejścia/wyjścia ([E8]: 7–10× względem kontenerów w trybie interpretowanym), a [E7] – że przeglądarka jest konkurencyjna właśnie wtedy, gdy dane wymieniane są w pamięci.

Pomiar H3 (JSON 1,9× szybszy od budowania obiektów JS przez `js-sys` dla wyników jednowierszowych) jest więc **wkładem własnym** w obszarze słabo pokrytym przez badania. Najbliższy precedens to DuckDB-Wasm [D1]: autorzy wskazują izolację pamięci WASM od sterty JS jako główną przeszkodę i rozwiązują ją, serializując wyniki jako strumienie Apache Arrow IPC, które JS czyta bezpośrednio ze sterty WASM. To potwierdza wniosek z H3 („JSON lub binarny format kolumnowy zamiast budowania obiektów przez `js-sys`”) i wskazuje konkretny format. Dodatkowo JS String Builtins (Wasm 3.0, [W10], [Q6]) zmniejszają koszt przekazywania tekstu.

### 2.3 Rozmiar modułu, kompilacja i rozgrzewka

[P2] zakłada zwartą reprezentację (62,5% rozmiaru asm.js, 85,3% kodu x86-64) i szybką, strumieniową kompilację. Flat-SQL: 545 KiB (206 KiB gzip), kompilacja 2,7 ms w Node.js; serwer wysyła moduł z typem `application/wasm` ([server/server.js](../../server/server.js)), więc przeglądarka może kompilować go strumieniowo – zgodne.

Yan i in. [W11] pokazali, że optymalizacje LLVM nie są dostrojone do WASM i wyższy poziom optymalizacji nie zawsze daje szybszy kod. Flat-SQL buduje się z `opt-level = 3` i `wasm-opt -O3` – warto w benchmarku porównać przynajmniej `opt-level = 2` i `"s"` (mniejszy moduł, a w świetle [W11] niekoniecznie wolniejszy).

Rozgrzewka w benchmarku (bez niej pierwszy mierzony silnik wypadał do 2× gorzej przy insertach – efekt przejścia Liftoff → TurboFan) jest zgodna z zaleceniami [B1] i [B2]. [B2] pokazuje jednak, że w wielu parach maszyna wirtualna–benchmark stabilny stan nie następuje nawet po rozgrzewce – zob. §8.

### 2.4 Izolacja, bezpieczeństwo i Privacy by Design

README obiecuje „przetwarzanie wrażliwych danych w izolowanym środowisku przeglądarki bez wysyłania ich na serwer”. Literatura rozdziela tu trzy różne własności:

| Własność | Literatura | Stan Flat-SQL | Ocena |
|---|---|---|---|
| Moduł nie zaszkodzi hostowi | [P2]: pamięć liniowa oddzielona od kodu i stosu silnika; [W4]: dowodliwie bezpieczne implementacje | zapewnia przeglądarka | zgodne |
| Błąd w module nie zaszkodzi danym modułu | [W2]: w pamięci liniowej brak kanarków stosu, ASLR, stron ochronnych; [W9] (2026): błędy binarne prowadzą do SQL injection i XS-Leaks | rdzeń i `flatsql-wasm` bez `unsafe`, rdzeń bez zależności – większość klas ataków z [W2] nie dotyczy kodu w bezpiecznym Ruście | zgodne (warto to podkreślić w pracy) |
| Dane są chronione przed innymi | [L1] ideał 6: szyfrowanie end-to-end, serwer przechowuje tylko szyfrogramy; [L11]: szyfrowane typy replikowane; [V1]: prywatność domyślnie i ochrona w całym cyklu życia; [L18]: kontrola dostępu w systemach local-first | IndexedDB nie jest szyfrowana i jest dostępna dla każdego skryptu tej samej domeny (np. przy XSS); po włączeniu synchronizacji serwer otrzymuje dane jawnym tekstem; serwer nie ma uwierzytelniania (prototyp na `127.0.0.1`) | **częściowo** |

**Wniosek:** „Privacy by Design” w obecnym kształcie oznacza minimalizację przesyłanych danych (lokalne przetwarzanie, synchronizacja opcjonalna), a nie ochronę kryptograficzną. Uczciwe sformułowanie w pracy: dane są domyślnie lokalne (zasada 2 z [V1]); pełna realizacja wymagałaby szyfrowania paczek zmian kluczem użytkownika ([L11]) i uwierzytelniania klientów ([L18]). Szyfrowanie end-to-end paczek `FSB1` jest architektonicznie możliwe, jeśli serwer stanie się przekaźnikiem nieprzezroczystych zmian: do rozstrzygnięcia LWW wystarczy jawny znacznik i identyfikator wiersza (np. skrót klucza), a wartości mogą być zaszyfrowane. Kosztem jest utrata trybu Cloud-First (`/api/query`) i walidacji schematu po stronie serwera, który dziś utrzymuje pełną replikę.

### 2.5 Pamięć i skala danych

MVP z [P2] ma 32-bitowe adresy (do 4 GiB pamięci liniowej). Wasm 3.0 dodał memory64 ([W10]), ale bez sztuczki ze stronami ochronnymi sprawdzanie granic kosztuje ponad 100% czasu wykonania, a dwupoziomowe strony ochronne obniżają to do 12,7% ([W6]); twórcy SpiderMonkey zalecają memory64 tylko przy faktycznej potrzebie > 4 GiB ([Q6]).

Yan i in. [W11] zmierzyli też, że programy WASM zużywają znacznie więcej pamięci niż odpowiedniki w JS (pamięć liniowa rośnie, ale nie jest oddawana jak przy GC). Flat-SQL trzyma każdy wiersz jako osobną alokację w `BTreeMap` (snapshot 100 tys. wierszy: 5,27 MB na dysku, w pamięci wielokrotnie więcej). Scenariusz z README „plik CSV 500 MB w przeglądarce” jest więc poza zasięgiem obecnej reprezentacji – właściwym narzędziem jest DuckDB-Wasm [D1] (kolumnowy, czytający pliki fragmentami), co [wasm-engines-comparison.md §4](../research/wasm-engines-comparison.md) już przyznaje. **Rekomendacja:** usunąć ten scenariusz z listy zastosowań Flat-SQL albo opisać go jako przypadek dla innego silnika; w części o ograniczeniach powołać się na [W6] i [D1].

### 2.6 Determinizm

[P2] dopuszcza w arytmetyce zmiennoprzecinkowej jedynie niedeterministyczne wzorce bitowe NaN. Codec Flat-SQL zapisuje `f64` bit po bicie ([storage/codec.rs](../../crates/flatsql-core/src/storage/codec.rs)), a porządek `Value` traktuje NaN jako równe – logicznie repliki są zgodne. Jeśli w przyszłości stan replik będzie porównywany po skrócie bajtów (np. przy uzgadnianiu zbiorów [L14]), warto kanonizować NaN przy zapisie.

## 3. Ideały local-first

Ocena według siedmiu ideałów z [L1] (sekcje 2.1–2.7 artykułu):

| Ideał [L1] | Flat-SQL | Ocena | Uwagi z literatury |
|---|---|---|---|
| 1. Brak spinnerów | zapytanie po kluczu ~10 µs, synchronicznie (H1) | zgodne | [D4]: cel 16 ms na interakcję, próg 100 ms – zapytania lokalne zużywają ułamek budżetu |
| 2. Praca nie jest uwięziona na jednym urządzeniu | synchronizacja przez serwer, repliki zbieżne (H6) | zgodne | [L20]: modele hybrydowe z serwerem są w praktyce najbardziej wykonalne |
| 3. Sieć jest opcjonalna | Service Worker, IndexedDB, trwały outbox, test offline | zgodne, z zastrzeżeniem | [L1] sam ostrzega, że magazyn przeglądarki bywa czyszczony razem z ciasteczkami; zob. §6.6 |
| 4. Płynna współpraca | LWW na cały wiersz | częściowo | współbieżne edycje różnych kolumn jednego wiersza – jedna ginie; [L1] wskazuje CRDT jako podstawę współpracy, a [L2] opisuje typy (np. mapy rejestrów) scalające zmiany na poziomie pól (§4.2) |
| 5. Długowieczność danych | udokumentowane formaty binarne z nagłówkami (`FLATSQL1`, `FSB1`), wspólne dla Rust i Node.js, CLI otwiera katalog serwera | częściowo | brak polityki zgodności wersji formatów i schematu dla klientów, którzy długo byli offline ([P1] rozdz. 4, [L9]) |
| 6. Bezpieczeństwo i prywatność domyślnie | dane lokalne domyślnie | częściowo | brak szyfrowania i uwierzytelniania (§2.4) |
| 7. Pełna kontrola użytkownika | eksport stanu (`exportState`, snapshot), przełącznik synchronizacji w demo | częściowo | dane w IndexedDB są w praktyce dostępne tylko przez aplikację; [L1] postuluje możliwość eksportu w otwartym formacie (np. JSON – jest `changes_to_json`, można go udostępnić jako eksport) |

## 4. Synchronizacja i rozwiązywanie konfliktów

### 4.1 Zegar HLC

Implementacja ([sync.rs](../../crates/flatsql-core/src/sync.rs)) odpowiada zwartej postaci z [C2]: 48 bitów czasu fizycznego i 16 bitów licznika w jednej liczbie 64-bitowej (w artykule – bity NTP, w projekcie – milisekundy), monotoniczność przy cofnięciu zegara i wchłanianie obcych znaczników (`observe`). Porządek `(time, node)` to porządek całkowity Lamporta [C1].

Czego brakuje: [C2] (sekcja 4) przewiduje regułę ignorowania wiadomości, które odsunęłyby zegar logiczny zbyt daleko od czasu fizycznego, oraz reset do czasu fizycznego z zalogowaniem incydentu. W Flat-SQL `observe` przyjmuje dowolną wartość, a serwer (`/sync/push`) nie sprawdza znaczników. Skutki:

- urządzenie z zegarem ustawionym np. rok do przodu wygrywa **wszystkie** konflikty przez ten rok,
- przez wchłanianie znaczników „przyszłość” rozlewa się na wszystkie repliki – zegary wszystkich klientów przeskakują,
- Kleppmann [P1, rozdz. 8] wprost zaznacza, że zegarom urządzeń, których nie kontrolujemy, nie można ufać (użytkownicy celowo zmieniają czas).

**Rekomendacja:** stała `MAX_DRIFT` (np. 60 s); serwer odrzuca lub oznacza zmiany ze znacznikiem `> czas_serwera + MAX_DRIFT`, a klient nie wchłania takich znaczników i sygnalizuje problem z zegarem. To zmiana kilkunastu linii w `Hlc::observe` i w obsłudze `/sync/push`, a zamyka scenariusz, którego test losowy nie pokrywa (zegary w testach są poprawne).

### 4.2 LWW per wiersz na tle CRDT

Model replikacji Flat-SQL to mapa rejestrów LWW indeksowana kluczem głównym, z tombstone'ami usunięć – formalnie typ opisany w [L2] (rejestr LWW, specyfikacje 8–9) i [L3]. Scalanie przez maksimum w porządku całkowitym jest łączne, przemienne i idempotentne, więc zbieżność (silna spójność ostateczna) wynika z teorii CRDT, a test `randomized_offline_sessions_converge` jest jej empirycznym potwierdzeniem; formalny dowód w stylu [L4] lub specyfikacja w [L19] byłyby naturalnym rozszerzeniem. Wysyłanie bieżącego stanu zmienionych wierszy zamiast historii operacji to synchronizacja delta-state [L6] – 100 kolejnych `UPDATE` jednego wiersza daje jedną zmianę.

Zastrzeżenie dotyczy granulacji. [P1, rozdz. 5] opisuje LWW jako zbieżność kosztem utraty współbieżnych zapisów. W Flat-SQL:

| Scenariusz offline | Wynik | Rozwiązania w literaturze i praktyce |
|---|---|---|
| A zmienia `qty`, B zmienia `player` w tym samym wierszu | zostaje cały wiersz z późniejszym znacznikiem, druga zmiana ginie bez śladu | LWW per kolumna / komórka: CRR [L24], cr-sqlite, Teilen-SQL [X5], zalecenie praktyków [Q10] |
| A i B zwiększają licznik o 1 | wynik +1 zamiast +2 | licznik CRDT (PN-Counter, [L2]) |
| wspólna edycja pola tekstowego | wygrywa jedna wersja | CRDT tekstu / Eg-walker [L16] |

**Rekomendacja:** w pracy wskazać, dla jakich danych LWW na wiersz jest właściwe (ekwipunek, ustawienia, katalog – dane rzadko edytowane współbieżnie) i jako kierunek rozwoju podać znaczniki per kolumna. Przechowywanie `Stamp` dla każdej kolumny zwiększa rozmiar wiersza, ale nie zmienia protokołu (paczka nadal niesie wiersz, tylko z wektorem znaczników).

### 4.3 Granice transakcji przy replikacji

Lokalnie transakcja jest atomowa (undo log, jedna ramka WAL). W synchronizacji ta granica nie jest zachowywana: `export_outbox(max)` ([db.rs](../../crates/flatsql-core/src/db.rs)) bierze do `max` wpisów w kolejności `seq` (`batchSize` = 1000 w [bridge/sync.js](../../bridge/sync.js)), a wpis wiersza modyfikowanego ponownie przesuwa się na koniec kolejki. Gdy outbox jest większy niż jedna paczka – typowo po dłuższej pracy offline – zmiany jednej transakcji mogą trafić do różnych paczek, a inne repliki mogą przez chwilę widzieć stan, który nigdy nie istniał w żadnej replice (np. wymiana przedmiotów między dwoma graczami, czyli dwa wiersze, widoczna tylko w połowie). Dopóki cały outbox mieści się w jednej paczce, problem nie występuje, bo serwer stosuje paczkę jako jedną transakcję, a klient – całą odpowiedź `pull`.

Bailis i in. [C8] pokazują, że **Monotonic Atomic View** (widoczne wszystkie albo żadne zapisy transakcji) da się zapewnić bez koordynacji, a więc bez utraty pracy offline; niemożliwe jest dopiero zapobieganie utraconym aktualizacjom. Helland [C7] i Bigtable [C6] przyjmują atomowość jednej encji (jednego wiersza) – to dokładnie gwarancja, którą Flat-SQL daje dziś między replikami.

**Rekomendacja:** albo udokumentować gwarancję wprost („transakcje są atomowe lokalnie; między replikami atomowa jest zmiana pojedynczego wiersza”), albo grupować outbox po transakcjach (`seq` początku i końca transakcji, eksport tylko pełnych grup) – serwer i tak stosuje każdą paczkę jako jedną transakcję (`apply_changes`), więc wystarczy, by paczka nie dzieliła transakcji klienta.

### 4.4 Usunięcia, tombstone'y i unikalność klucza

- Tombstone ze znacznikiem chroni przed „wskrzeszeniem” wiersza przez spóźniony `UPDATE` – zgodne z [L2]. Antidote SQL [L8] pozwala wybrać per tabela semantykę update-wins lub delete-wins; Flat-SQL stosuje czysty porządek znaczników (późniejsza operacja wygrywa), co przy braku kluczy obcych jest spójne i proste.
- CRR [L24] rozwiązuje usunięcia inaczej: zamiast tombstone'a ze znacznikiem każdy wiersz ma „długość przyczynową” – licznik zwiększany przy każdym wstawieniu i usunięciu (nieparzysta = wiersz istnieje, scalanie = maksimum). Dzięki temu ponowne wstawienie po usunięciu nie zależy od zegarów.
- Tombstone'y nie są usuwane (usuwa je tylko ponowne wstawienie wiersza). [L7] podaje kryterium bezpiecznego usuwania – stabilność przyczynową: tombstone można usunąć, gdy wszystkie repliki potwierdziły znacznik. Przy serwerze centralnym wystarczy, że serwer zna minimalny potwierdzony kursor wszystkich aktywnych klientów; klienci nieaktywni dłużej niż próg dostają pełny stan (`exportState`), co już jest obsłużone.
- **Unikalność klucza głównego jest lokalna.** Dwa urządzenia offline wstawiające wiersz o tym samym `id` nie dostaną błędu `constraint` – po synchronizacji jeden wiersz po cichu zastąpi drugi. [P1, rozdz. 9] wyjaśnia, że ograniczenia unikalności między replikami wymagają konsensusu, a rozdz. 5 – że LWW jest bezpieczne głównie dla kluczy zapisywanych raz. CRR [L24] wykrywa takie naruszenie przy scalaniu i jawnie cofa przegrywającą zmianę (wygrywa wcześniejsza, jak przy wykonaniu sekwencyjnym) – Flat-SQL robi odwrotnie (wygrywa późniejsza) i niczego nie zgłasza. **Rekomendacja:** w [flat-sql.md](../flat-sql.md) zalecić klucze generowane bez kolizji (UUID/ULID albo prefiks identyfikatora węzła) dla tabel, do których wstawia się offline.

### 4.5 Protokół synchronizacji

| Element Flat-SQL | Odpowiednik w literaturze | Ocena |
|---|---|---|
| zapisy lokalne + serwer porządkujący log (`seq`) | Bayou [C3]: zapisy do dowolnej repliki, serwer główny ustala kolejność zatwierdzeń, anti-entropy | zgodne (LWW zamiast procedur scalania aplikacji) |
| kursor klienta, epoka serwera, pełny stan przy nieznanym kursorze | offsety konsumentów i CDC ze snapshotem początkowym [P1, rozdz. 11] | zgodne |
| outbox ze scalaniem kolejnych zmian wiersza, kompaktowanie logu serwera | kompaktowanie logu po kluczu [P1, rozdz. 11], delta-state [L6] | zgodne |
| powtarzalne, idempotentne rundy (równe znaczniki ignorowane) | idempotencja wiadomości [C7], argument end-to-end [P1, rozdz. 12] | zgodne |
| pull z wykładniczym odstępem | [P1, rozdz. 12]: model push dla klientów reagujących na zmiany | częściowo – dla współpracy w czasie rzeczywistym potrzebny push (WebSocket/SSE) |
| serwer przyjmuje każdą zmianę | [L12]: odporność na nieuczciwych uczestników; [L18]: kontrola dostępu | luka (prototyp) |

Alternatywy na przyszłość: synchronizacja bez serwera przez Merkle-DAG [L13] lub uzgadnianie zakresów [L14] zamiast pełnego `exportState`.

### 4.6 Ewolucja schematu

`CREATE TABLE` z innej repliki jest ignorowany, jeśli tabela o tej nazwie już istnieje – bez porównania schematów ([db.rs, `apply_remote`](../../crates/flatsql-core/src/db.rs)). Dwie repliki, które offline utworzą tabelę o tej samej nazwie i różnych kolumnach, rozjadą się trwale: wiersze drugiej repliki będą odrzucane (`rejected`) przez `check_row`. To przykład problemu, który [L1] (brak „autorytatywnego” schematu bez centralnego serwera), [L9] i [L10] opisują jako otwarty, a [L26] (Cambria) rozwiązuje soczewkami tłumaczącymi dane między wersjami schematu.

**Rekomendacja:** w prototypie – wykrywać konflikt schematu i zgłaszać go jawnie (zamiast cichego odrzucania wierszy); w pracy – opisać jako świadome ograniczenie z odwołaniem do [L9]; w dalszych pracach – wersjonowanie schematu tabel i migracje deklarowane przez aplikację.

## 5. Model danych Flat-SQL („bez JOIN”)

Argumenty **za** płaskim modelem mają mocne oparcie w literaturze systemów rozproszonych:

- Helland [C7]: encja z unikalnym kluczem jest naturalnym zakresem serializowalności, a transakcje atomowe nie powinny obejmować wielu encji;
- Bigtable [C6]: atomowe operacje jednowierszowe;
- Dynamo [C4]: wiele usług potrzebuje wyłącznie dostępu po kluczu głównym;
- Cassandra [C5] i jej CQL: składnia SQL, obowiązkowy klucz, brak złączeń – przemysłowy odpowiednik Flat-SQL.

Argumenty **przeciw** pochodzą z literatury baz danych i aplikacji local-first:

- Stonebraker i Pavlo [M1]: systemy NoSQL niemal powszechnie dodały SQL, a systemy column-family odziedziczyły po Bigtable brak złączeń i indeksów wtórnych – co uznają za ograniczenie, nie zaletę;
- Riffle [D4]: w aplikacji local-first znaczna część kodu UI to złączenia i grupowania (np. utwory–albumy–artyści), a ich szybkość decyduje o responsywności;
- [P1, rozdz. 2]: relacje wiele-do-wielu bez złączeń przenoszą pracę do aplikacji.

Ważna obserwacja dla uzasadnienia w pracy: **granulacja replikacji nie zależy od języka zapytań**. CRR [L24] i cr-sqlite replikują wiersze/kolumny tabel relacyjnych, a Antidote SQL [L8] pokazuje, że prawdziwa trudność to więzy integralności (klucze obce, kaskady) przy współbieżnych usunięciach – nie same złączenia. Złączenia tylko do odczytu (bez `FOREIGN KEY`) nie skomplikowałyby synchronizacji Flat-SQL.

**Rekomendacja:** uzasadniać brak JOIN prostotą i przewidywalnością wykonania (jedna tabela, jeden plan dostępu) oraz brakiem więzów między tabelami (co upraszcza replikację), a nie samą replikacją. Jako dalsze prace: proste złączenie równościowe po indeksie bez więzów oraz zapytania reaktywne w stylu [D4].

## 6. Składowanie i trwałość

### 6.1 Baza w pamięci + WAL + snapshot

Wzorzec Flat-SQL (dane w pamięci, log tylko-redo z CRC, okresowy pełny snapshot, odtwarzanie = snapshot + ramki o większym LSN) jest klasyczny:

- DeWitt i in. [S1] – odtwarzanie baz w pamięci przez log i checkpointy;
- [P1, rozdz. 3] – bazy w pamięci zawdzięczają szybkość głównie brakowi kodowania struktur do formatu dyskowego, a trwałość zapewniają logiem i snapshotami;
- H-Store [S4] i analiza Harizopoulosa [S5] – w klasycznym silniku OLTP większość instrukcji idzie na logowanie, blokady, latche i bufor stron; jednowątkowy silnik w pamięci eliminuje trzy z czterech. Flat-SQL płaci tylko za logowanie (undo log + ramka WAL + wpis outboksu) – co tłumaczy wolniejsze od SQLite inserty i aktualizacje w H4;
- ARIES [S2] – Flat-SQL nie potrzebuje fazy undo na dysku, bo niezatwierdzone zmiany nigdy nie trafiają do WAL (bufor transakcji staje się ramką dopiero przy `COMMIT`).

Wybór logu fizycznego zamiast logowania poleceń: Malviya i in. [S6] mierzą ok. 1,5× większą przepustowość logowania poleceń kosztem dłuższego odtwarzania i wymogu deterministycznego wykonania. W Flat-SQL log fizyczny jest lepszy z dwóch powodów: wpisy WAL służą jednocześnie synchronizacji (zmiana = pełny wiersz + znacznik), a odtworzenie nie zależy od zegara ani od wyników wcześniejszych zapytań. **Zgodne**; warto ten argument przytoczyć w pracy.

Koszt checkpointu (kopiowanie całej bazy, 35 ms dla 3 MB) i odtwarzania (200 ms dla 100 tys. wierszy) ma znane kierunki optymalizacji: równoległe odtwarzanie [S7], struktury LSM [S3] lub snapshoty przyrostowe.

Dobrym kontrastem jest Zeta-Lite [D5] (preprint, 2026): przeglądarkowy silnik SQL o tej samej architekturze (jeden rdzeń dla serwera i przeglądarki, `wasm32-unknown-unknown` + `wasm-bindgen`), który zapisuje do OPFS tylko snapshot całej bazy na żądanie. Autor wprost przyznaje, że zmiany po ostatnim snapshocie giną przy zamknięciu karty. Flat-SQL rozwiązuje to przyrostowym WAL – to mocny argument na rzecz wyboru z H5.

### 6.2 Group commit

Zatwierdzenia z jednego „tiku” pętli zdarzeń zapisywane jednym zapisem do IndexedDB to group commit opisany po raz pierwszy w druku przez DeWitta i in. [S1]. Wynik H5 (1000 zatwierdzeń + jeden `flush()` = 5,6 µs na zatwierdzenie) jest dobrą ilustracją. **Zgodne.**

### 6.3 Trwałość w przeglądarce (IndexedDB)

`IndexedDbStore` otwiera transakcje z `durability: 'default'` ([bridge/stores/indexeddb.js](../../bridge/stores/indexeddb.js)). Od Chrome 121 domyślna trwałość to `relaxed` – zdarzenie `complete` następuje po przekazaniu danych do bufora systemu operacyjnego, a nie po zapisie na dysk ([Q1]); Firefox i Safari działały tak wcześniej. Zatem po `await db.flush()`:

- dane przetrwają zamknięcie karty i awarię przeglądarki,
- mogą zginąć przy awarii systemu lub zaniku zasilania.

Analogia z `synchronous=NORMAL` w SQLite z [architecture.md §4.3](../architecture.md) jest trafna, ale zdanie „po `await flush()` dane są trwałe” należy doprecyzować. `IndexedDbStore` już przyjmuje opcję `durability: 'strict'` – warto ją opisać w dokumentacji i zmierzyć jej koszt w H5 (spodziewany wzrost latencji pojedynczego zatwierdzenia). **Częściowo.**

### 6.4 Backend plikowy (CLI, serwer)

`FileStorage::write_snapshot` ([storage/file.rs](../../crates/flatsql-core/src/storage/file.rs)) wykonuje: zapis `snapshot.fsql.tmp` → `sync_all` → `rename` → `set_len(0)` na WAL → `sync_all` WAL. Pillai i in. [S8] pokazali, że kolejność utrwalania operacji na **różnych plikach** nie jest gwarantowana bez `fsync` katalogu po `rename`. Możliwy stan po zaniku zasilania: katalog nadal wskazuje stary snapshot (rename nieutrwalony), a WAL jest już pusty (obcięcie utrwalone) – utrata transakcji zatwierdzonych między dwoma checkpointami. Test „każdy prefiks WAL” tego nie wykryje, bo dotyczy jednego pliku.

`FsStore` w Node.js ([bridge/stores/fs.js](../../bridge/stores/fs.js)) nie wywołuje `fsync` w ogóle, choć tabela w [architecture.md §4.2](../architecture.md) przypisuje mu „opcjonalnie `fsync`” – rozbieżność dokumentacji.

**Rekomendacja:** po `rename` wykonać `fsync` katalogu (Linux/macOS) lub użyć zapisu z flagą write-through (Windows), a dopiero potem obcinać WAL; alternatywnie nie obcinać WAL, tylko zaczynać nowy plik WAL i usuwać stary po utrwaleniu snapshotu (ramki o LSN ≤ snapshot i tak są pomijane). W `FsStore` dodać `FileHandle.sync()` i poprawić tabelę w dokumentacji. **Luka.**

### 6.5 Testy odtwarzania po awarii

Model awarii z [S9] – utrwalony jest prefiks strumienia operacji wejścia/wyjścia, reszta ginie – odpowiada testowi `every_wal_prefix_recovers_to_a_committed_state` (ucięcie WAL w każdym bajcie). To dobra praktyka: w [S9] wszystkie 8 badanych baz zachowało się w tym modelu błędnie. Brakuje testu protokołu checkpointu między plikami w stylu ALICE [S8] (symulacja stanów: rename utracony / obcięcie utrwalone itd.). **Zgodne, do rozszerzenia.**

### 6.6 Usuwanie danych przez przeglądarkę

Lokalne dane w przeglądarce nie są gwarantowane:

- [L1] przypomina, że wyczyszczenie danych witryny usuwa też magazyn lokalny;
- Safari usuwa IndexedDB, LocalStorage i rejestracje Service Workera po 7 dniach używania przeglądarki bez interakcji użytkownika z witryną; aplikacje dodane do ekranu głównego mają własny licznik ([Q2]);
- magazyn jest domyślnie „best-effort” i może zostać usunięty przy braku miejsca; tryb trwały wymaga `navigator.storage.persist()` ([Q3], [Q4]).

W kodzie nie ma wywołania `navigator.storage.persist()`. Dla Local-First oznacza to, że **niewysłany outbox może zniknąć bez winy aplikacji**, co podważa ideał 3 i 7 z [L1]. **Rekomendacja:** wywołać `navigator.storage.persist()` przy pierwszym zapisie i pokazać stan w demo; w pracy opisać ograniczenie i wskazać instalację jako PWA oraz synchronizację jako zabezpieczenie. **Luka.**

## 7. Edge computing i Local-First vs Cloud-First (H1)

Literatura edge computing ([E1], [E2]) przenosi obliczenia bliżej użytkownika, by skrócić latencję, egzekwować polityki prywatności i maskować awarie chmury. Flat-SQL idzie krok dalej – wykonuje zapytania na urządzeniu. Uruchomienie tego samego modułu `.wasm` na serwerze odpowiada wizji WASM jako wspólnej warstwy kontinuum chmura–brzeg [E6] i pracom o lekkiej izolacji WASM na brzegu ([E3], [E4], [E5]). Prototyp ma jednak jeden centralny serwer, który jest jednocześnie hubem synchronizacji i bazą główną; zakładanych w prezentacji węzłów brzegowych synchronizowanych z bazą główną jeszcze nie ma ([conclusions.md §3.6](conclusions.md#36-edge-computing), [todo.md 4.10](../todo.md)).

Wynik H1 (11 µs lokalnie vs 20,9–102,5 ms przy RTT 20–100 ms) jest zgodny z tym, co przewiduje literatura: czas zapytania sieciowego jest ograniczony od dołu przez RTT. Riffle [D4] daje dobry punkt odniesienia dla interpretacji: przy budżecie 16 ms na klatkę jedno zapytanie przez sieć z RTT 20 ms już go przekracza. Colosi i in. [E7] pokazują ponadto, że przeglądarka jest konkurencyjna przy małych danych (wymiana w pamięci), a serwery z kompilacją AOT wygrywają przy dużych – co wspiera podział: dane operacyjne lokalnie (Flat-SQL), ciężka analityka gdzie indziej (DuckDB-Wasm [D1] albo serwer). **Zgodne.**

Uwaga metodyczna: symulacja opóźnienia w [benchmark-scenario.md](../research/benchmark-scenario.md) jest uczciwie opisana jako dolne ograniczenie (bez TLS, strat pakietów i zestawiania połączeń). Prace o sieciach mobilnych pokazałyby jeszcze większą przewagę Local-First – ale nie zostały zebrane w tej bazie (zob. [search-methodology.md](search-methodology.md)).

## 8. Metodyka benchmarków

Zestawienie z listą pułapek [B3] i pracami o pomiarach w środowiskach z JIT:

| Pułapka / zalecenie | Stan w projekcie | Ocena | Co zrobić |
|---|---|---|---|
| Powtarzalność [B3 §3.1], [B4] | skrypty, wersje, stałe ziarno, wyniki w JSON | zgodne | dołączyć surowe JSON do pracy (załącznik) |
| Brak optymalizacji systemu porównywanego [B3 §3.2] | SQLite w najlepszym układzie (`INTEGER PRIMARY KEY`), prepared statements | zgodne | – |
| Jabłka i pomarańcze [B3 §3.3] | trwałość porównywana z eksportem sql.js – opisane w zagrożeniach | częściowo | dodać SQLite WASM z VFS OPFS (`opfs-sahpool`, [Q7]) jako uczciwy punkt odniesienia dla H5 |
| Przebiegi zimne i gorące [B3 §3.5–3.6] | raportowane przebiegi po rozgrzewce; start częściowo (kompilacja, otwarcie bazy) | częściowo | osobna tabela „zimny start”: pobranie + kompilacja + otwarcie bazy + pierwsze zapytanie |
| Niepoprawne wyniki [B3 §3.8] | porównanie liczby wierszy | częściowo | porównywać sumę kontrolną wyników, nie tylko liczbę wierszy |
| Wiele uruchomień, przedziały ufności [B1] | jeden przebieg na konfigurację; szum ±30% opisany | częściowo | ≥ 10 uruchomień procesu, średnia z 95% przedziałem ufności lub mediana z IQR |
| Stabilny stan po rozgrzewce [B2] | stała rozgrzewka | częściowo | sprawdzić stabilność (np. wykres czasu kolejnych iteracji) dla najkrótszych operacji |
| Realistyczny rozkład dostępu [B5], [B4], [W7] | klucze losowane równomiernie | częściowo | dodać obciążenie w stylu YCSB (np. 95% odczytów, rozkład Zipfa) oraz pomiar na scenariuszu demo |
| Rozdzielczość zegara [B6], [Q5] | 100 µs w przeglądarce, opisane | zgodne | strona benchmarku z nagłówkami COOP/COEP daje 5 µs |

## 9. Rekomendacje

**Priorytet 1 – poprawność i rzetelność tez (małe zmiany):**

1. Ograniczenie dryfu HLC: odrzucanie znaczników z przyszłości po stronie serwera i klienta (§4.1) – `crates/flatsql-core/src/sync.rs`, `server/server.js`.
2. Kolejność `rename` → `fsync` katalogu → obcięcie WAL w `FileStorage`; `fsync` w `FsStore`; poprawka tabeli backendów (§6.4).
3. Doprecyzowanie gwarancji trwałości w przeglądarce (`relaxed` vs `strict`) w [architecture.md §4.3](../architecture.md) i pomiar `strict` w H5 (§6.3).
4. `navigator.storage.persist()` w `FlatDB.open` lub w demo (§6.6).
5. Usunięcie scenariusza „CSV 500 MB” z zastosowań Flat-SQL albo przypisanie go DuckDB-Wasm (§2.5).

**Priorytet 2 – semantyka synchronizacji:**

6. Zachowanie granic transakcji także przy outboksie większym niż jedna paczka (MAV, §4.3) lub jawny opis gwarancji „atomowość per wiersz”.
7. Zalecenie kluczy bezkolizyjnych dla wstawień offline w [flat-sql.md](../flat-sql.md) (§4.4).
8. Jawne wykrywanie konfliktów schematu (§4.6) i wersja formatu w protokole synchronizacji (§3, ideał 5).
9. Znaczniki per kolumna jako opcja tabeli (§4.2).

**Priorytet 3 – dalsze prace (do rozdziału „kierunki rozwoju”):**

10. Szyfrowanie end-to-end paczek zmian i uwierzytelnianie klientów (§2.4, [L11], [L18]).
11. Zapytania reaktywne i proste złączenia bez więzów (§5, [D4]).
12. Push zamiast pull w synchronizacji, usuwanie tombstone'ów po stabilności (§4.4–4.5).
13. Rozszerzenie metodyki benchmarków według §8.

## 10. Co projekt wnosi względem literatury

Uczciwie: żaden pojedynczy element nie jest nowy – LWW na HLC, WAL ze snapshotem, group commit, outbox i log z kursorami mają dobrze opisane odpowiedniki. Wartość prototypu leży w ich **połączeniu i pomiarze**:

1. jeden rdzeń bez zależności i bez I/O, uruchamiany identycznie w przeglądarce, na węźle edge i natywnie – praktyczna realizacja postulatu z [E6] (podobną drogą idzie Zeta-Lite [D5], ale bez przyrostowej trwałości i synchronizacji, za to z pełnym SQL i modułem ok. 14× większym po kompresji);
2. pomiar kosztu granicy JS↔WASM dla silnika bazy danych (H3) – obszar, którego nie obejmują [P2] ani [W1];
3. pomiar przyrostowej trwałości w IndexedDB z group commitem wobec eksportu całej bazy (H5);
4. dialekt SQL celowo ograniczony do modelu encji pod kluczem (w duchu [C7] i CQL), z wbudowaną synchronizacją – w odróżnieniu od CRR [L24], [L25], cr-sqlite czy Teilen-SQL [X5], które dodają warstwę replikacji do pełnej bazy relacyjnej (PostgreSQL przez Ecto w [L24], SQLite w [L25] i cr-sqlite).

Porównując się z tymi systemami, warto podkreślić różnicę celu: one zachowują pełny SQL i dokładają warstwę synchronizacji, Flat-SQL rezygnuje z części SQL w zamian za prostszy silnik, mniejszy moduł i synchronizację wbudowaną w format WAL.
