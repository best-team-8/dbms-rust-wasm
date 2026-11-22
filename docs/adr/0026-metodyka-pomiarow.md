# 0026. Metodyka pomiarów: powtórzenia procesu, sumy kontrolne, grupy warunków, dwie wersje silnika

- Status: obowiązuje
- Ostatnia zmiana: 2026-10-28

## Kontekst

Wyniki pomiarów mają być porównywalne między silnikami i między wersjami Flat-SQL, także przy pomiarach przyszłych. Pierwsze pomiary miały jeden proces na silnik i porównywały tylko liczbę zwróconych wierszy. Dwie konfiguracje Flat-SQL z tym samym kodem różniły się wtedy o kilkadziesiąt procent.

## Decyzja

1. Jednostką powtórzenia jest proces. Każdy pomiar działa w nowym procesie Node.js albo w nowej przeglądarce z pustym profilem, co najmniej 10 razy, w losowanej kolejności silników. Wynik to mediana z kwartylami. Zmiana między wersjami jest wykazana, gdy 95% przedziały ufności średnich się nie nakładają.
2. Poprawność wyników sprawdza 64-bitowa suma kontrolna postaci kanonicznej każdej odpowiedzi, a nie liczba wierszy. Po każdej fazie zapisu porównuje się też stan tabeli.
3. Silniki porównuje się tylko w grupach o tych samych warunkach: wszystko w pamięci, trwały zapis w IndexedDB w wątku głównym, silnik w Web Workerze, natywnie.
4. Narzędzie pomiarowe mierzy wskazaną wersję silnika. Prototyp (tag `v1-prototyp`) i wersję po optymalizacjach (tag `v2-optymalizacja`) mierzy to samo narzędzie. Zmiana obciążenia wymaga nowego identyfikatora obciążenia i ponownego pomiaru prototypu.
5. Surowe wyniki każdego procesu leżą w repozytorium razem ze skryptem, który tworzy z nich wszystkie tabele i wykresy.
6. Wersję po optymalizacjach porównano z innymi silnikami, łącząc jej wyniki z wynikami tych silników z pierwszego pomiaru. Różnicę między sesjami mierzy prototyp zmierzony w obu.

## Uzasadnienie

- W środowiskach z kompilatorem w trakcie działania (JIT) pojedynczy przebieg prowadzi do błędnych wniosków (Georges i in., 2007). Tu jeden proces dał narzut WebAssembly 1,1–1,8×, a dziesięć procesów 0,89–1,13×.
- Typowe błędy porównań baz to brak sprawdzenia wyników i porównywanie systemów w różnych warunkach (Raasveldt i in., 2018).
- Prototyp zmierzony w obu sesjach różni się o medianę 2,3%, a w 111 ze 113 par o mniej niż 10%. To o rząd mniej niż większość różnic między silnikami.

## Odrzucone warianty

- Wiele iteracji w jednym procesie. Nie pokazuje zmienności między uruchomieniami.
- Strona bez izolacji. Zegar miałby dokładność 100 µs zamiast 5 µs.
- Ponowny pomiar wszystkich silników razem z wersją po optymalizacjach. Około 6 godzin pracy dla silników, których kod się nie zmienił. Następny pełny pomiar powinien objąć wszystkie silniki w jednej sesji.
- Obciążenie z rozkładem Zipfa (YCSB). Zmiana obciążenia unieważniłaby porównanie z prototypem.

## Konsekwencje

- Pełny pomiar trwa kilka godzin. Narzędzie wznawia przerwany pomiar.
- Szczegóły: [metodyka benchmarku](../research/benchmark-scenario.md) i [instrukcja odtworzenia](../../bench/README.md).

## Historia

- 2026-10-06: pierwsze pomiary, jeden proces na silnik.
- 2026-10-24: obecna metodyka, pomiar prototypu przed optymalizacjami.
- 2026-10-28: porównanie wersji po optymalizacjach z silnikami z pierwszego pomiaru.
