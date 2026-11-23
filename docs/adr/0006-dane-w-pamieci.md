# 0006. Cała baza w pamięci, moduł 32-bitowy

- Status: obowiązuje
- Ostatnia zmiana: 2026-10-20

## Kontekst

Zapytania mają działać lokalnie i bez czekania (decyzja 0001), także w głównym wątku strony (decyzja 0007), gdzie nie ma synchronicznego dostępu do plików. Moduł WebAssembly może adresować pamięć liczbami 32-bitowymi (najwyżej 4 GiB) albo 64-bitowymi (rozszerzenie memory64).

## Decyzja

1. Cała baza jest w pamięci modułu. Wiersze leżą w drzewie uporządkowanym po kluczu głównym (`BTreeMap`). Indeks dodatkowy to mapa „wartość → zbiór kluczy”. Indeksów nie zapisuje się na dysk, tylko buduje przy wczytaniu bazy.
2. Dysk służy wyłącznie trwałości (decyzja 0016).
3. Silnik zostaje przy 32-bitowym WebAssembly (`wasm32`), bez memory64.

## Uzasadnienie

- Bazy w pamięci są szybkie głównie dlatego, że nie kodują danych do formatu dyskowego przy każdym dostępie. Trwałość zapewniają log i snapshot (Kleppmann, 2017, rozdz. 3; DeWitt i in., 1984).
- Silnik w pamięci wykonujący transakcje w jednym wątku nie płaci za blokady ani bufor stron (Stonebraker i in., 2007).
- Przy adresach 32-bitowych przeglądarka sprawdza granice pamięci sprzętowo, praktycznie za darmo. Przy 64-bitowych musi sprawdzać każdy dostęp programowo, co średnio podwaja jego koszt (Döllerer i Engelke, 2024).
- Docelowe dane są o rzędy wielkości mniejsze niż 4 GiB. Baza 100 000 wierszy zajmuje około 52 MiB. Na telefonie przeglądarka zamknie kartę, zanim zabraknie adresów.

## Odrzucone warianty

- Format stronicowy na dysku z buforem stron, jak w SQLite. W przeglądarce wymaga synchronicznego dostępu do plików, czyli Web Workera.
- Memory64. Spowalnia każde zapytanie u wszystkich użytkowników dla danych, których projekt i tak nie obsługuje.

## Konsekwencje

- Rozmiar bazy ograniczają pamięć urządzenia i 4 GiB modułu. Analiza dużych plików to zadanie dla silnika analitycznego, takiego jak DuckDB-Wasm.
- Odtworzenie bazy wymaga zdekodowania wszystkich wierszy, więc trwa dłużej niż w SQLite, który kopiuje strony pliku.
- Zużycie pamięci zmniejsza się przez oszczędniejszy układ wierszy (decyzja 0014), a nie przez większą przestrzeń adresową.

## Historia

- 2026-10-06: dane w pamięci od pierwszej wersji silnika.
- 2026-10-20: dopisano rezygnację z memory64.
