# ADR-0040: Planer zwraca koniunkty gwarantowane przez ścieżkę dostępu

| | |
|---|---|
| Status | przyjęta |
| Data | 2026-10-25 |
| Rodzaj | architektura |
| Powiązane | ADR-0004, ADR-0014, ADR-0039; zadania todo 2.1, 2.2 |

## Kontekst

Prototyp przyjmował zasadę zapisaną w [architecture.md §2](../architecture.md): ścieżka dostępu zwraca nadzbiór wierszy, a wykonawca sprawdza cały warunek `WHERE` na każdym wierszu kandydującym. Upraszczało to planer, ale Benchmark 1 pokazał koszt tej zasady w dwóch słabościach: `COUNT(*)` po indeksie czytał 1000 wierszy kategorii i trwał 438–457 µs wobec 63–69 µs w SQLite (S1), a zakres klucza ponownie sprawdzał warunek zakresu w każdym wierszu (S2, ponad połowa czasu zapytania według rozbicia kosztów). Dialekt ma jedną tabelę w zapytaniu i indeksy jednokolumnowe (ADR-0004), więc relacja między warunkiem a ścieżką dostępu jest prosta do wyznaczenia.

## Decyzja

1. Warunek `WHERE` jest przechowywany jako lista koniunktów. Planer (`plan::choose_terms`) zwraca razem ze ścieżką dostępu maskę koniunktów, które każdy zwrócony wiersz spełnia: równość albo `IN` wykorzystane do wyszukania, wszystkie porównania i `BETWEEN` na kolumnie zakresu, literał prawdziwy. Wykonawca liczy tylko pozostałe koniunkty, w kolejności zapisu i z semantyką `AND`.
2. Gwarancja opiera się na porządku wartości: planer zamienia literał na typ kolumny (`key_for`), a porządek `Value` dla wartości jednego typu jest tym samym porządkiem, którego używają operatory porównania (ADR-0014). Jedyny wyjątek, NULL przechowywany w indeksie i niespełniający żadnego porównania, usuwa dolna granica zakresu indeksu ustawiona za NULL.
3. `SELECT` z jedną grupą, samymi agregatami `COUNT(*)`, bez odwołań do kolumn poza warunkiem i z warunkiem w całości zagwarantowanym przez ścieżkę liczy wiersze bez ich czytania (`plan::count`: długość zbioru kluczy indeksu, liczba kluczy zakresu, liczba wierszy tabeli). `rowsScanned` wynosi wtedy 0.
4. `EXPLAIN` pokazuje w linii `FILTER` tylko niezagwarantowaną resztę warunku, a przy liczeniu bez odczytu dopisuje to do linii agregacji.

## Uzasadnienie

- Pomiar przed zmianą i po niej ([optimization-results.md §1–2](../research/optimization-results.md)): `COUNT(*)` po indeksie natywnie 486 µs → 1,66 µs, zakres klucza 96,8 µs → 56,1 µs, bez zmiany wyników w żadnym z porównań sum kontrolnych.
- SQLite odpowiada na `COUNT(*)` z samego indeksu (`USING COVERING INDEX`, `analysis/explain.txt`); indeks Flat-SQL przechowuje zbiór kluczy każdej wartości, więc liczba wierszy jest dostępna w czasie stałym [D2].
- Klasę błędów, przed którą chroniła dawna zasada, sprawdza test różnicowy planera: losowe warunki na tabeli z indeksami porównywane z pełnym przeglądem tabeli bez indeksów, z NULL, NaN, liczbami całkowitymi porównywanymi z ułamkowymi i wartościami logicznymi porównywanymi z 0 i 1 ([tests-results.md §3.2](../research/tests-results.md)). Kontrola negatywna (bez granicy za NULL, z maską obejmującą niezagwarantowany koniunkt) kończy się niepowodzeniem testu.

## Odrzucone alternatywy

- **Zachowanie dawnej zasady** – koszt mierzony w S1 i S2; bez tej zmiany liczenie po indeksie musiałoby czytać wiersze, żeby sprawdzić warunek.
- **Osobny licznik wierszy przy każdej wartości indeksu** – dodatkowa praca przy każdym zapisie, a długość zbioru kluczy jest dostępna bez niego.
- **Nowe drzewo wyrażenia z pozostałych koniunktów przy każdym wykonaniu** – alokacje przy każdym zapytaniu; lista z maską ich nie wymaga.
- **Odczyt wyłącznie z indeksu także dla innych zapytań (hipoteza O12)** – wymaga nowych zapytań w obciążeniu, więc nowej wersji benchmarku; zostaje hipotezą.

## Konsekwencje

- Opis planera w [architecture.md §2](../architecture.md) zmienia się: ścieżka dostępu nadal może zwrócić nadzbiór, ale wykonawca nie sprawdza ponownie tego, co ścieżka gwarantuje.
- Testy, które sprawdzały użycie indeksu liczbą odczytanych wierszy zapytania `COUNT(*)`, sprawdzają je zapytaniem zwracającym wiersze.
- Każda nowa ścieżka dostępu musi określić, które koniunkty gwarantuje; test różnicowy planera jest miejscem, które to sprawdza.

## Źródła

[D2]; [optimization-results.md §1–2](../research/optimization-results.md), [tests-results.md §3.2](../research/tests-results.md), [benchmark-results.md §1.11](../research/benchmark-results.md#111-słabe-strony-prototypu-i-hipotezy-przyczyn).
