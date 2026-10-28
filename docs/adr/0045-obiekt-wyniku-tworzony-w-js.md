# ADR-0045: Obiekt wyniku tworzony w kodzie JavaScriptu modułu, nazwy kolumn raz na plan

| | |
|---|---|
| Status | przyjęta; pkt 4 (wiersze budowane przez `js-sys`) zastąpiony przez [ADR-0046](0046-binarny-format-wyniku.md), funkcja `make_result` zastąpiona przez dekoder wyniku |
| Data | 2026-10-25 |
| Rodzaj | architektura |
| Powiązane | ADR-0019, ADR-0039, ADR-0041; zadanie todo 2.13 |

## Kontekst

Profil granicy JS↔WASM (todo 2.3, [optimization-results.md §10](../research/optimization-results.md#10-profil-granicy-jswasm)): odczyt po kluczu z wynikiem jako obiekty JS trwał w Node.js 7,58 µs, z czego wykonanie z parametrem 1,18 µs, a zamiana wyniku na obiekty 6,4 µs; dekodowanie UTF-8 i kod sklejający z przejściami WASM→JS po około 32% próbek. `result_to_js` w `flatsql-wasm` budował obiekt przez `js-sys`: pięć kluczy (`columns`, `rows`, `rowsAffected`, `rowsScanned`, `message`), nazwy kolumn i komunikat dekodowane z UTF-8 przy każdym wyniku, każdy klucz ustawiany osobnym wywołaniem `Reflect.set` przez kod sklejający. ADR-0019 opisuje ścieżkę obiektową jako budowaną z Rust przez `js-sys` i zaleca ścieżkę JSON.

## Decyzja

1. Obiekt wyniku `exec`/`run` tworzy funkcja JavaScriptu dołączona do modułu (`inline_js` wasm-bindgen, `make_result`), w której klucze są literałami kodu. Rust przekazuje jej w jednym wywołaniu nazwy kolumn, wiersze, obie liczby wierszy i komunikat (pusty komunikat jako `undefined`).
2. `PreparedStatement` w `flatsql-wasm` trzyma tablicę nazw kolumn w JavaScripcie razem z odwołaniem do nazw skompilowanego planu (`Arc<[String]>`, ADR-0041) i tworzy ją ponownie tylko wtedy, gdy plan ma inne nazwy, czyli po zmianie schematu.
3. Każdy wynik dostaje własną kopię tablicy nazw. Kształt i zawartość wyniku się nie zmieniają.
4. Wiersze i wartości są budowane jak dotąd; ich przekazywanie jest przedmiotem todo 2.4.

## Uzasadnienie

- Pomiar ([optimization-results.md §11](../research/optimization-results.md#11-obiekt-wyniku-tworzony-w-javascripcie)): odczyt po kluczu z wynikiem jako obiekty w Node.js 8,20 → 3,04 µs, w przeglądarce 8,99 → 3,95 µs, odczyt przez indeks 7,41 → 1,95 µs i 5,65 → 2,00 µs. Zamiana wyniku na obiekty JS w profilu granicy 6,4 → 1,1 µs, udział dekodowania UTF-8 34% → 16%, kodu sklejającego 31% → 15%.
- Obiekt tworzony literałem ma przy każdym wyniku ten sam kształt, a jego utworzenie to jedno przejście WASM→JS zamiast sześciu.
- Kopia tablicy nazw jest tańsza od jej utworzenia z UTF-8 i zachowuje niezależność wyników: aplikacja może zmieniać otrzymany wynik.

## Odrzucone alternatywy

- **Internowanie łańcuchów wasm-bindgen (`intern`, funkcja `enable-interning`)** – usuwa dekodowanie kluczy, ale zostawia osobne wywołanie przez kod sklejający dla każdego klucza i dodaje wyszukanie łańcucha w pamięci podręcznej; kod sklejający zajmował w profilu tyle samo próbek co dekodowanie.
- **Wspólna tablica nazw bez kopii** – zmiana nazw w jednym wyniku przez aplikację zmieniłaby je w kolejnych wynikach tej samej instrukcji.
- **Od razu format binarny (todo 2.4)** – większa zmiana; osobny krok mierzy koszt stałych części wyniku niezależnie od kosztu wartości, a funkcja tworząca obiekt w JavaScripcie przydaje się także przy dekodowaniu bufora.

## Konsekwencje

- Przy wyniku jednowierszowym ścieżka obiektowa jest szybsza od ścieżki JSON (Node.js 3,04 wobec 4,46 µs, przeglądarka 3,95 wobec 4,61 µs), więc zalecenie JSON z ADR-0019 przestaje dotyczyć takich wyników. Przy wielu wierszach JSON pozostaje w przeglądarce szybszy (pobranie 10 000 wierszy 9,2 wobec 13,9 ms) do czasu todo 2.4.
- Moduł WebAssembly większy o 288 B po kompresji gzip; kod JavaScriptu wyniku trafia do pakietu wasm-pack (`snippets/`), mosty się nie zmieniają.
- `Database.exec(sql)` bez instrukcji przygotowanej tworzy nazwy kolumn przy każdym wywołaniu; `FlatDB` wykonuje powtarzane instrukcje przez pamięć instrukcji przygotowanych (ADR-0041).
- Poprawność: [tests-results.md §3.7](../research/tests-results.md#37-nazwy-kolumn-wyniku-instrukcji-przygotowanej).

## Źródła

[optimization-results.md §10, §11](../research/optimization-results.md).
