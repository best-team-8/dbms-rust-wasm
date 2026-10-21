# Wnioski z analizy projektu i przeglądu literatury

Stan po rewizji z 20 października 2026. Dokument zbiera wnioski z analizy prototypu Flat-SQL, z rdzenia literatury ([README.md](README.md)) i z analizy rzeczywistych wdrożeń podobnych baz ([real-world-usage.md](../research/real-world-usage.md)). Szczegółowe uzasadnienia: [literature-comparison.md](literature-comparison.md); sposób wyszukiwania: [search-methodology.md](search-methodology.md). Zadania: **[../todo.md](../todo.md)** (optymalizacja prototypu) i **[../todo_future.md](../todo_future.md)** (rozszerzenia).

Identyfikatory w nawiasach (np. [W1]) odsyłają do wpisów w [README.md](README.md); hipotezy H1–H6 – do [benchmark-scenario.md](../research/benchmark-scenario.md).

## 1. Najważniejsze wnioski

1. **Flat-SQL nie jest lepszy od dojrzałych silników ogólnie, ale jest konkurencyjny w swojej kategorii:** mała baza SQL na dane operacyjne aplikacji, działająca w przeglądarce bez dodatkowej konfiguracji, z przyrostową trwałością i wbudowaną synchronizacją. Żadne z porównywanych rozwiązań nie łączy tych cech w jednym module.
2. **Architektura ma solidne oparcie w klasycznych pracach o pracy offline.** Trwały, skracany log zmian z idempotentną reintegracją to wzorzec z Coda [C10], lokalne zapisy porządkowane przez serwer – z Bayou [C3], a hybryda „lokalna kopia + serwer zatwierdzający” – z replikacji dwuwarstwowej Graya i in. [C9]. Współcześnie potwierdzają ją badanie aplikacji mobilnych, które bez takiego mechanizmu tracą dane [L27], i wywiady z twórcami aplikacji webowych [L20].
3. **Wyniki pomiarów są zgodne z literaturą.** Narzut WebAssembly (1,1–1,8×) mieści się w przedziale z [P2] i [W1].
4. **Największym kosztem jest granica JS↔WASM i reprezentacja danych w pamięci, a nie sam WebAssembly.** Recenzowanych pomiarów tej granicy dla silników baz nadal nie ma – pomiar H3 jest wkładem własnym; kierunek optymalizacji wskazuje DuckDB-Wasm [D1].
5. **Warstwa składowania realizuje klasyczny wzorzec** (dane w pamięci, WAL fizyczny, snapshot, group commit – [S1], [S4], [S6]). Przyrostowy WAL daje przewagę nad sql.js i nad Zeta-Lite [D5]. Słabym punktem jest checkpoint: synchroniczna kopia całej bazy to „naiwny snapshot” [S10], który w grze oznacza 2–3 klatki przerwy.
6. **Synchronizacja LWW per wiersz zapewnia zbieżność, ale gubi aktualizacje** – Gray i in. opisali to dla schematów ze znacznikami czasu już w 1996 r. [C9]. Kierunek: LWW per kolumna [L24] (optymalizacja istniejącego protokołu) i aktualizacje przemienne (`MERGE ADD`, rozszerzenie).
7. **Gwarancja „atomowy wiersz między replikami” jest uznanym wyborem, nie usterką** – taką samą daje Simba [L27], w duchu modelu encji Hellanda [C7]; Monotonic Atomic View [C8] jest osiągalne, gdyby okazała się za słaba.
8. **Wykryto luki poprawności w istniejących mechanizmach:** brak ograniczenia dryfu HLC [C2], niepewna kolejność utrwalania przy checkpoincie [S8], domyślnie „luźna” trwałość IndexedDB [Q1], możliwość usunięcia danych przez przeglądarkę [Q2]–[Q4], log serwera tylko w pamięci (każdy restart = pełny stan do wszystkich klientów), ciche konflikty schematu [L26] i klucza głównego [L24].
9. **„Privacy by Design” jest spełnione częściowo:** dane są domyślnie lokalne, ale nie są szyfrowane ani lokalnie, ani w synchronizacji, a serwer nie uwierzytelnia klientów ([L1], [W2], [V1]).
10. **Model „bez JOIN” ma mocne oparcie** ([C7], [M2]), **ale jest krytykowany** ([M1], [D4]). Teza o „przewidywalnym wykonaniu” wymaga doprecyzowania: przewidywalny jest plan, a ograniczony koszt mają tylko odczyty po kluczu lub indeksie z limitem [M2]. Złączenie po unikalnym kluczu nie psułoby przewidywalności ani replikacji ([M2], [L8]).
11. **Najmocniej potwierdzone zastosowania** to zbieranie danych w terenie offline, aplikacje osobiste na wielu urządzeniach i pamięć podręczna danych serwera w aplikacjach SaaS ([real-world-usage.md §4](../research/real-world-usage.md#4-zastosowania-flat-sql-potwierdzone-analizą)). Blokuje je niedojrzałość (replikacja częściowa, reguły serwera, uwierzytelnianie, wiele kart), nie koncepcja – i każde z tych rozszerzeń ma oparcie w literaturze ([C11], [C9], [Q12]).
12. **Tryb edge zrealizowano tylko częściowo.** Silnik działa na serwerze (ten sam moduł `.wasm` w Node.js), ale jako jeden centralny serwer, który jest hubem synchronizacji i bazą główną naraz. Węzłów brzegowych synchronizowanych z bazą główną jeszcze nie ma; fundamenty istnieją (rdzeń bez I/O, protokół push/pull, zbieżność LWW niezależna od topologii).
13. **Metodyka benchmarku jest staranna,** ale do mocnych tez brakuje porównania z SQLite WASM na OPFS, statystyki z wielu uruchomień i pomiaru pauz ([B1], [B3], [S10]).

## 2. Flat-SQL na tle istniejących rozwiązań

| Rozwiązanie | SQL | Trwałość w przeglądarce | Synchronizacja | Moduł (gzip) |
|---|---|---|---|---|
| **Flat-SQL** | okrojony (bez JOIN) | przyrostowa (WAL w IndexedDB), bez Workera i nagłówków | wbudowana | **206 KiB** (zmierzono) |
| sql.js | pełny | eksport całej bazy | brak | 315 KiB (zmierzono) |
| SQLite WASM (oficjalny) | pełny | OPFS, wymaga Web Workera | zewnętrzna (np. cr-sqlite) | ok. 0,8–1 MB bez kompresji* |
| PGlite | pełny PostgreSQL | IndexedDB / OPFS | zewnętrzna (Electric) | < 3 MB* |
| DuckDB-Wasm [D1] | pełny, analityczny | głównie odczyt plików | brak | ok. 2,5–2,8 MB* |
| Zeta-Lite [D5] | pełny PostgreSQL | snapshot do OPFS na żądanie | brak | 2,87 MB* |
| CRR [L24], cr-sqlite | pełny (warstwa nad PostgreSQL/SQLite) | zależnie od bazy | wbudowana, per kolumna | – |
| Simba [L27] (aplikacje mobilne) | API tabel (CRUD) | SQLite na urządzeniu | usługa, atomowość per wiersz | – |
| RxDB, PouchDB | brak SQL | IndexedDB | wbudowana | – |

\* deklaracje autorów, nie mierzono w projekcie

**Gdzie Flat-SQL wygrywa** (pomiary z [benchmark-results.md](../research/benchmark-results.md), Node.js, 100 tys. wierszy, porównanie z sql.js):

- odczyt po kluczu 9,7 µs zamiast 20,3 µs (ok. 2×), grupowanie 26 ms zamiast 130 ms (ok. 5×), top-k 1,5×;
- trwałe zatwierdzenie 0,34 ms zamiast 8,7 ms – koszt nie rośnie z rozmiarem bazy; przy group commit 5,6 µs na zatwierdzenie;
- moduł o ok. 1/3 mniejszy niż sql.js i kilkanaście razy mniejszy niż PGlite, DuckDB-Wasm czy Zeta-Lite;
- synchronizacja i praca offline bez dodatkowej warstwy; ten sam rdzeń w przeglądarce, na serwerze i natywnie;
- działa w wątku głównym, bez Web Workera i bez nagłówków COOP/COEP.

**Gdzie przegrywa:**

- język: brak złączeń, podzapytań i `CASE`;
- względem SQLite: inserty 1,6× wolniej, UPDATE/DELETE 1,7–3,1× wolniej, `COUNT` po indeksie 5,7× wolniej (indeksy pokrywające), snapshot 20× i odtworzenie 160× wolniej (SQLite kopiuje plik);
- skala: wszystkie dane w pamięci (limit 4 GiB `wasm32`, a reprezentacja wierszy zwielokrotnia rozmiar);
- synchronizacja: konflikt rozstrzygany dla całego wiersza; brak szyfrowania, uwierzytelniania i replikacji częściowej;
- odczyt po kluczu wolniejszy niż deklarowane przez autora Zeta-Lite 3,2–3,7 µs (inne warunki pomiaru);
- dojrzałość: prototyp kontra systemy produkcyjne.

**Zastrzeżenie:** przewaga w trwałości zapisu jest zmierzona tylko względem sql.js – najsłabszego pod tym względem wariantu SQLite. Oficjalny SQLite WASM z OPFS również zapisuje przyrostowo i nie został zmierzony. Poprawne sformułowanie brzmi „szybszy niż sql.js”, a nie „szybszy niż SQLite” ([todo.md 4.1](../todo.md)).

**Proponowane ujęcie w pracy:** Flat-SQL świadomie rezygnuje z części możliwości SQL (złączenia, podzapytania) w zamian za mały moduł, przyrostową trwałość bez wymagań środowiskowych i wbudowaną synchronizację Local-First. W typowych operacjach aplikacji (odczyt po kluczu, grupowanie, sortowanie) jest konkurencyjny wobec SQLite w WASM, a słabszy tam, gdzie SQLite korzysta z formatu stronicowego.

## 3. Wnioski z badań według obszarów

### 3.1 WebAssembly

- Narzut WASM na pełnych aplikacjach to średnio 1,45–1,55× ([W1]); przyczyny – presja na rejestry i dodatkowe kontrole przy każdym wywołaniu – tłumaczą spowolnienie rekurencyjnego `eval`.
- Optymalizacje LLVM nie są dostrojone do WASM, a programy WASM zużywają więcej pamięci niż JS ([W11]).
- Izolacja WASM chroni hosta przed modułem, ale nie dane modułu przed jego błędami ([P2], [W2]); bezpieczny Rust bez `unsafe` ogranicza tę klasę zagrożeń.
- Memory64 z Wasm 3.0 jest wolniejsze i warte użycia tylko powyżej 4 GiB ([Q6]).
- Wymiana danych z JS najlepiej działa przez binarny bufor czytany wprost z pamięci WASM ([D1]); JS String Builtins nie pomogą silnikowi w Ruście ([Q13]).

### 3.2 Local-First

- Projekt spełnia ideały szybkości, pracy na wielu urządzeniach i pracy offline; częściowo – współpracę, długowieczność danych, prywatność i kontrolę użytkownika ([L1]).
- W praktyce najbardziej wykonalne są modele hybrydowe z serwerem koordynującym – taki jest Flat-SQL ([L20], [C9]).
- Local-first nie zastąpi serwera przy transakcjach finansowych: urządzenie offline nie wie, czy środki nie zostały wydane gdzie indziej ([C9], [C8]); wszystkie znalezione systemy kasowe zatwierdzają płatności na serwerze.
- Dane w przeglądarce nie są gwarantowane: Safari usuwa je po 7 dniach bez interakcji, magazyn jest domyślnie „best-effort” ([Q2]–[Q4]).

### 3.3 Synchronizacja i konflikty

- Zegar HLC jest zaimplementowany w zwartej postaci z [C2], ale bez reguły ignorowania znaczników zbyt daleko w przyszłości – urządzenie ze złym zegarem wygrywa wszystkie konflikty ([C2] §4, [P1] rozdz. 8).
- LWW to poprawny CRDT ([L2], [L28]), ale gubi aktualizacje ([C9]); aktualizacje przemienne tego nie robią ([C9], [L8]).
- Atomowość per wiersz między replikami wystarcza typowym aplikacjom ([L27]); MAV jest osiągalne bez koordynacji, zapobieganie utraconym aktualizacjom – nie ([C8]).
- Unikalność klucza głównego jest gwarantowana tylko lokalnie; CRR [L24] wykrywa takie konflikty przy scalaniu, Coda [C10] unikała ich tymczasowymi identyfikatorami.
- Protokół (log z kursorami, scalanie zmian w outboksie, idempotentne rundy) odpowiada wzorcom z [C3], [C10], [C7] i [P1] rozdz. 11–12; luką jest log serwera tylko w pamięci.
- Replikacja częściowa ma gotowy wzór: filtr klienta i usuwanie lokalne wierszy, które z niego wyszły ([C11]).
- Ewolucja schematu w replikach to problem otwarty; Cambria [L26] tłumaczy dane między wersjami schematu.

### 3.4 Składowanie i trwałość

- Log fizyczny jest właściwym wyborem: te same wpisy służą odtwarzaniu i synchronizacji, a odtworzenie nie zależy od determinizmu zapytań ([S6]).
- Group commit opisali po raz pierwszy w druku DeWitt i in. ([S1]).
- Checkpoint przez synchroniczną kopię całego stanu daje skoki latencji; istnieją algorytmy bez przestoju, opracowane właśnie dla gier ([S10]).
- Bez `fsync` katalogu po `rename` kolejność utrwalania snapshotu i obcięcia WAL nie jest gwarantowana ([S8]).
- Od Chrome 121 `await flush()` gwarantuje przetrwanie awarii karty i przeglądarki, ale nie awarii systemu, chyba że użyje się `durability: 'strict'` ([Q1]).

### 3.5 Model danych

- Encja pod kluczem jako jednostka atomowości: [C7]; przemysłowy odpowiednik – CQL w Cassandrze ([M1]).
- Systemy NoSQL niemal powszechnie dodały SQL, a brak złączeń uznaje się za ograniczenie ([M1]).
- Przewidywalny koszt mają zapytania ograniczone – z limitem, bez złączeń albo ze złączeniem po kluczu głównym ([M2]).
- W aplikacjach local-first złączenia i reaktywność decydują o responsywności UI ([D4]); w modelu jednotabelowym zapytania reaktywne z aktualizacją przyrostową są prostsze do zrealizowania ([M3]) – to szansa, nie tylko ograniczenie.

### 3.6 Edge computing

- Według definicji [E1] urządzenie użytkownika jest brzegiem sieci – Flat-SQL wykonuje zapytania właśnie tam.
- **Stan prototypu:** jest jeden serwer ([server/server.js](../../server/server.js)) pełniący trzy role – hub synchronizacji (`/sync/push`, `/sync/pull`), API „Cloud-First” do porównań w H1 (`/api/query` z symulowanym opóźnieniem) i hosting dema. Ten serwer **jest** bazą główną: nie łączy się z żadną bazą wyżej, a jego log zmian jest tylko w pamięci. To, że działa w Node.js, nie jest ograniczeniem – węzeł edge też może być procesem Node.js w kilku regionach; brakuje **układu połączeń** i wdrożenia na platformach edge.
- **Założenie z prezentacji:** przeglądarki ↔ węzły edge ↔ baza główna. W literaturze to replikacja dwuwarstwowa z węzłami bazowymi ([C9]) albo multi-leader w topologii drzewa ([P1] rozdz. 5) i scenariusz CRR [L24]. Zbieżność LWW na HLC nie zależy od topologii, o ile każda zmiana w końcu dotrze wszędzie ([L2]), ale liczba utraconych aktualizacji rośnie z liczbą węzłów zapisujących ([C9]) – dla danych ryzykownych (np. unieważnianie sesji) potrzebny jest tryb jednego zapisującego.
- **Czego brakuje:** przekazywania w górę zmian przyjętych od klientów, rozsyłania w dół zmian z bazy głównej bez „echa”, trwałego logu i kursorów, uwierzytelniania między węzłami i testów w układzie klienci ↔ węzły ↔ centrala ([../todo_future.md 12.4](../todo_future.md), [../todo_future.md §7](../todo_future.md)).
- Na platformach typu Cloudflare Workers jednowątkowy obiekt ze stanem i trwałym magazynem (Durable Object) odpowiada modelowi wykonania Flat-SQL, a brak I/O w rdzeniu pozwala dopisać dla niego backend `Storage`.
- Lokalne zapytanie (11 µs) mieści się w budżecie 16 ms na klatkę z dużym zapasem; zapytanie przez sieć z RTT 20 ms już go przekracza ([D4], H1).

### 3.7 Metodyka pomiarów

- Spełnione: powtarzalność, optymalizacja systemu porównywanego, kontrola liczby wierszy, rozgrzewka ([B3], [B1]).
- Do uzupełnienia: wiele uruchomień i przedziały ufności ([B1]), osobne przebiegi zimne ([B3]), realistyczny rozkład dostępu ([B5]), zużycie pamięci ([W11]), pauzy checkpointu ([S10]), uczciwy punkt odniesienia dla trwałości (SQLite z OPFS, [Q7]), pomiar na telefonie.

## 4. Kierunek rozwoju: najlepszy w swojej kategorii

Przewyższenie wszystkich rozwiązań we wszystkim nie jest możliwe – pełny SQL kosztuje rozmiar modułu, analityka dużych plików wymaga silnika kolumnowego, a dojrzałość SQLite to dekady pracy. Realny cel to pierwsze miejsce na jasno zdefiniowanej tablicy wyników:

| Metryka | Teraz | Prowadzi dziś | Cel | Zadanie |
|---|---|---|---|---|
| Odczyt po kluczu z JS | 9,7 µs | Zeta-Lite 3,2–3,7 µs (wg autora) | < 3,5 µs | todo 2.3, 2.4 |
| Trwałe zatwierdzenie | 0,34 ms | Flat-SQL (wobec sql.js) | utrzymać, porównać z SQLite + OPFS | todo 4.1 |
| Najdłuższa pauza (checkpoint) | 35–49 ms | – | < 16 ms (jedna klatka) | todo 2.6, 4.10 |
| Rozmiar modułu | 206 KiB | Flat-SQL | < 150 KiB | todo 2.5 |
| Synchronizacja bez cichej utraty danych | LWW per wiersz | brak wbudowanej w silnikach SQL w przeglądarce | LWW per kolumna, `MERGE ADD`, jawne konflikty | future 2.5, 12.3, 12.5 |
| Zero konfiguracji | tak | Flat-SQL | utrzymać, dodać obsługę wielu kart | future 1.1 |
| Zapytania reaktywne | brak | Riffle/SKDB [D4] | aktualizacja przyrostowa w µs | future 5.1 |

Czego nie robić: gonić pełnego SQL/PostgreSQL (moduł urośnie kilkanaście razy), konkurować z DuckDB w analityce, wprowadzać wątki wymagające COOP/COEP (także do odtwarzania), zapisywać obrazu pamięci modułu zamiast przenośnego formatu, stroić silnik pod pojedynczy test ([B3]).

## 5. Zastosowania

Katalog zastosowań z ocenami i warunkami: [../use-cases.md](../use-cases.md); dowody z rzeczywistych wdrożeń: [../research/real-world-usage.md](../research/real-world-usage.md). W skrócie:

| Zastosowanie | Dowody z wdrożeń | Co jest potrzebne |
|---|---|---|
| Zbieranie danych w terenie offline | mocne | replikacja częściowa, uwierzytelnianie, reguły serwera, eksport do PostgreSQL (future 3.2, 4.1, §2, 11.1) |
| Aplikacje osobiste na wielu urządzeniach | mocne (architektura jak w Actual Budget) | baza na użytkownika, uwierzytelnianie, szyfrowanie (future 3.1, 4.1, 4.3) |
| Pamięć podręczna danych serwera (SaaS) | mocne | wiele kart, tabele serwera, tryb pamięci podręcznej, replikacja częściowa (future 1.1, 2.1, 1.2, 3.2) |
| Gry przeglądarkowe – zapis i ekwipunek | słabe–średnie (zapisy w chmurze) | checkpoint bez przestoju (todo 2.6), LWW per kolumna (future 12.3), `MERGE ADD` (future 2.5); w grach wieloosobowych serwer jako autorytet (future 2.1) |
| Kasy i sprzedaż offline | średnie | tabele serwera i zamiary zatwierdzane przez serwer (future 2.1, 2.4) |
| Repliki konfiguracji na edge (flagi, personalizacja) | średnie | węzły edge z bazą główną (future §7) |

**Nie polecać:** analityki dużych plików (DuckDB-Wasm [D1]), systemów płatności jako źródła prawdy ([C9]), piaskownic i nauki SQL wymagających zgodności z PostgreSQL/SQLite, wspólnej edycji tekstu w czasie rzeczywistym bez zewnętrznej biblioteki CRDT, danych większych niż pamięć urządzenia.

## 6. Wnioski z samego przeglądu literatury

- Pierwotny zbiór (77 PDF-ów) pokrywał dobrze WebAssembly, CRDT i składowanie, ale miał luki dokładnie tam, gdzie leżą atuty i plany projektu: praca offline klientów mobilnych, replikacja częściowa, synchronizacja tabel w aplikacjach, przewidywalność ograniczonego języka zapytań, checkpointy stanu w pamięci, utrzymanie widoków ([search-methodology.md §3](search-methodology.md#3-audyt-pokrycia-atutów-projektu)). Uzupełniono je ośmioma pracami.
- Rdzeń liczy 38 pozycji: 30 z pierwotnego zbioru i 8 nowych; 46 pozycji przeniesiono poza rdzeń z uzasadnieniem ([README.md §13](README.md#13-pozycje-poza-rdzeniem)).
- Trzy ważne pozycje pozostają niedostępne: książka [P1] (płatna), Teilen-SQL [X5] (tylko streszczenie), artykuł Niinikoski i Fagerholm [X6].
- Nadal nie ma recenzowanych badań wydajności IndexedDB i OPFS ani kosztu granicy JS↔WASM w silnikach baz – pomiary projektu częściowo wypełniają tę lukę.
- Notatka do [P1] powstała bez dostępu do tekstu książki – przed cytowaniem trzeba ją zweryfikować z egzemplarzem.

## 7. Słowniczek

| Pojęcie | Znaczenie |
|---|---|
| IndexedDB | wbudowana w przeglądarkę baza klucz–wartość na dysku; Flat-SQL zapisuje w niej WAL i snapshot |
| OPFS | Origin Private File System – prywatny folder plików strony; najszybszy dostęp tylko z Web Workera |
| WAL | Write-Ahead Log – dziennik zatwierdzonych zmian dopisywany na końcu; po awarii odtwarza się z niego stan |
| Snapshot / checkpoint | pełny obraz bazy zapisany w danym momencie, po którym WAL można wyczyścić |
| Group commit | zapis wielu zatwierdzeń naraz jedną operacją na dysku |
| `fsync` | wymuszenie fizycznego zapisu danych z bufora systemu na dysk |
| LWW | Last-Writer-Wins – w konflikcie wygrywa zmiana z późniejszym znacznikiem |
| Utracona aktualizacja | zmiana, która zniknęła, bo współbieżna zmiana z nowszym znacznikiem ją nadpisała |
| Aktualizacja przemienna | zmiana, której wynik nie zależy od kolejności (np. dodanie stałej do licznika) |
| HLC | Hybrid Logical Clock – zegar łączący czas fizyczny z licznikiem logicznym; nigdy się nie cofa |
| CRDT | typ danych, którego repliki zawsze dochodzą do tego samego stanu bez koordynacji |
| Tombstone | znacznik usuniętego wiersza, chroniący przed jego „wskrzeszeniem” przez spóźnioną zmianę |
| Stabilność przyczynowa | stan, w którym wszystkie repliki znają daną zmianę – wtedy jej metadane (np. tombstone) można usunąć |
| Outbox | kolejka lokalnych zmian czekających na wysłanie do serwera |
| Zapis wstępny / zamiar | zmiana wykonana lokalnie, która staje się ostateczna dopiero po zatwierdzeniu przez serwer |
| Replikacja częściowa | klient przechowuje tylko potrzebną mu część bazy (np. własne dane, swój region) |
| MAV | Monotonic Atomic View – widoczne są wszystkie albo żadne zmiany danej transakcji |
| Granica JS↔WASM | koszt przekazywania parametrów i wyników między JavaScriptem a modułem WebAssembly |
| MVP (WebAssembly) | pierwsza, minimalna wersja standardu z 2017 r. opisana w [P2] (m.in. pamięć do 4 GiB) |
| Indeks pokrywający | indeks zawierający wszystkie dane potrzebne zapytaniu, więc nie trzeba czytać wierszy |
| Zapytanie reaktywne | subskrypcja zapytania, której wynik silnik sam aktualizuje po zmianie danych |
| Szyfrowanie end-to-end | dane szyfrowane na urządzeniu; serwer przechowuje i przekazuje tylko szyfrogram |
| COOP/COEP | nagłówki HTTP włączające izolację strony (potrzebne m.in. dla `SharedArrayBuffer`, wątków WASM i dokładniejszych zegarów) |
