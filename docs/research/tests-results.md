# Wyniki testów poprawności

Opisy testów poprawności Flat-SQL i ich wyniki, przygotowane według [metodyki testów](tests-methodology.md). Każdy opis odpowiada na jedno pytanie o gwarancję silnika: trwałość zatwierdzonych danych, zbieżność replik albo stały format danych. Testy uruchamiane przy każdej zmianie wymienia też [benchmark-scenario.md §6](benchmark-scenario.md#6-walidacja-local-first-w-warunkach-offline-h6); tutaj opisano je pełniej, razem z modelem awarii, kontrolą negatywną i ograniczeniami.

## Środowisko

| | |
|---|---|
| System operacyjny | Windows 11 Home (kompilacja 26200), system plików NTFS |
| Procesor | AMD Ryzen 5 4600H (6 rdzeni, 12 wątków) |
| Pamięć | 16 GB |
| Rust | 1.99.0 (stable), cele `x86_64-pc-windows-msvc` i `wasm32-unknown-unknown` |
| Node.js | 24.21.0 lokalnie, 22 w CI |
| CI | GitHub Actions, [.github/workflows/build-test.yml](../../.github/workflows/build-test.yml), `ubuntu-latest` |

Wynik każdego testu podaje datę i commit (przez temat), na którym go uzyskano. Testy silnika uruchamia `cargo test --workspace`, testy end-to-end `npm run build && npm test`.

## Zestawienie

| Test | Pytanie | Status |
|---|---|---|
| [1.1](#11-utrwalenie-zmiany-nazwy-snapshotu-przed-obcięciem-wal) `committed_data_survives_crash_when_only_synced_writes_persist` | Czy po awarii w dowolnym miejscu pracy magazynu plikowego zostają wszystkie potwierdzone transakcje, gdy na dysku jest tylko to, co wymusiła synchronizacja? | zaliczony |
| [1.2](#12-eksperyment-synchronizacja-katalogu-w-windows) `windows_directory_flush_needs_write_access_and_backup_semantics`, `platform: a directory can be synced…` | Jak w Windows utrwalić wpisy katalogu po zmianie nazwy pliku? | zaliczony (eksperyment) |
| [1.3](#13-utrwalanie-zapisów-w-magazynie-plikowym-nodejs) `FsStore: flushed transactions survive a crash when only synced writes persist`, `FsStore: fsync can be turned off for WAL appends only` | Czy po `await flush()` i awarii magazyn plikowy Node.js zachowuje wszystkie utrwalone transakcje? | zaliczony |

## 1. Trwałość i odtwarzanie po awarii

Magazyn plikowy przechowuje bazę w dwóch plikach: `snapshot.fsql` (pełny obraz bazy z chwili ostatniego punktu kontrolnego, ang. *checkpoint*) i `wal.fsql` (dziennik zapisu z wyprzedzeniem, ang. *write-ahead log*, WAL: każda zatwierdzona transakcja dopisuje do niego jedną ramkę z sumą kontrolną). Checkpoint zapisuje nowy snapshot i obcina WAL. Odtworzenie bazy wczytuje snapshot i stosuje ramki WAL o numerze większym niż numer zapisany w snapshocie, więc ramki objęte już snapshotem są pomijane.

### 1.1 Utrwalenie zmiany nazwy snapshotu przed obcięciem WAL

| Element | Zawartość |
|---|---|
| Pytanie | Czy po awarii w dowolnym miejscu pracy magazynu plikowego `FileStorage` baza wraca do stanu ze wszystkimi potwierdzonymi transakcjami, jeśli system plików utrwalił tylko to, do czego zmusiło go wywołanie `fsync`? |
| Uzasadnienie | Checkpoint zapisuje snapshot do pliku tymczasowego, zmienia jego nazwę na `snapshot.fsql` i obcina WAL. Zmiana nazwy i utworzenie pliku są operacjami na katalogu. Pillai i in. [S8] pokazali, że systemy plików utrwalają je w różnej kolejności względem zapisów do plików, a pewność daje dopiero `fsync` katalogu. Jeśli obcięcie WAL trafi na dysk przed zmianą nazwy, po awarii zostaje stary snapshot i pusty WAL, czyli znikają wszystkie transakcje od poprzedniego checkpointu. To samo grozi nowo utworzonemu plikowi WAL: jego zawartość jest zsynchronizowana, ale wpis w katalogu nie. |
| Model awarii i założenia | Awaria może nastąpić po każdej operacji na plikach. Operacja zapisu, zmiany długości, utworzenia i zmiany nazwy jest na pewno utrwalona tylko wtedy, gdy po niej zakończył się `fsync` tego samego pliku (zapis i zmiana długości) albo `fsync` katalogu (utworzenie i zmiana nazwy). Test bierze najbardziej pesymistyczny stan tego modelu: na dysku jest wyłącznie to, co wymusiła synchronizacja. Za niezawodne uznano odczyt plików i samo wywołanie `fsync`. Test nie obejmuje innych podzbiorów utrwalonych operacji, częściowo zapisanych ramek ani utworzenia katalogu bazy w katalogu nadrzędnym. |
| Procedura | Plik `crates/flatsql-core/tests/checkpoint.rs`, test `committed_data_survives_crash_when_only_synced_writes_persist`. `FileStorage` działa na symulowanym systemie plików (`SimFs`), który zapisuje każdą operację z numerem pliku. Obciążenie: utworzenie tabeli i 9 transakcji (wstawienia, zmiany, usunięcia, jedna transakcja jawna z dwiema zmianami) oraz jeden checkpoint wywołany przez test. Obciążenie wykonano dwa razy: bez automatycznych checkpointów i z progiem 150 B, przy którym checkpoint następuje także w trakcie pracy. Dla każdego punktu awarii test buduje zawartość katalogu z operacji wymuszonych synchronizacją, otwiera bazę i porównuje tabelę z modelem w pamięci (`BTreeMap` aktualizowana tymi samymi zmianami). Wyrocznia: stan po *j* transakcjach, gdzie *j* jest nie mniejsze od liczby transakcji potwierdzonych przed awarią i nie większe od liczby transakcji rozpoczętych. Polecenie: `cargo test -p flatsql-core --test checkpoint`. |
| Kontrola negatywna | Na kodzie sprzed zmiany test zawiódł przy awarii po 9 z 31 operacji: `got None, expected the state after 1..=1 transactions`. Tabela utworzona w potwierdzonej transakcji zniknęła, bo wpis pliku `wal.fsql` w katalogu nie był utrwalony. Po dodaniu samej synchronizacji katalogu przy tworzeniu WAL test zawiódł przy awarii po 20 z 32 operacji, zaraz po obcięciu WAL w checkpoincie: `got None, expected the state after 3..=3 transactions`. Obcięcie było utrwalone, zmiana nazwy snapshotu nie, więc zniknęły trzy potwierdzone transakcje. |
| Wynik | Zaliczony. 35 punktów awarii przy 2 checkpointach i 49 punktów przy 4 checkpointach, przebieg poniżej 1 s. 23.10.2026, środowisko z początku dokumentu, commit „storage: sync directory before truncating the WAL”. |
| Wpływ na projekt | `FileStorage` synchronizuje katalog po zmianie nazwy snapshotu, przed obcięciem WAL, a także po utworzeniu pliku WAL i katalogu bazy. Kroki checkpointu są synchronizowane zawsze, także przy wyłączonym `fsync` zatwierdzeń (`--no-fsync` w CLI), więc wyłączenie go naraża tylko ostatnie transakcje. Operacje na plikach przeszły za interfejs `Vfs`, żeby test mógł podstawić symulację. |
| Ograniczenia | Test sprawdza jeden stan na punkt awarii: „nic ponad to, co wymuszone”. Nie dowodzi, że prawdziwy system plików dotrzymuje modelu, w szczególności że `fsync` katalogu w Windows utrwala zmianę nazwy (eksperyment 1.2). Utrata zasilania i błędy dysku nie są symulowane. |

### 1.2 Eksperyment: synchronizacja katalogu w Windows

| Element | Zawartość |
|---|---|
| Pytanie | Jak w Windows wymusić utrwalenie wpisów katalogu (zmiany nazwy pliku), skoro katalogu nie da się otworzyć zwykłym `File::open`? |
| Uzasadnienie | W systemach POSIX wpisy katalogu utrwala `fsync` deskryptora katalogu [S8]. Dokumentacja Windows nie opisuje odpowiednika. Flaga `MOVEFILE_WRITE_THROUGH` funkcji `MoveFileEx` według dokumentacji dotyczy tylko przeniesienia przez kopiowanie, czyli między woluminami. Hipoteza: wystarczy `FlushFileBuffers` na uchwycie katalogu, ale uchwyt musi być otwarty z flagą `FILE_FLAG_BACKUP_SEMANTICS` (inaczej katalogu nie da się otworzyć) i z prawem zapisu. |
| Założenia | NTFS, konto bez uprawnień administratora, katalog w katalogu tymczasowym użytkownika. Biblioteka standardowa Rust: `OpenOptions::custom_flags` i `File::sync_all`, które w Windows wywołuje `FlushFileBuffers`. |
| Procedura | Test `windows_directory_flush_needs_write_access_and_backup_semantics` w `crates/flatsql-core/tests/checkpoint.rs`, kompilowany tylko w Windows. Katalog jest otwierany na cztery sposoby, a dla otwartego uchwytu wywoływany jest `sync_all`. Test `directory_of_the_real_file_system_can_be_synced` wywołuje funkcję używaną przez `FileStorage` (`sync_dir`) na każdej platformie. To samo dla Node.js sprawdza test `platform: a directory can be synced with the open flags FsStore uses` w `test/fs-store.test.js`: `FileHandle.sync()` na katalogu otwartym flagami używanymi przez `FsStore`, a w Windows także flagami `'r'` i `'w'`. Wyniki zestawia tabela pod opisem. |
| Kontrola negatywna | Warianty bez prawa zapisu albo bez flagi kończą się błędem, więc test odróżnia działający sposób od niedziałających. |
| Wynik | Hipoteza potwierdzona (tabela). 23.10.2026, Windows 11 Home 26200, NTFS, Rust 1.99.0, commit „storage: sync directory before truncating the WAL”. |
| Wpływ na projekt | `sync_dir` w `crates/flatsql-core/src/storage/file.rs` otwiera katalog w Windows z prawem zapisu i flagą `FILE_FLAG_BACKUP_SEMANTICS`; w pozostałych systemach używa `File::open` i `fsync`. `FsStore` w Node.js otwiera katalog flagą `'r+'` w Windows (biblioteka libuv sama dodaje `FILE_FLAG_BACKUP_SEMANTICS`) i `'r'` w pozostałych systemach, gdzie otwarcie katalogu do zapisu kończy się błędem `EISDIR`. |
| Ograniczenia | Eksperyment pokazuje, że wywołanie się udaje. Czy NTFS utrwala przy nim zmianę nazwy przed późniejszymi zapisami, można sprawdzić tylko odcięciem zasilania, którego tu nie wykonano. Sprawdzono jedną wersję Windows; w Linuksie test `directory_of_the_real_file_system_can_be_synced` działa w CI. |

| Otwarcie katalogu | Wynik otwarcia | `sync_all` (`FlushFileBuffers`) |
|---|---|---|
| `File::open` (odczyt, bez flagi) | błąd 5, odmowa dostępu | – |
| zapis, bez `FILE_FLAG_BACKUP_SEMANTICS` | błąd 5, odmowa dostępu | – |
| odczyt i `FILE_FLAG_BACKUP_SEMANTICS` | otwarty | błąd 5, odmowa dostępu |
| zapis i `FILE_FLAG_BACKUP_SEMANTICS` | otwarty | udany |
| Node.js 24.21, `open(dir, 'r')` | otwarty | `EPERM` |
| Node.js 24.21, `open(dir, 'r+')` | otwarty | udany |
| Node.js 24.21, `open(dir, 'w')` | `EISDIR` | – |

### 1.3 Utrwalanie zapisów w magazynie plikowym Node.js

| Element | Zawartość |
|---|---|
| Pytanie | Czy magazyn plikowy Node.js (`FsStore`, używany przez serwer synchronizacji) po awarii zachowuje wszystkie transakcje, na które aplikacja doczekała się w `await flush()`, jeśli system plików utrwalił tylko to, do czego zmusiła go synchronizacja? |
| Uzasadnienie | `FsStore` ma ten sam układ plików co magazyn natywny z testu 1.1, ale zapisywał je funkcjami `appendFile`, `writeFile` i `rename` bez żadnego `fsync`. Zakończenie `flush()` oznaczało więc tylko przekazanie danych do pamięci podręcznej systemu operacyjnego. Ten sam problem kolejności zmiany nazwy i obcięcia WAL dotyczy checkpointu, a plik `meta.json` (identyfikator węzła, kursor synchronizacji) zastępowany przez zmianę nazwy mógł po awarii zostać pusty [S8]. |
| Model awarii i założenia | Jak w teście 1.1: awaria po dowolnej operacji na pliku, na dysku tylko operacje wymuszone przez `fsync` pliku albo katalogu. Za niezawodne uznano samo `fsync` i odczyt. Nie są modelowane utworzenie katalogu bazy ani częściowe zapisy. |
| Procedura | Plik `test/fs-store.test.js`, test `FsStore: flushed transactions survive a crash when only synced writes persist`. Moduł `test/support/crash-fs.js` podmienia funkcje `node:fs/promises` (także importowane po nazwie) na wersje, które wykonują prawdziwą operację i zapisują ją w tym samym modelu co test 1.1: utworzenie pliku, zmiana nazwy, zapis od przesunięcia, zmiana długości, `fsync` pliku, `fsync` katalogu. Obciążenie: `FlatDB` na `FsStore`, utworzenie tabeli i 9 transakcji jak w teście 1.1, każda zakończona `await flush()` i zapisem kursora do `meta.json`, jeden checkpoint wywołany przez test i próg checkpointu 300 B. Dla każdego punktu awarii zawartość katalogu z operacji wymuszonych trafia do nowego katalogu tymczasowego, który otwiera `FlatDB`; wyrocznia jak w teście 1.1 (model w `Map`). Drugi test sprawdza, że `fsync: false` wyłącza synchronizację tylko po dopisaniu do WAL, a checkpoint nadal synchronizuje plik i katalog. Polecenie: `node --test test/fs-store.test.js`. |
| Kontrola negatywna | Na `FsStore` sprzed zmiany test zawiódł przy awarii po 8 z 48 operacji: `got no table, expected the state after 1..1 transactions`; bez żadnego `fsync` po awarii nie zostaje nic. W nowym `FsStore` z celowo usuniętą synchronizacją katalogu po zmianie nazwy test zawiódł po 30 z 72 operacji: `got no table, expected the state after 3..3 transactions` (obcięty WAL, snapshot nieutrwalony). Test opcji `fsync: false` zawiódł na starym kodzie, bo opcji nie było. |
| Wynik | Zaliczony. 85 punktów awarii, 2 checkpointy, przebieg ok. 1 s. 23.10.2026, środowisko z początku dokumentu, commit „bridge: fsync the WAL, snapshot and meta.json in FsStore”. |
| Wpływ na projekt | `FsStore` synchronizuje każde dopisanie do WAL (`FileHandle.datasync`), a checkpoint i `meta.json` zapisuje przez plik tymczasowy z `fsync`, zmianę nazwy i `fsync` katalogu; WAL obcina dopiero potem. Opcja `new FsStore(dir, { fsync: false })` wyłącza synchronizację zatwierdzeń i `meta.json`, nie checkpointu. Urwany `meta.json` jest przy otwarciu traktowany jak pusty. |
| Ograniczenia | Jak w teście 1.1: jeden, najbardziej pesymistyczny stan na punkt awarii. Model obejmuje tylko funkcje `node:fs/promises` wywoływane przez `FsStore`; zachowanie libuv i systemu przy prawdziwej awarii nie jest sprawdzane. |

## 2. Synchronizacja replik

## 3. Zapytania i format danych
