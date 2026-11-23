# Zrobione: praca inżynierska

Zadania wykonane w ramach pracy inżynierskiej. Zakres pracy to warstwa lokalnego zapisu danych, benchmarki i sprawdzenie pracy offline, a do tego optymalizacje, które poprawiają wynik Flat-SQL w jego niszy albo czynią porównanie z innymi silnikami uczciwym. Bieżące zadania: [do zrobienia](todo.md), dalszy rozwój: [na później](future.md).

Rozmiar zadania: `[S]` pojedyncze funkcje i testy, `[M]` kilka modułów, nowe testy i pomiary, `[L]` przebudowa podsystemu.

## Poprawność zapisu i synchronizacji

Każde zadanie skończyło się testem. Opis testów: [testy](../testing/README.md).

- [S] Granica dryfu zegara: urządzenie z zegarem przesuniętym o ponad 60 s wstrzymuje synchronizację, a po poprawie zegara jego zmiany dostają nowe znaczniki ([decyzja 0020](../adr/0020-granica-dryfu-zegara.md)).
- [S] Kolejność utrwalania przy checkpoincie na plikach: utrwalenie katalogu przed obcięciem WAL, także w Windows ([decyzja 0018](../adr/0018-checkpoint.md)).
- [S] Utrwalanie zapisów w magazynie plikowym Node.js, z możliwością wyłączenia dla zatwierdzeń.
- [S] Tryb trwałości IndexedDB wybierany w `FlatDB.open`.
- [S] Prośba o trwały magazyn przy otwarciu bazy, żeby przeglądarka nie usunęła niewysłanych zmian.
- [M] Test wszystkich stanów dysku dopuszczonych po awarii checkpointu na plikach.
- [S] Jawne konflikty schematu zamiast cichego odrzucania wierszy ([decyzja 0023](../adr/0023-struktura-tabel-w-kodzie.md)).
- [S] Jeden zapis bitowy NaN, żeby te same dane dawały te same bajty.
- [M] Test zbieżności przy przesuniętych zegarach, podziałach sieci i restartach.
- [S] IndexedDB jako domyślny magazyn w przeglądarce, błąd bez magazynu w Node.js ([decyzja 0008](../adr/0008-magazyn-domyslny.md)).

## Pomiary

Metodyka: [decyzja 0026](../adr/0026-metodyka-pomiarow.md). Wyniki: [benchmarki](../research/benchmark-results.md).

- [L] Szeroki zestaw silników odniesienia: 23 konfiguracje w grupach „pamięć”, „trwały zapis”, „Worker” i „natywnie”.
- [S] Statystyka: 10 procesów na pomiar, mediana z kwartylami, przedziały ufności.
- [S] Zimny start, zużycie pamięci, suma kontrolna wyników.
- [S] Koszt trybu `strict` w IndexedDB, pauza checkpointu, koszt Web Workera, koszt utrwalania plików w Windows i Linuksie.

## Optymalizacje

Każda optymalizacja była mierzona przed zmianą i po niej. Wyniki: [optymalizacje](../research/optimization-results.md).

- [S] `COUNT(*)` po indeksie bez czytania wierszy.
- [S] Pomijanie warunku, który gwarantuje ścieżka dostępu.
- [M] Plan zapamiętany w instrukcji przygotowanej ([decyzja 0012](../adr/0012-plan-kompilowany-w-instrukcji.md)).
- [M] Warunki kompilowane przy przygotowaniu instrukcji.
- [M] Kolejka zmian prowadzona dopiero od włączenia synchronizacji ([decyzja 0021](../adr/0021-kolejka-zmian.md)).
- [M] `UPDATE` w miejscu, jeden odczyt zegara na instrukcję ([decyzja 0013](../adr/0013-sciezka-zapisu.md)).
- [S] Hurtowa budowa indeksów przy odtwarzaniu bazy.
- [M] Grupowanie przez tablicę mieszającą.
- [S] Magazyn plikowy Node.js z plikiem WAL otwartym między checkpointami.
- [S] Profil kosztu granicy JavaScript–WebAssembly.
- [S] Obiekt wyniku tworzony w JavaScripcie.
- [M] Wynik w jednym buforze binarnym ([decyzja 0015](../adr/0015-wynik-binarny-dla-js.md)).
- [M] Wiele wierszy jednym wywołaniem (`executeMany`).
- [L] Krótkie teksty w samej wartości zamiast planowanych wierszy w ciągłych blokach ([decyzja 0014](../adr/0014-krotkie-teksty-w-wartosci.md)).
- [M] Checkpoint kodowany krokami ([decyzja 0018](../adr/0018-checkpoint.md)).
- [S] Porównanie ustawień kompilacji; konfiguracja bez zmian ([decyzja 0009](../adr/0009-ustawienia-kompilacji.md)).
- [M] Ponowny pomiar obu wersji tym samym narzędziem (Benchmark 2).
