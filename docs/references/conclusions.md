# Wnioski z analizy projektu i przeglądu literatury

Stan na 18 października 2026. Dokument zbiera w jednym miejscu wnioski z analizy prototypu Flat-SQL i z przeglądu publikacji zebranych w [bazie wiedzy](README.md). Szczegółowe uzasadnienia: [literature-comparison.md](literature-comparison.md); sposób wyszukiwania i jego ograniczenia: [search-methodology.md](search-methodology.md). Zadania wynikające z tych wniosków: **[../todo.md](../todo.md)**.

Identyfikatory w nawiasach (np. [W1]) odsyłają do wpisów w [README.md](README.md); hipotezy H1–H6 – do [benchmark-scenario.md](../research/benchmark-scenario.md).

## 1. Najważniejsze wnioski

1. **Flat-SQL nie jest lepszy od dojrzałych silników ogólnie, ale jest konkurencyjny w swojej kategorii:** mała baza SQL na dane operacyjne aplikacji, działająca w przeglądarce bez dodatkowej konfiguracji, z przyrostową trwałością i wbudowaną synchronizacją. Żadne z porównywanych rozwiązań nie łączy wszystkich tych cech w jednym module.
2. **Wyniki pomiarów są zgodne z literaturą.** Narzut WebAssembly względem kodu natywnego (1,1–1,8×) mieści się w przedziale z [P2] i [W1].
3. **Największym kosztem jest granica JS↔WASM i reprezentacja danych w pamięci, a nie sam WebAssembly.** Pomiar dwóch ścieżek przekazywania wyników (H3) dotyczy obszaru słabo zbadanego – to wkład własny projektu.
4. **Warstwa składowania realizuje klasyczny, dobrze opisany wzorzec** (dane w pamięci, WAL, snapshot, group commit – [S1], [S4], [P1]). Przyrostowy WAL daje przewagę nad sql.js i nad najnowszym Zeta-Lite [D5], który zapisuje tylko snapshoty.
5. **Synchronizacja LWW per wiersz zapewnia zbieżność** (to poprawny typ CRDT – [L2]), **ale gubi współbieżne zmiany różnych kolumn jednego wiersza.** Badania i praktyka idą w stronę LWW per kolumna i liczników CRDT ([L24], [L25], [L8], [Q10]).
6. **Wykryto luki poprawności:** brak ograniczenia dryfu zegara HLC [C2], niepewna kolejność utrwalania przy checkpoincie [S8], domyślnie „luźna” trwałość IndexedDB w Chrome [Q1], możliwość usunięcia danych przez przeglądarkę [Q2]–[Q4], dzielenie transakcji między paczki synchronizacji [C8], konflikty schematu między replikami [L9], [L26].
7. **„Privacy by Design” jest spełnione częściowo:** dane są domyślnie lokalne, ale nie są szyfrowane ani lokalnie, ani w synchronizacji, a serwer nie uwierzytelnia klientów ([L1], [L11], [L18], [V1]).
8. **Model „bez JOIN” ma mocne oparcie** w pracach o systemach rozproszonych ([C7], [C6], [C4]), **ale jest krytykowany** w literaturze baz danych i aplikacji local-first ([M1], [D4]). Trudność replikacji leży w więzach integralności, nie w samych złączeniach [L8].
9. **Metodyka benchmarku jest staranna,** ale do mocnych tez brakuje porównania z SQLite WASM na OPFS oraz statystyki z wielu uruchomień ([B1], [B3]).
10. **Przegląd literatury jest ukierunkowany, nie systematyczny.** Pokrywa prace fundamentalne i najważniejsze nowe (2025–2026), ale wymaga uzupełnienia w bazach bibliograficznych uczelni.
11. **Tryb edge zrealizowano tylko częściowo.** Silnik działa na serwerze (ten sam moduł `.wasm` w Node.js), ale jako jeden centralny serwer, który jest jednocześnie hubem synchronizacji i bazą główną. Zakładanej w prezentacji architektury – węzłów brzegowych blisko użytkowników synchronizowanych z bazą główną – jeszcze nie ma. Fundamenty do jej dobudowania istnieją (rdzeń bez I/O, protokół push/pull, zbieżność LWW niezależna od układu połączeń).

## 2. Flat-SQL na tle istniejących rozwiązań

| Rozwiązanie | SQL | Trwałość w przeglądarce | Synchronizacja | Moduł (gzip) |
|---|---|---|---|---|
| **Flat-SQL** | okrojony (bez JOIN) | przyrostowa (WAL w IndexedDB), bez Workera i nagłówków | wbudowana | **206 KiB** (zmierzono) |
| sql.js | pełny | eksport całej bazy | brak | 315 KiB (zmierzono) |
| SQLite WASM (oficjalny) | pełny | OPFS, wymaga Web Workera | zewnętrzna (np. cr-sqlite) | ok. 1 MB* |
| PGlite | pełny PostgreSQL | IndexedDB / OPFS | zewnętrzna (Electric) | < 3 MB* |
| DuckDB-Wasm [D1] | pełny, analityczny | głównie odczyt plików | brak | ok. 2,5 MB* |
| Zeta-Lite [D5] | pełny PostgreSQL | snapshot do OPFS na żądanie | brak | 2,87 MB* |
| CRR [L24], [L25], cr-sqlite | pełny (warstwa nad PostgreSQL/SQLite) | zależnie od bazy | wbudowana, per kolumna | – |
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
- synchronizacja: konflikt rozstrzygany dla całego wiersza; brak szyfrowania i uwierzytelniania;
- odczyt po kluczu wolniejszy niż deklarowane przez autora Zeta-Lite ok. 3–4 µs (inne warunki pomiaru);
- dojrzałość: prototyp kontra systemy produkcyjne.

**Zastrzeżenie:** przewaga w trwałości zapisu jest zmierzona tylko względem sql.js – najsłabszego pod tym względem wariantu SQLite. Oficjalny SQLite WASM z OPFS również zapisuje przyrostowo i nie został zmierzony. Poprawne sformułowanie brzmi „szybszy niż sql.js”, a nie „szybszy niż SQLite”.

**Proponowane ujęcie w pracy:** Flat-SQL świadomie rezygnuje z części możliwości SQL (złączenia, podzapytania) w zamian za mały moduł, przyrostową trwałość bez wymagań środowiskowych i wbudowaną synchronizację Local-First. W typowych operacjach aplikacji (odczyt po kluczu, grupowanie, sortowanie) jest konkurencyjny wobec SQLite w WASM, a słabszy w operacjach, w których SQLite korzysta z formatu stronicowego.

## 3. Wnioski z badań według obszarów

### 3.1 WebAssembly

- Narzut WASM na pełnych aplikacjach to średnio 1,45–1,55× ([W1]); przyczyny – presja na rejestry i dodatkowe kontrole przy każdym wywołaniu – tłumaczą spowolnienie rekurencyjnego `eval` opisane w wynikach projektu.
- Optymalizacje LLVM nie są dostrojone do WASM; wyższy poziom optymalizacji nie zawsze daje szybszy kod, a programy WASM zużywają więcej pamięci niż JS ([W11]).
- Izolacja WASM chroni hosta przed modułem, ale nie dane modułu przed jego błędami ([P2], [W2], [W9]); bezpieczny Rust bez `unsafe` ogranicza tę klasę zagrożeń.
- Wasm 3.0 (2025) dodaje m.in. memory64, ale 64-bitowa pamięć ma wyraźny koszt sprawdzania granic ([W10], [W6]).
- Wymiana danych z JS najlepiej działa przez binarny bufor czytany wprost z pamięci WASM (Arrow w DuckDB-Wasm [D1]).

### 3.2 Local-First

- Projekt spełnia ideały szybkości, pracy na wielu urządzeniach i pracy offline; częściowo – współpracę, długowieczność danych, prywatność i kontrolę użytkownika ([L1]).
- W praktyce najbardziej wykonalne są modele hybrydowe z serwerem koordynującym – taki jest Flat-SQL ([L20]).
- Local-first nie zastąpi chmury m.in. przy transakcjach finansowych i dużych zbiorach danych ([L22]).
- Dane w przeglądarce nie są gwarantowane: Safari usuwa je po 7 dniach bez interakcji, magazyn jest domyślnie „best-effort” ([Q2]–[Q4]).

### 3.3 Synchronizacja i konflikty

- Zegar HLC jest zaimplementowany w zwartej postaci z [C2], ale bez reguły odrzucania znaczników zbyt daleko w przyszłości – urządzenie ze złym zegarem wygrywa wszystkie konflikty ([C2], [P1] rozdz. 8).
- Atomowość transakcji między replikami da się zapewnić bez koordynacji (Monotonic Atomic View); zapobieganie utraconym aktualizacjom – nie ([C8]).
- Unikalność klucza głównego jest gwarantowana tylko lokalnie; CRR [L24] wykrywa takie konflikty przy scalaniu i jawnie cofa przegraną zmianę.
- Sposób rozstrzygania konfliktów można deklarować w SQL per tabela lub kolumna (Antidote SQL [L8]).
- Ewolucja schematu w replikach to problem otwarty; Cambria [L26] tłumaczy dane między wersjami schematu.
- Protokół (log z kursorami, kompaktowanie, idempotentne rundy) odpowiada wzorcom z [C3], [C7] i [P1] rozdz. 11–12.

### 3.4 Składowanie i trwałość

- Log fizyczny jest właściwym wyborem: te same wpisy służą odtwarzaniu i synchronizacji, a odtworzenie nie zależy od determinizmu zapytań ([S6]).
- Group commit opisali po raz pierwszy w druku DeWitt i in. ([S1]).
- Bez `fsync` katalogu po `rename` kolejność utrwalania snapshotu i obcięcia WAL nie jest gwarantowana ([S8]); test „każdy prefiks WAL” odpowiada modelowi awarii z [S9], ale nie obejmuje protokołu między plikami.
- Od Chrome 121 `await flush()` gwarantuje przetrwanie awarii karty i przeglądarki, ale nie awarii systemu, chyba że użyje się `durability: 'strict'` ([Q1]).

### 3.5 Model danych

- Encja pod kluczem jako jednostka atomowości: [C7], [C6], [C4]; najbliższy przemysłowy odpowiednik to CQL w Cassandrze [C5].
- Systemy NoSQL niemal powszechnie dodały SQL, a brak złączeń jest uznawany za ograniczenie ([M1]).
- W aplikacjach local-first złączenia i reaktywność decydują o responsywności UI ([D4]); w modelu jednotabelowym reaktywne zapytania z aktualizacją przyrostową są znacznie prostsze do zrealizowania – to szansa, nie tylko ograniczenie.

### 3.6 Edge computing

- WebAssembly jest uznawany za wspólną warstwę wykonawczą od urządzeń po chmurę ([E6], [E3]–[E5]); uruchomienie tego samego modułu na serwerze wpisuje się w ten kierunek.
- **Stan prototypu:** jest jeden serwer ([server/server.js](../../server/server.js)) pełniący naraz trzy role – hub synchronizacji (`/sync/push`, `/sync/pull`), API „Cloud-First” do porównań w H1 (`/api/query` z symulowanym opóźnieniem) i hosting dema. Ten serwer **jest** bazą główną: nie łączy się z żadną bazą wyżej, a jego log zmian jest przechowywany tylko w pamięci. To, że działa w Node.js, nie jest ograniczeniem – węzeł edge też może być procesem Node.js uruchomionym w kilku regionach; brakuje **układu połączeń** i wdrożenia na platformach edge.
- **Założenie z prezentacji:** przeglądarki ↔ węzły edge (repliki blisko użytkowników) ↔ baza główna. W literaturze to replikacja multi-leader w topologii drzewa ([P1] rozdz. 5) i dokładnie scenariusz CRR [L24] (synchronizacja urządzeń brzegowych z chmurą); węzeł edge może dalej obsługiwać użytkowników przy niedostępnej centrali ([E2]). Zbieżność LWW na HLC nie zależy od topologii, o ile każda zmiana w końcu dotrze wszędzie ([L2]).
- **Czego brakuje do tego trybu:** przekazywania w górę zmian przyjętych od klientów (dziś celowo nie trafiają do outboksu serwera, by nie wracały „echem”), rozsyłania w dół zmian z bazy głównej bez odsyłania ich z powrotem, trwałego logu i kursorów, uwierzytelniania między węzłami oraz testów w układzie klienci ↔ kilka węzłów ↔ centrala ([../todo.md](../todo.md), zadania 4.10–4.11).
- Na platformach typu Cloudflare Workers stan między żądaniami wymaga trwałego magazynu platformy; jednowątkowy obiekt ze stanem w pamięci i trwałym magazynem (np. Durable Object) odpowiada modelowi wykonania Flat-SQL, a brak I/O w rdzeniu pozwala dopisać dla niego backend `Storage`.
- Przeglądarka jest konkurencyjna przy małych danych, serwery z kompilacją AOT przy dużych ([E7]) – co wspiera podział: dane operacyjne lokalnie, ciężka analityka gdzie indziej.
- Lokalne zapytanie (11 µs) mieści się w budżecie 16 ms na klatkę z dużym zapasem; zapytanie przez sieć z RTT 20 ms już go przekracza ([D4], H1).

### 3.7 Metodyka pomiarów

- Spełnione: powtarzalność, optymalizacja systemu porównywanego, kontrola liczby wierszy, rozgrzewka ([B3], [B2]).
- Do uzupełnienia: wiele uruchomień i przedziały ufności ([B1]), osobne przebiegi zimne ([B3]), realistyczny rozkład dostępu ([B5], [B4], [W7]), zużycie pamięci ([W11]), uczciwy punkt odniesienia dla trwałości (SQLite z OPFS, [Q7]).

## 4. Kierunek rozwoju: najlepszy w swojej kategorii

Przewyższenie wszystkich rozwiązań we wszystkim nie jest możliwe – pełny SQL kosztuje rozmiar modułu, analityka dużych plików wymaga silnika kolumnowego, a dojrzałość SQLite to dekady pracy. Realny cel to pierwsze miejsce na jasno zdefiniowanej tablicy wyników:

| Metryka | Teraz | Prowadzi dziś | Cel |
|---|---|---|---|
| Odczyt po kluczu z JS | 9,7 µs | Zeta-Lite ~3–4 µs (wg autora) | < 3 µs |
| Trwałe zatwierdzenie | 0,34 ms | Flat-SQL (wobec sql.js) | utrzymać, porównać z SQLite + OPFS |
| Rozmiar modułu | 206 KiB | Flat-SQL | < 150 KiB |
| Synchronizacja bez cichej utraty danych | LWW per wiersz | brak wbudowanej w silnikach SQL w przeglądarce | LWW per kolumna, liczniki, atomowe transakcje |
| Zero konfiguracji | tak | Flat-SQL | utrzymać, dodać obsługę wielu kart |
| Zapytania reaktywne | brak | Riffle/SKDB [D4] | aktualizacja przyrostowa w µs |

Czego nie robić: gonić pełnego SQL/PostgreSQL (moduł urośnie kilkanaście razy), konkurować z DuckDB w analityce, wprowadzać wątki wymagające COOP/COEP, stroić silnik pod pojedynczy test ([B3]). Kolejne kroki: [../todo.md](../todo.md).

## 5. Zastosowania

Pełny katalog zastosowań – z podziałem danych w grach, przepisem na portfel kryptowalut, scenariuszem z backendem, synchronizacją z PostgreSQL/SQLite i ograniczeniami – jest w [../use-cases.md](../use-cases.md). Poniżej skrót.

### 5.1 Ocena zastosowań z prezentacji projektu

| Zastosowanie ([ProductPresentation.pdf](../ProductPresentation.pdf)) | Dopasowanie | Uzasadnienie i warunki |
|---|---|---|
| Gry przeglądarkowe (ekwipunek, statystyki) | bardzo dobre | odczyty w µs, trwały zapis 0,34 ms, group commit pozwala utrwalać zmiany w każdej klatce; liczniki (waluta, doświadczenie) na wielu urządzeniach wymagają scalania sumującego; w grach wieloosobowych serwer musi pozostać autorytetem, bo dane lokalne kontroluje użytkownik |
| Analityka CSV 500 MB | słabe | dane w pamięci w reprezentacji wierszowej, limit `wasm32`, brak przetwarzania kolumnowego – właściwy jest DuckDB-Wasm [D1]; Flat-SQL nadaje się do zbiorów rzędu setek tysięcy wierszy |
| Agregacja metryk i telemetria na edge | dobre, z warunkiem | paczkowanie zdarzeń to wzorzec outboksu i group commitu [S1], grupowanie jest szybkie; wymaga węzłów edge przekazujących paczki do bazy głównej, których prototyp jeszcze nie ma (dziś jest jeden centralny serwer), a na platformach edge – backendu `Storage` dla ich magazynu |
| Sesje i autoryzacja na edge | ryzykowne | spójność ostateczna: unieważniona sesja może być jeszcze ważna na innym węźle do czasu synchronizacji ([P1] rozdz. 9); bezpieczny wariant to replika tylko do odczytu z krótkim czasem życia sesji i natychmiastowym rozsyłaniem unieważnień |
| Edytory graficzne – katalog assetów | dobre | metadane i filtrowanie po indeksach, praca offline, synchronizacja; same pliki binarne poza tabelą (OPFS/IndexedDB); relacje wiele-do-wielu (tagi) wymagają modelowania bez JOIN |
| Portfele kryptowalut | częściowe | historia transakcji – dobre (lokalnie, prywatnie, szybkie filtrowanie); kluczy prywatnych nie przechowywać w bazie ani w IndexedDB (brak szyfrowania, dostęp skryptów domeny – [W2], [W9]), lecz w WebCrypto lub portfelu sprzętowym; local-first nie zastępuje serwera przy transakcjach finansowych [L22] |
| Geolokalizacja i personalizacja na edge | bardzo dobre | dane referencyjne rzadko zmieniane przez jednego autora (LWW wystarcza), odczyt po kluczu w µs; potrzebne węzły edge synchronizowane z bazą główną i rozsyłanie zmian z centrali (push) |
| Feature flags / testy A/B | bardzo dobre | jak wyżej; przydział wariantu liczony lokalnie z identyfikatora użytkownika; zmiany flag rozsyłane w sekundach |

### 5.2 Zastosowania wynikające z obecnych mocnych stron

- **Aplikacje terenowe offline** (inspekcje, inwentaryzacja, serwis, ankiety, dostawy) – trwały outbox przetrwa zamknięcie aplikacji bez sieci.
- **Aplikacje osobiste na wielu urządzeniach** (notatki, zadania, nawyki, dziennik treningów) – szybki start dzięki małemu modułowi, praca offline, synchronizacja.
- **Autozapis i szkice** (formularze wieloetapowe, edytory) – zapis przy każdej zmianie bez spowalniania interfejsu.
- **Lokalna replika danych referencyjnych** (katalog produktów, cenniki, słowniki, rozkłady jazdy) – głównie odczyty po kluczu, przyrostowa synchronizacja.
- **Rozszerzenia przeglądarki i widżety osadzane na cudzych stronach** – działa w wątku głównym bez Workera i bez nagłówków COOP/COEP, a SQLite z trwałością na OPFS wymaga Web Workera.
- **Edukacja przy słabym internecie** – treści i postępy offline, synchronizacja po powrocie sieci.
- **Bufor telemetrii po stronie klienta** – zdarzenia zbierane offline i wysyłane paczkami.
- **Narzędzia administracyjne** – wspólny format plików pozwala otworzyć bazę serwera w CLI, a w przyszłości w panelu WinUI.

### 5.3 Zastosowania odblokowane przez planowane funkcje

| Funkcja (zob. [../todo.md](../todo.md)) | Nowe zastosowania |
|---|---|
| LWW per kolumna + liczniki (`MERGE ADD`) | wspólne listy i tablice zadań, stany magazynowe liczone z wielu skanerów, ekonomia gry na wielu urządzeniach |
| Zapytania reaktywne | interfejsy i pulpity aktualizowane bez ręcznego odświeżania |
| Szyfrowanie end-to-end i lokalne | dane wrażliwe: zdrowie, dzienniki, finanse osobiste (śledzenie wydatków), dokumenty firmowe |
| Złączenia tylko do odczytu | proste CRM, katalogi z relacjami |
| Węzły edge synchronizowane z bazą główną, wdrożenie na platformie edge | scenariusze edge z prezentacji (flagi, personalizacja, telemetria) w rzeczywistym układzie: użytkownik → najbliższy węzeł → centrala |

**Nie polecać:** analityki dużych plików, systemów płatności i rozliczeń jako źródła prawdy, wspólnej edycji tekstu w czasie rzeczywistym (wymaga CRDT tekstu, np. [L16]), danych większych niż pamięć urządzenia.

## 6. Wnioski z samego przeglądu literatury

- Zebrano 77 publikacji i dokumentów w otwartym dostępie; korzystano wyłącznie z legalnych źródeł (wersje autorskie, repozytoria uczelni, otwarty dostęp wydawców).
- Trzy ważne pozycje pozostają niedostępne: książka [P1] (płatna), Teilen-SQL [X5] (tylko streszczenie), artykuł Niinikoski i Fagerholm [X6] (jeszcze nieopublikowany).
- Notatka do [P1] powstała bez dostępu do tekstu książki – przed cytowaniem trzeba ją zweryfikować z egzemplarzem.
- Nie znaleziono recenzowanych badań wydajności IndexedDB i OPFS ani kosztu granicy JS↔WASM w silnikach baz danych – pomiary projektu częściowo wypełniają tę lukę.
- Do uzupełnienia: przeszukanie Scopus, Web of Science, IEEE Xplore i ACM DL zapytaniami z [search-methodology.md §5](search-methodology.md#5-jak-uzupełnić-przegląd-biblioteka-uczelni).

## 7. Słowniczek

| Pojęcie | Znaczenie |
|---|---|
| IndexedDB | wbudowana w przeglądarkę baza klucz–wartość na dysku; Flat-SQL zapisuje w niej WAL i snapshot |
| OPFS | Origin Private File System – prywatny folder plików strony w przeglądarce; najszybszy dostęp tylko z Web Workera |
| WAL | Write-Ahead Log – dziennik zatwierdzonych zmian dopisywany na końcu; po awarii odtwarza się z niego stan |
| Snapshot / checkpoint | pełny obraz bazy zapisany w danym momencie, po którym WAL można wyczyścić |
| Group commit | zapis wielu zatwierdzeń naraz jedną operacją na dysku |
| `fsync` | wymuszenie fizycznego zapisu danych z bufora systemu na dysk |
| LWW | Last-Writer-Wins – w konflikcie wygrywa zmiana z późniejszym znacznikiem |
| HLC | Hybrid Logical Clock – zegar łączący czas fizyczny z licznikiem logicznym; nigdy się nie cofa |
| CRDT | typ danych, którego repliki zawsze dochodzą do tego samego stanu bez koordynacji |
| Tombstone | znacznik usuniętego wiersza, chroniący przed jego „wskrzeszeniem” przez spóźnioną zmianę |
| Outbox | kolejka lokalnych zmian czekających na wysłanie do serwera |
| MAV | Monotonic Atomic View – widoczne są wszystkie albo żadne zmiany danej transakcji |
| Granica JS↔WASM | koszt przekazywania parametrów i wyników między JavaScriptem a modułem WebAssembly |
| MVP (WebAssembly) | pierwsza, minimalna wersja standardu z 2017 r. opisana w [P2] (m.in. pamięć do 4 GiB) |
| Indeks pokrywający | indeks zawierający wszystkie dane potrzebne zapytaniu, więc nie trzeba czytać wierszy |
| Zapytanie reaktywne | subskrypcja zapytania, której wynik silnik sam aktualizuje po zmianie danych |
| Szyfrowanie end-to-end | dane szyfrowane na urządzeniu; serwer przechowuje i przekazuje tylko szyfrogram |
| COOP/COEP | nagłówki HTTP włączające izolację strony (potrzebne m.in. dla `SharedArrayBuffer` i dokładniejszych zegarów) |
