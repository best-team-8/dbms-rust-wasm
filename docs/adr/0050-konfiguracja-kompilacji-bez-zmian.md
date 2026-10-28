# ADR-0050: Kompilacja modułu WebAssembly bez zmian: opt-level 3, wasm-opt -O3, alokator domyślny, bez SIMD

| | |
|---|---|
| Status | przyjęta |
| Data | 2026-10-28 |
| Rodzaj | badania |
| Powiązane | ADR-0013, ADR-0039; zadanie todo 2.5 |

## Kontekst

Todo 2.5 zakładało, że optymalizacje LLVM nie są dostrojone do WebAssembly, więc mniejszy moduł (`opt-level = "s"` albo `"z"`) nie musi być wolniejszy [W11], że alokator domyślny `wasm32-unknown-unknown` (port dlmalloc) jest wolniejszy od talc w operacjach z wieloma alokacjami i że `+simd128` może przyspieszyć pętle. Moduł budowany jest z `opt-level = 3`, LTO, jedną jednostką kodu i `wasm-opt -O3`.

## Decyzja

Konfiguracja kompilacji pozostaje bez zmian: `opt-level = 3`, LTO, `codegen-units = 1`, `wasm-opt -O3`, alokator domyślny, bez `+simd128`.

## Uzasadnienie

Siedem wariantów wersji po todo 2.6, każdy z jedną zmianą, zmierzonych w Node.js obciążeniem i pamięcią po wczytaniu, 10 uruchomień na przemian z wersją bazową ([optimization-results.md §16](../research/optimization-results.md#16-poziomy-optymalizacji-alokator-i-simd)):

- `opt-level = "s"` i `"z"`: moduł po kompresji gzip mniejszy o 17% i 25%, ale operacje wolniejsze o 5–40% i 13–78%.
- `opt-level = 2`: moduł mniejszy o 5%, wstawianie wolniejsze o 10–17%, zakres klucza o 11%.
- bez `wasm-opt`: czasy w rozrzucie (wstawienie jednym wywołaniem −2%), moduł większy o 6%; `wasm-opt -O4`: bez zysku.
- talc: bez zysku w operacjach z wieloma alokacjami (wstawianie, odtworzenie, `UPDATE`, pobranie 10 000 wierszy w rozrzucie), pamięć po wczytaniu +1%.
- `+simd128`: odtworzenie −5%, reszta w rozrzucie, moduł mniejszy o 1%; moduł z instrukcjami SIMD nie wczyta się w Safari starszym niż 16.4.

## Odrzucone alternatywy

- **`opt-level = "s"`/`"z"`** – rozmiar kosztem czasu; hipoteza „mniejszy nie musi być wolniejszy” nie potwierdziła się dla tego silnika.
- **`opt-level = 2`** – wolniejsze wstawianie.
- **talc** – brak zysku, dodatkowa zależność.
- **`+simd128`** – 5% w jednej operacji nie równoważy utraty starszych przeglądarek.
- **`wasm-opt -O4`, bez `wasm-opt`** – bez zysku czasu; `-O3` zmniejsza moduł o 6%.

## Konsekwencje

- Kod i konfiguracja bez zmian; tag `po-2.5` wskazuje commit z opisem pomiaru.
- Warianty różniące się tylko konfiguracją (`Cargo.toml` obszaru roboczego, `.cargo/config.toml`) mają ten sam skrót plików kodu co wersja bazowa, bo skrót obejmuje katalogi `crates`, `bridge` i `server`; identyfikuje je katalog wariantu i opis.

## Źródła

[W11]; [optimization-results.md §16](../research/optimization-results.md#16-poziomy-optymalizacji-alokator-i-simd).
