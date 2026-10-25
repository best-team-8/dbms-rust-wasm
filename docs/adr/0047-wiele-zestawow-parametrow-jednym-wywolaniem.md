# ADR-0047: Instrukcja zapisu dla wielu zestawów parametrów jednym wywołaniem, obciążenie benchmark-2

| | |
|---|---|
| Status | przyjęta |
| Data | 2026-10-25 |
| Rodzaj | architektura |
| Powiązane | ADR-0008, ADR-0039, ADR-0041, ADR-0044, ADR-0046; zadanie todo 2.14 |

## Kontekst

Wstawienie 100 000 wierszy przygotowanym `INSERT` w jednej transakcji trwało w Benchmarku 1 343–391 ms w WASM wobec 214 ms natywnie, a po wcześniejszych optymalizacjach 298 ms w Node.js wobec 163 ms natywnie. Pomiar granicy ([optimization-results.md §13](../research/optimization-results.md#13-wstawianie-wielu-wierszy-jednym-wywołaniem)): wywołanie z pięcioma parametrami instrukcji, która niczego nie zapisuje, kosztuje 1,37–1,47 µs na wiersz, czyli około połowy czasu wstawienia (2,6–2,8 µs); każda wartość przechodzi przez kod sklejający kilkoma wywołaniami (sprawdzenie typu, odczyt liczby, kodowanie tekstu), a każda instrukcja odczytuje zegar przez `Date.now()`.

## Decyzja

1. `Database::execute_many(prepared, next)` w rdzeniu wykonuje przygotowany `INSERT`, `UPDATE` albo `DELETE` dla każdego zestawu parametrów, który zwraca źródło `next` (zapisuje parametry do podanego wektora). Rdzeń nie zna formatu przekazania, więc tej samej funkcji używa natywny benchmark i moduł WASM. Inne instrukcje dają błąd rodzaju `unsupported`.
2. Wszystkie wykonania są jedną instrukcją: stosują się razem albo wcale (atomowość instrukcji jak w ADR-0008; w otwartej transakcji wycofywana jest tylko ta instrukcja), a ich wiersze dostają znaczniki z jednego odczytu zegara (rozszerzenie ADR-0044). Wynikiem jest liczba zmienionych wierszy.
3. `flatsql-wasm`: `Database.executeMany(statement, rows)` i `FlatDB.executeMany`; `rows` to tablica tablic parametrów. JavaScript koduje wiersze do bufora w binarnym formacie parametrów (liczba wartości wiersza, potem znacznik i dane: NULL, FALSE, TRUE, liczba jako f64, tekst jako długość i UTF-8, BigInt jako i64) porcjami około 64 KiB, każda pobierana przez Rust po zużyciu poprzedniej. Typy i błędy są te same co w `execute` (wspólna reguła liczby całkowitej i ułamkowej, ten sam komunikat dla nieobsługiwanego typu i BigInt spoza zakresu).
4. Obciążenie pomiarowe dostaje operację `insert_many` (wiersze fazy wstawienia do drugiej tabeli o tej samej definicji, w jednej transakcji, na końcu obciążenia, żeby nie zmieniać wcześniejszych operacji) i identyfikator `benchmark-2`. Runner przyjmuje wyniki zgodnego obciążenia `benchmark-1` (natywny benchmark starszych wersji). Adapter Flat-SQL używa `executeMany`, gdy mierzona wersja je ma, a w innym wypadku pętli `execute` z fazy wstawienia, więc Benchmark 2 porównuje najszybszy sposób wstawienia wielu wierszy, jaki daje każda wersja.

## Uzasadnienie

- Pomiar ([optimization-results.md §13](../research/optimization-results.md#13-wstawianie-wielu-wierszy-jednym-wywołaniem)): `insert_many` w Node.js 283 → 160 ms, w przeglądarce 273 → 151 ms, natywnie 145 ms; stosunek WASM do natywnego 1,04–1,10 wobec 1,8 przy wstawianiu po jednym wierszu. Kod sklejający 19,5% → 0,9% próbek.
- Atomowość całego wsadu odpowiada intuicji wywołania „wstaw te wiersze” i jest tańsza niż transakcja na wiersz; pętla `execute` nadal daje semantykę wiersz po wierszu.
- Porcje 64 KiB ograniczają pamięć kodowania niezależnie od liczby wierszy, a koszt przejścia przez granicę na porcję (kilka mikrosekund) jest pomijalny przy około 1400 wierszach w porcji.

## Odrzucone alternatywy

- **Wsad kolumnowy (tablice typowane dla liczb, jeden bufor tekstów)** – wymaga od aplikacji przestawienia wierszy na kolumny; kodowanie wierszy w JavaScripcie zajmuje około 10% próbek wstawienia, więc format kolumnowy mógłby oszczędzić najwyżej tyle.
- **`executeMany` czytające tablice wierszy z Rust przez `js-sys`** – zostawia kilka wywołań kodu sklejającego na wartość, czyli koszt zmierzony w wariancie bez zapisu (1,37–1,47 µs na wiersz).
- **Cały wsad w jednym buforze** – szczyt pamięci równy podwójnemu rozmiarowi wsadu.
- **Osobna instrukcja dla każdego zestawu** – semantyka pętli `execute`, dostępna już dziś.
- **Zmiana znaczenia operacji `insert` zamiast nowej operacji** – wyniki `insert` przestałyby być porównywalne z Benchmarkiem 1.

## Konsekwencje

- Moduł WebAssembly większy o 2,5 KiB po kompresji gzip.
- Natywny benchmark mierzonej wersji i `suite.js` zgłaszają `benchmark-2`; raporty Benchmarku 1 pozostają przy `benchmark-1`.
- Parametry są nadal kopiowane do wiersza (tekst parametru i tekst w wierszu to dwie alokacje); przeniesienie bez kopii to możliwa dalsza zmiana.
- Poprawność: [tests-results.md §3.9](../research/tests-results.md#39-wiele-zestawów-parametrów-jednym-wywołaniem).

## Źródła

[optimization-results.md §13](../research/optimization-results.md#13-wstawianie-wielu-wierszy-jednym-wywołaniem).
