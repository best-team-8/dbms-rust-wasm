# Wyniki optymalizacji silnika

Dokument opisuje optymalizacje Flat-SQL wykonane po Benchmarku 1. Każdy opis wychodzi od słabej strony prototypu ([benchmark-results.md §1.11](benchmark-results.md#111-słabe-strony-prototypu-i-hipotezy-przyczyn)) i jej przyczyny wskazanej przez rozbicie kosztów ([§1.12](benchmark-results.md#112-rozbicie-kosztów-silnika-pomiar-pomocniczy)), profil albo plan zapytania, a kończy się pomiarem przed zmianą i po niej. Pełny pomiar wersji końcowej razem z prototypem, według [metodyki](benchmark-scenario.md), jest w [benchmark-results.md](benchmark-results.md); testy poprawności zmienionego kodu opisuje [tests-results.md](tests-results.md).

## Opis optymalizacji

| Element | Zawartość |
|---|---|
| Słabość | objaw z Benchmarku 1: operacja, wartość, porównanie z innym silnikiem albo z wykonaniem natywnym |
| Hipoteza | przyczyna i jej dowód: rozbicie kosztów, profil, plan zapytania, analiza kodu |
| Zmiana | co zmieniono, jakie rozwiązania odrzucono i dlaczego, z literaturą |
| Pomiar | polecenia, wersja przed i po (commit przez temat), środowisko, liczba uruchomień, obciążenie procesora przed pomiarem |
| Wynik | wartości przed i po w operacji docelowej oraz wielkości niezależne od obciążenia maszyny: liczba alokacji, rozmiar snapshotu, liczba odczytanych wierszy |
| Wpływ na inne operacje | zmiana pozostałych operacji obciążenia, rozmiaru modułu, czasu startu i pamięci |
| Ograniczenia | czego pomiar nie pokazuje i co go zaburza |

Status: *potwierdzona* (zmiana usunęła wskazany koszt w przewidywanym zakresie), *częściowo* (zysk mniejszy od przewidywanego, z podaną przyczyną) albo *odrzucona* (pomiar nie wykazał zysku albo zmiana pogorszyła inną operację). Optymalizacja odrzucona zostaje w dokumencie razem z pomiarem.

Pomiar przed i po wykonuje się w tej samej sesji, wersja po wersji, bez innych aplikacji obciążających procesor (scenariusz, §2). Zmianę uznaje się za wykazaną według kryterium ze scenariusza (§5.4); szybki pomiar z kilkoma uruchomieniami podaje rozrzut. Opis odwołuje się do literatury identyfikatorami z [indeksu](../references/README.md), do kodu przez plik i funkcję, a nie odwołuje się do rejestru decyzji ani list zadań.

### Procedura pomiaru pojedynczej zmiany

Każdą zmianę mierzy się trzema narzędziami, a surowe wyniki leżą w [data/optimization/](data/optimization/) (opis katalogów w jego README):

1. Pomiar przed i po (`bench/compare.js`): wersja sprzed zmiany (worktree poprzedniego stanu zbudowany przez `bench/build-version.js`) i wersja ze zmianą są mierzone tym samym runnerem co w Benchmarku 1, na przemian, każda w 10 nowych procesach Node.js i 10 procesach benchmarku natywnego; konfiguracje „Flat-SQL” (wynik jako obiekty JS) i „Flat-SQL (JSON)”. Przy każdej operacji obciążenia podaje się medianę z kwartylami obu wersji i to, czy 95% przedziały ufności średnich są rozłączne. Sumy kontrolne wyników obu wersji muszą być równe: optymalizacja nie może zmienić żadnej odpowiedzi. Zmiany granicy JS↔WASM mierzy się dodatkowo w przeglądarce.
2. Rozbicie kosztów (program `cost-split` z [benchmark-1/analysis](data/benchmark-1/analysis/cost-split/)): te same części operacji w obu wersjach, uruchamiane skryptem `run.mjs`, który buduje program z rdzeniem wskazanej wersji. Program mierzy też czas przygotowania każdej instrukcji obciążenia, bo część optymalizacji przenosi pracę z wykonania do przygotowania.
3. Liczba alokacji pamięci w tych samych operacjach (program `allocations`), niezależna od obciążenia maszyny.

Rozmiar modułu WebAssembly (gzip -9) obu wersji zapisuje się w manifeście pomiaru. Przy kilkudziesięciu porównaniach w jednym pomiarze pojedyncza operacja może mieć rozłączne przedziały ufności przypadkiem; taką różnicę w operacji, której zmieniony kod nie wykonuje, opisuje się jako zmienność maszyny.

## Zestawienie

| Optymalizacja | Słabość | Operacja docelowa: przed → po | Status |
|---|---|---|---|
| [1. Liczenie wierszy po indeksie](#1-liczenie-wierszy-po-indeksie-bez-ich-czytania) | S1 | `COUNT(*)` po indeksie natywnie 486 µs → 1,66 µs, Node.js 498 µs → 9,5 µs | potwierdzona |

## 1. Liczenie wierszy po indeksie bez ich czytania

| Element | Zawartość |
|---|---|
| Słabość | `SELECT COUNT(*) FROM items WHERE category = ?` trwał w Benchmarku 1 438 µs w przeglądarce i 457 µs natywnie, wobec 63–69 µs w SQLite (sql.js i oficjalny SQLite WASM). Ten sam czas natywnie i w WebAssembly oznacza koszt samego silnika. |
| Hipoteza | Indeks pomocniczy przechowuje dla każdej wartości zbiór kluczy głównych, więc liczba wierszy jest długością tego zbioru. Wykonawca pobierał jednak każdy z 1000 wierszy kategorii z drzewa klucza głównego, ponownie sprawdzał warunek `category = ?` i dopiero wtedy zwiększał licznik (`EXPLAIN`: `INDEX LOOKUP` z `FILTER category = …`). SQLite odpowiada z samego indeksu (`USING COVERING INDEX`). Rozbicie kosztów potwierdziło przyczynę: zapytanie trwało natywnie 489–495 µs, odczyt długości zbioru kluczy 66–71 ns, wyszukanie 1000 wierszy po kluczach z indeksu 205–207 µs, a odczyt wartości kolumny i porównanie w każdym wierszu dokładały kolejne 210–235 µs. Odczyt wartości kosztuje tyle co wyszukanie, bo wiersz i jego tekst to osobne obszary sterty, więc każdy wiersz to kilka chybień pamięci podręcznej procesora. |
| Zmiana | Planer (`plan::choose_covered` w `crates/flatsql-core/src/plan.rs`) zwraca razem ze ścieżką dostępu informację, czy ścieżka gwarantuje cały warunek `WHERE`: tak jest, gdy warunek składa się wyłącznie z koniunktów, które ścieżka wykorzystała (równość albo `IN` z literałami przy wyszukaniu po kluczu lub indeksie, porównania i `BETWEEN` na kolumnie zakresu). Gdy zapytanie ma jedną grupę, wszystkie agregaty to `COUNT(*)`, a projekcja, `HAVING` i `ORDER BY` nie odwołują się do kolumn, wykonawca (`execute_select` w `exec.rs`) liczy wiersze funkcją `plan::count` bez ich czytania: długość zbioru kluczy przy wyszukaniu po indeksie, suma długości przy `IN` i zakresie indeksu, liczba kluczy w zakresie drzewa klucza głównego, liczba wierszy tabeli bez warunku. `EXPLAIN` dopisuje do linii agregacji „rows counted by the access path, not read”, a `rowsScanned` wynosi 0, bo żaden wiersz nie został odczytany. Gwarancja wymaga, żeby porządek wartości w drzewie był zgodny z operatorami porównania; tak jest dla wartości jednego typu, a planer zamienia literał na typ kolumny przed wyszukaniem. Wyjątkiem był NULL, który indeks przechowuje, a który nie spełnia żadnego porównania: zakres indeksu bez dolnej granicy zaczyna się teraz za NULL. Odrzucono osobny licznik wierszy przy każdej wartości indeksu (dodatkowa praca przy każdym zapisie, a długość zbioru jest dostępna w czasie stałym) oraz ogólny odczyt wyłącznie z indeksu dla innych zapytań niż `COUNT(*)`, który wymaga nowych zapytań w obciążeniu. |
| Pomiar | `bench/compare.js`, przed: tag `prototyp`, po: zmiana z commita „core: count rows by the access path when it guarantees the whole WHERE clause” przed zatwierdzeniem (skrót różnicy w manifeście), 10 uruchomień na wersję na przemian, 25.10.2026, laptop i oprogramowanie jak w Benchmarku 1, obciążenie procesora przed pomiarem 11,8%. Rozbicie kosztów: 3 rundy w obu wersjach. Dane: [data/optimization/01-count-by-index/](data/optimization/01-count-by-index/). |
| Wynik | `COUNT(*)` po indeksie: natywnie 486 (478–488) µs → 1,66 (1,65–1,68) µs, Node.js z wynikiem jako obiekty 498 (482–537) µs → 9,47 (8,67–10,1) µs, z wynikiem jako JSON 484 (461–520) µs → 5,58 (4,61–6,76) µs; przedziały ufności rozłączne we wszystkich trzech. Odczytanych wierszy: 1000 → 0. Alokacje zapytania: 24 → 23. W Node.js pozostały czas to prawie w całości koszt granicy JS↔WASM (wynik jednowierszowy), więc Flat-SQL odpowiada teraz szybciej niż SQLite WASM w tej samej grupie (59,6 µs w Benchmarku 1). |
| Wpływ na inne operacje | Pozostałe operacje bez zmiany wykazanej kryterium przedziałów ufności, z jednym wyjątkiem: wstawienie 100 000 wierszy w Node.js (konfiguracja obiektowa) 427 → 402 ms; zmieniony kod nie bierze udziału we wstawianiu, a w konfiguracji JSON i natywnie wstawienie się nie zmieniło, więc to zmienność maszyny. Czas przygotowania instrukcji `COUNT` bez zmiany (2,03–2,23 µs przed, 2,07–2,13 µs po). Moduł WebAssembly: 219 575 → 221 642 B po kompresji gzip (+2,0 KiB). Wyniki wszystkich zapytań i stan tabeli zgodne w obu wersjach (460 porównań). |
| Ograniczenia | Liczenie bez odczytu dotyczy tylko `COUNT(*)` z warunkiem w całości przejętym przez ścieżkę dostępu; `COUNT(*)` z dodatkowym warunkiem na innej kolumnie nadal czyta wiersze. Poprawność potwierdza test różnicowy planera ([tests-results.md §3.2](tests-results.md#32-te-same-odpowiedzi-przy-każdej-ścieżce-dostępu)); dowodu formalnego nie przeprowadzono. |

Status: potwierdzona.
