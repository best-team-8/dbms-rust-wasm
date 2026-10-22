# ADR-0010: Group commit w przeglądarce

| | |
|---|---|
| Status | przyjęta |
| Data | 2026-10-06 (bridge, [architecture.md §4.3](../architecture.md)) |
| Rodzaj | architektura |
| Powiązane | ADR-0009, ADR-0012, ADR-0013; todo 1.4 |

## Kontekst

`FlatDB.exec()` zwraca wynik synchronicznie, a zapis do IndexedDB jest asynchroniczny. Osobny zapis na dysk dla każdej transakcji byłby kosztowny przy wielu drobnych zmianach (np. zapis stanu gry w każdej klatce).

## Decyzja

Jeśli transakcja wyprodukowała bajty WAL, `FlatDB` planuje `flush()` w najbliższym mikrozadaniu (`queueMicrotask`), więc wszystkie zatwierdzenia z jednego „tiku” pętli zdarzeń trafiają do IndexedDB jednym zapisem. `await db.flush()` jest punktem trwałości: wszystko zatwierdzone do tej chwili jest zapisane. Operacje na magazynie są serializowane w jednej kolejce. Semantyka odpowiada `synchronous=NORMAL` w SQLite.

## Uzasadnienie

- Group commit – jeden zapis logu dla grupy transakcji – to klasyczna technika baz w pamięci [S1]; w pomiarze 1000 zatwierdzeń + jeden `flush()` kosztuje 5,6 µs na zatwierdzenie (H5).
- Aplikacja nie czeka na dysk przy zapisie, a gdy potrzebuje gwarancji, wywołuje `await flush()`.

## Odrzucone alternatywy

- **Zapis na dysk przy każdym zatwierdzeniu, z oczekiwaniem** – wymusiłby asynchroniczne API zapisu.
- **Zapis wyłącznie na żądanie aplikacji** – łatwo zgubić dane, gdy aplikacja zapomni wywołać zapis.

## Konsekwencje

- Zatwierdzenia z ostatnich milisekund mogą zginąć przy nagłym zamknięciu karty, ale baza zawsze wraca do spójnego stanu.
- Gwarancja po `await flush()` zależy od trybu trwałości IndexedDB: domyślnie (`relaxed`, Chrome 121+) przetrwa awarię karty i przeglądarki, nie systemu [Q1]; opcja `strict` – todo 1.4.

## Źródła

[S1], [Q1]; [benchmark-results.md §3](../research/benchmark-results.md).
