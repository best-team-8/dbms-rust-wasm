# ADR-0022: Podział list zadań – optymalizacja i rozszerzenia

| | |
|---|---|
| Status | zastąpiona przez [ADR-0029](0029-listy-zadan-wedlug-zakresu-pracy.md) |
| Data | 2026-10-20 |
| Rodzaj | proces |
| Powiązane | ADR-0021; [todo.md](../todo.md), [future/todo.md](../future/todo.md) |

## Kontekst

Wcześniejsza lista zadań mieszała poprawki istniejącego prototypu z nowymi funkcjami i integracjami, przez co trudno było ustalić zakres pracy inżynierskiej i odróżnić to, co prototyp już obiecuje, od tego, co dopiero ma obiecywać.

## Decyzja

- **[todo.md](../todo.md)** zawiera wyłącznie zadania ulepszające to, co prototyp już ma (silnik, składowanie, istniejący protokół synchronizacji, bridge, serwer-hub, benchmarki, dokumentacja prototypu), wynikające z literatury lub z analizy wdrożeń.
- **[future/todo.md](../future/todo.md)** zawiera rozszerzenia – nowe możliwości potrzebne dojrzałemu produktowi – pod warunkiem, że nie odbierają atutów projektu (ADR-0001–ADR-0006, ADR-0013).
- Zadania usunięte po weryfikacji z literaturą są wymienione z powodem w [todo.md §5](../todo.md).

## Uzasadnienie

- Granica „istniejące a nowe” odpowiada podziałowi „praca inżynierska a dalszy rozwój” i ułatwia uzgodnienie zakresu z promotorem.
- Każde zadanie ma podstawę w literaturze lub we wdrożeniach, więc łatwo sprawdzić, czy jest jeszcze potrzebne.

## Odrzucone alternatywy

- **Jedna lista z priorytetami** – nie oddziela poprawek tego, co już jest obiecane, od nowych obietnic.

## Konsekwencje

- Zadanie, które zmienia sposób działania istniejącej funkcji bez nowej składni ani API (np. LWW per kolumna), trafia do `todo.md`; nowa składnia lub nowe API (np. `MERGE ADD`) – do `todo_future.md`.
- Zadanie, którego wykonanie wymaga decyzji, kończy się nowym zapisem w rejestrze ADR.

## Źródła

[literature.md](../references/literature.md), [use-cases.md](../research/use-cases.md).
