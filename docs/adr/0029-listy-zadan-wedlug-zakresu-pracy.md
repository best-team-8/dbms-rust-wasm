# ADR-0029: Listy zadań według zakresu pracy inżynierskiej

| | |
|---|---|
| Status | przyjęta; podział zadań na wymagane i dodatkowe (pkt 1) zastąpiony przez [ADR-0039](0039-zadania-optymalizacji-z-rozbicia-kosztow.md) |
| Data | 2026-10-21 |
| Rodzaj | proces |
| Powiązane | ADR-0020, ADR-0021, ADR-0022 (zastępuje); [todo.md](../todo.md), [future/todo.md](../future/todo.md) |

## Kontekst

ADR-0022 podzielił zadania na optymalizacje istniejącego prototypu ([todo.md](../todo.md)) i rozszerzenia do dojrzałości ([future/todo.md](../future/todo.md)). Po przeglądzie okazało się, że `todo.md` jest za duże jak na pracę inżynierską: zawiera też optymalizacje, które nie wpływają ani na wyniki benchmarków, ani na punkty karty pracy (np. semantyka konfliktów w synchronizacji, trwały log serwera).

Karta pracy wymaga trzech rzeczy: analizy porównawczej silników WASM ze scenariuszem badawczym, implementacji warstwy lokalnego składowania oraz benchmarków z walidacją architektury Local-First w warunkach offline. Teza pracy: Flat-SQL jest konkurencyjny w swojej niszy – mała baza SQL na dane operacyjne aplikacji w przeglądarce, z przyrostową trwałością i wbudowaną synchronizacją ([use-cases.md](../research/use-cases.md), [literature.md](../references/literature.md)).

## Decyzja

1. **`todo.md` to zakres pracy inżynierskiej.** Zadanie jest w nim, jeśli poprawia wynik Flat-SQL w metrykach niszy (odczyt po kluczu, trwały zapis, rozmiar modułu, zimny start, typowe zapisy), czyni porównanie z innymi silnikami uczciwym i wiarygodnym albo realizuje punkt karty pracy. Zadania bez dopisku są wymagane; dopisek „dobrze mieć” oznacza zadania wykonywane, jeśli starczy czasu.
2. **`todo_future.md` to wszystko po pracy inżynierskiej:** nowe możliwości oraz optymalizacje prototypu odłożone na później (§12).
3. Przeniesione zadania dostają nowe numery w `todo_future.md`; dawne numery z `todo.md` nie są używane ponownie, a `todo.md` §6 prowadzi tabelę przeniesień.

Przeniesione 21.10.2026: todo 0.3, 0.4, 3.1–3.5, 4.5, 4.8, 4.11 → future 12.1–12.10. Zadanie todo 2.7 (zwarta reprezentacja wierszy) zostaje w `todo.md` jako „dobrze mieć”, bo poprawiłoby wyniki snapshotu, odtwarzania i pamięci w benchmarkach. Dodano todo 2.8 (szybszy `UPDATE`).

## Uzasadnienie

- **Zakres zgodny z kartą i tezą.** Wynik pracy zależy od silnika i jego porównania z innymi rozwiązaniami; synchronizacja jest potrzebna w zakresie walidacji pracy offline, którą obecny prototyp zapewnia (test scenariusza offline, losowy test zbieżności replik).
- **Szeroki zestaw punktów odniesienia (todo 4.1) jest wymagany w całości** – teza „najlepszy w niszy” wymaga porównania z silnikami działającymi w tych samych warunkach i z granicami niszy (ADR-0020).
- **Przeniesione zadania nie zmieniają wyników pracy:** semantyka konfliktów (LWW per kolumna, konflikty klucza, granice transakcji, tombstone'y) nie jest mierzona w benchmarkach, a jej ograniczenia są opisane w literaturze ([C9], [L27], [L28]); trwały log serwera wpływa na wydajność synchronizacji po restarcie, nie na poprawność; YCSB i dokładniejszy zegar przeglądarki poprawiają metodykę w niewielkim stopniu dla silnika w pamięci; koszt LWW per kolumna nie ma czego mierzyć bez LWW per kolumna.

## Odrzucone alternatywy

- **Zachowanie podziału z ADR-0022 („istniejące a nowe”)** – miesza zakres pracy z dalszym rozwojem i utrudnia uzgodnienie zakresu z promotorem.
- **Jedna lista z poziomami priorytetu** – zadania po pracy i tak wymagają osobnego planu razem z rozszerzeniami.

## Konsekwencje

- Odnośniki do przeniesionych zadań w ADR i dokumentach wskazują `todo_future.md` (np. „future 12.4” zamiast „todo 3.2”).
- Konsekwencja ADR-0022 („zmiana działania istniejącej funkcji bez nowej składni trafia do `todo.md`”) przestaje obowiązywać – o miejscu decyduje zakres pracy.
- Kolejność w `todo.md`: sekcja 1 → 2.1, 2.2, 2.8 → 2.3–2.5 → sekcja 4 → zadania „dobrze mieć”; sekcja 0 równolegle.

## Źródła

Karta pracy; [use-cases.md](../research/use-cases.md); [literature.md](../references/literature.md).
