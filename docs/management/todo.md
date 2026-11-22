# Lista zadań: praca inżynierska

Zakres pracy inżynierskiej: warstwa lokalnego zapisu danych, benchmarki i sprawdzenie pracy offline, a do tego optymalizacje, które poprawiają wynik Flat-SQL w jego niszy albo czynią porównanie z innymi silnikami uczciwym. Zakres jest zrobiony poza dwoma zadaniami porządkowymi. Dalszy rozwój: [na później](future.md).

Rozmiar zadania: `[S]` pojedyncze funkcje i testy, `[M]` kilka modułów, nowe testy i pomiary, `[L]` przebudowa podsystemu.

## Do zrobienia

### [S] Tezy w README i prezentacji

Doprecyzować hasła z pierwotnego opisu projektu:

- „0 ms latencji” to brak opóźnienia sieci, a nie zerowy czas wykonania;
- „Privacy by Design” to dane domyślnie lokalne, ale nieszyfrowane; izolacja WebAssembly chroni przeglądarkę, a nie dane;
- „szybszy niż sql.js” we wszystkich odczytach, ale wolniejszy w zapisach i odtwarzaniu bazy;
- transakcja jest atomowa lokalnie, a między urządzeniami atomowy jest wiersz;
- dane po `await flush()` przetrwają awarię karty i przeglądarki, ale nie systemu (w trybie domyślnym);
- „przewidywalne wykonanie” to plan zawsze jednotabelowy i widoczny, a ograniczony koszt mają tylko odczyty po kluczu lub indeksie;
- „tryb edge” to dziś jeden serwer; węzły edge są planowane.

### [S] Zastosowania w README i prezentacji

Analizę pliku CSV 500 MB przypisać DuckDB-Wasm. Przy portfelach kryptowalut wykluczyć przechowywanie kluczy. Sesje na edge opisać jako ryzykowne, bo spójność jest tylko ostateczna. Dodać zastosowania potwierdzone analizą wdrożeń: zbieranie danych w terenie, aplikacje osobiste na wielu urządzeniach, kopia danych serwera.

## Zrobione

### Poprawność zapisu i synchronizacji

Każde zadanie skończyło się testem. Opis testów: [testy](../testing/README.md).

- [x] [S] Granica dryfu zegara: urządzenie z zegarem przesuniętym o ponad 60 s wstrzymuje synchronizację, a po poprawie zegara jego zmiany dostają nowe znaczniki ([decyzja 0020](../adr/0020-granica-dryfu-zegara.md)).
- [x] [S] Kolejność utrwalania przy checkpoincie na plikach: utrwalenie katalogu przed obcięciem WAL, także w Windows ([decyzja 0018](../adr/0018-checkpoint.md)).
- [x] [S] Utrwalanie zapisów w magazynie plikowym Node.js, z możliwością wyłączenia dla zatwierdzeń.
- [x] [S] Tryb trwałości IndexedDB wybierany w `FlatDB.open`.
- [x] [S] Prośba o trwały magazyn przy otwarciu bazy, żeby przeglądarka nie usunęła niewysłanych zmian.
- [x] [M] Test wszystkich stanów dysku dopuszczonych po awarii checkpointu na plikach.
- [x] [S] Jawne konflikty schematu zamiast cichego odrzucania wierszy ([decyzja 0023](../adr/0023-struktura-tabel-w-kodzie.md)).
- [x] [S] Jeden zapis bitowy NaN, żeby te same dane dawały te same bajty.
- [x] [M] Test zbieżności przy przesuniętych zegarach, podziałach sieci i restartach.
- [x] [S] IndexedDB jako domyślny magazyn w przeglądarce, błąd bez magazynu w Node.js ([decyzja 0008](../adr/0008-magazyn-domyslny.md)).

### Pomiary

Metodyka: [decyzja 0026](../adr/0026-metodyka-pomiarow.md). Wyniki: [benchmarki](../research/benchmark-results.md).

- [x] [L] Szeroki zestaw silników odniesienia: 23 konfiguracje w grupach „pamięć”, „trwały zapis”, „Worker” i „natywnie”.
- [x] [S] Statystyka: 10 procesów na pomiar, mediana z kwartylami, przedziały ufności.
- [x] [S] Zimny start, zużycie pamięci, suma kontrolna wyników.
- [x] [S] Koszt trybu `strict` w IndexedDB, pauza checkpointu, koszt Web Workera, koszt utrwalania plików w Windows i Linuksie.

### Optymalizacje

Każda optymalizacja była mierzona przed zmianą i po niej. Wyniki: [optymalizacje](../research/optimization-results.md).

- [x] [S] `COUNT(*)` po indeksie bez czytania wierszy.
- [x] [S] Pomijanie warunku, który gwarantuje ścieżka dostępu.
- [x] [M] Plan zapamiętany w instrukcji przygotowanej ([decyzja 0012](../adr/0012-plan-kompilowany-w-instrukcji.md)).
- [x] [M] Warunki kompilowane przy przygotowaniu instrukcji.
- [x] [M] Kolejka zmian prowadzona dopiero od włączenia synchronizacji ([decyzja 0021](../adr/0021-kolejka-zmian.md)).
- [x] [M] `UPDATE` w miejscu, jeden odczyt zegara na instrukcję ([decyzja 0013](../adr/0013-sciezka-zapisu.md)).
- [x] [S] Hurtowa budowa indeksów przy odtwarzaniu bazy.
- [x] [M] Grupowanie przez tablicę mieszającą.
- [x] [S] Magazyn plikowy Node.js z plikiem WAL otwartym między checkpointami.
- [x] [S] Profil kosztu granicy JavaScript–WebAssembly.
- [x] [S] Obiekt wyniku tworzony w JavaScripcie.
- [x] [M] Wynik w jednym buforze binarnym ([decyzja 0015](../adr/0015-wynik-binarny-dla-js.md)).
- [x] [M] Wiele wierszy jednym wywołaniem (`executeMany`).
- [x] [L] Krótkie teksty w samej wartości zamiast planowanych wierszy w ciągłych blokach ([decyzja 0014](../adr/0014-krotkie-teksty-w-wartosci.md)).
- [x] [M] Checkpoint kodowany krokami ([decyzja 0018](../adr/0018-checkpoint.md)).
- [x] [S] Porównanie ustawień kompilacji; konfiguracja bez zmian ([decyzja 0009](../adr/0009-ustawienia-kompilacji.md)).
- [x] [M] Ponowny pomiar obu wersji tym samym narzędziem (Benchmark 2).
