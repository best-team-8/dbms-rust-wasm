# ADR-0009: Trwałość – fizyczny WAL tylko-redo i snapshot

| | |
|---|---|
| Status | przyjęta |
| Data | 2026-10-06 (silnik, [flatsql-core/src/storage/README.md](../../crates/flatsql-core/src/storage/README.md#model-snapshot-i-wal)) |
| Rodzaj | architektura |
| Powiązane | ADR-0006, ADR-0008, ADR-0010, ADR-0011, ADR-0016; todo 1.2, 1.6, 2.6 |

## Kontekst

Dane w pamięci (ADR-0006) muszą przetrwać zamknięcie aplikacji i awarię, a koszt trwałego zatwierdzenia nie może rosnąć z rozmiarem bazy – tak jest w sql.js, który zapisuje cały plik przy każdym utrwaleniu.

## Decyzja

1. **WAL** – tylko dopisywanie; każda ramka ma LSN, liczbę operacji i sumę CRC32; operacje są **fizyczne i idempotentne** (`Put` = pełny wiersz, `Delete` = klucz).
2. **Snapshot (checkpoint)** – pełny obraz zatwierdzonego stanu z sumą CRC32, zapisywany, gdy WAL przekroczy próg; potem WAL jest czyszczony.
3. **Odtwarzanie** = snapshot + ramki o LSN większym niż LSN snapshotu; odczyt WAL kończy się na pierwszej ramce niekompletnej lub z błędną sumą.

## Uzasadnienie

- Koszt zatwierdzenia jest proporcjonalny do zmiany, nie do bazy (H5: 0,34 ms wobec 8,7 ms w sql.js).
- Log fizyczny zamiast logowania poleceń: logowanie poleceń daje ok. 1,5× większą przepustowość, ale wymaga deterministycznego wykonania i dłużej odtwarza [S6]; w Flat-SQL te same wpisy WAL służą synchronizacji (zmiana = wiersz + znacznik), a odtworzenie nie zależy od zegara ani wyników zapytań.
- Bez fazy undo, bo niezatwierdzone zmiany nie trafiają do WAL (ADR-0008); model „log + checkpoint” dla baz w pamięci opisał [S1].
- Test ucinający WAL w każdym bajcie sprawdza, że baza zawsze wraca do stanu po ostatniej pełnej transakcji.

## Odrzucone alternatywy

- **Eksport całej bazy przy zatwierdzeniu (sql.js)** – koszt rośnie liniowo z bazą.
- **Tylko snapshot na żądanie (Zeta-Lite [D5])** – zmiany po ostatnim snapshocie giną.
- **Logowanie poleceń (VoltDB)** – zob. wyżej.
- **Równoległe odtwarzanie (SiloR)** – wymaga wątków, czyli COOP/COEP (ADR-0013).

## Konsekwencje

- Checkpoint kopiuje całą bazę synchronicznie (35–49 ms) – to „naiwny snapshot” z [S10], który daje przestój w interfejsie (todo 2.6).
- Protokół checkpointu na plikach wymaga `fsync` katalogu przed obcięciem WAL [S8] (todo 1.2, 1.6).

## Źródła

[S1], [S6], [S8], [S10], [D5], [P1] rozdz. 3.
