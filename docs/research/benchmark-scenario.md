# Scenariusz badawczy i metodyka

Dokument definiuje pytania badawcze, hipotezy, obciążenie testowe, silniki odniesienia, metryki, procedurę pomiaru i zagrożenia dla trafności wyników. Ta sama metodyka obowiązuje we wszystkich pomiarach porównywanych ze sobą: „Benchmark 1” mierzy prototyp przed optymalizacjami, a kolejne pomiary mierzą tym samym narzędziem zarówno prototyp, jak i wersję po zmianach. Wyniki: [benchmark-results.md](benchmark-results.md). Opis narzędzia: [bench/README.md](../../bench/README.md).

## 1. Pytania badawcze i hipotezy

| | Pytanie | Hipoteza |
|---|---|---|
| H1 | O ile lokalne wykonanie zapytania (WASM w procesie aplikacji) jest szybsze od modelu Cloud-First? | Lokalne zapytanie po kluczu trwa mikrosekundy; zapytanie przez sieć jest ograniczone od dołu czasem RTT, więc różnica wynosi 2–4 rzędy wielkości. |
| H2 | Jaki jest koszt uruchamiania silnika w WebAssembly zamiast natywnie? | Ten sam kod Rust w WASM jest 1,2–3× wolniejszy od natywnego, zależnie od rodzaju operacji. |
| H3 | Ile kosztuje granica Rust↔JavaScript i jak ją ograniczyć? | Dla małych wyników dominuje koszt przekazywania danych; serializacja do JSON w Rust jest szybsza niż budowanie obiektów JS przez `js-sys`. |
| H4 | Jak Flat-SQL wypada na tle dojrzałych silników SQL w WASM? | Flat-SQL jest konkurencyjny w zapytaniach punktowych, agregacji i sortowaniu, słabszy w operacjach, w których SQLite korzysta z formatu stronicowego (snapshot i odtwarzanie) i z indeksów pokrywających. |
| H5 | Jaki jest koszt trwałego zatwierdzenia w przeglądarce? | Przyrostowy WAL w IndexedDB daje koszt zależny od rozmiaru zmiany i niezależny od rozmiaru bazy; eksport całej bazy (sql.js) rośnie liniowo z jej rozmiarem. |
| H6 | Czy aplikacja zachowuje pełną funkcjonalność offline i czy repliki są zbieżne po odzyskaniu połączenia? | Tak: odczyty i zapisy działają bez serwera, niewysłane zmiany przetrwają restart aplikacji, a po synchronizacji wszystkie repliki mają identyczny stan. |

Oprócz hipotez pomiar odpowiada na pytania pomocnicze: jaki jest rozmiar modułu i czas zimnego startu każdego silnika, ile pamięci zajmuje baza 100 000 wierszy, jak długo checkpoint blokuje wątek główny, ile kosztuje przeniesienie silnika do Web Workera, ile kosztuje tryb trwałości `strict` w IndexedDB i ile kosztuje `fsync` przy zapisie plików.

## 2. Środowisko

| Element | Wartość |
|---|---|
| Sprzęt | laptop, AMD Ryzen 5 4600H (6 rdzeni, 12 wątków), 16 GB RAM, dysk SSD NVMe 512 GB (SK hynix) |
| System | Windows 11 Home 10.0.26200, plan zasilania „Zrównoważony”, zasilanie sieciowe |
| Node.js | 24.21.0 |
| Przeglądarka | Microsoft Edge 156 (silnik Chromium 156, V8), tryb bez okna (headless), sterowana przez playwright-core 1.64.1 |
| Rust | 1.99.0, profil `release` (`opt-level = 3`, LTO, `codegen-units = 1`), `wasm-opt -O3` |
| Wersje silników | dokładnie przypięte w `package.json`: sql.js 1.14.2, @sqlite.org/sqlite-wasm 3.54.1-build1 (SQLite 3.54.1), @journeyapps/wa-sqlite 2.0.8, @electric-sql/pglite 0.5.11, @duckdb/duckdb-wasm 1.32.0, alasql 4.21.0, dexie 4.4.7, pouchdb-browser 9.0.0, rxdb 17.6.0 |

Runner zapisuje w pliku `manifest.json` każdego pomiaru wersje wszystkich składników, model procesora, system, wersję mierzonego Flat-SQL (`git describe` i skrót commita) oraz rejestr konfiguracji silników. Przed pomiarem zamyka się inne aplikacje, a laptop pracuje na zasilaniu sieciowym. Systemu nie strojono (bez wyłączania trybu turbo i bez przypisywania procesów do rdzeni), bo wynik ma opisywać typowe urządzenie użytkownika.

## 3. Obciążenie

Tabela wspólna dla silników SQL:

```sql
CREATE TABLE items (id INTEGER PRIMARY KEY, category TEXT, price FLOAT, qty INT, name TEXT);
CREATE INDEX items_category ON items (category);
```

Wiersz `i` powstaje deterministycznie: `(i, 'c' || (i % 100), ((i * 7919) % 10000) / 100.0, i % 50, 'item-' || i)`. Daje to 100 kategorii po 1000 wierszy i ceny rozłożone pseudolosowo; każda cena występuje dokładnie 10 razy. Domyślnie N = 100 000 wierszy. `INTEGER PRIMARY KEY` czyni kolumnę aliasem `rowid` w SQLite, czyli najszybszym układem tabeli tego silnika. PostgreSQL (PGlite) i DuckDB dostają te same typy w swoim dialekcie (`DOUBLE PRECISION`, `DOUBLE`, `VARCHAR`), a parametry `$1, $2…` zamiast `?` w PGlite.

| Operacja | Zapytanie | Wykonań w uruchomieniu |
|---|---|---:|
| Wstawienie | N × przygotowane `INSERT INTO items VALUES (?, ?, ?, ?, ?)` w jednej transakcji | 1 |
| Odczyt po kluczu | `SELECT * FROM items WHERE id = ?` (losowe `id`) | 10 000 |
| COUNT po indeksie | `SELECT COUNT(*) FROM items WHERE category = ?` | 500 |
| Zakres klucza | `SELECT SUM(price) FROM items WHERE id BETWEEN ? AND ?` (1000 kolejnych kluczy) | 200 |
| Pełny skan z filtrem | `SELECT COUNT(*) FROM items WHERE price > 50 AND qty < 10` | 20 |
| Grupowanie | `SELECT category, COUNT(*), AVG(price), MAX(qty) FROM items GROUP BY category` | 20 |
| Top-k | `SELECT id, price FROM items ORDER BY price DESC, id ASC LIMIT 10` | 20 |
| Pobranie wyników | `SELECT * FROM items WHERE id < 10000` (10 000 wierszy do JS) | 20 |
| UPDATE 10% | `UPDATE items SET qty = qty + 1 WHERE id % 10 = 0` | 1 |
| DELETE 10% | `DELETE FROM items WHERE id % 10 = 1` | 1 |
| Snapshot | zapis całej bazy do bajtów: Flat-SQL `snapshot()`, sql.js `export()`, SQLite WASM `sqlite3_js_db_export`, PGlite `dumpDataDir`, czysty JS `JSON.stringify` | 1 |
| Odtworzenie | baza z tych bajtów: `Database.restore`, `new SQL.Database(bytes)`, `sqlite3_deserialize`, `PGlite.create({ loadDataDir })`, `JSON.parse` z odbudową indeksu | 1 |
| Wstawienie jednym wywołaniem (od wersji obciążenia `benchmark-2`) | te same N wierszy do tabeli `items_many` o tej samej definicji, w jednej transakcji; Flat-SQL: `executeMany`, gdy wersja je ma, w innym wypadku N × `execute` | 1 |

Ostatnia operacja doszła po Benchmarku 1 i jest wykonywana na końcu, więc wcześniejsze operacje pozostają takie same jak w Benchmarku 1; mierzą ją tylko adaptery Flat-SQL i natywny benchmark. Zapytania odczytu są przygotowywane raz i wykonywane z parametrami, a ich wyniki są w całości przenoszone do JavaScriptu jako tablice wierszy, tak jak w aplikacji. Silniki, których API nie ma przygotowanych instrukcji (PGlite), wykonują za każdym razem zapytanie z parametrami. Kryterium `id` w top-k rozstrzyga remisy cen, więc odpowiedź jest jednoznaczna w każdym silniku.

Parametry każdej operacji pochodzą z generatora liniowego o stałym ziarnie (osobne ziarno dla każdej operacji), są wyliczane przed pomiarem i są identyczne we wszystkich silnikach, środowiskach i uruchomieniach. Liczba wykonań rozgrzewających nie przesuwa więc ciągu kluczy mierzonych.

Silniki z trwałym magazynem wykonują dodatkowo fazę zapisu trwałego: 100 zatwierdzeń po jednym wierszu, z których każde czeka na trwałość, potem 1000 zatwierdzeń wysłanych bez czekania i jedno oczekiwanie na trwałość wszystkich (*group commit*), na koniec zamknięcie bazy, ponowne otwarcie i pierwsze zapytanie. W tej grupie wstawienie, UPDATE i DELETE są mierzone do chwili, w której zmiana jest trwała.

## 4. Silniki i grupy warunków uruchomienia

Wynik porównuje się tylko w obrębie grupy silników uruchomionych w tych samych warunkach: tego samego wątku, tego samego rodzaju API (synchroniczne albo asynchroniczne) i tego samego magazynu. Zestawienie silnika pracującego w pamięci z silnikiem, który każdy zapis utrwala na dysku, mierzyłoby różnicę warunków zamiast różnicy silników [B3].

| Grupa | Warunki | Środowisko | Konfiguracje |
|---|---|---|---|
| pamięć | dane w pamięci, silnik w wątku wywołującym (proces Node.js, wątek główny przeglądarki), bez nagłówków i bez Workera | Node.js, przeglądarka | Flat-SQL (wyniki jako obiekty JS), Flat-SQL (wyniki jako JSON), sql.js, SQLite WASM (`:memory:`, API oo1), PGlite (pamięć, API asynchroniczne), DuckDB-Wasm (pakiet synchroniczny), AlaSQL, czysty JavaScript na `Map` |
| trwały zapis | magazyn IndexedDB, wątek główny przeglądarki, zapis mierzony do trwałości | przeglądarka | Flat-SQL z IndexedDB (tryb trwałości domyślny i `strict`), sql.js z eksportem całej bazy do IndexedDB, wa-sqlite z `IDBBatchAtomicVFS` (`synchronous=full` i `normal`), PGlite z IndexedDB, Dexie.js (tryb domyślny i `strict`), PouchDB, RxDB na magazynie Dexie |
| Worker | silnik w dedykowanym Web Workerze; każde wywołanie to komunikat `postMessage` w obie strony mierzony w wątku głównym | przeglądarka | Flat-SQL w pamięci, Flat-SQL z IndexedDB, SQLite WASM z `opfs-sahpool` (OPFS), PGlite z OPFS (`opfs-ahp`), DuckDB-Wasm w pamięci |
| natywnie | ten sam rdzeń Rust skompilowany natywnie, bez WASM i bez granicy JS | proces CLI | Flat-SQL (`flatsql bench`) |

Każdy silnik pracuje w konfiguracji domyślnej albo zalecanej w swojej dokumentacji. Wyjątki są jawne w nazwie konfiguracji (`strict`, `synchronous=normal`). DuckDB-Wasm w przeglądarce zwykle działa we własnym Workerze; w grupie „pamięć” użyto jego pakietu synchronicznego, który wykonuje ten sam silnik w wątku wywołującym, a w grupie „Worker” ten sam pakiet działa w Workerze narzędzia. Jednowątkowa kompilacja DuckDB i VFS `opfs-sahpool` nie używają `SharedArrayBuffer`.

Bazy bez SQL (Dexie.js, PouchDB, RxDB) wykonują te same operacje przez własne API. Odwzorowanie opisano w kodzie adapterów i w tabeli poniżej; to, czego API nie wyraża (np. `GROUP BY`), liczy kod JavaScript adaptera, a ten czas wchodzi do wyniku. Odpowiedzi są porównywane sumą kontrolną tak samo jak w silnikach SQL.

| Operacja | Dexie.js | PouchDB | RxDB |
|---|---|---|---|
| Odczyt po kluczu | `get(id)` | `get(_id)` | `findOne` z kluczem głównym (pamięć podręczna dokumentów RxDB, jeśli dokument był już czytany) |
| COUNT po indeksie | `where('category').equals(c).count()` | trwały widok z reduktorem `_count` | `count()` z indeksem `category` |
| Zakres klucza | zakres po kluczu, suma w JS | `allDocs` z zakresem kluczy, suma w JS | selektor na kluczu głównym, suma w JS |
| Pełny skan, grupowanie | odczyt wszystkich obiektów, filtr i agregacja w JS | `allDocs`, filtr i agregacja w JS | `find`, agregacja w JS |
| Top-k | indeks `price` malejąco, remisy w JS | trwały widok po `price` malejąco, remisy w JS | `find` z sortowaniem `price, id` i limitem |
| UPDATE, DELETE | `modify` / `delete` w jednej transakcji | `allDocs` i `bulkDocs` | `find` i `bulkUpsert` / `bulkRemove` |
| Zatwierdzenie | `add`, każde we własnej transakcji IndexedDB | `put` | `insert` |

PouchDB buduje widoki leniwie przy pierwszym zapytaniu; budowę mierzy się osobno zaraz po wstawieniu danych. RxDB zwraca wynik powtórzonego zapytania z pamięci podręcznej zapytań, dopóki kolekcja się nie zmieniła; adapter opróżnia tę pamięć przed każdym zapytaniem, bez zmiany samego zapytania, więc każde wywołanie jest wykonywane przez magazyn. Zapytania po zakresie klucza głównego dostają wskazówkę indeksu klucza (pole `index` zapytania, opisane w dokumentacji RxDB), bo bez niej planer RxDB wybierał indeks `category` i czytał całą kolekcję (zmierzono 48 ms zamiast 22 ms dla 1000 z 2000 dokumentów). Pierwsza wersja adaptera wymuszała ponowne wykonanie dodatkowym warunkiem na polu bez indeksu, co psuło wybór indeksu; pomiar RxDB powtórzono z poprawionym adapterem.

Silniki rozważane, ale niemierzone:

| Silnik | Powód |
|---|---|
| Zeta-Lite [D5] | kod zamknięty, brak pakietu ani publicznego modułu do pobrania (stan na październik 2026); wyniki podają osobno wartości deklarowane przez autora w preprincie (odczyt po kluczu, rozmiar modułu), oznaczone jako niepotwierdzone pomiarem i nieporównywalne wprost, bo warunki pomiaru autora są inne |
| absurd-sql | projekt nierozwijany od 2021 r., jego rolę przejęły VFS na OPFS (mierzony `opfs-sahpool`) i IndexedDB (mierzony wa-sqlite) |
| SQLite z VFS `opfs` | wymaga nagłówków COOP/COEP i `SharedArrayBuffer`; dokumentacja SQLite wskazuje `opfs-sahpool` jako szybszy wariant bez tych wymagań [Q7] |

## 5. Metodyka pomiaru

### 5.1 Powtórzenia i kolejność

Jednostką powtórzenia jest proces. Każdy pomiar (jeden silnik, jeden rodzaj zadania) działa w nowym procesie Node.js albo w nowym procesie przeglądarki z pustym profilem i jest powtarzany co najmniej 10 razy. Kompilator JIT, odśmiecanie pamięci i rozmieszczenie danych w pamięci zmieniają się między procesami bardziej niż między iteracjami jednej pętli, więc dopiero powtórzenie procesu pokazuje zmienność, z którą trzeba porównywać różnice [B1]. Pierwsza wersja narzędzia mierzyła każdy silnik w jednym procesie i pokazała narzut WebAssembly 1,1–1,8× w operacjach wykonywanych w całości w silniku; dziesięć procesów w Benchmarku 1 dało w tych samych operacjach 0,89–1,13×. Kolejność silników w każdym uruchomieniu jest inna, losowana generatorem o ziarnie równym numerowi uruchomienia; żaden silnik nie jest systematycznie mierzony jako pierwszy po starcie maszyny ani zaraz po innym, obciążającym silniku.

### 5.2 Rozgrzewka

Przed pomiarem każdy silnik wykonuje w tym samym procesie skrócony przebieg na osobnej bazie: 10 000 wierszy i do 200 wykonań każdego zapytania. V8 kompiluje WebAssembly najpierw kompilatorem bazowym, a gorące funkcje optymalizuje później; bez rozgrzewki pierwsza faza płaciłaby za tę kompilację. Każda operacja odczytu ma dodatkowo n/10 wykonań rozgrzewających tuż przed pomiarem.

### 5.3 Pomiar czasu

Czas pojedynczej operacji to czas pętli wszystkich jej wykonań podzielony przez ich liczbę. W pętli są tylko wywołanie silnika, dwa odczyty zegara i zapisanie referencji do wyniku; parametry są przygotowane wcześniej, a sumę kontrolną liczy się po pętli. Z pojedynczych próbek wyznacza się też p50, p95 i p99 w obrębie uruchomienia. Zegar: `performance.now()` w JavaScripcie, `std::time::Instant` w Rust.

W przeglądarce strona pomiarowa jest izolowana (*cross-origin isolated*, nagłówki COOP i COEP), dzięki czemu `performance.now()` ma rozdzielczość 5 µs zamiast 100 µs [Q5]. Żadna mierzona konfiguracja nie korzysta z możliwości, które izolacja odblokowuje (`SharedArrayBuffer`, wątki WebAssembly), więc nagłówki zmieniają tylko przyrząd. Percentyle operacji krótszych niż kilkadziesiąt mikrosekund pozostają w przeglądarce skwantowane co 5 µs; średnia z pętli tego problemu nie ma.

Operacja odczytu ma limit 20 s na uruchomienie. Silnik, który go przekroczy, kończy pętlę po co najmniej trzech wykonaniach, a tabela wyników podaje liczbę wykonanych powtórzeń. Limit dotyczy tylko silników, w których jedno wykonanie trwa dziesiątki milisekund (np. PGlite z IndexedDB).

### 5.4 Statystyka

Dla każdej metryki zbiera się jedną wartość z każdego uruchomienia (np. średni czas odczytu po kluczu w tym uruchomieniu). Wynik to mediana tych wartości z pierwszym i trzecim kwartylem; mediana jest odporna na pojedyncze zakłócone uruchomienie. Plik `summary.json` zawiera też średnią z 95% przedziałem ufności (rozkład t Studenta). Zmianę między dwiema wersjami silnika uznaje się za wykazaną, gdy 95% przedziały ufności średnich nie zachodzą na siebie [B1]; mniejsze różnice opisuje się jako mieszczące się w rozrzucie.

### 5.5 Kontrola wyników

Każdy wynik zapytania jest zapisywany w postaci kanonicznej: liczba (całkowita, zmiennoprzecinkowa, `BigInt`, `Decimal`) jako liczba całkowita równa wartości pomnożonej przez 10^6 i zaokrąglonej, tekst z prefiksem, `NULL` jako znacznik. Wiersze zapytań bez `ORDER BY` są sortowane. Z tego tekstu liczy się 64-bitową sumę kontrolną (dwa 32-bitowe skróty FNV-1a z różnymi mnożnikami) osobno dla każdej operacji: po pierwszych trzech wykonaniach i po wszystkich. Po wstawieniu, UPDATE, DELETE, odtworzeniu, zatwierdzeniach i ponownym otwarciu silnik zwraca stan tabeli: `COUNT(*)`, `SUM(qty)`, `SUM(price)`. Benchmark natywny w Rust liczy tę samą sumę tym samym algorytmem.

Raport porównuje sumy kontrolne i stan tabeli każdego uruchomienia każdego silnika z silnikiem odniesienia grupy. Wyniki zapytań mają co najwyżej kilkaset tysięcy w części całkowitej i są sumami najwyżej 1000 cen, więc błąd zaokrąglenia jest o rzędy wielkości mniejszy niż 10^-6 i różna kolejność sumowania w silnikach nie zmienia sumy kontrolnej. Inaczej jest ze stanem tabeli: suma 100 000 cen (około 5·10^6) zależy od kolejności dodawania już na szóstym miejscu po przecinku, a silniki dokumentowe zwracają dokumenty w innej kolejności niż silniki SQL. Stan tabeli porównuje się więc z tolerancją względną 10^-9, a raport podaje, ile porównań jej wymagało. Sama liczba wierszy, porównywana we wstępnych pomiarach, nie wykryłaby np. złego wiersza zwróconego po kluczu [B3].

### 5.6 Zimny start

Zimny start to czas od rozpoczęcia importu silnika do wyniku pierwszego zapytania w nowym procesie, bez pamięci podręcznej. Składa się z trzech faz: import modułu JavaScript z pobraniem, kompilacją i utworzeniem instancji WebAssembly; otwarcie bazy z utworzeniem tabeli i indeksu; pierwszy zapis i pierwszy odczyt. W przeglądarce każdy pomiar to nowy proces z nowym profilem, więc nie ma pamięci podręcznej HTTP ani skompilowanego kodu. Pliki są serwowane z `localhost`, czyli czas pobrania nie zawiera sieci; rozmiar modułu podaje się osobno, żeby dało się przeliczyć pobranie dla dowolnego łącza. Rozmiar to suma wszystkich plików silnika pobranych przez stronę (z wyłączeniem kodu narzędzia), skompresowanych gzip na poziomie 9. W Node.js zimny start mierzy się od importu silnika w świeżym procesie; start samego Node.js jest zapisywany osobno.

### 5.7 Pamięć

Pamięć mierzy się w osobnym procesie: silnik otwiera pustą bazę, następuje pomiar, wczytanie N wierszy i drugi pomiar. Rozmiar pamięci liniowej WebAssembly odczytuje sonda, która przed załadowaniem silnika opakowuje API `WebAssembly` i zapamiętuje (słabymi referencjami) każdą utworzoną lub wyeksportowaną pamięć. Pamięć liniowa nie maleje i nie jest zwracana systemowi [W11], więc jej rozmiar po wczytaniu jest pamięcią, którą silnik zajmuje od tej chwili, łącznie ze szczytem zużycia podczas wczytywania. W Node.js dochodzi przyrost sterty JS po pełnym odśmiecaniu (`--expose-gc`), który jest miarą pamięci silników napisanych w JavaScripcie; w przeglądarce `performance.measureUserAgentSpecificMemory()`.

### 5.8 Trwały zapis i tryb `strict`

Zatwierdzenie jest trwałe, gdy magazyn potwierdził zapis: w Flat-SQL po `await db.flush()`, w sql.js po zapisaniu wyeksportowanego pliku w IndexedDB, w pozostałych silnikach po rozwiązaniu obietnicy zapisu. IndexedDB w Chromium od wersji 121 domyślnie kończy transakcję w trybie `relaxed`, czyli po przekazaniu danych systemowi operacyjnemu; tryb `strict` czeka na zapis na dysk [Q1]. Konfiguracje „strict” Flat-SQL i Dexie.js mierzą koszt tej gwarancji. wa-sqlite z `IDBBatchAtomicVFS` wybiera tryb przez `PRAGMA synchronous` (`full` to `strict` i jest domyślne w SQLite). SQLite z `opfs-sahpool`, PGlite i bazy dokumentowe utrwalają każdą transakcję osobno, więc 1000 zatwierdzeń kosztuje w nich 1000 utrwaleń.

### 5.9 Pauza checkpointu

Checkpoint Flat-SQL zapisuje snapshot całej bazy i obcina WAL; kodowanie snapshotu jest synchroniczne. Podczas `checkpoint()` w wątku głównym działa sonda: pętla komunikatów `MessageChannel`, która mierzy przerwy między kolejnymi obrotami pętli zdarzeń. Wynik to najdłuższa przerwa dla baz 10 000, 50 000 i 100 000 wierszy, na tle budżetu jednej klatki przy 60 Hz (16,7 ms). Ten sam pomiar dla sql.js obejmuje eksport całej bazy i zapis pliku do IndexedDB. Sonda działa także przez 200 ms bezczynności, co pokazuje przerwy niezwiązane z silnikiem.

### 5.10 Koszt Workera

Koszt przeniesienia silnika do Web Workera to różnica median tej samej operacji w konfiguracji Flat-SQL w Workerze i Flat-SQL w wątku głównym (oba w pamięci, wyniki jako obiekty JS). Dla odczytu po kluczu różnica jest kosztem komunikatu w obie strony, a dla pobrania 10 000 wierszy dochodzi kopiowanie wyniku przez *structured clone*.

### 5.11 Local-First a Cloud-First (H1)

Ten sam silnik i ta sama tabela (10 000 wierszy) po obu stronach. Zapytanie `SELECT * FROM bench_remote WHERE id = <losowe>`:

- lokalnie: `db.exec(sql)` w procesie (parsowanie, wykonanie, przekazanie wyniku), 2000 powtórzeń, pierwsze 200 odrzucone;
- przez API: `POST /api/query` do serwera Node.js z tym samym silnikiem, z opóźnieniem sieci symulowanym po stronie serwera: 0 (localhost), 20, 50 i 100 ms; 40 żądań (20 dla co najmniej 50 ms), pierwsze 3 odrzucone (rozgrzanie połączenia keep-alive).

Symulacja zakłada opóźnienie równe RTT, bez strat pakietów i bez ustanawiania połączenia i TLS, więc wynik jest dolnym ograniczeniem kosztu modelu Cloud-First. W Node.js klient używa `node:http` z keep-alive, bo wbudowany `fetch` (undici) dodaje w Windows około 15 ms do każdego żądania; serwer odmierza opóźnienie dokładnie (timer i dopełnienie przez `setImmediate`), bo `setTimeout` w Windows ma kwant około 15,6 ms. W przeglądarce strona i serwer synchronizacji mają różne porty, więc zapytanie jest żądaniem cross-origin z nagłówkami CORS.

### 5.12 Koszt utrwalania plików

Ta sama faza zapisu trwałego co w przeglądarce (wstawienie N wierszy, 100 zatwierdzeń, 1000 zatwierdzeń z jednym oczekiwaniem, checkpoint, ponowne otwarcie) na plikach: `FileStorage` w natywnym CLI (`flatsql bench --storage KATALOG`) i `FsStore` w Node.js, każdy z `fsync` i bez. Dla serwera synchronizacji mierzy się rundę z jedną zmianą (`/sync/push` i `/sync/pull`) wobec serwera z katalogiem danych (`FsStore`) i serwera w pamięci. Katalog roboczy jest na tym samym dysku SSD co system.

### 5.13 Procedura

Polecenia, wymagania, czas trwania i sposób sprawdzenia wyniku opisuje instrukcja odtworzenia badań: [bench/README.md](../../bench/README.md). Runner zapisuje wynik każdego procesu w osobnym pliku `raw/<środowisko>/<rodzaj>/<silnik>/NN.json` i można go przerwać i wznowić tym samym poleceniem. Skrypt raportu jest samodzielny (tylko moduły wbudowane Node.js) i odtwarza wszystkie tabele i wykresy z surowych wyników. Narzędzie pochodzi zawsze z bieżącego kodu, a mierzona wersja silnika z osobnej kopii roboczej, więc prototyp (tag `v1-prototyp`) i wersję po optymalizacjach (tag `v2-optymalizacja`) mierzy to samo narzędzie. Pomiar ręczny (inna przeglądarka, telefon) wykonuje się stroną `bench/browser/manual.html`; wynik ma ten sam format co pomiar automatyczny.

## 6. Walidacja Local-First w warunkach offline (H6)

Walidacja ma trzy poziomy – od deterministycznych testów jednostkowych po ręczny scenariusz w przeglądarce.

### 6.1 Testy silnika (Rust, `crates/flatsql-core/tests/`)

| Test | Weryfikuje |
|---|---|
| `every_wal_prefix_recovers_to_a_committed_state` | ucięcie WAL w każdym bajcie → baza równa stanowi po ostatniej pełnej transakcji |
| `torn_wal_tail_is_discarded`, `corrupted_snapshot_is_rejected` | urwany zapis i uszkodzony snapshot |
| `frames_covered_by_snapshot_are_skipped` | awaria między snapshotem a obcięciem WAL |
| `outbox_survives_restart_and_ack_is_durable` | niewysłane zmiany przetrwają restart; potwierdzone nie są wysyłane ponownie |
| `concurrent_offline_edits_converge_with_last_writer_wins`, `delete_wins_over_older_update` | konflikty edycji i usunięć |
| `randomized_offline_sessions_converge` | 25 ziaren × 300 losowych operacji na 3 replikach z przerywaną łącznością → identyczny stan wszystkich replik i serwera |

### 6.2 Testy end-to-end (Node.js, `test/offline-sync.test.js`)

Prawdziwy serwer HTTP, klienci `FlatDB` + `SyncClient`:

1. dwie repliki synchronizują dane przez serwer,
2. **serwer zostaje wyłączony** → 50 zapisów na replice A i edycja na replice B kończą się lokalnie (bez czekania na sieć), odczyty działają, runda synchronizacji kończy się statusem `offline`, outbox = 51,
3. aplikacja A zostaje **zamknięta i otwarta ponownie** offline → outbox nadal = 51, dane na miejscu,
4. **serwer startuje ponownie** (nowy proces, ten sam katalog danych, nowa epoka) → po synchronizacji outboxy są puste, a A, B i serwer mają identyczną zawartość tabeli,
5. konflikt: obie repliki offline edytują ten sam wiersz – na wszystkich wygrywa późniejsza edycja, niezależnie od kolejności ponownego połączenia,
6. zapisy wykonane przez cloud API docierają do replik lokalnych.

### 6.3 Scenariusz ręczny w przeglądarce (demo)

1. Pierwsza wizyta w przeglądarce bez danych strony, serwer z katalogiem danych (`npm run start:persistent`): wczytanie danych, zapytania – status „zsynchronizowano”.
2. „Symuluj brak sieci” → zapisy lokalne, licznik oczekujących zmian rośnie.
3. Zatrzymanie serwera → zapis → **odświeżenie strony**: aplikacja startuje z cache Service Workera, dane z IndexedDB, liczba oczekujących zmian zachowana.
4. Ponowne uruchomienie serwera, bez odświeżania strony → zmiany zostają wysłane, serwer zwraca je w zapytaniu przez API.

Kryterium zaliczenia (H6): wszystkie kroki zakończone bez utraty zatwierdzonych (spłukanych) danych i ze zbieżnym stanem replik. Wynik: [tests-results.md §2.5](tests-results.md#25-praca-offline-w-aplikacji-demo).

## 7. Zagrożenia dla trafności

### 7.1 Trafność wewnętrzna

Szum pomiarowy pochodzi z kompilatora JIT, odśmiecania pamięci, procesów w tle, zmian częstotliwości procesora i temperatury laptopa. Ogranicza go powtarzanie pomiaru w nowych procesach, losowa kolejność silników i raportowanie mediany z kwartylami; rozrzut każdego pomiaru jest podany przy wyniku, a raport wymienia operacje o największym współczynniku zmienności. Uruchomienia nie są w pełni niezależne, bo dzielą tę samą maszynę i sesję systemu; trend w czasie (np. nagrzanie się laptopa) rozkłada się dzięki losowej kolejności na wszystkie silniki.

Adaptery silników odniesienia napisał autor Flat-SQL. Raasveldt i in. wskazują, że lepsza znajomość własnego systemu jest częstym źródłem nieuczciwych porównań [B3]. Każdy silnik używa więc API i konfiguracji zalecanych w jego dokumentacji, a odstępstwa (opróżnianie pamięci podręcznej zapytań RxDB i wskazówka indeksu, budowa widoków PouchDB poza pomiarem odczytów, pakiet synchroniczny DuckDB) są opisane wyżej i w kodzie adapterów. Przykład z RxDB pokazuje, że sposób użycia API potrafi zmienić wynik około stu razy (zakres klucza 2,9 s w pierwszej wersji adaptera, 30 ms po poprawce); dla silników, których API autor zna słabiej, wynik należy traktować jako górne ograniczenie czasu. Sumy kontrolne potwierdzają, że wszystkie adaptery zwracają te same odpowiedzi, ale nie potwierdzają, że wybrano najszybszy sposób ich uzyskania w danym silniku.

Wynik każdej operacji zawiera koszt przekazania odpowiedzi do JavaScriptu w postaci tablic wierszy. Dla silników, których naturalnym formatem są kolumny Apache Arrow (DuckDB-Wasm) albo obiekty (AlaSQL, bazy dokumentowe), zawiera więc też konwersję do tego formatu. Aplikacja, która korzysta z formatu natywnego silnika, zapłaciłaby mniej.

Gwarancje trwałości różnią się między silnikami mimo wspólnej definicji („obietnica zapisu rozwiązana”): tryb `relaxed` IndexedDB nie chroni przed awarią systemu, a magazyn plikowy bez `fsync` nie chroni przed utratą zasilania. Wyniki trwałego zapisu podaje się więc razem z trybem trwałości.

RxDB odczytuje dokument po kluczu z własnej pamięci podręcznej, gdy dokument był wcześniej odczytany; w obciążeniu poprzedza to sprawdzenie stanu tabeli, które czyta wszystkie dokumenty. Wynik RxDB w odczycie po kluczu opisuje więc tę pamięć podręczną; odczyt z IndexedDB jest wolniejszy.

Przeglądarka działa w trybie bez okna (headless). Silnik JavaScript, IndexedDB i OPFS są te same co w trybie z oknem, ale strona nie rysuje klatek, więc pomiar pauzy checkpointu nie obejmuje kosztu renderowania.

### 7.2 Trafność zewnętrzna

Wszystkie pomiary wykonano na jednym laptopie z Windows. Wartości bezwzględne nie przenoszą się na inne urządzenia, a zwłaszcza na telefony; przenoszą się proporcje między silnikami w tej samej grupie. Koszt `fsync` i trybu `strict` zależy od systemu plików i dysku.

Obciążenie jest syntetyczne: jedna tabela, równomierny rozkład kluczy i stały zestaw zapytań. Rzeczywiste aplikacje mają rozkład dostępu skośny (np. rozkład Zipfa, który przyjmuje benchmark magazynów adresowanych kluczem YCSB), wiele tabel i zapytania zależne od interfejsu. Wynik opisuje więc zachowanie silników w typowych operacjach niszy Flat-SQL; wydajność konkretnej aplikacji trzeba zmierzyć na jej obciążeniu.

Symulowana sieć pomija zmienność opóźnień, utratę pakietów, TLS i ustanawianie połączeń, więc przewaga modelu Local-First jest w niej zaniżona.

### 7.3 Trafność wniosków

Wstępne wyniki z 6 października 2026 pochodzą z jednego uruchomienia procesu na silnik, bez sumy kontrolnej i z innym zapytaniem top-k; nie porównuje się ich z Benchmarkiem 1 liczbowo. Wnioski o zmianie wydajności między wersjami silnika opierają się wyłącznie na pomiarach wykonanych tym samym narzędziem, w tym samym środowisku i według tej procedury.
