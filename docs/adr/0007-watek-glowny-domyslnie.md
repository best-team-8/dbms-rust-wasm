# 0007. Silnik w głównym wątku strony, Web Worker jako opcja

- Status: obowiązuje
- Ostatnia zmiana: 2026-10-24

## Kontekst

Przeglądarka wykonuje kod strony w jednym wątku, który także rysuje interfejs. Bazę można uruchomić w tym wątku albo w Web Workerze, czyli osobnym wątku, z którym rozmawia się wiadomościami. Konkurencyjne silniki z trwałym zapisem (SQLite na prywatnym systemie plików, DuckDB-Wasm) muszą działać w Workerze, bo potrzebują synchronicznego dostępu do plików. Wielowątkowy WebAssembly wymaga dodatkowo nagłówków serwera włączających izolację strony (COOP i COEP).

## Decyzja

Flat-SQL domyślnie działa w wątku, z którego jest wywoływany, z synchronicznym API: `db.exec()` od razu zwraca wynik. Nie wymaga Workera ani nagłówków izolacji. Uruchomienie w Workerze jest dozwolone jako opcja. Wielowątkowość, która wymaga nagłówków izolacji, jest wykluczona.

## Uzasadnienie

- Wynik od razu. Zapytanie jest wywołaniem funkcji, bez `await` i bez wiadomości między wątkami. Gra może odczytać dane w środku klatki. Z tego samego powodu Riffle wykonuje zapytania synchronicznie w wątku interfejsu (Litt i in., 2023).
- Osadzanie bez przebudowy aplikacji: jeden import, bez osobnego pliku workera i bez nagłówków serwera. Działa na zwykłym hostingu plików, w rozszerzeniach i widżetach.
- Jest to możliwe, bo dane są w pamięci (decyzja 0006), a zapis na dysk jest asynchroniczny (decyzja 0017).
- Pomiar kosztu Workera: każde wywołanie kosztuje dodatkowo 56 µs, a pobranie 10 000 wierszy dodatkowo 11 ms. Przy odczycie po kluczu, który w wątku głównym trwa kilka mikrosekund, to kilkanaście razy więcej.

## Odrzucone warianty

- Worker jako wymóg. Interfejs nigdy by się nie zacinał, ale API stałoby się asynchroniczne, każde zapytanie płaciłoby za przesłanie między wątkami, a osadzanie wymagałoby przebudowy aplikacji. Flat-SQL straciłby wyróżnik wobec SQLite.
- Wątki WebAssembly. Wymagają nagłówków izolacji, które blokują zasoby z innych domen i wymagają kontroli nad serwerem.

## Konsekwencje

- Każda długa operacja blokuje interfejs, więc trzeba ją skracać, a nie przenosić. Dlatego checkpoint jest dzielony na kroki (decyzja 0018).
- Obsługę wielu kart trzeba dobudować. Naturalną drogą jest `SharedWorker`, czyli Worker jako opcja.

## Historia

- 2026-10-06: tryb domyślny od pierwszej wersji mostu JavaScript.
- 2026-10-24: zmierzono koszt Workera; wątek główny zostaje domyślny.
