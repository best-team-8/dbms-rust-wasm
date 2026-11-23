# Awarie zapisu na pliki

Test sprawdza, że po awarii w dowolnym miejscu pracy magazynu plikowego baza wraca do stanu ze wszystkimi potwierdzonymi transakcjami. Dotyczy obu magazynów plikowych: `FileStorage` w Rust (narzędzie natywne) i `FsStore` w Node.js (serwer synchronizacji). Oba miały błędy, które test wykrył i które poprawiono.

## Dlaczego

Checkpoint zapisuje snapshot do pliku tymczasowego, zmienia jego nazwę na `snapshot.fsql` i obcina WAL. Zmiana nazwy jest operacją na katalogu. System plików może utrwalić ją później niż obcięcie WAL, jeśli program nie wymusi utrwalenia katalogu (Pillai i in., 2014). Wtedy po awarii zostaje stary snapshot i pusty WAL, a transakcje od poprzedniego checkpointu znikają. `FsStore` w ogóle nie wymuszał utrwalenia.

## Model awarii

Awaria może nastąpić po każdej operacji na plikach. Operacje wymuszone przez `fsync` pliku albo katalogu są na dysku. Każda inna wcześniejsza operacja może być na dysku albo nie, niezależnie od kolejności, a zapis może być urwany w połowie. Przy wyłączonym `fsync` zatwierdzeń wymagane są tylko transakcje objęte zakończonym checkpointem.

Model nie obejmuje urwania zapisu w innym miejscu niż połowa, uszkodzenia bajtów (wykrywają je sumy kontrolne), utworzenia katalogu bazy ani drugiej awarii w trakcie odtwarzania. Zakłada, że `fsync` działa zgodnie z opisem.

## Jak

System plików jest symulowany: każda operacja magazynu jest zapisywana. Obciążenie to utworzenie tabeli i 9 transakcji (wstawienia, zmiany, usunięcia, jedna transakcja jawna) z checkpointami. Dla każdego punktu awarii test buduje z zapisanych operacji każdy stan dysku dopuszczony przez model i otwiera bazę. Wzorcem jest model w pamięci aktualizowany tymi samymi zmianami: stan po *j* transakcjach, gdzie *j* leży między liczbą transakcji potwierdzonych a rozpoczętych przed awarią. Odtworzona baza dostaje jeszcze jeden wiersz, który musi być w niej po ponownym otwarciu. Tak wychodzą śmieci po urwanym zapisie.

Gdy stanów w jednym punkcie awarii jest za dużo, test losuje je ze stałym ziarnem.

```bash
cargo test -p flatsql-core --test checkpoint
```

```bash
node --test test/fs-store.test.js
```

Pełne przebiegi: `FLATSQL_CRASH_STATES=4000000 cargo test -p flatsql-core --release --test checkpoint every_permitted` i `FLATSQL_CRASH_STATES=4096 node --test --test-name-pattern="every permitted" test/fs-store.test.js`.

## Wynik

| Wariant | Punkty awarii | Stany dopuszczone przez model | Sprawdzone przy każdej zmianie | Pełny przebieg |
|---|---:|---:|---:|---|
| Rust, `fsync`, bez automatycznego checkpointu | 35 | 76 | 76 | wszystkie |
| Rust, `fsync`, checkpoint co 150 B | 49 | 112 | 112 | wszystkie |
| Rust, bez `fsync`, bez automatycznego checkpointu | 24 | 118 953 | 17 517 | wszystkie |
| Rust, bez `fsync`, checkpoint co 150 B | 36 | 2 959 137 | 65 489 | wszystkie |
| Node.js, `fsync`, checkpoint co 300 B | 85 | 226 | 226 | wszystkie |
| Node.js, bez `fsync`, checkpoint co 300 B | 54 | ok. 4,8·10¹¹ | 3 068 | 174 791 losowych |

Wszystkie sprawdzone stany odtwarzają stan zatwierdzony. Przebieg przy każdej zmianie trwa 3,0 s w Rust i 1,6 s w Node.js. Pomiar z 23.10.2026.

## Kontrola negatywna

- Kod sprzed poprawki w Rust zgubił tabelę z potwierdzonej transakcji, bo wpis pliku WAL w katalogu nie był utrwalony. Po pierwszej poprawce zgubił trzy transakcje, bo obcięcie WAL utrwaliło się przed zmianą nazwy snapshotu.
- `FsStore` sprzed poprawki nie zachował po awarii niczego.
- Otwarcie bazy bez checkpointu po odrzuceniu urwanego końca WAL gubiło zapis wykonany po odtworzeniu.
- Kroki checkpointu utrwalane tylko przy włączonym `fsync` dawały nieczytelny snapshot.
- Urwany plik `meta.json` (identyfikator urządzenia, kursor) przerywał otwarcie bazy.

## Utrwalenie katalogu w Windows

Dokumentacja Windows nie opisuje, jak utrwalić wpisy katalogu. Eksperyment pokazał, że katalog trzeba otworzyć z prawem zapisu i flagą `FILE_FLAG_BACKUP_SEMANTICS`, a potem wywołać `FlushFileBuffers`:

| Otwarcie katalogu | Wynik otwarcia | Utrwalenie |
|---|---|---|
| odczyt, bez flagi | odmowa dostępu | – |
| zapis, bez flagi | odmowa dostępu | – |
| odczyt z flagą | otwarty | odmowa dostępu |
| zapis z flagą | otwarty | udane |
| Node.js, `open(dir, 'r')` | otwarty | `EPERM` |
| Node.js, `open(dir, 'r+')` | otwarty | udane |

Eksperyment pokazuje tylko, że wywołanie się udaje. Czy NTFS utrwala przy nim zmianę nazwy przed późniejszymi zapisami, można sprawdzić jedynie odcięciem zasilania.

## Skutek

Oba magazyny utrwalają katalog po zmianie nazwy snapshotu, przed obcięciem WAL, a także po utworzeniu pliku WAL. Kroki checkpointu są utrwalane zawsze. Opis protokołu: [decyzja 0018](../adr/0018-checkpoint.md).
