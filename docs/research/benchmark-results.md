# Wyniki benchmarków i walidacji offline

Pomiary z 6 października 2026 według [scenariusza badawczego](benchmark-scenario.md): AMD Ryzen 5 4600H, 16 GB RAM, Windows 11, Node.js 22.23.2, Chromium 152, Rust 1.99, sql.js 1.14.2, N = 100 000 wierszy. Polecenie: `node bench/run.js --rows 100000 --native`.

## 1. Node.js (V8) – operacje w pamięci

Średni czas operacji (w nawiasie p95). Insert – czas całej transakcji i przepustowość.

| Operacja | Flat-SQL WASM (obiekty) | Flat-SQL WASM (JSON) | sql.js (SQLite) | czysty JS (`Map`) | Flat-SQL natywnie |
|---|---:|---:|---:|---:|---:|
| Insert, 1 transakcja | 758 ms (132 tys./s) | 555 ms (180 tys./s) | 344 ms (291 tys./s) | 45 ms (2,2 mln/s) | 223 ms (447 tys./s) |
| PK lookup | 18,6 µs (24,4) | **9,7 µs** (9,5) | 20,3 µs (26,6) | 0,64 µs | 2,3 µs |
| Lookup po indeksie + COUNT | 613 µs (975) | 573 µs (801) | **100 µs** (136) | 2,6 µs | 442 µs |
| Zakres PK (1000 wierszy) + SUM | 153 µs | 124 µs | 128 µs | 78 µs | 85 µs |
| Pełny skan z filtrem | 13,4 ms | 11,5 ms | 9,7 ms | 2,0 ms | 8,2 ms |
| GROUP BY (100 grup) | 27,9 ms | **26,4 ms** | 130,3 ms | 3,8 ms | 15,1 ms |
| ORDER BY … LIMIT 10 | 7,4 ms | **7,1 ms** | 10,6 ms | 1,7 ms | 6,5 ms |
| Pobranie 10 000 wierszy do JS | 16,8 ms | 18,5 ms | 16,9 ms | 1,0 ms | – |
| UPDATE 10% wierszy | 57,8 ms | 56,2 ms | **18,0 ms** | 6,0 ms | 30,9 ms |
| DELETE 10% wierszy | 43,4 ms | 41,6 ms | **25,1 ms** | 6,8 ms | 25,2 ms |
| Snapshot / eksport | 49,0 ms | 43,4 ms | **2,2 ms** | 28,5 ms | 25,8 ms |
| Odtworzenie / import | 200 ms | 192 ms | **1,2 ms** | 51 ms | 114 ms |
| Rozmiar snapshotu | 5,27 MB | 5,27 MB | 4,70 MB | 3,17 MB (JSON) | 5,27 MB |

| Moduł WebAssembly | Rozmiar | gzip | Kompilacja (`WebAssembly.compile`) |
|---|---:|---:|---:|
| Flat-SQL (Rust → wasm32) | 545 KiB | **206 KiB** | 2,7 ms |
| sql.js (SQLite → Emscripten) | 643 KiB | 315 KiB | 3,3 ms |

Wszystkie silniki zwróciły te same liczby wierszy dla każdego zapytania i po odtworzeniu snapshotu (90 000).

## 2. Przeglądarka (Chromium 152) – operacje w pamięci

`performance.now()` w przeglądarce ma rozdzielczość 100 µs, więc wiarygodne są średnie (percentyle krótkich operacji pominięto).

| Operacja | Flat-SQL WASM (obiekty) | Flat-SQL WASM (JSON) | sql.js | czysty JS |
|---|---:|---:|---:|---:|
| Insert, 1 transakcja | 765 ms (131 tys./s) | 580 ms (172 tys./s) | 280 ms (357 tys./s) | 54 ms |
| PK lookup | 15,5 µs | **11,4 µs** | 18,0 µs | 1,1 µs |
| Lookup po indeksie + COUNT | 614 µs | 569 µs | **72 µs** | 0,8 µs |
| Zakres PK + SUM | 132 µs | 152 µs | 100 µs | 29 µs |
| Pełny skan z filtrem | 11,1 ms | 11,4 ms | 9,0 ms | 1,4 ms |
| GROUP BY | 28,6 ms | **26,1 ms** | 121,8 ms | 2,8 ms |
| ORDER BY … LIMIT 10 | **7,0 ms** | 7,2 ms | 11,8 ms | 1,4 ms |
| Pobranie 10 000 wierszy | 21,1 ms | **14,9 ms** | 24,9 ms | 1,0 ms |
| UPDATE 10% | 51,9 ms | 41,0 ms | **14,7 ms** | 4,9 ms |
| DELETE 10% | 42,4 ms | 38,0 ms | **22,0 ms** | 9,5 ms |
| Snapshot / eksport | 57,5 ms | 40,6 ms | **1,2 ms** | 22,1 ms |
| Odtworzenie / import | 187 ms | 200 ms | **1,1 ms** | 49,8 ms |

| Moduł | Pobranie (localhost) | Kompilacja |
|---|---:|---:|
| Flat-SQL | 21,7 ms | 28,8 ms |
| sql.js | 16,8 ms | 51,9 ms |

## 3. Trwałość w przeglądarce (IndexedDB)

| Pomiar (50 000 wierszy) | Wynik |
|---|---:|
| Flat-SQL: 50 000 insertów w 1 transakcji – wykonanie | 211,5 ms |
| Flat-SQL: … + zapis WAL (2,86 MB) w IndexedDB | 225,2 ms |
| Flat-SQL: pojedyncze zatwierdzenie + `await flush()` | **0,34 ms** (p95 0,5 ms) |
| Flat-SQL: 1000 zatwierdzeń + 1 `flush()` (group commit) | 5,6 ms (5,6 µs / zatwierdzenie) |
| Flat-SQL: checkpoint (snapshot 3,02 MB do IndexedDB) | 34,9 ms |
| Flat-SQL: ponowne otwarcie bazy (IndexedDB → pamięć) | 49,3 ms (odtworzenie: 40,0 ms) |
| sql.js: zatwierdzenie = `export()` całej bazy (1,68 MB) + zapis do IndexedDB | **8,68 ms** |

Ten sam pomiar dla 10 000 wierszy: Flat-SQL 0,53 ms, sql.js 3,84 ms (plik 0,33 MB).

## 4. Local-First vs Cloud-First

PK lookup na tabeli 10 000 wierszy (ten sam silnik lokalnie i na serwerze).

| Tryb | Node.js – średnio | Node.js – p95 | Przeglądarka – średnio |
|---|---:|---:|---:|
| lokalnie (WASM w procesie, z parsowaniem SQL) | **0,011 ms** | 0,012 ms | ≈ 0,011 ms |
| cloud API, localhost | 0,63 ms | 0,78 ms | 4,0 ms |
| cloud API, RTT 20 ms | 20,9 ms | 21,3 ms | 23,8 ms |
| cloud API, RTT 50 ms | 52,3 ms | 52,8 ms | 60,2 ms |
| cloud API, RTT 100 ms | 102,5 ms | 103,1 ms | 108,6 ms |

## 5. Wpływ optymalizacji silnika (natywnie, 100 000 wierszy)

Pierwsza wersja silnika vs wersja po profilowaniu (commit „engine: speed up predicate evaluation, grouping and top-k”):

| Operacja | Przed | Po | Zmiana |
|---|---:|---:|---:|
| Insert | 344 tys./s | 447 tys./s | +30% |
| Pełny skan z filtrem | 15,9 ms | 8,2 ms | −48% |
| GROUP BY | 27,9 ms | 15,1 ms | −46% |
| ORDER BY … LIMIT 10 | 27,5 ms | 6,5 ms | −76% |
| Odtworzenie snapshotu | 145 ms | 114 ms | −21% |

Główna przyczyna wolnego filtrowania okazała się nieoczywista: rekurencyjna funkcja `eval` z dużym `match` (wiele inlinowanych gałęzi, obsługa błędów) miała ramkę stosu ~500 B i zapisywała wszystkie rejestry przy **każdym** wywołaniu – także dla odczytu kolumny. Obsługa liści wyrażeń inline i osobny ewaluator predykatów skróciły koszt porównania `kolumna < stała` z ~90 ns do ~30 ns.

## 6. Interpretacja

**H1 – Local-First vs Cloud-First: potwierdzona.** Lokalne zapytanie (11 µs) jest ~57× szybsze niż zapytanie do serwera na tej samej maszynie i ~1 900× / ~9 300× szybsze przy RTT 20 / 100 ms. Czas zapytania przez sieć jest praktycznie równy RTT – koszt wykonania na serwerze jest pomijalny, a skrócić go może tylko przeniesienie danych bliżej użytkownika. W przeglądarce dochodzi jeszcze ~3–8 ms narzutu `fetch` na każde żądanie.

**H2 – narzut WebAssembly: potwierdzona z zastrzeżeniem.** Dla operacji obliczeniowych wewnątrz silnika WASM jest 1,1–1,8× wolniejszy od kodu natywnego (top-k 1,10×, lookup po indeksie 1,30×, pełny skan 1,41×, zakres 1,45×, snapshot/odtworzenie 1,68×, GROUP BY 1,74×). Większe różnice (PK lookup 4,3×, insert 2,5×) wynikają głównie z kosztu granicy JS↔WASM (konwersja parametrów i wyników), a nie z wolniejszego kodu – przy operacjach trwających pojedyncze mikrosekundy koszt przejścia przez granicę dominuje.

**H3 – koszt interfejsu Rust↔JS: potwierdzona.** Zwracanie wyniku jako JSON (jedno przekazanie łańcucha + `JSON.parse`) jest ~1,9× szybsze niż budowanie obiektów JS z Rust dla wyników jednowierszowych (9,7 vs 18,6 µs). Dla dużych wyników różnica zależy od środowiska (Node.js: porównywalnie; przeglądarka: JSON 1,4× szybszy). Wniosek projektowy: domyślna ścieżka bridge'a powinna używać JSON lub binarnego formatu kolumnowego, a nie budowania obiektów przez `js-sys`.

**H4 – Flat-SQL vs SQLite (sql.js): potwierdzona.**
- Flat-SQL jest szybszy w zapytaniach punktowych (2,1×), grupowaniu (4,9×; plan SQLite to `SCAN items USING INDEX items_category` – przegląd w kolejności indeksu i pobranie każdego wiersza po `rowid`, czyli 100 000 losowych odczytów, podczas gdy Flat-SQL agreguje w jednym sekwencyjnym przebiegu) i top-k (1,5×; SQLite: `USE TEMP B-TREE FOR ORDER BY`, Flat-SQL: ograniczony bufor k najlepszych wierszy), porównywalny w zakresach i pobieraniu wyników, nieco wolniejszy w pełnym skanie (1,2×).
- SQLite wygrywa wyraźnie tam, gdzie korzysta ze swojej architektury stronicowej: snapshot i odtworzenie to kopia pliku (20× i 160× szybciej), a `COUNT(*)` po indeksie liczy wpisy w B-drzewie indeksu bez odczytu wierszy (`USING COVERING INDEX`, 5,7×). Flat-SQL pobiera każdy wiersz z drzewa klucza głównego i ponownie sprawdza filtr; dalsze kierunki: indeksy pokrywające / zliczanie po indeksie, pominięcie filtra rezydualnego, gdy ścieżka dostępu pokrywa cały warunek, oraz zapis wierszy w ciągłych blokach zamiast osobnych alokacji.
- Inserty, UPDATE i DELETE: SQLite 1,6–3,1× szybszy; w Flat-SQL każda zmiana tworzy wpis w undo logu, kodowaną operację WAL i referencję w outboksie (koszt wbudowanej trwałości i synchronizacji, którego sql.js nie ponosi, bo ich nie ma).

**H5 – trwałość: potwierdzona.** Trwałe zatwierdzenie w Flat-SQL kosztuje ~0,3–0,5 ms niezależnie od rozmiaru bazy (zapisywana jest tylko ramka WAL), a przy group commit ~6 µs na transakcję. W sql.js jedyną drogą do trwałości jest eksport całej bazy – 3,8 ms przy 10 000 i 8,7 ms przy 50 000 wierszy (wzrost liniowy z rozmiarem). Koszt Flat-SQL przenosi się na checkpoint (35 ms dla 3 MB) wykonywany rzadko, w tle, oraz na czas otwarcia bazy (49 ms dla 50 000 wierszy).

**H6 – praca offline i zbieżność: potwierdzona.** Wszystkie testy z sekcji 6 scenariusza przechodzą (82 testy Rust, 14 testów end-to-end). Scenariusz ręczny w przeglądarce wykonano w pełni: przy zatrzymanym serwerze aplikacja uruchomiła się z cache Service Workera, odtworzyła bazę z IndexedDB w 8,7 ms (10 ramek WAL), zapis wykonał się lokalnie w 0,6 ms, niewysłana zmiana przetrwała odświeżenie strony, a po ponownym uruchomieniu serwera została wysłana automatycznie i była widoczna w zapytaniu do serwera. Repliki osiągają identyczny stan także w losowych sesjach z konfliktami.

## 7. Wnioski

1. Przeniesienie wykonania zapytań do klienta daje latencję na poziomie mikrosekund – o 2–4 rzędy wielkości mniejszą niż w modelu Cloud-First – co realizuje założenie „latencji 0 ms”.
2. Silnik napisany w Rust i skompilowany do WASM jest konkurencyjny wobec SQLite w WASM w typowych zapytaniach aplikacji (odczyty punktowe, agregacje, sortowanie), przy module o ~1/3 mniejszym po kompresji.
3. Największymi kosztami okazały się granica JS↔WASM i reprezentacja danych w pamięci (osobne alokacje wierszy), a nie sam WebAssembly; to wyznacza kierunki dalszej optymalizacji.
4. Przyrostowy WAL z group commitem jest właściwą strategią trwałości w przeglądarce – koszt zatwierdzenia nie rośnie z rozmiarem bazy, w przeciwieństwie do eksportu całego pliku.
5. Model płaskich tabel adresowanych kluczem umożliwił prostą, deterministyczną synchronizację (LWW per wiersz), która zachowuje pełną funkcjonalność offline.
