# ADR-0038: Metodyka benchmarku: powtórzenia procesu, sumy kontrolne, grupy warunków, wskazana wersja silnika

| | |
|---|---|
| Status | przyjęta |
| Data | 2026-10-24 |
| Rodzaj | proces |
| Powiązane | ADR-0020 (zastępuje), ADR-0013, ADR-0019, ADR-0037; zadania todo 4.1–4.4, 4.6, 4.7, 4.9, 4.10, 4.12, 4.13 |

## Kontekst

ADR-0020 ustalił punkty odniesienia (sql.js, czysty JS, Flat-SQL natywnie, serwer z symulowanym RTT) i metodykę wstępnych pomiarów: jeden proces na silnik, średnia z pętli, zgodność wyników sprawdzana liczbą wierszy. Uzupełnienia wskazane w ADR-0020 (wiele uruchomień [B1], zimny start [B3], pamięć, pauzy checkpointu, szerszy zestaw silników) są zadaniami todo §4. ADR-0037 przesunął §4 przed optymalizacje i wymaga, żeby ta sama metodyka i to samo narzędzie mierzyły prototyp i późniejsze wersje. Zakłada to ADR-0001 i ADR-0002 (teza o lokalnych zapytaniach i pracy offline jest weryfikowana pomiarem) oraz ADR-0013 (wątek główny jako tryb domyślny, Worker jako opcja – koszt opcji trzeba zmierzyć).

## Decyzja

1. **Jednostka powtórzenia to proces.** Każdy pomiar (silnik × rodzaj zadania) działa w nowym procesie Node.js albo nowej przeglądarce z pustym profilem, co najmniej 10 razy; kolejność silników losowana ziarnem równym numerowi uruchomienia. Wynik: mediana z kwartylami; zmiana między wersjami wykazana, gdy 95% przedziały ufności średnich (t Studenta) się nie zachodzą.
2. **Suma kontrolna wyników** zamiast liczby wierszy: 64-bitowy skrót postaci kanonicznej (liczby × 10^6 zaokrąglone, wiersze zapytań bez `ORDER BY` sortowane) każdej operacji oraz stan tabeli (`COUNT`, `SUM(qty)`, `SUM(price)`) po każdej fazie zapisu; ten sam algorytm w benchmarku natywnym. Top-k dostaje kryterium rozstrzygające remisy (`ORDER BY price DESC, id ASC`), bo bez niego odpowiedź nie jest jednoznaczna.
3. **Grupy warunków**: „pamięć” (ten sam wątek, Node.js i przeglądarka), „trwały zapis” (IndexedDB, wątek główny, zapis mierzony do trwałości), „Worker” (każde wywołanie to `postMessage` w obie strony), „natywnie”. Wyniki porównuje się tylko w grupie.
4. **Silniki** (wersje dokładnie przypięte w `package.json`): sql.js, oficjalny SQLite WASM (`:memory:` i `opfs-sahpool`), wa-sqlite z `IDBBatchAtomicVFS`, PGlite (pamięć, IndexedDB, OPFS), DuckDB-Wasm (pakiet synchroniczny w wątku wywołującym i w Workerze), AlaSQL, Dexie.js, PouchDB, RxDB, czysty JS. Każdy w konfiguracji domyślnej albo zalecanej; odstępstwa w nazwie konfiguracji. Zeta-Lite nie jest mierzony: kod zamknięty, brak modułu do pobrania; wyniki podają osobno wartości deklarowane przez autora, oznaczone jako niepotwierdzone pomiarem.
5. **Przeglądarka sterowana automatycznie** (playwright-core, zainstalowany Microsoft Edge), strona izolowana nagłówkami COOP/COEP wyłącznie dla zegara 5 µs i `measureUserAgentSpecificMemory`; żadna konfiguracja nie używa `SharedArrayBuffer`.
6. **Narzędzie mierzy wskazaną wersję silnika** (`--flatsql KATALOG`, np. worktree tagu `prototyp`); adaptery używają tylko API prototypu. Natywne obciążenie należy do mierzonej wersji i zgłasza identyfikator `WORKLOAD`; niezgodny identyfikator przerywa pomiar.
7. **Surowe wyniki w repozytorium** (`docs/research/data/<benchmark>/raw/`) razem z samodzielnym skryptem, który tworzy z nich tabele i wykresy SVG.

## Uzasadnienie

- Georges i in. pokazują, że zmienność między uruchomieniami maszyny wirtualnej jest większa niż w obrębie jednego uruchomienia, a wnioski z pojedynczego przebiegu bywają odwrócone; zalecają wiele uruchomień i przedziały ufności [B1]. We wstępnych pomiarach dwie konfiguracje Flat-SQL wykonujące ten sam kod wstawiania różniły się o kilkadziesiąt procent w pojedynczym przebiegu.
- Raasveldt i in. wymieniają jako typowe błędy porównań baz danych brak sprawdzenia poprawności wyników, porównywanie systemów w różnych warunkach i przebiegi tylko „na gorąco” [B3]. Porównanie liczby wierszy nie odróżnia poprawnej odpowiedzi od niepoprawnej o tej samej długości; przykładem było dawne zapytanie top-k, w którym każda cena występuje 10 razy, więc kolejność 10 zwróconych wierszy zależała od silnika. Grupy warunków oddzielają silniki w pamięci od silników utrwalających każdy zapis.
- Losowana kolejność ogranicza wpływ trendów w czasie (nagrzewanie laptopa) i pamięci podręcznej systemu na jeden silnik.
- Automatyzacja przeglądarki daje te same powtórzenia procesu co w Node.js; nowy profil przy każdym uruchomieniu daje prawdziwy zimny start (bez pamięci podręcznej HTTP i skompilowanego kodu). Zegar 100 µs bez izolacji uniemożliwiał wiarygodne percentyle operacji mikrosekundowych [Q5].
- Pomiar wersji wskazanej katalogiem realizuje ADR-0037: nowsza wersja narzędzia (np. z nowym silnikiem odniesienia) mierzy też stan wyjściowy.

## Odrzucone alternatywy

- **Wiele iteracji w jednym procesie zamiast wielu procesów** – nie pokazuje zmienności między uruchomieniami [B1].
- **Strona bez izolacji cross-origin** – percentyle operacji krótszych niż 100 µs byłyby nieczytelne; izolacja nie zmienia działania żadnego mierzonego silnika.
- **Rozkład Zipfa i obciążenie YCSB** [B5] – odłożone (todo_future 12.8); zmiana obciążenia unieważniłaby porównanie z prototypem, jeśli nastąpi po Benchmarku 1, więc pozostaje poza „benchmark-1”.
- **Pomijanie silników, których API różni się od SQL** – bazy dokumentowe są konkurencją w aplikacjach offline; mierzone przez własne API z odwzorowaniem opisanym w metodyce i kontrolą wyników.
- **Własny Worker DuckDB (AsyncDuckDB)** – ten sam silnik co pakiet synchroniczny, a własny protokół komunikatów utrudniałby porównanie kosztu Workera z innymi silnikami.
- **SQLite z VFS `opfs`** – wymaga `SharedArrayBuffer` i nagłówków; `opfs-sahpool` jest wskazany przez dokumentację SQLite jako szybszy [Q7].

## Konsekwencje

- Pełny pomiar trwa kilka godzin (najdłużej silniki wstawiające wiersz po wierszu z osobnym kosztem wywołania: DuckDB-Wasm, PGlite); runner wznawia przerwany pomiar.
- Wyniki z 6 października 2026 pozostają w dokumentacji jako wstępne; Benchmark 1 jest stanem wyjściowym dla todo §2.
- Zmiana obciążenia (zapytań, danych, ziaren) wymaga nowego identyfikatora `WORKLOAD` i ponownego pomiaru prototypu.
- Pomiar na telefonie (todo 4.7) i koszt utrwalania w Linuksie (część todo 4.13) używają tego samego narzędzia: trybu ręcznego i runnera.

## Źródła

[B1], [B3], [B5], [Q1], [Q5], [Q7], [W11]; [benchmark-scenario.md](../research/benchmark-scenario.md), [bench/README.md](../../bench/README.md).
