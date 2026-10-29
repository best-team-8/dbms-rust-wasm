# ADR-0056: Pomiar na telefonie poza zakresem pracy inżynierskiej

| | |
|---|---|
| Status | przyjęta |
| Data | 2026-10-29 |
| Rodzaj | proces |
| Powiązane | ADR-0029, ADR-0037, ADR-0038, ADR-0039 (pkt 4 w części dotyczącej zadania 4.7); zadanie [future/todo.md 1.6](../future/todo.md) (dawne todo 4.7) |

## Kontekst

Zadanie 4.7 z [todo.md](../todo.md) przewidywało powtórzenie pomiarów H1–H5 na telefonie, z osobnym czasem otwarcia bazy z IndexedDB, bo na wolnych urządzeniach lokalny odczyt bywa wolniejszy od sieci ([use-cases.md](../research/use-cases.md#4-wnioski-projektowe-z-cudzych-wdrożeń)). Narzędzie jest gotowe: tryb ręczny `bench/browser/serve.js` z `adb reverse` ([bench/README.md](../../bench/README.md)). ADR-0039 (pkt 4) uznał wszystkie zadania `todo.md`, także z sekcji 4, za wymagane.

Benchmark 1 (24.10.2026) i Benchmark 2 (28.10.2026) wykonano na jednym laptopie, bez podłączonego telefonu. Tekst pracy opisuje jedno urządzenie i jeden silnik JavaScript jako ograniczenie trafności zewnętrznej (podrozdziały 5.7 i 9.4).

## Decyzja

1. Zadanie 4.7 przechodzi do [future/todo.md](../future/todo.md) jako 1.6 „Pomiar na telefonie”. Numer 4.7 w `todo.md` nie jest używany ponownie; tabela w §6 `todo.md` wskazuje nowe miejsce.
2. Praca opisuje brak pomiaru na urządzeniu mobilnym jako ograniczenie, tak jak dotąd.

## Uzasadnienie

- **Zakres pracy** (ADR-0029): `todo.md` zawiera zadania, które poprawiają wynik Flat-SQL w metrykach jego niszy, czynią porównanie z innymi silnikami uczciwym albo realizują punkty karty pracy. Punkty karty (warstwa lokalnego składowania, benchmarki, walidacja offline) realizują Benchmark 1 i 2 na laptopie oraz testy z [tests-results.md](../research/tests-results.md). Pomiar na telefonie żadnego z nich nie zmienia.
- **Wnioski z hipotez dotyczą zmierzonego środowiska.** H1–H5 porównują silniki i wersje na tym samym urządzeniu. Telefon dałby wartości dla innej klasy urządzeń i mógłby zmienić proporcje między silnikami, ale nie zmienia werdyktów dla laptopa, na którym zmierzono obie wersje. Metodyka ([benchmark-scenario.md §7.2](../research/benchmark-scenario.md#72-trafność-zewnętrzna)) już ogranicza wnioski do zmierzonego środowiska.
- **Wymaga urządzenia, którego nie było przy obu pomiarach.** Pomiar tylko jednej wersji na telefonie, bez Benchmarku 1 na tym samym urządzeniu, nie dałby porównania prototypu z wersją po optymalizacjach w tych samych warunkach (ADR-0037, ADR-0038).

## Odrzucone alternatywy

- **Pomiar przed oddaniem pracy** – wymagałby powtórzenia pomiarów obu wersji z tagów na telefonie i dopisania wyników do rozdziałów 6 i 7, przy braku wpływu na werdykty hipotez.
- **Pomiar tylko czasu otwarcia bazy na telefonie** – jedna liczba z innego urządzenia bez odniesienia do pozostałych silników; zostaje częścią zadania 1.6.

## Konsekwencje

- `todo.md` nie ma otwartych zadań sekcji 4; ograniczenie do jednego urządzenia zostaje w tekście pracy.
- Zadanie [future/todo.md 1.2](../future/todo.md) (pamięć podręczna danych serwera) zależy teraz od pomiaru z 1.6.

## Źródła

- [use-cases.md §4](../research/use-cases.md#4-wnioski-projektowe-z-cudzych-wdrożeń) – wnioski z cudzych wdrożeń (Notion: odczyt lokalny „ścigający się” z API).
- [benchmark-scenario.md §7.2](../research/benchmark-scenario.md#72-trafność-zewnętrzna) – trafność zewnętrzna.
