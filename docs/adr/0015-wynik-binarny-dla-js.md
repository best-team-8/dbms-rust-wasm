# 0015. Wynik przekazywany do JavaScriptu w jednym buforze binarnym

- Status: obowiązuje
- Ostatnia zmiana: 2026-10-25

## Kontekst

Moduł WebAssembly ma własną pamięć, a obiekty JavaScriptu żyją poza nią. Każdy wynik trzeba przenieść przez tę granicę. Przy operacjach trwających mikrosekundy to przeniesienie dominuje. W prototypie odczyt po kluczu trwał natywnie 2,3 µs, a z JavaScriptu około 9 µs. Profil pokazał, że czas zabiera dekodowanie tekstów UTF-8 i wiele osobnych wywołań przez kod sklejający Rust z JavaScriptem.

## Decyzja

1. Wykonawca koduje wynik zapytania wprost z wierszy tabeli do binarnego formatu wierszowego, bez pośredniej kopii wartości.
2. Format: 32-bajtowy nagłówek, potem wartości wiersz po wierszu (znacznik typu i dane; liczby jako 64-bitowe liczby zmiennoprzecinkowe, bo tak widzi je JavaScript), na końcu wszystkie teksty jednym ciągiem UTF-8.
3. Funkcja JavaScriptu dołączona do modułu zamienia bufor na obiekt wyniku jednym wywołaniem. Wszystkie teksty dekoduje jednym wywołaniem `TextDecoder`.
4. Obiekt wyniku ma stały kształt, a nazwy kolumn instrukcji przygotowanej powstają w JavaScripcie raz na plan.
5. Format jest wewnętrzny dla pakietu `flatsql-wasm`: koder i dekoder powstają w jednej kompilacji, więc nie wymaga wersji. API JavaScriptu się nie zmienia. Wynik jako JSON (`execJson`) zostaje do porównań.

## Uzasadnienie

- Tak samo rozwiązuje to DuckDB-Wasm: JavaScript czyta wynik wprost z pamięci modułu (Kohn i in., 2022).
- Zmierzone w przeglądarce: odczyt po kluczu 8,78 → 3,28 µs, pobranie 10 000 wierszy 11,4 → 1,55 ms. Pobranie wielu wierszy jest teraz w WebAssembly szybsze niż natywnie, bo moduł nie kopiuje wierszy do wyniku pośredniego.
- Jedno wywołanie dekodera na wynik zamiast jednego na wartość: wywołanie `TextDecoder` kosztuje około 0,2 µs niezależnie od długości tekstu.

## Odrzucone warianty

- Format kolumnowy Apache Arrow. Wymaga biblioteki w JavaScripcie i zamiany kolumn na wiersze.
- Wynik budowany z Rusta obiekt po obiekcie (`js-sys`). To był koszt, który zmierzono.
- Internowanie stałych tekstów przez wasm-bindgen. Usuwa dekodowanie, ale zostawia osobne wywołanie na każdy klucz.
- Mechanizm JS String Builtins. Służy językom, które używają tekstów JavaScriptu jako własnych. Rust trzyma tekst we własnej pamięci, także natywnie.
- Wynik leniwy, czytający bufor przy dostępie. Zmienia API i czas życia danych.

## Konsekwencje

- Moduł trzyma bufor ostatniego wyniku (do 1 MiB).
- Odczyt po kluczu wciąż jest 2,7–3,6 raza wolniejszy niż natywnie, bo każde wywołanie przekracza granicę. Operacje przekazujące dużo danych naraz mają narzut bliski zeru.

## Historia

- 2026-10-06: wynik jako obiekty budowane z Rusta albo jako JSON; JSON był szybszy.
- 2026-10-25: obiekt wyniku tworzony w JavaScripcie, potem jeden bufor binarny. Wynik jako obiekty jest teraz szybszy od JSON.
