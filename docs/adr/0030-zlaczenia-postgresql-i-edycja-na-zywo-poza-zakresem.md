# ADR-0030: Złączenia, integracja z PostgreSQL i wspólna edycja na żywo poza zakresem

| | |
|---|---|
| Status | przyjęta |
| Data | 2026-10-21 |
| Rodzaj | proces |
| Powiązane | ADR-0004, ADR-0015, ADR-0023; dawne future 5.2, 6.3, 11.1–11.4 |

## Kontekst

[todo_future.md](../todo_future.md) planował trzy rozszerzenia, które wynikały z analizy wdrożeń i literatury, ale nie z założeń projektu:

- **5.2 złączenia tylko do odczytu po kluczu głównym** – oznaczone jako „decyzja autora”, bo zmieniają tożsamość dialektu; ADR-0004 zastrzega, że ich wprowadzenie wymaga nowej decyzji;
- **6.3 współdzielona treść edytowana na żywo** – aktualizacje dokumentu z biblioteki CRDT (np. Yjs) zapisywane jako niezmienne wiersze, wzór z tldraw i Excalidraw;
- **sekcja 11 integracja z istniejącą bazą główną** – eksport do PostgreSQL na potrzeby analityki, łącznik CDC w obie strony, mapowanie typów i kluczy, konflikty między źródłami; wzór z PowerSync, ElectricSQL i Community Health Toolkit.

## Decyzja

Wszystkie trzy rozszerzenia wypadają z planów (decyzja autora):

1. Flat-SQL pozostaje dialektem jednotabelowym; ADR-0004 obowiązuje bez zmian i nie planuje się złączeń także w wariancie tylko do odczytu.
2. Wspólna edycja dokumentów na żywo nie jest zastosowaniem Flat-SQL; wspólną edycję tekstu realizuje zewnętrzna biblioteka CRDT poza silnikiem (ADR-0015).
3. Serwer Flat-SQL jest własną bazą główną (ADR-0023); synchronizacji z istniejącą bazą serwerową (PostgreSQL) projekt nie zapewnia.

Numery 5.2, 6.3 i 11.1–11.4 nie są używane ponownie; tabela „Poza zakresem” w [todo_future.md](../todo_future.md#poza-zakresem-decyzja-autora) podaje powody.

## Uzasadnienie

- **Złączenia.** Jednotabelowość to część definicji języka, a nie brak wynikający z niedojrzałości. Złączenie po unikalnym kluczu nie psułoby przewidywalności ani replikacji [M2], [L8], więc przeszkoda nie jest techniczna, tylko dotyczy tożsamości dialektu. Krytyka braku złączeń [M1], [D4] zostaje omówiona w pracy ([literature-comparison.md §5](../references/literature-comparison.md#5-model-danych-flat-sql-bez-join)).
- **Edycja na żywo.** Analiza wdrożeń wymienia współpracę na żywo nad dokumentami wśród zastosowań, w które Flat-SQL nie powinien celować ([real-world-usage.md §4](../research/real-world-usage.md#4-zastosowania-flat-sql-potwierdzone-analizą)); edytory tego typu opierają się na CRDT tekstu (Yjs, Automerge), a Flat-SQL byłby dla nich tylko magazynem.
- **PostgreSQL.** Łącznik z istniejącą bazą serwerową to zakres osobnych produktów (PowerSync, ElectricSQL [Q11]) i osobny podsystem (CDC, mapowanie typów, znaczniki HLC dla zapisów spoza Flat-SQL). Community Health Toolkit przenosi dane do PostgreSQL osobnym narzędziem. Przechowywanie paczek w bazie SQL backendu daje adapter magazynu serwera ([sync-server.md §6](../future/components/sync-server.md#6-dane-na-serwerze)) bez mapowania danych.
- Żadne z trzech rozszerzeń nie dotyczy zakresu pracy inżynierskiej (ADR-0029).

## Odrzucone alternatywy

- **Zachowanie zadań jako odległych rozszerzeń** – plan zawierałby zmianę tożsamości dialektu i funkcje osobnych produktów bez potwierdzonej potrzeby w docelowych zastosowaniach.
- **Złączenia jako opcja wyłączona domyślnie** – nadal zmieniają definicję dialektu i rozbudowują planer.

## Konsekwencje

- Odnośniki do dawnych zadań w ADR-0004, ADR-0015, [conclusions.md](../references/conclusions.md), [literature-comparison.md](../references/literature-comparison.md), [real-world-usage.md](../research/real-world-usage.md), [use-cases.md](../use-cases.md) i w indeksie literatury wskazują tabelę „Poza zakresem”.
- Zastosowania wymagające analityki danych wszystkich użytkowników w PostgreSQL (zbieranie danych w terenie) potrzebują osobnego narzędzia.
- Przy współbieżnej edycji tego samego pola tekstowego wygrywa jedna wersja (LWW) – ograniczenie opisane w pracy.

## Źródła

Decyzja autora; [M1], [M2], [L8], [D4], [Q11]; [real-world-usage.md §4](../research/real-world-usage.md#4-zastosowania-flat-sql-potwierdzone-analizą); [literature-comparison.md §5](../references/literature-comparison.md#5-model-danych-flat-sql-bez-join).
