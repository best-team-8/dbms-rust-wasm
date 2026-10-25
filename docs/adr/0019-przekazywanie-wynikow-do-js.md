# ADR-0019: Przekazywanie wyników do JS – JSON teraz, bufor binarny docelowo

| | |
|---|---|
| Status | przyjęta; tworzenie obiektu wyniku w ścieżce obiektowej i zalecenie JSON dla wyników jednowierszowych zastąpione przez [ADR-0045](0045-obiekt-wyniku-tworzony-w-js.md) |
| Data | 2026-10-06 (`flatsql-wasm`, [architecture.md §7](../architecture.md)); kierunek doprecyzowany 2026-10-20 |
| Rodzaj | architektura |
| Powiązane | ADR-0005, ADR-0011; todo 2.3, 2.4 |

## Kontekst

Moduł WASM ma własną pamięć (tablicę bajtów); obiekty i napisy JavaScriptu żyją poza nią. Każdy wynik zapytania trzeba przenieść przez tę granicę. Przy operacjach trwających mikrosekundy to przeniesienie dominuje: odczyt po kluczu trwa natywnie 2,3 µs, a z JS 9,7 µs (H2, H3).

## Decyzja

`flatsql-wasm` udostępnia dwie ścieżki: obiekty i tablice JS budowane z Rusta przez `js-sys` (`exec`/`run`) oraz wynik serializowany do JSON w Ruście i parsowany przez `JSON.parse` (`execJson`/`runJson`). Na podstawie H3 preferowana jest ścieżka JSON. Docelowo wynik ma trafiać do jednego bufora binarnego w pamięci WASM, czytanego z JS bez pośrednich obiektów (todo 2.4).

## Uzasadnienie

- H3: JSON jest ok. 1,9× szybszy od budowania obiektów przez `js-sys` dla wyników jednowierszowych – jedno przejście przez granicę zamiast wielu.
- DuckDB-Wasm rozwiązuje ten sam problem, przekazując wyniki jako Arrow IPC czytane wprost ze sterty WASM [D1]; zmniejszenie liczby przejść daje więcej niż przyspieszanie pojedynczego przejścia.
- Recenzowanych pomiarów tej granicy dla silników baz nie znaleziono – pomiar H3 jest wkładem własnym projektu.

## Odrzucone alternatywy

- **JS String Builtins (Wasm 3.0)** – pozwalają kodowi WASM trzymać odwołania do napisów JS bez kopiowania; służą językom, które używają napisów JS jako własnych (Kotlin, Dart, Java). Rdzeń Flat-SQL musi przechowywać, porównywać i zapisywać tekst we własnej pamięci, także natywnie, gdzie JavaScriptu nie ma (ADR-0005, ADR-0011), więc kopiowania nie uniknie [Q13].
- **Tylko obiekty `js-sys`** – wolniejsze w pomiarze H3.

## Konsekwencje

- Cel: odczyt po kluczu poniżej ok. 3,5 µs (Zeta-Lite deklaruje 3,2–3,7 µs [D5]); najpierw profil kosztu (todo 2.3).
- Format bufora binarnego stanie się częścią API bridge'a i będzie wymagał wersjonowania.

## Źródła

[D1], [D5], [Q13], [W1]; [benchmark-results.md §6](../research/benchmark-results.md).
