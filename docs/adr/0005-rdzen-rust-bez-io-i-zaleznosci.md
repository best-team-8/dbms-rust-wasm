# ADR-0005: Jeden rdzeń w Rust bez I/O i bez zależności

| | |
|---|---|
| Status | przyjęta |
| Data | 2026-10-06 (silnik, [architecture.md §1](../architecture.md)) |
| Rodzaj | architektura |
| Powiązane | ADR-0011, ADR-0012, ADR-0013; future §9 |

## Kontekst

Ten sam silnik ma działać w przeglądarce, w Node.js (serwer synchronizacji, węzeł edge) i natywnie (CLI, przyszły panel WinUI przez interfejs C). Każde środowisko ma inny magazyn: IndexedDB, pliki, OPFS, magazyny platform edge.

## Decyzja

`flatsql-core` to czysta biblioteka Rust bez zależności zewnętrznych i bez operacji wejścia/wyjścia, kompilowana do `wasm32-unknown-unknown` i natywnie. Silnik tylko produkuje bajty: każda zatwierdzona transakcja trafia jako ramka WAL do bufora (`take_wal()`), snapshot to tablica bajtów. O tym, gdzie bajty zapisać, decyduje host – w przeglądarce magazyny z `bridge/stores/`, natywnie trait `Storage`.

## Uzasadnienie

- Identyczny kod w każdym środowisku; serwer używa tego samego modułu `.wasm` co przeglądarka.
- Magazyn i wątek wybiera host, więc nowy backend (np. OPFS, magazyn platformy edge) nie wymaga zmiany silnika (ADR-0012, ADR-0013).
- Trwałość jest asynchroniczna i nie blokuje zapytań, które są synchroniczne.
- Brak zależności daje mały moduł (545 KiB, 206 KiB gzip) i pełną kontrolę nad formatami.

## Odrzucone alternatywy

- **Silnik z wbudowanym, synchronicznym I/O (jak VFS w SQLite)** – wiązałby silnik z jednym magazynem; synchroniczny dostęp do plików w przeglądarce istnieje tylko w Web Workerze.
- **Biblioteki parsera SQL, kodeka i CRC** – większy moduł, mniejsza kontrola nad formatem.

## Konsekwencje

- Host musi odbierać bajty WAL (`takeWal()`) i je zapisywać – robią to `FlatDB` i `Persistent`.
- Własny parser, kodek i CRC trzeba utrzymywać i testować.
- Interfejs C dla innych języków (future 9.1) to cienka warstwa nad tym samym rdzeniem.

## Źródła

[P2] (cel przenośności WASM); [architecture.md §1, §8](../architecture.md).
