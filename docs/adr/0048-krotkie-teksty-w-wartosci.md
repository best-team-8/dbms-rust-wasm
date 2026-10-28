# ADR-0048: Krótkie teksty przechowywane w wartości, wiersze bez bloków ciągłych

| | |
|---|---|
| Status | przyjęta |
| Data | 2026-10-28 |
| Rodzaj | architektura |
| Powiązane | ADR-0011, ADR-0014, ADR-0039; zadanie todo 2.7 |

## Kontekst

Todo 2.7 zakładało wiersze w ciągłych blokach zamiast osobnych alokacji drzewa: pamięć liniowa po wczytaniu 100 000 wierszy 42 MiB w Benchmarku 1 (54,8 MiB w wersji po todo 2.14 przy obciążeniu z drugą tabelą), odtworzenie 116 ms. Licznik alokacji pokazał, że odtworzenie 100 000 wierszy wykonuje 319 540 alokacji: wektor wartości wiersza i osobna alokacja każdego tekstu (dwa teksty na wiersz w obciążeniu, `c42` i `item-12345`). `Value::Text` trzymało `String`, więc każdy tekst, także dwuznakowy, był osobnym obiektem na stercie.

## Decyzja

1. `Value::Text` trzyma typ `Text`: tekst do 22 bajtów UTF-8 (14 na wasm32) w samej wartości, dłuższy w jednej alokacji (`Box<str>`). `Value` zachowuje rozmiar (24 B, na wasm32 16 B). Postać zależy tylko od długości, więc równość, porządek i skrót są tekstu; jedyny `unsafe` (odczyt bajtów zapisanych w miejscu jako `str`) opiera się na niezmienniku konstruktora.
2. Porównanie tekstów (`Ord for Text`) nie jest rozwijane w miejscu wywołania (`#[inline(never)]`).
3. Wiersz pozostaje wektorem wartości w drzewie wierszy; bez bloków ciągłych i bez wierszy kodowanych jako bajty.

## Uzasadnienie

- Doświadczenie z układami tych samych 100 000 wierszy na wasm32 (`layout-wasm32.txt`): dzisiejszy układ 13,5 MiB żądanej pamięci, 309 095 alokacji, budowa 9,0 ms, pełny przegląd 1,43 ms; teksty w wartości 12,3 MiB, 109 095 alokacji, 7,0 ms, 1,17 ms; wartości w jednym bloku na tabelę z numerem wiersza w drzewie 16,1 MiB, 24,8 ms, 2,18 ms; wiersze kodowane jako bajty 8,1 MiB, ale przegląd 2,73 ms. Natywnie te same proporcje.
- Pomiar zmiany ([optimization-results.md §14](../research/optimization-results.md#14-krótkie-teksty-w-wartości)): pamięć liniowa po wczytaniu 54,8 → 51,8 MiB, alokacje odtworzenia 319 540 → 119 440, w przeglądarce zakres klucza 51,2 → 45,4 µs (−11%), grupowanie 10,9 → 9,36 ms (−14%), snapshot 24,3 → 21,9 ms (−10%), odtworzenie 49,1 → 46,3 ms (−6%).
- Pierwsza wersja z porównaniem tekstów rozwijanym w miejscu spowalniała wstawianie o 11–15% (natywnie i w WASM); bez rozwijania wstawianie wróciło do czasu sprzed zmiany.

## Odrzucone alternatywy

- **Wartości w ciągłym bloku na tabelę z numerem wiersza w drzewie (kierunek z todo 2.7)** – w doświadczeniu więcej pamięci i wolniejsza budowa (na wasm32 24,8 wobec 9,0 ms), bo teksty nadal są osobnymi alokacjami, a blok rośnie przez kopiowanie.
- **Wiersze kodowane jako bajty** – o 40–50% mniej pamięci, ale przegląd wolniejszy o 54–88%, bo każda wartość jest dekodowana przy odczycie, i przebudowa wykonawcy (wyrażenia działają na `&[Value]`).
- **`Box<[Value]>` zamiast `Vec<Value>`** – 0,7 MiB mniej na 100 000 wierszy, bez zmiany liczby alokacji; zysk zbyt mały wobec zmian w zapisie w miejscu.
- **Porównanie tekstów rozwijane w miejscu** – spowolnienie wstawiania opisane wyżej.
- **memory64** – nie dotyczy rozmiaru danych tej pracy, a kosztuje wydajność [W6], [Q6].

## Konsekwencje

- Natywnie top-k (sortowanie po liczbach) wolniejszy o 18% i grupowanie o 8%, a budowa indeksu dłuższa o 1,5–5 ms; przyczyny nie ustalono (sprawdzone: rozwijanie porównania i równości tekstów, wskazówki `#[inline]`, rozmiary typów, kod maszynowy `Value::cmp`). W WebAssembly te operacje nie zwolniły.
- Moduł WebAssembly większy o 4,2 KiB po kompresji gzip.
- Konstrukcja tekstu z `String` przez `.into()`; kod spoza rdzenia (CLI, narzędzia pomiarowe) działa ze starszymi wersjami bez zmian.
- Poprawność: [tests-results.md §3.10](../research/tests-results.md#310-krótkie-teksty-w-wartości).

## Źródła

[W6], [Q6], [W11]; [optimization-results.md §14](../research/optimization-results.md#14-krótkie-teksty-w-wartości).
