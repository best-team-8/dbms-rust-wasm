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
| [1.2](#12-eksperyment-synchronizacja-katalogu-w-windows) `windows_directory_flush_needs_write_access_and_backup_semantics` | Jak w Windows utrwalić wpisy katalogu po zmianie nazwy pliku? | zaliczony (eksperyment) |

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
| Procedura | Test `windows_directory_flush_needs_write_access_and_backup_semantics` w `crates/flatsql-core/tests/checkpoint.rs`, kompilowany tylko w Windows. Katalog jest otwierany na cztery sposoby, a dla otwartego uchwytu wywoływany jest `sync_all`. Wyniki zestawia tabela pod opisem. Test `directory_of_the_real_file_system_can_be_synced` wywołuje funkcję używaną przez `FileStorage` (`sync_dir`) na każdej platformie. |
| Kontrola negatywna | Warianty bez prawa zapisu albo bez flagi kończą się błędem, więc test odróżnia działający sposób od niedziałających. |
| Wynik | Hipoteza potwierdzona (tabela). 23.10.2026, Windows 11 Home 26200, NTFS, Rust 1.99.0, commit „storage: sync directory before truncating the WAL”. |
| Wpływ na projekt | `sync_dir` w `crates/flatsql-core/src/storage/file.rs` otwiera katalog w Windows z prawem zapisu i flagą `FILE_FLAG_BACKUP_SEMANTICS`; w pozostałych systemach używa `File::open` i `fsync`. |
| Ograniczenia | Eksperyment pokazuje, że wywołanie się udaje. Czy NTFS utrwala przy nim zmianę nazwy przed późniejszymi zapisami, można sprawdzić tylko odcięciem zasilania, którego tu nie wykonano. Sprawdzono jedną wersję Windows; w Linuksie test `directory_of_the_real_file_system_can_be_synced` działa w CI. |

| Otwarcie katalogu | Wynik otwarcia | `sync_all` (`FlushFileBuffers`) |
|---|---|---|
| `File::open` (odczyt, bez flagi) | błąd 5, odmowa dostępu | – |
| zapis, bez `FILE_FLAG_BACKUP_SEMANTICS` | błąd 5, odmowa dostępu | – |
| odczyt i `FILE_FLAG_BACKUP_SEMANTICS` | otwarty | błąd 5, odmowa dostępu |
| zapis i `FILE_FLAG_BACKUP_SEMANTICS` | otwarty | udany |

## 2. Synchronizacja replik

## 3. Zapytania i format danych
