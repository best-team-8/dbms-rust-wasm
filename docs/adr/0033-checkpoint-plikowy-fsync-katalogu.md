# ADR-0033: Checkpoint na plikach – synchronizacja katalogu przed obcięciem WAL

| | |
|---|---|
| Status | przyjęta |
| Data | 2026-10-23 |
| Rodzaj | architektura |
| Powiązane | ADR-0005, ADR-0009 (realizuje konsekwencję o `fsync` katalogu), ADR-0011; todo 1.2, 1.3, 1.6, 4.13 |

## Kontekst

`FileStorage` (CLI i przyszły panel WinUI) wykonywał checkpoint w kolejności: zapis `snapshot.fsql.tmp` i `fsync`, `rename` na `snapshot.fsql`, obcięcie `wal.fsql` i jego `fsync`. Zmiana nazwy jest operacją na katalogu, a systemy plików utrwalają operacje na katalogu i zapisy do różnych plików w dowolnej kolejności, dopóki katalog nie zostanie zsynchronizowany [S8]. Test z symulowanym systemem plików ([tests-results.md §1.1](../research/tests-results.md#11-utrwalenie-zmiany-nazwy-snapshotu-przed-obcięciem-wal)) pokazał dwa przypadki utraty potwierdzonych transakcji: utrwalone obcięcie WAL przy nieutrwalonej zmianie nazwy snapshotu oraz nowy plik WAL bez utrwalonego wpisu w katalogu. Zadanie 1.2 dopuszczało dwie poprawki: `fsync` katalogu albo nowy plik WAL przy każdym checkpoincie.

## Decyzja

1. Kolejność checkpointu: zapis pliku tymczasowego, jego `fsync`, `rename`, **`fsync` katalogu**, obcięcie WAL, `fsync` WAL. Katalog jest synchronizowany także po utworzeniu pliku WAL i (w katalogu nadrzędnym) po utworzeniu katalogu bazy.
2. Kroki checkpointu są synchronizowane zawsze. `with_fsync(false)` / `--no-fsync` wyłącza tylko `fsync` po zatwierdzeniu transakcji, więc naraża ostatnie transakcje, nigdy starsze.
3. W Windows katalog jest otwierany z prawem zapisu i flagą `FILE_FLAG_BACKUP_SEMANTICS`, a następnie wywoływany jest `FlushFileBuffers` (`File::sync_all`); w systemach POSIX – `File::open` i `fsync`. Wystarcza biblioteka standardowa, bez zależności (ADR-0005).
4. `FileStorage` wykonuje operacje na plikach przez interfejs `Vfs`; `OsFs` to prawdziwy system plików, testy podstawiają symulację zapisującą każdą operację.

## Uzasadnienie

- Pomijanie ramek objętych snapshotem (po LSN) sprawia, że pozostawiony stary WAL jest nieszkodliwy. Jedynym wymaganiem jest więc, żeby obcięcie WAL nie trafiło na dysk przed zmianą nazwy; `fsync` katalogu między nimi to wymusza. Tę kolejność wskazali Pillai i in. jako typowy błąd protokołów aktualizacji przez zmianę nazwy [S8].
- Po poprawce test przechodzi dla wszystkich 35 i 49 punktów awarii (2 i 4 checkpointy), a kontrola negatywna odtwarza oba przypadki utraty danych.
- Eksperyment w Windows 11 ([tests-results.md §1.2](../research/tests-results.md#12-eksperyment-synchronizacja-katalogu-w-windows)): katalogu nie da się otworzyć bez `FILE_FLAG_BACKUP_SEMANTICS`, a `FlushFileBuffers` na uchwycie tylko do odczytu kończy się odmową dostępu; z prawem zapisu wywołanie się udaje.
- Koszt to jeden `fsync` katalogu na checkpoint (i na utworzenie pliku), a nie na zatwierdzenie.
- Interfejs `Vfs` pozwala sprawdzić protokół bez awarii zasilania i jest podstawą testu z todo 1.6.

## Odrzucone alternatywy

- **Nowy plik WAL przy każdym checkpoincie** (numer pliku zapisany w snapshocie) – utworzenie nowego pliku i usunięcie starego to także operacje na katalogu, więc przed usunięciem starego WAL i tak potrzebny jest `fsync` katalogu; do tego zmienia się układ katalogu danych wspólny dla Rust i Node.js (ADR-0011).
- **`MoveFileEx` z `MOVEFILE_WRITE_THROUGH` w Windows** – według dokumentacji flaga dotyczy tylko przeniesienia przez kopiowanie (między woluminami), a wywołanie wymagałoby deklaracji funkcji systemowej spoza biblioteki standardowej.
- **Bez synchronizacji katalogu w Windows** – NTFS prowadzi dziennik metadanych, ale dokumentacja nie gwarantuje kolejności utrwalenia zmiany nazwy względem obcięcia innego pliku.
- **Synchronizacja checkpointu zależna od `--no-fsync`** – utrata danych sprzed checkpointu wykracza poza obietnicę tej opcji (ostatnie transakcje).

## Konsekwencje

- Dodatkowy `fsync` katalogu przy każdym checkpoincie; koszt do zmierzenia w todo 4.13.
- Gwarancja w Windows zależy od tego, czy NTFS utrwala zmianę nazwy przy `FlushFileBuffers` na uchwycie katalogu; tego nie da się sprawdzić bez odcięcia zasilania.
- `FsStore` w Node.js ma ten sam protokół bez żadnego `fsync` (todo 1.3).
- Pełny przegląd stanów po awarii (inne podzbiory operacji, częściowe zapisy) – todo 1.6.

## Źródła

[S8]; [tests-results.md §1.1–1.2](../research/tests-results.md#1-trwałość-i-odtwarzanie-po-awarii).
