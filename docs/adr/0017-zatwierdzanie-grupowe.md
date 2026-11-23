# 0017. Zatwierdzenia z jednej chwili zapisywane na dysk jednym zapisem

- Status: obowiązuje
- Ostatnia zmiana: 2026-10-23

## Kontekst

`FlatDB.exec()` zwraca wynik synchronicznie, a zapis do IndexedDB jest asynchroniczny. Osobny zapis na dysk dla każdej transakcji byłby kosztowny przy wielu drobnych zmianach, na przykład gdy gra zapisuje stan w każdej klatce.

## Decyzja

Gdy transakcja dała nowe bajty WAL, `FlatDB` planuje zapis w najbliższym mikrozadaniu (`queueMicrotask`). Wszystkie zatwierdzenia z jednego obrotu pętli zdarzeń trafiają do IndexedDB jednym zapisem. `await db.flush()` jest punktem trwałości: wszystko zatwierdzone do tej chwili jest zapisane. Operacje na magazynie idą jedną kolejką. Gwarancja odpowiada ustawieniu `synchronous=NORMAL` w SQLite.

## Uzasadnienie

- Jeden zapis logu dla grupy transakcji (group commit) to klasyczna technika baz w pamięci (DeWitt i in., 1984).
- Zmierzone: trwałe zatwierdzenie jednego wiersza 0,36 ms, a 1000 zatwierdzeń z jednym oczekiwaniem na trwałość 5,1 µs na zatwierdzenie.
- Aplikacja nie czeka na dysk przy zapisie. Gdy potrzebuje gwarancji, wywołuje `await flush()`.

## Odrzucone warianty

- Zapis na dysk przy każdym zatwierdzeniu, z oczekiwaniem. Wymusza asynchroniczne API zapisu.
- Zapis tylko na żądanie aplikacji. Łatwo zgubić dane, gdy aplikacja zapomni go wywołać.

## Konsekwencje

- Zatwierdzenia z ostatnich milisekund mogą zginąć przy nagłym zamknięciu karty, ale baza zawsze wraca do spójnego stanu.
- Gwarancja po `await flush()` zależy od trybu IndexedDB. Domyślny tryb Chrome (`relaxed`) przetrwa awarię karty i przeglądarki, ale nie systemu. `FlatDB.open({ durability: 'strict' })` czeka na dysk, co kosztuje około 1 ms na zatwierdzenie zamiast 0,4 ms.

## Historia

- 2026-10-06: zatwierdzanie grupowe w pierwszej wersji mostu JavaScript.
- 2026-10-23: opcja `durability` w `FlatDB.open`.
