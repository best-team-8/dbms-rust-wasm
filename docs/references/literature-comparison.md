# Flat-SQL na tle literatury

Dokument zestawia założenia i wyniki prototypu (gałąź `dbms-2-engine-prototype`, dokumenty [architecture.md](../architecture.md), [flat-sql.md](../flat-sql.md), [README](../../README.md), [docs/research](../research/)) z publikacjami z rdzenia [bazy wiedzy](README.md). Identyfikatory w nawiasach kwadratowych (np. [W1]) odsyłają do wpisów w indeksie; zadania – do [todo.md](../todo.md) (optymalizacja prototypu) i [todo_future.md](../todo_future.md) (rozszerzenia). Stan po rewizji z 20 października 2026.

Ocena w tabelach:

- **zgodne** – rozwiązanie odpowiada temu, co literatura uznaje za poprawne lub sprawdzone,
- **częściowo** – kierunek zgodny, ale z istotnym zastrzeżeniem albo brakującym elementem,
- **luka** – literatura opisuje problem, którego projekt nie rozwiązuje albo którego dokumentacja nie wymienia.

## 1. Podsumowanie

| # | Obszar | Założenie / wynik projektu | Literatura | Ocena |
|---|---|---|---|---|
| 1 | Narzut WebAssembly (H2) | 1,1–1,8× wolniej niż natywnie w operacjach wewnątrz silnika | [P2]: prawie wszystkie jądra < 2×; [W1]: średnio 1,45–1,55×, maks. 2,5× | zgodne |
| 2 | Granica JS↔WASM (H3) | JSON 1,9× szybszy od obiektów `js-sys` dla jednego wiersza | [P2], [W1] nie mierzą; [D1]: wyniki jako Arrow IPC czytane wprost ze sterty WASM | zgodne, **wkład własny** |
| 3 | Izolacja i Privacy by Design | dane przetwarzane „w izolowanym środowisku przeglądarki” | [P2], [W2]: izolacja chroni hosta, nie dane modułu; [L1], [V1]: szyfrowanie, ochrona w całym cyklu życia | częściowo |
| 4 | Siedem ideałów local-first | szybkość, offline, synchronizacja | [L1] | 3 zgodne, 4 częściowo (§3) |
| 5 | Zegar HLC | 48 bitów ms + 16 bitów licznika, wchłanianie obcych znaczników | [C2] §4: także ignorowanie znaczników zbyt daleko w przyszłości; [P1] rozdz. 8 | luka |
| 6 | LWW per wiersz | deterministyczna zbieżność bez koordynacji | [L2], [L28]: poprawny CRDT; [C9]: znaczniki czasu gubią aktualizacje; [L24]: LWW per atrybut | częściowo |
| 7 | Atomowość między replikami | transakcja atomowa lokalnie, między replikami – wiersz | [C7], [L27]: wiersz/encja jako jednostka atomowości; [C8]: MAV osiągalne bez koordynacji | zgodne (do udokumentowania) |
| 8 | Trwały outbox, praca offline | outbox odbudowywany z WAL, przetrwa restart | [C10]: trwały, skracany log reintegracji; [L27]: aplikacje bez tego tracą dane | zgodne |
| 9 | Log synchronizacji serwera | log w pamięci, nowa epoka po każdym restarcie | [P1] rozdz. 11, [C10]: trwały log z offsetami | luka |
| 10 | Ewolucja schematu w replikach | „replikowana w uproszczony sposób” | [L1], [L26]: problem otwarty, wzór soczewek | luka (znana) |
| 11 | Model „bez JOIN” | upraszcza wykonanie i replikację | [C7] za; [M1], [D4] przeciw; [M2]: przewidywalny koszt tylko dla zapytań ograniczonych; [L8]: trudność leży w więzach | częściowo |
| 12 | Baza w pamięci + WAL + snapshot, group commit | trwałość przyrostowa, koszt ∝ zmianie | [S1], [S4], [S6], [P1] rozdz. 3 | zgodne |
| 13 | Checkpoint | synchroniczna kopia całej bazy (35–49 ms) | [S10]: „naiwny snapshot” daje skoki latencji; są algorytmy bez przestoju | częściowo |
| 14 | Trwałość w przeglądarce | „po `await flush()` dane są trwałe” | [Q1]: od Chrome 121 domyślnie `relaxed` – trwałe wobec awarii karty, nie systemu | częściowo (doprecyzować) |
| 15 | Backend plikowy | `tmp` + `fsync` + `rename` + obcięcie WAL | [S8]: bez `fsync` katalogu kolejność `rename` i obcięcia WAL nie jest gwarantowana | luka |
| 16 | Usuwanie danych przez przeglądarkę | dane w IndexedDB „na zawsze” | [L1], [Q2]–[Q4]: magazyn best-effort, 7-dniowy limit Safari | luka |
| 17 | Local-First vs Cloud-First (H1) | µs lokalnie vs RTT w chmurze | [D4] (cel 16 ms, próg 100 ms), [E1] | zgodne |
| 18 | Metodyka benchmarków | rozgrzewka, kontrola wyników, JSON | [B3]: zimne i gorące przebiegi osobno; [B1]: wiele uruchomień; [B5]: rozkład Zipfa | częściowo |

## 2. WebAssembly jako platforma silnika

### 2.1 Narzut względem kodu natywnego (H2)

Praca twórców standardu [P2] pokazała wydajność bliską natywnej na PolyBenchC (7 z 24 jąder w granicy 10%, prawie wszystkie poniżej 2×). Jangda i in. [W1] na pełnych programach SPEC CPU zmierzyli średnio 1,55× (Chrome) i 1,45× (Firefox), z maksimum 2,5×, i przypisali narzut głównie większej liczbie dostępów do pamięci (presja na rejestry), dodatkowym skokom (m.in. kontrola przepełnienia stosu przy każdym wywołaniu) i większemu kodowi.

Wynik Flat-SQL – 1,1–1,8× dla operacji wewnątrz silnika ([benchmark-results.md §6](../research/benchmark-results.md)) – mieści się w tym przedziale. Obserwacja z §5 wyników (rekurencyjny `eval` z ramką ~500 B zapisywał rejestry przy każdym wywołaniu; obsługa liści wyrażeń bez wywołania funkcji skróciła porównanie z ~90 do ~30 ns) jest praktycznym potwierdzeniem diagnozy z [W1]: w WASM każde wywołanie jest droższe niż natywnie, więc spłaszczanie gorących ścieżek opłaca się bardziej.

**Wniosek:** zgodne. W pracy warto zestawić trzy liczby: [P2], [W1] i własny pomiar, oraz zaznaczyć, że narzut 4,3× dla odczytu po kluczu wynika z granicy JS↔WASM, a nie z samego WebAssembly.

### 2.2 Granica JS↔WASM (H3)

[P2] stawia prostą współpracę z platformą WWW za cel, ale nie mierzy kosztu przekazywania danych; [W1] ocenia narzut wywołań obsługiwanych przez JS jako pomijalny i nie bada go dalej. Ponowne wyszukiwanie (20.10.2026, [search-methodology.md](search-methodology.md)) nie znalazło recenzowanych pomiarów tej granicy dla silników baz danych.

Pomiar H3 (JSON 1,9× szybszy od budowania obiektów JS przez `js-sys` dla wyników jednowierszowych) jest więc **wkładem własnym**. Najbliższy precedens to DuckDB-Wasm [D1]: autorzy wskazują izolację pamięci WASM od sterty JS jako główną przeszkodę i serializują wyniki jako strumienie Apache Arrow IPC, które JS czyta bezpośrednio ze sterty WASM. To potwierdza kierunek „binarny bufor zamiast obiektów” ([todo.md 2.4](../todo.md)). Nie potwierdza natomiast wcześniejszej sugestii, że pomogą JS String Builtins z Wasm 3.0: mechanizm służy językom, które używają łańcuchów JS jako własnych (z GC), a Rust trzyma tekst w pamięci liniowej i kopiowania nie uniknie [Q13] – zadanie usunięto.

### 2.3 Rozmiar modułu, kompilacja i poziomy optymalizacji

[P2] zakłada zwartą reprezentację (62,5% rozmiaru asm.js, 85,3% kodu x86-64) i kompilację strumieniową. Flat-SQL: 545 KiB (206 KiB gzip), kompilacja 2,7 ms w Node.js; serwer wysyła moduł z typem `application/wasm`, a inicjalizacja `wasm-bindgen` kompiluje go strumieniowo – zgodne. Analiza wdrożeń potwierdza, że rozmiar ma znaczenie: Notion ładuje SQLite asynchronicznie, bo kilkaset KB biblioteki opóźniało start strony ([real-world-usage.md §3](../research/real-world-usage.md#3-wnioski-projektowe-z-cudzych-wdrożeń)).

Yan i in. [W11] pokazali, że optymalizacje LLVM nie są dostrojone do WASM i wyższy poziom optymalizacji nie zawsze daje szybszy kod. Flat-SQL buduje się z `opt-level = 3` i `wasm-opt -O3` – warto porównać przynajmniej `opt-level = 2`, `"s"` i `"z"` ([todo.md 2.5](../todo.md)).

Rozgrzewka w benchmarku (bez niej pierwszy mierzony silnik wypadał do 2× gorzej przy insertach – efekt przejścia Liftoff → TurboFan) jest zgodna z zaleceniami [B1]; wiele uruchomień procesu – zob. §8.

### 2.4 Izolacja, bezpieczeństwo i Privacy by Design

README obiecuje „przetwarzanie wrażliwych danych w izolowanym środowisku przeglądarki bez wysyłania ich na serwer”. Literatura rozdziela tu trzy własności:

| Własność | Literatura | Stan Flat-SQL | Ocena |
|---|---|---|---|
| Moduł nie zaszkodzi hostowi | [P2]: pamięć liniowa oddzielona od kodu i stosu silnika | zapewnia przeglądarka | zgodne |
| Błąd w module nie zaszkodzi danym modułu | [W2]: w pamięci liniowej brak kanarków stosu, ASLR, stron ochronnych | rdzeń i `flatsql-wasm` bez `unsafe`, rdzeń bez zależności – klasy ataków z [W2] w większości nie dotyczą bezpiecznego Rusta | zgodne (warto podkreślić w pracy) |
| Dane są chronione przed innymi | [L1] ideał 6: szyfrowanie end-to-end; [L11]: szyfrowane typy replikowane; [V1]: prywatność domyślnie i ochrona w całym cyklu życia | IndexedDB nie jest szyfrowana i jest dostępna dla każdego skryptu tej samej domeny (np. przy XSS); serwer synchronizacji dostaje dane jawnym tekstem i nie uwierzytelnia klientów | **częściowo** |

**Wniosek:** „Privacy by Design” oznacza dziś minimalizację przesyłanych danych (lokalne przetwarzanie, synchronizacja opcjonalna – zasada 2 z [V1]), a nie ochronę kryptograficzną ([todo.md 0.1](../todo.md)). Szyfrowanie end-to-end paczek `FSB1` jest architektonicznie możliwe, jeśli serwer stanie się przekaźnikiem nieprzezroczystych zmian: do rozstrzygnięcia LWW wystarczy jawny znacznik i skrót klucza, a wartości mogą być zaszyfrowane [L11]. Kosztem jest utrata `/api/query` i walidacji po stronie serwera dla zaszyfrowanych tabel ([todo_future.md 4.3–4.4](../todo_future.md)).

### 2.5 Pamięć i skala danych

MVP z [P2] ma 32-bitowe adresy (do 4 GiB pamięci liniowej). Wasm 3.0 dodał memory64, ale 64-bitowa pamięć jest wyraźnie wolniejsza i twórcy SpiderMonkey zalecają ją tylko przy faktycznej potrzebie > 4 GiB [Q6]. Yan i in. [W11] zmierzyli, że programy WASM zużywają znacznie więcej pamięci niż JS, bo pamięć liniowa rośnie, ale nie jest oddawana. Flat-SQL trzyma każdy wiersz jako osobną alokację w `BTreeMap` (snapshot 100 tys. wierszy: 5,27 MB na dysku, w pamięci wielokrotnie więcej). Scenariusz „plik CSV 500 MB w przeglądarce” jest poza zasięgiem tej reprezentacji – właściwym narzędziem jest DuckDB-Wasm [D1]. **Rekomendacja:** przypisać scenariusz DuckDB-Wasm ([todo.md 0.2](../todo.md)), zmierzyć pamięć ([todo.md 4.4](../todo.md)) i zmniejszyć ją zwartą reprezentacją wierszy ([todo.md 2.7](../todo.md)).

### 2.6 Determinizm

[P2] dopuszcza w arytmetyce zmiennoprzecinkowej jedynie niedeterministyczne wzorce bitowe NaN. Codec Flat-SQL zapisuje `f64` bit po bicie, a porządek `Value` traktuje NaN jako równe – logicznie repliki są zgodne. Przy porównywaniu replik po skrócie bajtów trzeba kanonizować NaN przy zapisie ([todo.md 1.8](../todo.md)).

## 3. Ideały local-first

Ocena według siedmiu ideałów z [L1]:

| Ideał [L1] | Flat-SQL | Ocena | Uwagi z literatury |
|---|---|---|---|
| 1. Brak spinnerów | zapytanie po kluczu ~10 µs, synchronicznie (H1) | zgodne | [D4]: cel 16 ms na interakcję – zapytania lokalne zużywają ułamek budżetu; wyjątek: checkpoint (§6.3) |
| 2. Praca nie jest uwięziona na jednym urządzeniu | synchronizacja przez serwer, repliki zbieżne (H6) | zgodne | [L20]: modele hybrydowe z serwerem są w praktyce najbardziej wykonalne |
| 3. Sieć jest opcjonalna | Service Worker, IndexedDB, trwały outbox, test offline | zgodne, z zastrzeżeniem | wzorzec pracy bez połączenia z [C10]; [L1] ostrzega, że magazyn przeglądarki bywa czyszczony (§6.7) |
| 4. Płynna współpraca | LWW na cały wiersz | częściowo | współbieżne edycje różnych kolumn jednego wiersza – jedna ginie [C9]; scalanie na poziomie pól [L24] |
| 5. Długowieczność danych | udokumentowane formaty binarne z nagłówkami (`FLATSQL1`, `FSB1`), wspólne dla Rust i Node.js | częściowo | brak polityki zgodności wersji formatów i schematu dla klientów długo offline ([P1] rozdz. 4, [L26]) |
| 6. Bezpieczeństwo i prywatność domyślnie | dane lokalne domyślnie | częściowo | brak szyfrowania i uwierzytelniania (§2.4) |
| 7. Pełna kontrola użytkownika | eksport stanu (`exportState`, snapshot), przełącznik synchronizacji w demo | częściowo | brak eksportu do pliku w otwartym formacie – jest `changes_to_json` ([todo_future.md 1.3](../todo_future.md)) |

## 4. Synchronizacja i rozwiązywanie konfliktów

### 4.1 Zegar HLC

Implementacja ([sync.rs](../../crates/flatsql-core/src/sync.rs)) odpowiada zwartej postaci z [C2]: 48 bitów czasu fizycznego i 16 bitów licznika w jednej liczbie 64-bitowej, monotoniczność przy cofnięciu zegara i wchłanianie obcych znaczników (`observe`). Porządek `(time, node)` to porządek całkowity z rozstrzyganiem remisów identyfikatorem węzła.

Czego brakuje: [C2] (sekcja 4) przewiduje regułę **ignorowania** wiadomości, które odsunęłyby zegar logiczny zbyt daleko od fizycznego, z zalogowaniem incydentu i powiadomieniem administratora. W Flat-SQL `observe` przyjmuje dowolną wartość, a serwer nie sprawdza znaczników. Skutki:

- urządzenie z zegarem ustawionym np. rok do przodu wygrywa **wszystkie** konflikty przez ten rok,
- przez wchłanianie znaczników „przyszłość” rozlewa się na wszystkie repliki,
- Kleppmann [P1, rozdz. 8] zaznacza, że zegarom urządzeń, których nie kontrolujemy, nie można ufać.

**Rekomendacja:** stała `MAX_DRIFT`; serwer odrzuca zmiany ze znacznikiem `> czas_serwera + MAX_DRIFT`, klient nie wchłania takich znaczników i sygnalizuje problem ([todo.md 1.1](../todo.md)), test z rozjechanymi zegarami ([todo.md 1.9](../todo.md)).

### 4.2 LWW per wiersz na tle CRDT

Model replikacji Flat-SQL to mapa rejestrów LWW indeksowana kluczem, z tombstone'ami usunięć – formalnie typ z [L2]. Scalanie przez maksimum w porządku całkowitym jest łączne, przemienne i idempotentne, więc zbieżność wynika z teorii CRDT [L28], a test `randomized_offline_sessions_converge` jest jej empirycznym potwierdzeniem. Wysyłanie bieżącego stanu zmienionych wierszy zamiast historii operacji to synchronizacja delta-state [L28] – 100 kolejnych `UPDATE` jednego wiersza daje jedną zmianę.

Zastrzeżenie dotyczy tego, co zbieżność kosztuje. Gray i in. [C9] opisali to już w 1996 r.: schemat „nowszy znacznik wygrywa” daje zbieżność, ale **gubi aktualizacje** (problem utraconej aktualizacji); wyjątkiem są aktualizacje przemienne, np. dodawanie stałej do liczby. [P1, rozdz. 5] dodaje, że LWW jest bezpieczne głównie dla kluczy zapisywanych raz. W Flat-SQL:

| Scenariusz offline | Wynik | Rozwiązania w literaturze i praktyce |
|---|---|---|
| A zmienia `qty`, B zmienia `player` w tym samym wierszu | zostaje cały wiersz z późniejszym znacznikiem, druga zmiana ginie bez śladu | LWW per kolumna: CRR [L24], cr-sqlite, Teilen-SQL [X5] – [todo.md 3.1](../todo.md) |
| A i B zwiększają licznik o 1 | wynik +1 zamiast +2 | aktualizacje przemienne [C9], licznik CRDT [L2], kolumny addytywne [L8] – [todo_future.md 2.5](../todo_future.md) |
| wspólna edycja pola tekstowego | wygrywa jedna wersja | CRDT tekstu poza silnikiem [L1] – [todo_future.md 6.3](../todo_future.md) |

**Rekomendacja:** w pracy wskazać, dla jakich danych LWW na wiersz jest właściwe (ekwipunek, ustawienia, katalog – dane rzadko edytowane współbieżnie; Coda w praktyce prawie nie miała konfliktów [C10]), a jako pierwsze ulepszenie wprowadzić znaczniki per kolumna – protokół się nie zmienia, rośnie tylko rozmiar wiersza.

### 4.3 Granice transakcji przy replikacji

Lokalnie transakcja jest atomowa (undo log, jedna ramka WAL). W synchronizacji ta granica nie jest zachowywana: `export_outbox(max)` bierze do `max` wpisów w kolejności `seq` (`batchSize` = 1000 w [bridge/sync.js](../../bridge/sync.js)), a wiersz modyfikowany ponownie przesuwa się na koniec kolejki. Gdy outbox jest większy niż jedna paczka, zmiany jednej transakcji mogą trafić do różnych paczek i inne repliki mogą przez chwilę widzieć stan pośredni.

Literatura daje tu dwie odpowiedzi. Bailis i in. [C8] pokazują, że **Monotonic Atomic View** (widoczne wszystkie albo żadne zapisy transakcji) da się zapewnić bez koordynacji. Z drugiej strony Helland [C7] przyjmuje atomowość jednej encji, a Simba [L27] – system synchronizacji tabel dla aplikacji mobilnych – świadomie daje atomowość tylko pojedynczego wiersza, bo atomowa synchronizacja wielu wierszy okazała się dla większości aplikacji zbędna i kosztowna. Flat-SQL daje dziś dokładnie gwarancję Simby.

**Rekomendacja:** udokumentować gwarancję wprost („transakcje są atomowe lokalnie; między replikami atomowa jest zmiana pojedynczego wiersza”) i dopiero przy konkretnej potrzebie grupować outbox po transakcjach ([todo.md 3.5](../todo.md)).

### 4.4 Usunięcia, tombstone'y i unikalność klucza

- Tombstone ze znacznikiem chroni przed „wskrzeszeniem” wiersza przez spóźniony `UPDATE` – zgodne z [L2]. Antidote SQL [L8] pozwala wybrać per tabela semantykę update-wins lub delete-wins; Flat-SQL stosuje czysty porządek znaczników, co przy braku kluczy obcych jest spójne.
- CRR [L24] zamiast tombstone'a ze znacznikiem używa „długości przyczynowej” – licznika wstawień i usunięć (nieparzysta = wiersz istnieje, scalanie = maksimum), więc ponowne wstawienie po usunięciu nie zależy od zegarów.
- Tombstone'y nie są usuwane. Bezpieczne kryterium to stabilność przyczynowa – usunięcie, gdy wszystkie repliki potwierdziły znacznik [L28]. Przy serwerze centralnym wystarczy minimalny potwierdzony kursor aktywnych klientów; klienci nieaktywni dłużej niż próg dostają pełny stan ([todo.md 3.4](../todo.md)).
- **Unikalność klucza głównego jest lokalna.** Dwa urządzenia offline wstawiające wiersz o tym samym `id` nie dostaną błędu – po synchronizacji jeden wiersz po cichu zastąpi drugi. [P1, rozdz. 9]: ograniczenia unikalności między replikami wymagają konsensusu. CRR [L24] wykrywa takie naruszenie przy scalaniu i cofa przegrywającą zmianę, Simba [L27] zgłasza konflikty aplikacji, a Coda [C10] unikała kolizji tymczasowymi identyfikatorami zastępowanymi przy reintegracji. **Rekomendacja:** klucze bez kolizji w [flat-sql.md](../flat-sql.md) ([todo.md 0.3](../todo.md)) i raport konfliktów ([todo.md 3.3](../todo.md)).

### 4.5 Protokół synchronizacji

| Element Flat-SQL | Odpowiednik w literaturze | Ocena |
|---|---|---|
| zapisy lokalne + serwer porządkujący log (`seq`) | Bayou [C3]: zapisy do dowolnej repliki, serwer główny ustala kolejność, anti-entropy | zgodne (LWW zamiast procedur scalania aplikacji) |
| outbox w WAL, przetrwa restart; kolejne zmiany wiersza scalane w jedną | Coda [C10]: trwały log zmian offline, nowszy zapis unieważnia wcześniejsze wpisy; delta-state [L28] | zgodne |
| ponowne dostarczenie paczki nie zmienia stanu | idempotencja wiadomości [C7], argument end-to-end [P1, rozdz. 12]; Coda odtwarza log w jednej transakcji [C10] | zgodne |
| kursor klienta, epoka serwera, pełny stan przy nieznanym kursorze | offsety konsumentów i CDC ze snapshotem początkowym [P1, rozdz. 11] | zgodne |
| log serwera tylko w pamięci, epoka losowana przy starcie | trwały log z offsetami [P1, rozdz. 11] | **luka** – każdy restart wysyła wszystkim pełny stan ([todo.md 3.2](../todo.md)) |
| pull z wykładniczym odstępem | [P1, rozdz. 12]: model push dla klientów reagujących na zmiany | częściowo – push w [todo_future.md 6.2](../todo_future.md) |
| serwer przyjmuje każdą zmianę | Gray i in. [C9]: właściciel kopii głównej zatwierdza zmiany wstępne według kryterium akceptacji; Bayou [C3]: sprawdzanie zależności | luka prototypu ([todo_future.md §2, 4.1](../todo_future.md)) |
| każdy klient dostaje całą bazę | Cimbiosys [C11]: replikacja częściowa z filtrem klienta i usuwaniem lokalnym wierszy spoza filtra | luka (rozszerzenie – [todo_future.md 3.2](../todo_future.md)) |

### 4.6 Ewolucja schematu

`CREATE TABLE` z innej repliki jest ignorowany, jeśli tabela o tej nazwie już istnieje – bez porównania schematów ([db.rs, `apply_remote`](../../crates/flatsql-core/src/db.rs)). Dwie repliki, które offline utworzą tabelę o tej samej nazwie i różnych kolumnach, rozjadą się trwale: wiersze drugiej repliki będą odrzucane przez `check_row`. [L1] opisuje brak „autorytatywnego” schematu jako problem otwarty, a Cambria [L26] rozwiązuje go soczewkami tłumaczącymi dane między wersjami schematu.

**Rekomendacja:** w prototypie – wykrywać konflikt i zgłaszać go jawnie ([todo.md 1.7](../todo.md)); w dalszych pracach – wersjonowanie schematu i migracje ([todo_future.md 6.1](../todo_future.md)) oraz schemat zarządzany przez backend ([todo_future.md 2.6](../todo_future.md)).

## 5. Model danych Flat-SQL („bez JOIN”)

Argumenty **za** płaskim modelem:

- Helland [C7]: encja z unikalnym kluczem jest naturalnym zakresem serializowalności, a transakcje atomowe nie powinny obejmować wielu encji;
- Armbrust i in. [M2]: zapytania z limitem bez złączeń (albo ze złączeniem po unikalnym kluczu) mają koszt niezależny od rozmiaru bazy – to formalna wersja argumentu o przewidywalności;
- Stonebraker i Pavlo [M1]: przemysłowym odpowiednikiem jest CQL w Cassandrze – składnia SQL nad modelem klucz–wiersz;
- Simba [L27]: tabela z wierszem jako jednostką synchronizacji wystarcza typowym aplikacjom mobilnym.

Argumenty **przeciw**:

- [M1]: systemy NoSQL niemal powszechnie dodały SQL, a brak złączeń uznaje się za ograniczenie, nie zaletę; [M2] krytykuje języki, które dla skalowalności po prostu usuwają złączenia;
- Riffle [D4]: w aplikacji local-first znaczna część kodu UI to złączenia i grupowania, a ich szybkość decyduje o responsywności;
- [P1, rozdz. 2]: relacje wiele-do-wielu bez złączeń przenoszą pracę do aplikacji.

Dwie obserwacje ważne dla uzasadnienia w pracy:

1. **Granulacja replikacji nie zależy od języka zapytań.** CRR [L24] i cr-sqlite replikują wiersze i kolumny tabel relacyjnych, a Antidote SQL [L8] pokazuje, że prawdziwa trudność to więzy integralności (klucze obce, kaskady) przy współbieżnych usunięciach – nie same złączenia.
2. **„Przewidywalne” nie znaczy „ograniczone”.** Plan Flat-SQL jest zawsze jednotabelowy i widoczny w `EXPLAIN` oraz `rowsScanned`, ale pełny skan i `GROUP BY` rosną z tabelą. W sensie [M2] ograniczony koszt mają tylko odczyty po kluczu lub indeksie z `LIMIT`. Z tej samej pracy wynika, że złączenie po unikalnym kluczu głównym nie psuje przewidywalności.

**Rekomendacja:** uzasadniać brak JOIN prostotą i przewidywalnością planu oraz brakiem więzów między tabelami, a nie samą replikacją; doprecyzować tezę o przewidywalności ([todo.md 0.1](../todo.md)); jako dalsze prace – złączenia tylko do odczytu po kluczu głównym ([todo_future.md 5.2](../todo_future.md), decyzja autora) i zapytania reaktywne ([todo_future.md 5.1](../todo_future.md)).

## 6. Składowanie i trwałość

### 6.1 Baza w pamięci + WAL + snapshot

Wzorzec Flat-SQL (dane w pamięci, log tylko-redo z CRC, okresowy pełny snapshot, odtwarzanie = snapshot + ramki o większym LSN) jest klasyczny:

- DeWitt i in. [S1] – odtwarzanie baz w pamięci przez log i checkpointy;
- [P1, rozdz. 3] – bazy w pamięci zawdzięczają szybkość głównie brakowi kodowania struktur do formatu dyskowego, a trwałość zapewniają logiem i snapshotami; WAL nie potrzebuje fazy undo, bo niezatwierdzone zmiany nigdy do niego nie trafiają;
- H-Store [S4] – w klasycznym silniku OLTP większość pracy idzie na logowanie, blokady, latche i bufor stron; jednowątkowy silnik w pamięci eliminuje trzy z czterech. Flat-SQL płaci tylko za logowanie (undo log + ramka WAL + wpis outboksu) – co tłumaczy wolniejsze od SQLite inserty i aktualizacje w H4.

Wybór logu fizycznego zamiast logowania poleceń: Malviya i in. [S6] mierzą ok. 1,5× większą przepustowość logowania poleceń kosztem dłuższego odtwarzania i wymogu deterministycznego wykonania. W Flat-SQL log fizyczny jest lepszy, bo wpisy WAL służą jednocześnie synchronizacji (zmiana = pełny wiersz + znacznik), a odtworzenie nie zależy od zegara ani wyników zapytań. **Zgodne.**

Dobrym kontrastem jest Zeta-Lite [D5]: silnik o tej samej architekturze (jeden rdzeń, `wasm32-unknown-unknown` + `wasm-bindgen`), który zapisuje do OPFS tylko snapshot na żądanie, więc zmiany po ostatnim snapshocie giną przy zamknięciu karty. Flat-SQL rozwiązuje to przyrostowym WAL – mocny argument dla H5.

### 6.2 Group commit

Zatwierdzenia z jednego „tiku” pętli zdarzeń zapisywane jednym zapisem do IndexedDB to group commit opisany po raz pierwszy w druku przez DeWitta i in. [S1]. Wynik H5 (1000 zatwierdzeń + jeden `flush()` = 5,6 µs na zatwierdzenie) jest dobrą ilustracją. **Zgodne.**

### 6.3 Checkpoint i przestoje

Checkpoint kopiuje całą bazę synchronicznie: 35 ms dla 3 MB w przeglądarce, 49 ms dla snapshotu 100 tys. wierszy w Node.js. Cao i in. [S10] nazywają to „naiwnym snapshotem” i pokazują na przykładzie gier MMO (stan spójny na granicy ticku), że daje on skoki latencji; algorytmy z kopiowaniem przy aktualizacji i ich warianty bez blokad dają niemal stałą latencję kosztem dodatkowej pamięci. Przy budżecie 16 ms na klatkę [D4] obecny checkpoint to 2–3 klatki przerwy – istotne dla gier, czyli pierwszego zastosowania z prezentacji.

**Rekomendacja:** checkpoint porcjami między zadaniami pętli zdarzeń z kopiowaniem wierszy zmienianych w trakcie albo snapshot przyrostowy ([todo.md 2.6](../todo.md)), mierzony najdłuższą pauzą ([todo.md 4.10](../todo.md)). Odrzucono równoległe odtwarzanie: wymaga wątków WASM, czyli nagłówków COOP/COEP, co przeczy atutowi „zero konfiguracji”. **Częściowo.**

### 6.4 Trwałość w przeglądarce (IndexedDB)

`IndexedDbStore` otwiera transakcje z `durability: 'default'` ([bridge/stores/indexeddb.js](../../bridge/stores/indexeddb.js)). Od Chrome 121 domyślna trwałość to `relaxed` – zdarzenie `complete` następuje po przekazaniu danych do bufora systemu operacyjnego ([Q1]). Po `await db.flush()` dane przetrwają zamknięcie karty i awarię przeglądarki, ale mogą zginąć przy awarii systemu.

Analogia z `synchronous=NORMAL` w SQLite z [architecture.md §4.3](../architecture.md) jest trafna, ale zdanie „po `await flush()` dane są trwałe” należy doprecyzować. `IndexedDbStore` już przyjmuje `durability: 'strict'` – warto udostępnić ją w `FlatDB`, opisać i zmierzyć ([todo.md 1.4, 4.9](../todo.md)). **Częściowo.**

### 6.5 Backend plikowy (CLI, serwer)

`FileStorage::write_snapshot` ([storage/file.rs](../../crates/flatsql-core/src/storage/file.rs)) wykonuje: zapis `snapshot.fsql.tmp` → `sync_all` → `rename` → `set_len(0)` na WAL → `sync_all` WAL. Pillai i in. [S8] pokazali, że kolejność utrwalania operacji na **różnych plikach** nie jest gwarantowana bez `fsync` katalogu po `rename`. Możliwy stan po zaniku zasilania: katalog nadal wskazuje stary snapshot, a WAL jest już pusty – utrata transakcji zatwierdzonych między dwoma checkpointami. `FsStore` w Node.js ([bridge/stores/fs.js](../../bridge/stores/fs.js)) nie wywołuje `fsync` w ogóle, choć [architecture.md §4.2](../architecture.md) przypisuje mu „opcjonalnie `fsync`”.

**Rekomendacja:** `fsync` katalogu po `rename` i dopiero potem obcięcie WAL (albo nowy plik WAL zamiast obcinania); `fsync` w `FsStore` i poprawka dokumentacji ([todo.md 1.2, 1.3](../todo.md)). **Luka.**

### 6.6 Testy odtwarzania po awarii

Test `every_wal_prefix_recovers_to_a_committed_state` (ucięcie WAL w każdym bajcie) odpowiada modelowi awarii, w którym utrwalony jest prefiks zapisu jednego pliku – dobra praktyka. Brakuje testu protokołu checkpointu między plikami w stylu ALICE [S8] ([todo.md 1.6](../todo.md)). **Zgodne, do rozszerzenia.**

### 6.7 Usuwanie danych przez przeglądarkę

- [L1] przypomina, że wyczyszczenie danych witryny usuwa też magazyn lokalny;
- Safari usuwa IndexedDB po 7 dniach używania przeglądarki bez interakcji z witryną ([Q2]);
- magazyn jest domyślnie „best-effort”; tryb trwały wymaga `navigator.storage.persist()` ([Q3], [Q4]).

W kodzie nie ma wywołania `navigator.storage.persist()`, więc **niewysłany outbox może zniknąć bez winy aplikacji**. Analiza wdrożeń pokazuje, że kasy i gry dodają z tego powodu kopię na serwerze lub eksport do pliku. **Rekomendacja:** `persist()` przy otwarciu bazy ([todo.md 1.5](../todo.md)), eksport do pliku ([todo_future.md 1.3](../todo_future.md)); w pracy opisać ograniczenie. **Luka.**

## 7. Edge computing i Local-First vs Cloud-First (H1)

Shi i in. [E1] definiują „edge” jako dowolne zasoby obliczeniowe na drodze między źródłem danych a chmurą – także smartfon. Flat-SQL realizuje tę definicję wprost: zapytania wykonują się na urządzeniu. Prototyp ma przy tym jeden centralny serwer, który jest jednocześnie hubem synchronizacji i bazą główną; zakładanych w prezentacji węzłów brzegowych synchronizowanych z bazą główną jeszcze nie ma ([conclusions.md §3.6](conclusions.md#36-edge-computing), [todo_future.md §7](../todo_future.md)). W literaturze replikacji taki układ to replikacja dwuwarstwowa Graya i in. [C9] – z węzłami bazowymi jako warstwą zatwierdzającą – a w wariancie z rozstrzyganiem na węzłach: multi-leader w topologii drzewa ([P1] rozdz. 5) i scenariusz CRR [L24].

Wynik H1 (11 µs lokalnie vs 20,9–102,5 ms przy RTT 20–100 ms) jest zgodny z tym, co przewiduje literatura: czas zapytania sieciowego jest ograniczony od dołu przez RTT. Riffle [D4] daje punkt odniesienia: przy budżecie 16 ms na klatkę jedno zapytanie przez sieć z RTT 20 ms już go przekracza. Podział „dane operacyjne lokalnie, ciężka analityka gdzie indziej” wspiera DuckDB-Wasm [D1] jako inna klasa systemu. **Zgodne.**

Uwaga metodyczna: symulacja opóźnienia w [benchmark-scenario.md](../research/benchmark-scenario.md) jest uczciwie opisana jako dolne ograniczenie (bez TLS, strat pakietów i zestawiania połączeń). Analiza wdrożeń dodaje zastrzeżenie w drugą stronę: na wolnych urządzeniach lokalny odczyt z dysku bywa wolniejszy od sieci (Notion „ściga” oba odczyty) – stąd pomiar na telefonie ([todo.md 4.7](../todo.md)).

## 8. Metodyka benchmarków

| Pułapka / zalecenie | Stan w projekcie | Ocena | Co zrobić |
|---|---|---|---|
| Powtarzalność [B3] | skrypty, wersje, stałe ziarno, wyniki w JSON | zgodne | dołączyć surowe JSON do pracy |
| Brak optymalizacji systemu porównywanego [B3] | SQLite w najlepszym układzie (`INTEGER PRIMARY KEY`), prepared statements | zgodne | – |
| Jabłka i pomarańcze [B3] | trwałość porównywana z eksportem sql.js – opisane w zagrożeniach | częściowo | SQLite WASM z `opfs-sahpool` [Q7] jako uczciwy punkt odniesienia dla H5 ([todo.md 4.1](../todo.md)) |
| Przebiegi zimne i gorące [B3] | raportowane przebiegi po rozgrzewce | częściowo | osobna tabela „zimny start” ([todo.md 4.3](../todo.md)) |
| Niepoprawne wyniki [B3] | porównanie liczby wierszy | częściowo | suma kontrolna wyników ([todo.md 4.6](../todo.md)) |
| Wiele uruchomień, przedziały ufności [B1] | jeden przebieg na konfigurację; szum ±30% opisany | częściowo | ≥ 10 uruchomień procesu ([todo.md 4.2](../todo.md)) |
| Realistyczny rozkład dostępu [B5] | klucze losowane równomiernie | częściowo | YCSB z rozkładem Zipfa i scenariusz demo ([todo.md 4.5](../todo.md)) |
| Ogony latencji, nie tylko średnie | percentyle operacji; brak pomiaru pauz | częściowo | pauza checkpointu na tle 16 ms ([todo.md 4.10](../todo.md), [S10], [D4]) |
| Rozdzielczość zegara [Q5] | 100 µs w przeglądarce, opisane | zgodne | strona benchmarku z COOP/COEP daje 5 µs ([todo.md 4.8](../todo.md)) |

## 9. Rekomendacje

Pełna lista z nakładem: [todo.md](../todo.md) (optymalizacja prototypu) i [todo_future.md](../todo_future.md) (rozszerzenia).

**Priorytet 1 – poprawność i rzetelność tez (małe zmiany):**

1. Ograniczenie dryfu HLC (§4.1) – todo 1.1.
2. `fsync` katalogu przed obcięciem WAL, `fsync` w `FsStore` (§6.5) – todo 1.2, 1.3.
3. Doprecyzowanie trwałości IndexedDB i `persist()` (§6.4, §6.7) – todo 1.4, 1.5.
4. Doprecyzowanie tez: Privacy by Design, „szybszy niż sql.js”, atomowość per wiersz, przewidywalność (§2.4, §4.3, §5) – todo 0.1.
5. Scenariusz „CSV 500 MB” przypisany DuckDB-Wasm (§2.5) – todo 0.2.

**Priorytet 2 – ulepszenia istniejącej synchronizacji i składowania:**

6. LWW per kolumna (§4.2) – todo 3.1.
7. Trwały log serwera (§4.5) – todo 3.2.
8. Jawne konflikty schematu i klucza głównego, klucze bez kolizji (§4.4, §4.6) – todo 1.7, 3.3, 0.3.
9. Checkpoint bez przestoju (§6.3) – todo 2.6.
10. Usuwanie tombstone'ów (§4.4) – todo 3.4.

**Priorytet 3 – rozszerzenia (do rozdziału „kierunki rozwoju”):**

11. Reguły serwera według replikacji dwuwarstwowej: tabele serwera, walidacja, zamiary, liczniki `MERGE ADD` (§4.2, §4.5) – future §2.
12. Replikacja częściowa z filtrami (§4.5) – future 3.2.
13. Szyfrowanie end-to-end i uwierzytelnianie (§2.4) – future §4.
14. Zapytania reaktywne i złączenia po kluczu (§5) – future 5.1, 5.2.
15. Push zamiast pull, wersjonowanie schematu (§4.5, §4.6) – future 6.1, 6.2.

## 10. Co projekt wnosi względem literatury

Uczciwie: żaden pojedynczy element nie jest nowy – LWW na HLC, WAL ze snapshotem, group commit, outbox i log z kursorami mają dobrze opisane odpowiedniki, w tym w klasycznych pracach o pracy offline ([C3], [C9], [C10]). Wartość prototypu leży w ich **połączeniu i pomiarze**:

1. jeden rdzeń bez zależności i bez I/O, uruchamiany identycznie w przeglądarce, na serwerze i natywnie – realizacja celu przenośności z [P2] (podobną drogą idzie Zeta-Lite [D5], ale bez przyrostowej trwałości i synchronizacji, za to z pełnym SQL i modułem ok. 14× większym po kompresji);
2. pomiar kosztu granicy JS↔WASM dla silnika bazy danych (H3) – obszar bez recenzowanych pomiarów;
3. pomiar przyrostowej trwałości w IndexedDB z group commitem wobec eksportu całej bazy (H5);
4. dialekt SQL celowo ograniczony do modelu encji pod kluczem (w duchu [C7], [M2] i CQL), z synchronizacją wbudowaną w format WAL – w odróżnieniu od CRR [L24], cr-sqlite czy Teilen-SQL [X5], które dodają warstwę replikacji do pełnej bazy relacyjnej, i od Simby [L27], która jest usługą obok bazy, a nie silnikiem.

Porównując się z tymi systemami, warto podkreślić różnicę celu: one zachowują pełny SQL i dokładają warstwę synchronizacji, Flat-SQL rezygnuje z części SQL w zamian za prostszy silnik, mniejszy moduł i synchronizację wbudowaną w format WAL.
