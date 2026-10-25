# ADR-0041: Plan zapamiętany w instrukcji przygotowanej, pamięć instrukcji w moście

| | |
|---|---|
| Status | przyjęta |
| Data | 2026-10-25 |
| Rodzaj | architektura |
| Powiązane | ADR-0005, ADR-0008, ADR-0013, ADR-0019, ADR-0039, ADR-0040; zadania todo 2.9, 2.10, 2.13, 2.4 |

## Kontekst

Instrukcja przygotowana (`Prepared`) przechowywała tylko drzewo składni, więc każde wykonanie od nowa wiązało nazwy kolumn, wstawiało parametry jako literały, wybierało ścieżkę dostępu, kopiowało nazwy kolumn i formatowało komunikat `SELECT n`. Rozbicie kosztów Benchmarku 1 przypisało tej pracy około połowy z 2,3 µs natywnego odczytu po kluczu, a samo wyszukanie w drzewie to 0,2 µs. W moście `FlatDB.run` po każdym wywołaniu, także po `SELECT`, sprawdzał przez granicę JS↔WASM, czy powstały nowe bajty WAL (hipoteza O2), a `FlatDB.query(sql, params)` przygotowywał instrukcję przy każdym wywołaniu i zostawiał ją odśmiecaczowi (O3).

## Decyzja

1. SELECT, INSERT, UPDATE i DELETE są kompilowane względem katalogu (`exec::compile`): nazwy rozwiązane, parametry `?` zostają odwołaniami (`BExpr::Param`, wartości przy wykonaniu w kontekście `expr::Cx`), nazwy kolumn wyniku w jednej współdzielonej tablicy (`QueryResult::columns: Arc<[String]>`). Instrukcja przygotowana kompiluje się przy `prepare`; instrukcja wykonywana raz kompiluje się i od razu wykonuje.
2. Ścieżka dostępu (`plan::AccessPlan`): warunek bez parametrów dostaje ścieżkę raz, przy kompilacji; pojedynczy warunek na kluczu albo kolumnie indeksowanej porównanej z parametrami (`col = ?`, `col < ?`, `col BETWEEN ? AND ?`) dostaje szablon wypełniany wartością przy wykonaniu; każdy inny warunek z parametrami ma ścieżkę wybieraną przy każdym wykonaniu. Gdy wartość nie pasuje do szablonu (NULL, typ, który nie może prowadzić wyszukania), decyduje wybór ogólny, więc szablon daje zawsze tę samą ścieżkę co wybór ogólny.
3. Skompilowana postać jest ważna dla wersji katalogu, dla której powstała. Wersja zmienia się przy każdej zmianie schematu (utworzenie i usunięcie tabeli lub indeksu, ich wycofanie, odtworzenie z WAL, zmiana z innej repliki) i pochodzi z licznika wspólnego dla wszystkich baz procesu, więc instrukcja przygotowana na jednej bazie i wykonana na innej kompiluje się od nowa. Instrukcja, której nie da się skompilować przy `prepare` (np. tabela jeszcze nie istnieje), kompiluje się przy wykonaniu i wtedy zgłasza błąd, jak dotąd.
4. `QueryResult::message` zawiera komunikat tylko dla instrukcji zmieniających dane lub schemat; dla `SELECT` jest pusty, bo wynikiem są wiersze.
5. Most: `PreparedStatement.readOnly`; `FlatDB.run` i `runJson` nie sprawdzają WAL po instrukcjach tylko do odczytu. `FlatDB` ma pamięć ostatnio używanych instrukcji (LRU, domyślnie 64, opcja `statementCache`): `query(sql, params)` korzysta z niej od pierwszego wywołania, `exec(sql)` od drugiego wykonania tego samego tekstu, a wypierane instrukcje są zwalniane jawnie.

## Uzasadnienie

- Instrukcja przygotowana SQLite jest kompilowana raz i wykonywana wielokrotnie [D2]; ten sam podział pracy daje natywnie odczyt po kluczu 2,71 → 1,27 µs i `COUNT(*)` po indeksie 1,73 → 0,70 µs, a liczbę alokacji odczytu po kluczu 26 → 6 ([optimization-results.md §3](../research/optimization-results.md)). Cel odczytu po kluczu poniżej 3,5 µs w przeglądarce (ADR-0019) jest nieosiągalny, dopóki sam silnik potrzebuje ponad 2 µs (ADR-0039).
- Unieważnianie wersją katalogu jest tańsze od sprawdzania przy każdym wykonaniu, czy tabela, kolumny i indeksy są takie jak przy kompilacji: jedno porównanie liczby. Wykonanie jest szeregowe (ADR-0008), więc schemat nie zmienia się w trakcie wykonania instrukcji.
- Szablon tylko dla pojedynczego warunku obejmuje typowe odczyty aplikacji (po kluczu, po indeksie, zakres) bez powtarzania całej logiki wyboru ścieżki w postaci symbolicznej; pozostałe warunki z parametrami płacą wybór ścieżki, który jest mały wobec przeglądanych przez nie wierszy.
- Komunikat `SELECT n` i kopia nazw kolumn kosztowały natywnie około 0,25 µs na wynik, a w JavaScripcie każdy z tych łańcuchów jest dekodowany z UTF-8 (H3); liczba wierszy jest w samym wyniku.
- Pamięć instrukcji w moście daje `FlatDB.query` z parametrem 24,5 → 8,5 µs. Zasada „od drugiego wykonania” w `exec(sql)` chroni przed zapełnieniem pamięci zapytaniami z wartościami wpisanymi w tekst, które nigdy się nie powtarzają.

## Odrzucone alternatywy

- **Kompilacja przy pierwszym wykonaniu zamiast przy `prepare`** – przesuwa koszt na pierwsze zapytanie; przygotowanie służy właśnie temu, żeby ten koszt ponieść wcześniej. Błędy nadal pojawiają się przy wykonaniu, jak dotąd.
- **Sprawdzanie przy wykonaniu, czy schemat tabeli jest ten sam (porównanie definicji)** – droższe niż porównanie wersji i nie obejmuje indeksów.
- **Szablon ścieżki dla dowolnego warunku z parametrami** – wymagałby symbolicznego wyboru ścieżki z rozstrzyganiem NULL i typów przy wykonaniu; zysk dla warunków złożonych jest mały.
- **Pamięć instrukcji w rdzeniu dla `Database::execute(sql)`** – zapytania z literałami (np. lokalne zapytanie porównania Local-First) mają za każdym razem inny tekst i nie trafiałyby do pamięci; w moście pamięć obejmuje `query` z parametrami, gdzie tekst się powtarza.
- **Zachowanie komunikatu `SELECT n`** – koszt przy każdym wyniku bez informacji, której nie ma w wierszach.

## Konsekwencje

- Przygotowanie instrukcji jest droższe (natywnie odczyt po kluczu 1,6–1,9 → 2,9–3,2 µs), wykonanie tańsze; zapytanie wykonywane raz z tekstu SQL kosztuje tyle co przed zmianą (4,7–5,0 µs natywnie).
- Zmiana API: `QueryResult::columns` jest typu `Arc<[String]>`, `message` jest pusty dla `SELECT` (opis w `bridge/index.d.ts`, aplikacja demonstracyjna pokazuje wtedy liczbę wierszy).
- Na skompilowanej postaci opierają się kolejne zadania: wyrażenia kompilowane do domknięć (2.10), nazwy kolumn przekazywane do JS raz (2.13), binarny format wyników (2.4).
- Moduł WebAssembly rośnie o 9 KiB po kompresji (+4%).

## Źródła

[D2]; [optimization-results.md §3](../research/optimization-results.md), [tests-results.md §3.3](../research/tests-results.md), [hyphotetical-optimalizations.md](../hyphotetical-optimalizations.md) (O2, O3).
