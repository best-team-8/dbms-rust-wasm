# ADR-0015: Synchronizacja – zegar HLC i LWW per wiersz z tombstone'ami

| | |
|---|---|
| Status | przyjęta |
| Data | 2026-10-06 (silnik, [architecture.md §5.1](../architecture.md)) |
| Rodzaj | architektura |
| Powiązane | ADR-0002, ADR-0004, ADR-0016, ADR-0017, ADR-0018; todo 1.1, 1.9; future 2.5, 12.3, 12.5, 12.6 |

## Kontekst

Przy pracy offline (ADR-0002) każde urządzenie przyjmuje zapisy, więc konflikty są nieuniknione ([P1] rozdz. 5). Potrzebna jest reguła, która na każdej replice daje ten sam wynik bez koordynacji, niezależnie od kolejności dostarczenia zmian.

## Decyzja

1. Każda zmiana ma znacznik `Stamp { time, node }`: `time` z hybrydowego zegara logicznego (HLC – 48 bitów milisekund i 16 bitów licznika w jednej liczbie 64-bitowej), `node` – losowy identyfikator repliki, zapamiętywany trwale. Zegar nigdy się nie cofa i wchłania znaczniki otrzymane od innych replik.
2. **Last-writer-wins per wiersz:** zmiana zdalna jest stosowana tylko, gdy jej znacznik jest większy od znacznika wiersza lub jego tombstone'u.
3. Usunięte wiersze zostawiają tombstone ze znacznikiem usunięcia, żeby spóźniony `UPDATE` nie wskrzesił wiersza.

## Uzasadnienie

- Scalanie przez maksimum w porządku całkowitym jest łączne, przemienne i idempotentne, więc repliki zbiegają się do tego samego stanu – to rejestr LWW z teorii CRDT ([L2], [L28]); potwierdza to test 25 ziaren × 300 operacji na 3 replikach.
- HLC zachowuje przyczynowość jak zegar logiczny i pozostaje blisko czasu fizycznego [C2]; płaski model sprawia, że konflikt zawsze dotyczy jednego klucza (ADR-0004).
- Rozwiązanie jest proste i działa offline; Actual Budget stosuje w praktyce ten sam zegar.

## Odrzucone alternatywy

- **Wektory wersji i scalanie przez aplikację** – większe metadane i obowiązek pisania procedur scalania.
- **Pełna historia operacji (event sourcing, CRDT operacyjne)** – rosnący log; Flat-SQL przesyła stan wierszy (delta-state, [L28]).
- **CRDT tekstu w silniku** – poza modelem; wspólną edycję tekstu realizuje zewnętrzna biblioteka (dawne future 6.3, poza zakresem według ADR-0030).

## Konsekwencje

- LWW zbiega się kosztem **utraconych aktualizacji**: współbieżna zmiana z mniejszym znacznikiem znika bez śladu [C9]. Dotyczy to zmian różnych kolumn jednego wiersza (future 12.3 – LWW per kolumna) i liczników (future 2.5 – aktualizacje przemienne).
- Zegar urządzenia ustawiony w przyszłość wygrywa wszystkie konflikty – potrzebna reguła ignorowania znaczników spoza granicy dryfu [C2] (todo 1.1).
- Unikalność klucza jest tylko lokalna – dwa wstawienia tego samego klucza offline nadpisują się po cichu (future 12.1, 12.5).
- Tombstone'y rosną – usuwanie po stabilności przyczynowej (future 12.6).

## Źródła

[C2], [C9], [L2], [L24], [L28], [P1] rozdz. 5, 8, 9.
