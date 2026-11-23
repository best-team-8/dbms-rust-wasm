# 0014. Krótkie teksty przechowywane w samej wartości

- Status: obowiązuje
- Ostatnia zmiana: 2026-10-28

## Kontekst

Baza 100 000 wierszy zajmowała ponad 50 MiB pamięci modułu, a odtworzenie wykonywało około 320 000 alokacji. Każdy tekst, także dwuznakowy, był osobnym obiektem na stercie. Pierwotny plan zakładał wiersze w ciągłych blokach pamięci zamiast osobnych alokacji.

## Decyzja

1. Tekst do 14 bajtów UTF-8 (22 w wersji natywnej) jest przechowywany w samej wartości, dłuższy w jednej alokacji. Rozmiar wartości się nie zmienia (16 B w WebAssembly). Równość, porządek i skrót zależą tylko od treści tekstu.
2. Porównanie tekstów nie jest rozwijane w miejscu wywołania (`#[inline(never)]`).
3. Wiersz zostaje wektorem wartości w drzewie. Bez bloków ciągłych i bez wierszy kodowanych jako bajty.
4. Odczyt bajtów tekstu zapisanego w wartości to jedyne użycie `unsafe` w rdzeniu. Opiera się na warunku sprawdzanym w konstruktorze.

## Uzasadnienie

- Doświadczenie z czterema układami tych samych 100 000 wierszy: teksty w wartości dały najmniej pamięci przy najszybszym przeglądzie. Bloki ciągłe zużyły więcej pamięci i budowały się prawie trzy razy dłużej. Wiersze kodowane jako bajty zajęły 40–50% mniej pamięci, ale przegląd był o 54–88% wolniejszy.
- Zmierzone: pamięć po wczytaniu 54,8 → 51,8 MiB, alokacje odtworzenia 319 540 → 119 440, w przeglądarce zakres klucza −11%, grupowanie −14%, snapshot −10%.
- Porównanie tekstów rozwijane w miejscu spowalniało wstawianie o 11–15%.

## Odrzucone warianty

- Wartości w ciągłym bloku na tabelę. Więcej pamięci i wolniejsza budowa, bo teksty i tak są osobnymi alokacjami.
- Wiersze kodowane jako bajty. Każdy odczyt musi dekodować wartość.
- memory64 (decyzja 0006).

## Konsekwencje

- Natywnie sortowanie po liczbach zwolniło o 18%, a grupowanie o 8%. Przyczyny nie ustalono. W WebAssembly te operacje nie zwolniły.
- Odtworzenie bazy wciąż jest wolniejsze niż w SQLite, bo trzeba zbudować drzewo z pojedynczych wierszy.

## Historia

- 2026-10-28: teksty w wartości zamiast planowanych bloków ciągłych.
