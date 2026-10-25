# ADR-0046: Wynik w binarnym formacie wierszowym, kodowany wprost z tabeli i dekodowany w JS jednym wywołaniem

| | |
|---|---|
| Status | przyjęta |
| Data | 2026-10-25 |
| Rodzaj | architektura |
| Powiązane | ADR-0005, ADR-0019, ADR-0039, ADR-0041, ADR-0045; zadanie todo 2.4 |

## Kontekst

ADR-0019 wyznaczył jako cel wynik w jednym buforze w pamięci WASM, czytany z JS bez pośrednich obiektów. Po todo 2.13 (ADR-0045) wynik jednowierszowy nie zależy już od stałych części wyniku, ale każda wartość nadal przechodziła przez granicę osobno: liczba przez wywołanie kodu sklejającego, tekst przez osobne dekodowanie UTF-8. Pomiar granicy (skrypt z todo 2.3 rozszerzony o wynik 10 000 wierszy, [optimization-results.md §12](../research/optimization-results.md#12-binarny-format-wyniku)): pobranie 10 000 wierszy w Node.js 9,98 ms, z czego wykonanie z budową `QueryResult` w Rust 3,01 ms, a zamiana 50 000 wartości na obiekty JS około 7 ms; dekodowanie UTF-8 36% próbek. Rozbicie kosztów natywnie: zebranie 10 000 wierszy w `QueryResult` (kopia każdej wartości) 4,0–4,2 ms i 40 014 alokacji.

## Decyzja

1. Rdzeń: wykonawca SELECT oddaje wiersze odbiorcy (`RowSink`): zbiera je w `QueryResult` jak dotąd albo koduje wprost z wierszy tabeli w binarnym formacie wyniku (`result::ResultEncoder`), bez kopii wartości. `Database::execute_prepared_encoded` wykonuje instrukcję przygotowaną z wynikiem w tym formacie; inne instrukcje kodują zebrany `QueryResult` (`QueryResult::encode`).
2. Format wierszowy, little-endian: nagłówek 32 B (liczba wierszy, wartości w wierszu, wiersze zmienione i przejrzane jako f64, położenie obszaru tekstu, długość komunikatu w jednostkach UTF-16), potem wartości wiersz po wierszu jako znacznik i dane (NULL, FALSE, TRUE bez danych; liczba jako f64, także INT, bo tak widzi ją JavaScript; tekst jako długość w jednostkach UTF-16), na końcu obszar tekstu: UTF-8 wszystkich tekstów i komunikatu.
3. `flatsql-wasm`: `exec`, `execAll` i `run` kodują wynik do bufora trzymanego w `Database` (bufor większy niż 1 MiB nie jest zachowywany) i zamieniają go na obiekt wyniku jednym wywołaniem funkcji JavaScriptu dołączonej do modułu (`decode_result`): liczby czytane z bajtów, wszystkie teksty dekodowane jednym wywołaniem `TextDecoder` (z `ignoreBOM`, bo U+FEFF na początku to tekst) i cięte według długości UTF-16. Widoki pamięci modułu powstają ponownie tylko po jej powiększeniu. Obiekt wyniku, jego kształt i nazwy kolumn raz na plan zostają jak w ADR-0045.
4. Format jest wewnętrzny dla pakietu `flatsql-wasm`: koder i dekoder powstają w jednej kompilacji, więc nie wymaga wersjonowania; API JavaScriptu się nie zmienia. Ścieżka JSON (`execJson`, `runJson`) zostaje do porównań.

## Uzasadnienie

- Pomiar ([optimization-results.md §12](../research/optimization-results.md#12-binarny-format-wyniku)): pobranie 10 000 wierszy w Node.js 10,8 → 1,93 ms, w przeglądarce 14,0 → 1,59 ms; odczyt po kluczu 3,12 → 2,60 µs i 3,99 → 3,29 µs, czyli poniżej celu 3,5 µs z ADR-0019 (Zeta-Lite deklaruje 3,2–3,7 µs [D5]). Natywnie kodowanie 10 000 wierszy wprost z tabeli 0,70–0,96 ms i 16 alokacji wobec 4,0–4,2 ms i 40 014 alokacji przy zebraniu wyniku.
- Kierunek jest ten sam co w DuckDB-Wasm (wynik czytany z pamięci WebAssembly bez obiektów pośrednich [D1]), w formacie wierszowym, bo wynik ma postać tablic wierszy.
- Jedno wywołanie dekodera tekstu na wynik zamiast jednego na wartość: wywołanie `TextDecoder` dla krótkiego tekstu kosztuje w V8 około 0,2 µs niezależnie od długości.

## Odrzucone alternatywy

- **Parametry w tablicach typowanych dla pojedynczych instrukcji (część pierwotnego zakresu todo 2.4)** – pomiar todo 2.3 pokazał, że wykonanie z parametrem trwa w WASM tyle co natywnie (1,18 µs), więc przekazanie parametru jest pomijalne; kodowanie parametrów ma sens przy wielu wierszach naraz i przechodzi do todo 2.14.
- **Arrow IPC jak w DuckDB-Wasm [D1]** – format kolumnowy wymaga biblioteki do odczytu w JavaScripcie i zamiany kolumn na wiersze; rdzeń nie ma zależności.
- **Krótkie teksty ASCII dekodowane pętlą w JavaScripcie** – szybsze od `TextDecoder` tylko poniżej 13 znaków, bo V8 łączy dłuższe łańcuchy w łańcuchy złożone spłaszczane przy cięciu.
- **Widoki pamięci tworzone przy każdym wyniku** – `DataView` i `Uint8Array` kosztowały około 0,3 µs na wynik (odczyt po kluczu 2,07 µs wobec 1,74 µs z widokami zachowanymi).
- **Wynik leniwy (obiekt czytający bufor przy dostępie)** – zmienia API wyniku i czas życia danych; poza zakresem pracy.

## Konsekwencje

- ADR-0019 zastąpiony: cel z ADR-0019 jest osiągnięty, a ścieżka obiektowa jest szybsza od JSON przy wyniku jednowierszowym i przy 10 000 wierszy. Punkt 4 ADR-0045 (wiersze budowane przez `js-sys`) zastąpiony.
- Moduł WebAssembly większy o 2,4 KiB po kompresji gzip.
- `Database` w `flatsql-wasm` trzyma bufor ostatniego wyniku (do 1 MiB).
- Mniej alokacji w ścieżce obiektowej zmienia stan sterty modułu dla dalszych operacji w tym samym procesie; pomiar pokazał szybsze zapisy, snapshot i odtworzenie po zmianie, bez osobnego potwierdzenia przyczyny.
- Poprawność: [tests-results.md §3.8](../research/tests-results.md#38-binarny-format-wyniku).

## Źródła

[D1], [D5]; [optimization-results.md §12](../research/optimization-results.md#12-binarny-format-wyniku).
