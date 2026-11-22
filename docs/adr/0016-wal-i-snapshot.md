# 0016. Trwałość przez dziennik zapisu i snapshot we własnym, przenośnym formacie

- Status: obowiązuje
- Ostatnia zmiana: 2026-10-25

## Kontekst

Dane w pamięci (decyzja 0006) muszą przetrwać zamknięcie aplikacji i awarię. Koszt trwałego zatwierdzenia nie może rosnąć z rozmiarem bazy, jak w sql.js, który przy każdym zapisie eksportuje cały plik. Pliki muszą być czytelne w każdym środowisku, w którym działa rdzeń, i po aktualizacji silnika.

## Decyzja

1. Dziennik zapisu z wyprzedzeniem (WAL) jest tylko dopisywany. Każda ramka ma numer kolejny, liczbę operacji i sumę kontrolną CRC32. Operacje opisują wynik, a nie polecenie: „zapisz ten wiersz” albo „usuń ten klucz”. Powtórzenie operacji daje ten sam stan.
2. Snapshot to pełny obraz zatwierdzonego stanu z sumą kontrolną. Powstaje przy checkpoincie, gdy WAL przekroczy próg, a potem WAL jest czyszczony.
3. Odtworzenie bazy to snapshot plus ramki WAL o numerze większym niż w snapshocie. Odczyt WAL kończy się na pierwszej ramce urwanej albo z błędną sumą.
4. Snapshot, ramki WAL i paczki synchronizacji mają własny, opisany format binarny z nagłówkiem, numerem wersji formatu i sumą kontrolną. Ten sam katalog danych (`snapshot.fsql`, `wal.fsql`) otwiera serwer w Node.js i narzędzie natywne.

## Uzasadnienie

- Koszt zatwierdzenia zależy od zmiany, a nie od bazy: około 0,36 ms w IndexedDB niezależnie od jej rozmiaru.
- Log opisujący wynik zamiast polecenia jest wolniejszy od logowania poleceń (Malviya i in., 2014), ale te same wpisy służą synchronizacji, a odtworzenie nie zależy od zegara ani od wyników zapytań.
- Niezatwierdzone zmiany nie trafiają do WAL (decyzja 0011), więc odtworzenie niczego nie cofa. Model „log i checkpoint” dla baz w pamięci opisał DeWitt i in. (1984).
- Numer wersji formatu pozwala czytać stare dane po aktualizacji silnika, bo dane żyją dłużej niż kod (Kleppmann, 2017, rozdz. 4).
- Test ucinający WAL w każdym bajcie sprawdza, że baza zawsze wraca do stanu po ostatniej pełnej transakcji.

## Odrzucone warianty

- Eksport całej bazy przy zatwierdzeniu, jak w sql.js. Koszt rośnie z bazą.
- Tylko snapshot na żądanie, jak w Zeta-Lite. Zmiany po ostatnim snapshocie giną.
- Snapshot jako zrzut pamięci modułu. Zawiera wskaźniki 32-bitowe, nieczytelne natywnie, i zmienia się przy każdej kompilacji.
- Gotowy format, na przykład Protocol Buffers. Zależność zewnętrzna (decyzja 0005).
- Odtwarzanie równoległe. Wymaga wątków, czyli nagłówków izolacji strony (decyzja 0007).

## Konsekwencje

- Odtworzenie wymaga zdekodowania i wstawienia wszystkich wierszy: 46 ms dla 100 000 wierszy w przeglądarce, wobec około 1 ms w SQLite, który kopiuje strony.
- Protokół checkpointu na plikach wymaga utrwalenia katalogu przed obcięciem WAL (decyzja 0018).

## Historia

- 2026-10-06: format i protokół pierwszej wersji silnika.
- 2026-10-25: snapshot w formacie 2 z flagą synchronizacji i zwartą kolejką zmian (decyzja 0021); format 1 nadal jest czytany.
