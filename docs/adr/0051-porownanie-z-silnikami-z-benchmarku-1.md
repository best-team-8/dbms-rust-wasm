# ADR-0051: Wersja po optymalizacjach porównywana z silnikami odniesienia z Benchmarku 1, silniki oznaczone według roli

| | |
|---|---|
| Status | przyjęta |
| Data | 2026-10-28 |
| Rodzaj | proces |
| Powiązane | ADR-0037, ADR-0038, ADR-0039; Benchmark 2 (todo, „Kolejność”) |

## Kontekst

Benchmark 2 zmierzył prototyp i wersję po optymalizacjach (tag `po-2.5`) na przemian, ale tylko w konfiguracjach Flat-SQL. Wynik pokazuje zmianę silnika. Miejsca wśród innych silników, które Benchmark 1 podał dla prototypu w każdej grupie warunków, z niego nie widać. ADR-0038 wymaga porównywania silników w tej samej grupie i tym samym narzędziem; ADR-0037 – tej samej metodyki dla prototypu i późniejszych wersji. Pełny pomiar wszystkich silników trwa około 6 godzin (Benchmark 1: 860 procesów), głównie przez silniki wstawiające wiersz po wierszu (PGlite, DuckDB-Wasm, PouchDB). Wersje pakietów silników odniesienia i ich adaptery od Benchmarku 1 się nie zmieniły; w narzędziu zmienił się tylko adapter Flat-SQL i doszła operacja `insert_many` na końcu obciążenia.

## Decyzja

1. Miejsce wersji po optymalizacjach wyznacza się z wyników silników odniesienia z Benchmarku 1 (`docs/research/data/benchmark-1/raw`) i wyników Flat-SQL z Benchmarku 2 (`after/raw`), w tych samych grupach warunków; skrypt `docs/research/data/benchmark-2/engines-report.mjs`.
2. Różnicę między sesjami mierzy prototyp, zmierzony w obu: raport podaje zmianę jego median we wszystkich parach konfiguracja–operacja. Miejsce wersji po optymalizacjach jest niepewne, gdy inny silnik grupy różni się od niej o mniej niż 10%, i jest oznaczone gwiazdką.
3. Miejsce liczy się według mediany wśród domyślnych konfiguracji silników grupy, bez czystego JavaScriptu i bez wariantów z innym ustawieniem trwałości.
4. Silniki mają rolę według analizy porównawczej ([wasm-engines-comparison.md](../research/wasm-engines-comparison.md) §4–§5): pokrewne (sql.js, SQLite WASM, wa-sqlite, PouchDB, RxDB; Zeta-Lite jako niemierzony), inne zastosowanie (PGlite, DuckDB-Wasm), punkty odniesienia (AlaSQL, Dexie.js, czysty JavaScript). Wykresy oznaczają rolę kolorem i kształtem znacznika; raport podaje iloraz czasu Flat-SQL i najszybszego pokrewnego silnika grupy.

## Uzasadnienie

- Prototyp zmierzony w obu sesjach różni się w 113 parach o medianę −2,3% (w 103 parach Benchmark 2 szybszy), 90. percentyl wartości bezwzględnej 6,7%, 111 par w granicach ±10%; największa różnica to zimny start w przeglądarce (−11 do −12%). Przeglądarka zaktualizowała się w tym czasie z 156.0.4314.42 do 156.0.4314.50. Różnica jest o rząd wielkości mniejsza niż większość różnic między silnikami, a próg 10% obejmuje wszystkie pary obciążenia.
- Sumy kontrolne wersji po optymalizacjach zgadzają się z wynikami Flat-SQL z Benchmarku 1 w 630 porównaniach, a te – z wynikami wszystkich silników odniesienia; porównanie dotyczy więc tych samych odpowiedzi.
- Raasveldt i in. wskazują porównywanie systemów w różnych warunkach jako typowy błąd [B3]; warunki (maszyna, system, wersje pakietów, narzędzie, grupy) są tu te same, a jedyną różnicą jest sesja, której wpływ zmierzono.
- Podział na silniki pokrewne i pozostałe wynika z niszy Flat-SQL opisanej w analizie porównawczej: DuckDB-Wasm i PGlite rozwiązują inny problem, więc wygrana z nimi albo przegrana nie mówi o konkurencyjności Flat-SQL w jego zastosowaniu.

## Odrzucone alternatywy

- **Ponowny pomiar wszystkich silników razem z wersją po optymalizacjach** – około 6 godzin pracy laptopa bez innych obciążeń dla silników, których kod się nie zmienił; odłożony. Wykonany później, zastąpi pkt 1–2 tej decyzji, a obecne wyniki pozostaną do porównania.
- **Wyniki Benchmarku 2 przeskalowane o różnicę sesji** – różnica nie jest stała (od −12% do +4% w zależności od operacji), więc korekta wprowadzałaby własny błąd; zamiast niej miejsce bliskie innemu silnikowi jest oznaczone jako niepewne.
- **Ranking wszystkich konfiguracji, także wariantów `strict` i czystego JavaScriptu** – ten sam silnik liczyłby się kilka razy, a czysty JavaScript jest dolną granicą, nie silnikiem bazy danych.

## Konsekwencje

- [benchmark-results.md §2.9](../research/benchmark-results.md#29-flat-sql-po-optymalizacjach-na-tle-innych-silników) podaje tabele w układzie Benchmarku 1 z miejscem Flat-SQL przed i po optymalizacjach; dane i wykresy: `docs/research/data/benchmark-2/engines.md`, `engines.json`, `figures/07`–`12`.
- Wnioski o zimnym starcie wymagają uwzględnienia różnicy sesji (−11%); porównanie na przemian z prototypem pozostaje miarą zmiany samego silnika.
- Następny pomiar wszystkich silników (np. po nowych optymalizacjach) mierzy w jednej sesji także wersję bieżącą.

## Źródła

[B1], [B3]; [benchmark-results.md §2.9](../research/benchmark-results.md#29-flat-sql-po-optymalizacjach-na-tle-innych-silników), [wasm-engines-comparison.md](../research/wasm-engines-comparison.md).
