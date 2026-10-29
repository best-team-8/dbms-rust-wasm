# ADR-0016: Outbox jako referencje do wierszy, odbudowywany z WAL

| | |
|---|---|
| Status | przyjęta; zdanie o zmianach schematu w pkt 1 zastąpione przez [ADR-0027](0027-struktura-w-kodzie-aplikacji-tylko-dokladanie.md); moment prowadzenia outboxa wierszy i zapis outboxa w snapshocie zastąpione przez [ADR-0043](0043-outbox-po-wlaczeniu-synchronizacji.md) |
| Data | 2026-10-06 (silnik, [flatsql-core/README.md](../../crates/flatsql-core/README.md#synchronizacja)) |
| Rodzaj | architektura |
| Powiązane | ADR-0009, ADR-0015, ADR-0017 |

## Kontekst

Zmiany wykonane offline muszą czekać na wysłanie, przetrwać zamknięcie aplikacji i nie zostać wysłane dwa razy. Badania aplikacji mobilnych pokazują, że bez takiego mechanizmu zmiany giną, gdy aplikacja zostanie zamknięta w trakcie przerwy w sieci [L27].

## Decyzja

1. Lokalna zmiana wiersza zapisuje w outboksie tylko **referencję** `(tabela, klucz) → seq`; przy eksporcie odczytywany jest bieżący stan wiersza albo tombstone, więc wiele kolejnych zmian jednego wiersza daje jedną zmianę do wysłania. Zmiany schematu są przechowywane w całości.
2. Każda lokalna operacja w WAL niesie swój numer `seq`, a potwierdzenie serwera jest zapisywane w WAL jako `Ack(seq)`; outbox jest odbudowywany przy odtwarzaniu.
3. Zmiany otrzymane z serwera są zapisywane w WAL z `seq = 0` i nie trafiają do outboksu (brak „echa”).

## Uzasadnienie

- To akademicki wzorzec pracy bez połączenia: trwały log zmian offline, w którym nowszy zapis unieważnia wcześniejsze wpisy, odtwarzany po połączeniu [C10].
- Wysyłanie bieżącego stanu zmienionych wierszy to synchronizacja delta-state [L28] – 100 kolejnych `UPDATE` jednego wiersza daje jedną zmianę.
- Outbox w WAL nie wymaga osobnego magazynu ani osobnej procedury odtwarzania (ADR-0009).

## Odrzucone alternatywy

- **Outbox jako osobna tabela lub osobny magazyn** – dodatkowy zapis przy każdej zmianie i ryzyko rozjazdu z danymi.
- **Kolejka wszystkich operacji** – rośnie bez potrzeby i powtarza pośrednie stany.

## Konsekwencje

- Niewysłany outbox przetrwa restart aplikacji offline (test `outbox_survives_restart_and_ack_is_durable`), ale nie usunięcie danych przez przeglądarkę – stąd `navigator.storage.persist()` (todo 1.5).
- Wiersz zmieniany ponownie przesuwa się na koniec kolejki, więc zmiany jednej transakcji mogą trafić do różnych paczek (ADR-0018).

## Źródła

[C10], [L27], [L28].
