# ADR-0014: System typów – cztery typy, koercja bezstratna, porządek całkowity

| | |
|---|---|
| Status | przyjęta |
| Data | 2026-10-06 (silnik, [flat-sql.md §2](../flat-sql.md)) |
| Rodzaj | architektura |
| Powiązane | ADR-0004, ADR-0011; todo 1.8 |

## Kontekst

Dialekt ma być prosty (ADR-0004), a wartości muszą dawać się porządkować – są kluczami `BTreeMap` w tabelach i indeksach – oraz kodować identycznie na wszystkich replikach.

## Decyzja

1. Typy: `INT` (64-bitowe), `FLOAT` (64-bitowe), `TEXT` (UTF-8), `BOOL`, a w każdej kolumnie także `NULL`; popularne aliasy (`INTEGER`, `VARCHAR`, `DECIMAL`…) są akceptowane, długość i precyzja ignorowane.
2. Koercja przy zapisie tylko bezstratna: `INT` → `FLOAT` zawsze, `FLOAT` → `INT` tylko dla wartości całkowitych, `0`/`1` → `BOOL`; tekst w kolumnie liczbowej to błąd `type`.
3. `Value` ma porządek całkowity: liczby porównywane numerycznie bez względu na `INT`/`FLOAT`, NaN na końcu – dzięki temu każda wartość może być kluczem.
4. Porównanie tekstu z liczbą to błąd, nie niejawna konwersja.

## Uzasadnienie

- Porządek całkowity jest warunkiem użycia `BTreeMap` dla kluczy i indeksów.
- Bezstratna koercja i jawne błędy typów chronią dane, które przez synchronizację trafiają na inne repliki – błąd na jednej replice nie rozlewa się po cichu.

## Odrzucone alternatywy

- **Dynamiczne typy kolumn jak w SQLite** – wartość o innym typie w tej samej kolumnie utrudnia porządek i porównania między replikami.
- **Typy z precyzją (`DECIMAL(p,s)`), daty, typy binarne** – poza zakresem prototypu; typ wektorowy rozważany w future 5.6.

## Konsekwencje

- Kolumny agregatów i wyrażeń mają przewidywalne typy (`SUM` całkowita, gdy wszystkie wartości są `INT`; `AVG` zawsze `FLOAT`).
- NaN jest kodowany bit po bicie – do porównywania replik po skrócie trzeba go kanonizować (todo 1.8) [P2].

## Źródła

[P2] §2.5; [flat-sql.md §2](../flat-sql.md), [architecture.md §2](../architecture.md).
