# 0018. Checkpoint kodowany krokami, a na plikach z utrwaleniem katalogu

- Status: obowiązuje
- Ostatnia zmiana: 2026-10-28

## Kontekst

Checkpoint zapisuje snapshot całej bazy i obcina WAL. W prototypie snapshot był kodowany jednym wywołaniem i blokował główny wątek strony na 40 ms przy 100 000 wierszy, czyli na 2–3 klatki obrazu. Na plikach checkpoint zmieniał nazwę pliku snapshotu i obcinał WAL. System plików może jednak utrwalić te operacje w innej kolejności, niż je wykonano. Test z symulowanym systemem plików znalazł przypadki utraty potwierdzonych transakcji.

## Decyzja

1. Rdzeń koduje snapshot krokami. Snapshot jest stanem z chwili rozpoczęcia. Przed zmianą wiersza, do którego kroki jeszcze nie doszły, zachowywana jest jego kopia (kopia przy zmianie). Zmiana schematu rozpoczyna snapshot od nowa.
2. `FlatDB` wykonuje checkpoint krokami po 4096 wierszy i oddaje sterowanie pętli zdarzeń między krokami. W IndexedDB snapshot jest zapisywany jako `Blob`, co zajmuje wątek krócej niż zapis tablicy bajtów.
3. Na plikach kolejność to: zapis pliku tymczasowego, jego utrwalenie (`fsync`), zmiana nazwy, utrwalenie katalogu, obcięcie WAL. Kroki checkpointu są utrwalane zawsze, także gdy aplikacja wyłączyła `fsync` po zatwierdzeniach.
4. W Windows katalog jest otwierany z prawem zapisu i flagą `FILE_FLAG_BACKUP_SEMANTICS`, bo tylko wtedy da się go utrwalić. Wystarcza biblioteka standardowa Rusta.

## Uzasadnienie

- Naiwny snapshot całego stanu wstrzymuje aplikację na czas rosnący z rozmiarem stanu. Kopia przy zmianie daje pauzę prawie stałą kosztem pamięci (Cao i in., 2011).
- Zmierzone: najdłuższa pauza przy 100 000 wierszy 40,3 → 2,6 ms, na poziomie zakłóceń w bezczynności. Cały checkpoint trwa tyle samo.
- Brak utrwalenia katalogu po zmianie nazwy pliku to typowy błąd dojrzałych aplikacji (Pillai i in., 2014). Oba magazyny plikowe Flat-SQL miały ten błąd. Po poprawce test odtwarza stan zatwierdzony w każdym stanie dysku dopuszczonym przez model awarii.
- Koszt to jedno utrwalenie katalogu na checkpoint, a nie na zatwierdzenie.

## Odrzucone warianty

- Snapshot przyrostowy (tylko zmienione wiersze). Zmienia format odtwarzania i model awarii.
- Checkpoint w Web Workerze. Wymaga kopii bazy albo pamięci współdzielonej z nagłówkami izolacji (decyzja 0007).
- Blokowanie zapisów na czas checkpointu. API zapisu jest synchroniczne.
- Nowy plik WAL przy każdym checkpoincie. Utworzenie i usunięcie pliku to też operacje na katalogu, więc i tak trzeba go utrwalić.

## Konsekwencje

- Wiersze zmieniane w trakcie checkpointu zajmują pamięć kopii do jego końca.
- Bazy w IndexedDB zapisanej nową wersją nie odczyta wersja starsza, bo snapshot jest zapisany jako `Blob`.
- W Windows gwarancja zależy od tego, czy NTFS utrwala zmianę nazwy przy utrwaleniu katalogu. Nie da się tego sprawdzić bez odcięcia zasilania.

## Historia

- 2026-10-23: utrwalenie katalogu w checkpoincie na plikach.
- 2026-10-28: snapshot kodowany krokami zamiast jednym wywołaniem.
