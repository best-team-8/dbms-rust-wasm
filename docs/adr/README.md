# Rejestr decyzji projektowych (ADR)

Każda istotna decyzja projektu Flat-SQL ma tu osobny zapis: z jakiego założenia wynika, dlaczego ją podjęto, jakie alternatywy odrzucono i co z niej wynika. Decyzje sprzed utworzenia rejestru (20.10.2026) odtworzono z README, prezentacji projektu, [architecture.md](../architecture.md), [flat-sql.md](../flat-sql.md), dokumentów badań i historii repozytorium – ich data to data wprowadzenia do repozytorium.

Identyfikatory literatury w nawiasach (np. [C7]) odsyłają do [indeksu literatury](../references/README.md); numery zadań – do [todo.md](../todo.md) i [todo_future.md](../todo_future.md).

## Rodzaje decyzji

- **założenie** – co Flat-SQL obiecuje (standard projektu); zmiana założenia zmienia tożsamość projektu;
- **architektura** – jak założenia są spełnione; można ją zmienić, jeśli wymaga tego założenie;
- **proces** – jak prowadzone są badania i prace.

## Spis

| Nr | Decyzja | Rodzaj | Status |
|---|---|---|---|
| [0001](0001-zapytania-lokalne-bez-serwera.md) | Zapytania wykonywane lokalnie, bez udziału serwera | założenie | przyjęta |
| [0002](0002-local-first-z-serwerem-koordynujacym.md) | Local-First: praca offline, synchronizacja w tle, serwer koordynujący | założenie | przyjęta |
| [0003](0003-privacy-by-design-dane-domyslnie-lokalne.md) | Privacy by Design: dane domyślnie lokalne | założenie | przyjęta |
| [0004](0004-dialekt-flat-sql-bez-zlaczen.md) | Dialekt Flat-SQL: jedna tabela, obowiązkowy klucz, bez złączeń | założenie | przyjęta |
| [0005](0005-rdzen-rust-bez-io-i-zaleznosci.md) | Jeden rdzeń w Rust bez I/O i bez zależności | architektura | przyjęta |
| [0006](0006-dane-w-pamieci.md) | Dane w całości w pamięci | architektura | przyjęta |
| [0007](0007-wasm32-bez-memory64.md) | Cel `wasm32`, bez memory64 | architektura | przyjęta |
| [0008](0008-transakcje-undo-log-wykonanie-szeregowe.md) | Transakcje: undo log, atomowość instrukcji, wykonanie szeregowe | architektura | przyjęta |
| [0009](0009-wal-fizyczny-i-snapshot.md) | Trwałość: fizyczny WAL tylko-redo i snapshot | architektura | przyjęta |
| [0010](0010-group-commit-w-przegladarce.md) | Group commit w przeglądarce | architektura | przyjęta |
| [0011](0011-przenosny-format-plikow.md) | Przenośny format plików zamiast obrazu pamięci | architektura | przyjęta |
| [0012](0012-indexeddb-domyslnie-opfs-jako-opcja.md) | IndexedDB jako magazyn domyślny, OPFS jako opcja | architektura | przyjęta |
| [0013](0013-watek-glowny-domyslnie-worker-jako-opcja.md) | Wątek główny jako tryb domyślny, Worker jako opcja | architektura | przyjęta |
| [0014](0014-system-typow.md) | System typów: cztery typy, koercja bezstratna, porządek całkowity | architektura | przyjęta |
| [0015](0015-hlc-i-lww-per-wiersz.md) | Synchronizacja: zegar HLC i LWW per wiersz z tombstone'ami | architektura | przyjęta |
| [0016](0016-outbox-w-wal.md) | Outbox jako referencje do wierszy, odbudowywany z WAL | architektura | przyjęta; zdanie o zmianach schematu w pkt 1 zastąpione przez [ADR-0027](0027-struktura-w-kodzie-aplikacji-tylko-dokladanie.md) |
| [0017](0017-protokol-push-pull-z-kursorem.md) | Protokół push/pull z kursorem i epoką | architektura | przyjęta; pkt 2 zastąpiony przez [ADR-0023](0023-konfigurowalne-tryby-serwera-synchronizacji.md) |
| [0018](0018-atomowosc-wiersza-miedzy-replikami.md) | Między replikami atomowy jest wiersz | architektura | przyjęta |
| [0019](0019-przekazywanie-wynikow-do-js.md) | Przekazywanie wyników do JS: JSON teraz, bufor binarny docelowo | architektura | przyjęta |
| [0020](0020-punkty-odniesienia-benchmarku.md) | Punkty odniesienia i metodyka benchmarku | proces | przyjęta |
| [0021](0021-rdzen-literatury.md) | Rdzeń literatury i kryteria doboru | proces | przyjęta; kryterium włączenia i skład rdzenia zastąpione przez [ADR-0031](0031-rdzen-literatury-wedlug-cytowan-w-pracy.md) |
| [0022](0022-podzial-list-zadan.md) | Podział list zadań: optymalizacja i rozszerzenia | proces | zastąpiona przez [ADR-0029](0029-listy-zadan-wedlug-zakresu-pracy.md) |
| [0023](0023-konfigurowalne-tryby-serwera-synchronizacji.md) | Konfigurowalne tryby serwera synchronizacji | architektura | przyjęta |
| [0024](0024-bez-wiazan-dla-pythona.md) | Bez wiązań dla Pythona i synchronizacji w procesie backendu | proces | przyjęta; zakres zawężony przez [ADR-0025](0025-serwer-synchronizacji-za-bramka-backendu.md) |
| [0025](0025-serwer-synchronizacji-za-bramka-backendu.md) | Serwer synchronizacji za bramką backendu | architektura | przyjęta |
| [0026](0026-reguly-w-backendzie-mechanizmy-w-serwerze.md) | Reguły w backendzie, mechanizmy w serwerze synchronizacji | architektura | przyjęta |
| [0027](0027-struktura-w-kodzie-aplikacji-tylko-dokladanie.md) | Struktura tabel w kodzie aplikacji, między wersjami tylko dokładanie | architektura | przyjęta |
| [0028](0028-kursor-jako-numer-przyjecia.md) | Kursor synchronizacji jako numer przyjęcia nadany przez serwer | architektura | przyjęta |
| [0029](0029-listy-zadan-wedlug-zakresu-pracy.md) | Listy zadań według zakresu pracy inżynierskiej | proces | przyjęta |
| [0030](0030-zlaczenia-postgresql-i-edycja-na-zywo-poza-zakresem.md) | Złączenia, integracja z PostgreSQL i wspólna edycja na żywo poza zakresem | proces | przyjęta |
| [0031](0031-rdzen-literatury-wedlug-cytowan-w-pracy.md) | Rdzeń literatury według cytowań w pracy dyplomowej | proces | przyjęta |
| [0032](0032-metodyka-testow-poprawnosci.md) | Metodyka testów poprawności oddzielna od metodyki benchmarków | proces | przyjęta |

## Zasady prowadzenia rejestru

1. Nowa decyzja dostaje kolejny numer i plik `NNNN-krotki-opis.md` według szablonu poniżej; wpis trafia do spisu.
2. Treści przyjętej decyzji się nie zmienia. Gdy decyzja się zmienia, powstaje nowy zapis, a stary dostaje status „zastąpiona przez ADR-NNNN” (dozwolone są tylko poprawki literówek i odnośników).
3. Odrzucone ważne alternatywy zapisuje się w sekcji „Odrzucone alternatywy” decyzji, której dotyczą; osobny zapis ze statusem „odrzucona” – gdy alternatywa była wcześniej planowana.
4. Decyzja, która wynika z wykonanego zadania, odsyła do tego zadania, a zadanie – do decyzji.

## Szablon

```markdown
# ADR-NNNN: Tytuł

| | |
|---|---|
| Status | proponowana / przyjęta / zastąpiona przez ADR-NNNN / odrzucona |
| Data | RRRR-MM-DD |
| Rodzaj | założenie / architektura / proces |
| Powiązane | ADR-…, zadania … |

## Kontekst
Jaki problem lub jakie założenie wymaga decyzji.

## Decyzja
Co postanowiono – jednym akapitem.

## Uzasadnienie
Dlaczego, z odwołaniem do założeń, pomiarów i literatury.

## Odrzucone alternatywy
Co rozważano i dlaczego odrzucono.

## Konsekwencje
Co zyskujemy, co tracimy, jakie zadania z tego wynikają.

## Źródła
```
