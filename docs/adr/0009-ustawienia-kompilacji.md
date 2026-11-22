# 0009. Kompilacja nastawiona na szybkość, bez zmian względem standardu

- Status: obowiązuje
- Ostatnia zmiana: 2026-10-28

## Kontekst

Moduł WebAssembly można kompilować pod szybkość albo pod rozmiar, z innym alokatorem pamięci i z instrukcjami wektorowymi (SIMD). Literatura sugerowała, że optymalizacje kompilatora LLVM nie są dostrojone do WebAssembly, więc mniejszy moduł nie musi być wolniejszy (Yan i in., 2021).

## Decyzja

Moduł jest kompilowany z `opt-level = 3`, optymalizacją całego programu (LTO), jedną jednostką kodu (`codegen-units = 1`) i `wasm-opt -O3`. Alokator pamięci jest domyślny, bez instrukcji SIMD.

## Uzasadnienie

Zmierzono siedem wariantów, każdy z jedną zmianą, w Node.js, po 10 uruchomień na przemian z wersją bazową:

- kompilacja pod rozmiar (`"s"` i `"z"`) zmniejsza moduł o 17–25%, ale spowalnia operacje o 5–78%;
- `opt-level = 2` spowalnia wstawianie o 10–17%;
- `wasm-opt -O4` nie daje zysku, a bez `wasm-opt` moduł jest o 6% większy;
- alokator talc nie przyspiesza operacji z wieloma alokacjami;
- SIMD przyspiesza tylko odtworzenie bazy, o 5%, a moduł z SIMD nie wczyta się w Safari starszym niż 16.4.

Hipoteza „mniejszy nie musi być wolniejszy” nie sprawdziła się dla tego silnika.

## Odrzucone warianty

Wszystkie powyższe warianty: każdy albo spowalnia silnik, albo nic nie daje.

## Konsekwencje

Brak zmian w kodzie i konfiguracji. Przy dużej zmianie silnika warto powtórzyć ten pomiar.

## Historia

- 2026-10-28: pomiar wariantów kompilacji, konfiguracja bez zmian.
