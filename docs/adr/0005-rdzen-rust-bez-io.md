# 0005. Jeden rdzeń w Rust, bez operacji wejścia/wyjścia i bez zależności

- Status: obowiązuje
- Ostatnia zmiana: 2026-10-20

## Kontekst

Ten sam silnik ma działać w przeglądarce, w Node.js (serwer synchronizacji) i natywnie (narzędzie wiersza poleceń, w przyszłości aplikacje przez interfejs C). Każde z tych środowisk zapisuje dane gdzie indziej: w IndexedDB, w plikach, w prywatnym systemie plików przeglądarki.

## Decyzja

`flatsql-core` to biblioteka Rust bez zależności zewnętrznych, która sama niczego nie zapisuje na dysk. Kompiluje się do WebAssembly (`wasm32-unknown-unknown`) i natywnie. Silnik tylko produkuje bajty: każda zatwierdzona transakcja trafia jako ramka dziennika zapisu (WAL) do bufora, a snapshot bazy to tablica bajtów. Gdzie te bajty zapisać, decyduje program, który osadza silnik: w przeglądarce magazyny z `bridge/stores/`, natywnie implementacja `Storage`.

## Uzasadnienie

- Ten sam kod działa wszędzie. Serwer używa tego samego modułu `.wasm` co przeglądarka.
- Nowy magazyn, na przykład prywatny system plików przeglądarki, nie wymaga zmian w silniku.
- Zapis na dysk jest asynchroniczny i nie blokuje zapytań, które są synchroniczne.
- Brak zależności daje mały moduł: 287 KiB wszystkich plików silnika po kompresji, najmniej wśród silników SQL w WebAssembly.

## Odrzucone warianty

- Silnik, który sam czyta i zapisuje pliki, jak SQLite. Wiązałby silnik z jednym magazynem, a synchroniczny dostęp do plików w przeglądarce istnieje tylko w Web Workerze.
- Gotowe biblioteki parsera SQL, kodowania i sum kontrolnych. Większy moduł i mniejsza kontrola nad formatem danych.

## Konsekwencje

- Program osadzający musi odbierać bajty WAL i je zapisywać. Robią to `FlatDB` w JavaScripcie i `Persistent` w Rust.
- Własny parser, kodek i sumę kontrolną trzeba samemu utrzymywać i testować.
- Interfejs C dla innych języków byłby cienką warstwą nad tym samym rdzeniem.

## Historia

- 2026-10-06: architektura pierwszej wersji silnika.
