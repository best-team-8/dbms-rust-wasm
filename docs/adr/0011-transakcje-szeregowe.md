# 0011. Transakcje wykonywane po kolei, z dziennikiem wycofania

- Status: obowiązuje
- Ostatnia zmiana: 2026-10-06

## Kontekst

Silnik w pamięci, wywoływany synchronicznie z jednego wątku, potrzebuje transakcji niejawnych (każda instrukcja osobno) i jawnych (`BEGIN … COMMIT` albo `ROLLBACK`). Musi też umieć cofnąć instrukcję, która zawiodła w połowie.

## Decyzja

1. Transakcje wykonują się po kolei w jednym wątku, bez blokad i bez wielu wersji wierszy.
2. Każda zmiana wiersza albo schematu zapisuje poprzedni stan w dzienniku wycofania (undo log). Błąd w połowie instrukcji cofa tę instrukcję, także wewnątrz jawnej transakcji.
3. Zmiany transakcji są kodowane do bufora. Przy `COMMIT` bufor staje się jedną ramką WAL, a zmiany stają się widoczne dla synchronizacji. `ROLLBACK` nie zostawia śladu w WAL.
4. W trakcie jawnej transakcji nie można zrobić snapshotu, wyeksportować kolejki zmian ani przyjąć zmian z innego urządzenia.

## Uzasadnienie

- Wykonanie po kolei w jednym wątku to uznana technika dla danych w pamięci. Lokalna izolacja jest wtedy pełna i nic nie kosztuje (Stonebraker i in., 2007; Kleppmann, 2017, rozdz. 7).
- Niezatwierdzone zmiany nigdy nie trafiają do WAL, więc odtwarzanie po awarii nie musi niczego wycofywać (decyzja 0016).

## Odrzucone warianty

- Wiele wersji wierszy i transakcje współbieżne (MVCC), jak w Zeta-Lite. W jednym wątku to złożoność bez korzyści. Izolację dla pamięci agentów opisuje decyzja 0025.
- Wykonanie wielowątkowe. Wymaga nagłówków izolacji strony (decyzja 0007).

## Konsekwencje

- Długa transakcja blokuje pozostałe, a w wątku głównym także interfejs.
- Każda zmiana kosztuje wpis w dzienniku wycofania i w WAL, a przy włączonej synchronizacji także w kolejce zmian. Dlatego zapisy są wolniejsze niż w SQLite, a odczyty szybsze.

## Historia

- 2026-10-06: model transakcji pierwszej wersji silnika.
