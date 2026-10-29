# ADR-0008: Transakcje – undo log, atomowość instrukcji, wykonanie szeregowe

| | |
|---|---|
| Status | przyjęta |
| Data | 2026-10-06 (silnik, [flatsql-core/README.md](../../crates/flatsql-core/README.md#transakcje)) |
| Rodzaj | architektura |
| Powiązane | ADR-0006, ADR-0009, ADR-0018 |

## Kontekst

Silnik w pamięci, wywoływany synchronicznie z jednego wątku, potrzebuje transakcji: niejawnych (autocommit) i jawnych (`BEGIN … COMMIT/ROLLBACK`), z cofaniem częściowo wykonanych instrukcji.

## Decyzja

1. Transakcje wykonują się szeregowo, w jednym wątku – bez blokad i wielu wersji wierszy.
2. Każda zmiana wiersza lub schematu zapisuje poprzednią wersję w undo logu; błąd w połowie instrukcji cofa tę instrukcję, także wewnątrz jawnej transakcji (atomowość na poziomie instrukcji).
3. Operacje są kodowane do bufora transakcji; przy `COMMIT` bufor staje się jedną ramką WAL, a wpisy outboksu – widoczne dla synchronizacji; `ROLLBACK` nie zostawia śladu w WAL.
4. Snapshot, eksport outboksu i przyjmowanie zmian zdalnych są zabronione w trakcie jawnej transakcji.

## Uzasadnienie

- Wykonanie szeregowe w jednym wątku to uznana technika dla danych w pamięci: izolacja serializowalna lokalnie jest „za darmo” ([S4], [P1] rozdz. 7).
- Skoro niezatwierdzone zmiany nigdy nie trafiają do WAL, log nie potrzebuje fazy undo przy odtwarzaniu (ADR-0009).

## Odrzucone alternatywy

- **MVCC i współbieżne transakcje (jak Zeta-Lite [D5])** – złożoność bez korzyści w jednym wątku; wiele przeplatanych transakcji to granica koncepcji ([use-cases.md](../research/use-cases.md)).
- **Wielowątkowe wykonanie** – wymaga `SharedArrayBuffer`, czyli nagłówków COOP/COEP (ADR-0013).

## Konsekwencje

- Długa transakcja blokuje pozostałe – w wątku głównym także interfejs.
- Koszt każdej zmiany: wpis w undo logu, kodowana operacja WAL i referencja w outboksie – stąd wolniejsze od SQLite inserty i aktualizacje (H4).

## Źródła

[S4], [P1] rozdz. 7, [D5].
