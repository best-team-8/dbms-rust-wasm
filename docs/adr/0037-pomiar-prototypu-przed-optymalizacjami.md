# ADR-0037: Pomiar prototypu przed optymalizacjami, ta sama metodyka w obu pomiarach

| | |
|---|---|
| Status | przyjęta |
| Data | 2026-10-24 |
| Rodzaj | proces |
| Powiązane | ADR-0020, ADR-0029, ADR-0038; zadania todo §1, §2, §4 |

## Kontekst

Dotychczasowa kolejność w [todo.md](../todo.md) przewidywała optymalizacje silnika (§2: 2.1, 2.2, 2.8, 2.3–2.5) przed badaniami i benchmarkami (§4). Wyniki z 6 października 2026 ([benchmark-results.md](../research/benchmark-results.md)) pochodzą z jednego uruchomienia procesu na silnik, bez rozrzutu, bez sumy kontrolnej wyników i tylko dla sql.js i czystego JavaScriptu jako punktów odniesienia. Gdyby optymalizacje poprzedziły nową metodykę, praca nie miałaby wiarygodnego stanu wyjściowego: różnicę „przed” i „po” trzeba by liczyć między pomiarem starą metodyką a pomiarem nową, czyli mieszać zmianę silnika ze zmianą sposobu mierzenia.

## Decyzja

1. Kolejność w `todo.md`: sekcja 1 → sekcja 4 → sekcja 2 → zadania „dobrze mieć”. Sekcja 4 jest fizycznie przeniesiona przed sekcję 2; numery zadań się nie zmieniają.
2. Najpierw powstaje narzędzie pomiarowe z §4 i pełny pomiar prototypu („Benchmark 1”). Stan silnika w tym pomiarze oznacza tag `prototyp`.
3. Po optymalizacjach z §2 oba stany (tag `prototyp` i bieżący) są mierzone tym samym narzędziem, w tym samym środowisku i według tej samej procedury. Runner przyjmuje katalog z wersją silnika (np. worktree tagu), więc nowsza wersja narzędzia mierzy też starszy silnik.
4. Lista słabych stron prototypu z Benchmarku 1, z hipotezami przyczyn, jest wejściem do §2: każda optymalizacja wskazuje, którą słabość adresuje, i jest oceniana w metryce, w której słabość wystąpiła.

## Uzasadnienie

- **Porównanie przed/po wymaga jednej metodyki.** Zmiana liczby uruchomień, rozgrzewki, sposobu liczenia średniej albo zestawu zapytań zmienia wynik niezależnie od silnika [B1], [B3]. Raasveldt i in. wymieniają porównywanie wyników z różnych konfiguracji pomiaru jako typowy błąd porównań baz danych [B3].
- **Rozrzut decyduje o tym, co jest zmianą.** W pojedynczych przebiegach dwie konfiguracje wykonujące ten sam kod różniły się o kilkadziesiąt procent ([benchmark-scenario.md §7](../research/benchmark-scenario.md)). Bez wielu uruchomień procesu nie da się stwierdzić, czy optymalizacja o 20% jest efektem, czy szumem [B1].
- **Optymalizacje mają cel w danych.** Kierunki w §2 (np. 2.8 – szybszy `UPDATE`) są dziś hipotezami z analizy kodu. Pomiar prototypu z profilem i planami zapytań pozwala je potwierdzić albo odrzucić przed pracą nad nimi.
- **Zakres pracy się nie zmienia** (ADR-0029): §4 była wymagana w całości, zmienia się tylko jej miejsce w kolejności.

## Odrzucone alternatywy

- **Optymalizacje, potem pomiar** (dotychczasowa kolejność) – brak stanu wyjściowego zmierzonego tą samą metodyką; wkład optymalizacji nie dałby się oddzielić od zmiany metodyki.
- **Pomiar prototypu starym narzędziem, nowe narzędzie dopiero po optymalizacjach** – ten sam problem, a do tego stary runner nie ma rozrzutu ani sumy kontrolnej, więc wynik „przed” nie spełnia wymagań [B1].
- **Zamrożenie narzędzia w tagu i pomiar każdego stanu jego własną wersją** – poprawki samego narzędzia (np. nowy silnik odniesienia) nie objęłyby wtedy stanu wyjściowego. Jedna, najnowsza wersja narzędzia mierzy obie wersje silnika.

## Konsekwencje

- §2 zaczyna się dopiero po Benchmarku 1; jego wyniki i lista słabych stron są w [benchmark-results.md](../research/benchmark-results.md).
- Interfejs adapterów w `bench/` musi działać z API silnika z tagu `prototyp`. Jeśli optymalizacja zmieni API (np. 2.4 – binarny format wyników), adapter wykrywa dostępne metody, zamiast wymagać nowych.
- Wyniki z 6 października 2026 zostają w dokumentacji jako wstępne, uzyskane inną metodyką; nie są porównywane z Benchmarkiem 1 wprost.

## Źródła

[B1], [B3]; [benchmark-scenario.md](../research/benchmark-scenario.md), [benchmark-results.md](../research/benchmark-results.md).
