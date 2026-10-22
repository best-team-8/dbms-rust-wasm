# ADR-0020: Punkty odniesienia i metodyka benchmarku

| | |
|---|---|
| Status | przyjęta |
| Data | 2026-10-06 ([benchmark-scenario.md](../research/benchmark-scenario.md), [wasm-engines-comparison.md §5](../research/wasm-engines-comparison.md)) |
| Rodzaj | proces |
| Powiązane | todo §4 |

## Kontekst

Projekt ma porównać Flat-SQL z istniejącymi silnikami WASM i zweryfikować hipotezy H1–H6. Każdy punkt odniesienia musi działać w tych samych warunkach co badany silnik, inaczej porównanie jest „jabłkami i pomarańczami” [B3].

## Decyzja

1. Uczestnicy: Flat-SQL WASM (dwie ścieżki wyników), Flat-SQL natywnie (koszt samego WASM), **sql.js** (dojrzały SQL w pamięci WASM, działa identycznie w Node.js i przeglądarce bez Workera i nagłówków), czysty JavaScript na `Map` (dolna granica), serwer HTTP z tym samym silnikiem i symulowanym RTT (Cloud-First).
2. Metodyka: rozgrzewka przed pomiarem, stałe ziarno danych i kluczy, przygotowane instrukcje, SQLite w najlepszym układzie (`INTEGER PRIMARY KEY`), porównanie liczby wierszy między silnikami, surowe wyniki w JSON.
3. PGlite i DuckDB-Wasm nie są mierzone – to inne klasy systemów i wymagają Workera lub OPFS.

## Uzasadnienie

- sql.js to jedyny dojrzały silnik SQL w WASM działający w tych samych warunkach co Flat-SQL (wątek główny, bez nagłówków).
- Rozgrzewka jest konieczna: bez niej pierwszy mierzony silnik wypadał do 2× gorzej (przejście kompilatorów V8) [B1].
- Symulowane RTT bez TLS i strat pakietów daje dolne ograniczenie kosztu Cloud-First – wniosek H1 jest więc ostrożny.

## Odrzucone alternatywy

- **Tylko SQLite WASM z OPFS jako punkt odniesienia** – wymaga Workera, więc nie mierzy tych samych warunków; planowany jako dodatkowy punkt odniesienia dla trwałości (todo 4.1).

## Konsekwencje

- Przewaga w trwałości dotyczy sql.js, nie SQLite ogólnie – teza „szybszy niż sql.js” (todo 0.1), uzupełnienie o SQLite z OPFS, PGlite i Zeta-Lite (todo 4.1).
- Do uzupełnienia według literatury: wiele uruchomień [B1], przebiegi zimne [B3], rozkład Zipfa [B5], pamięć, pauzy checkpointu, pomiar na telefonie (todo 4.2–4.10, future 12.8).

## Źródła

[B1], [B3], [B5].
