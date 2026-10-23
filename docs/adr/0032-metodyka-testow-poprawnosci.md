# ADR-0032: Metodyka testów poprawności oddzielna od metodyki benchmarków

| | |
|---|---|
| Status | przyjęta |
| Data | 2026-10-23 |
| Rodzaj | proces |
| Powiązane | ADR-0020, ADR-0029; zadania todo §1; [tests-methodology.md](../research/tests-methodology.md) |

## Kontekst

Zadania z [todo.md §1](../todo.md) poprawiają trwałość i synchronizację, a każde kończy się testem. Wyniki tych testów wejdą do pracy dyplomowej obok benchmarków (rozdziały 5 i 8 pracy). Dotąd metodykę miały tylko pomiary wydajności ([benchmark-scenario.md](../research/benchmark-scenario.md), ADR-0020). Testy poprawności były opisane tylko w §6 tego dokumentu: tabelą z nazwą testu i jednym zdaniem o tym, co weryfikuje, oraz listą kroków scenariusza end-to-end i ręcznego. Brakowało modelu awarii, wyroczni, kontroli negatywnej i ograniczeń, a bez nich wynik „test przechodzi” nie nadaje się do pracy.

## Decyzja

1. Testy poprawności mają własną, ogólną metodykę w [docs/research/tests-methodology.md](../research/tests-methodology.md): poziomy testów, zasady projektowania (model awarii, wyrocznia, determinizm, kontrola negatywna), techniki dla trwałości, synchronizacji i formatu danych oraz układ opisu testu.
2. Opisy testów i ich wyniki trafiają do `docs/research/tests-results.md`, pogrupowane według obszarów, z jednym opisem środowiska na początku.
3. Benchmarki zachowują osobną metodykę ([benchmark-scenario.md](../research/benchmark-scenario.md)); test poprawności nie mierzy kosztu.
4. Dokumenty w `docs/research/` nie odwołują się do rejestru ADR, bo wchodzą do tekstu pracy, a rejestr – nie. Rejestr odsyła do nich, one do rejestru nie.

## Uzasadnienie

- **Inne pytanie, inna powtarzalność.** Benchmark odpowiada „ile”, a wynik ma rozkład i wymaga wielu uruchomień [B1]. Test poprawności odpowiada „czy” i ma być deterministyczny. Wspólna metodyka mieszałaby kryteria zaliczenia z kryteriami statystycznymi.
- **Wynik musi mieć granice.** Testy trwałości symulują awarię według modelu systemu plików; bez jawnego modelu i ograniczeń zaliczony test sugeruje odporność na awarie, których nie sprawdzano [S8].
- **Kontrola negatywna jako dowód.** Test, który nie zawiódł na kodzie sprzed poprawki, nie pokazuje, że poprawka czegokolwiek dotyczy. W pracy to jedyny sposób, żeby wykazać, że test wykrywa opisany problem.
- **Ogólność.** Metodyka nie jest przypisana do §1: te same zasady obowiązują dla testów zapytań, formatu i późniejszych rozszerzeń.

## Odrzucone alternatywy

- **Osobny rejestr weryfikacji z plikiem na każdy test** (wzorem rejestru ADR) – drugi rejestr obok ADR, opisy rozproszone po wielu plikach i niedopasowane do układu rozdziałów pracy.
- **Rozszerzenie [benchmark-scenario.md](../research/benchmark-scenario.md)** – jeden dokument dla kosztu i poprawności miesza kryteria powtarzalności i rozrasta się poza rozdział metodyki pomiarów.
- **Opis tylko w kodzie testów i wiadomościach commitów** – nie da się go cytować, a uzasadnienie i ograniczenia giną.

## Konsekwencje

- Każde zadanie z §1 kończy się opisem testu według metodyki; pierwsze z nich zakłada `tests-results.md`.
- Koszt poprawek (np. dodatkowe utrwalanie plików) mierzy benchmark w ostatecznej wersji (todo §4), a nie zadania z §1.
- Do ustalenia: czy walidacja offline z [benchmark-scenario.md §6](../research/benchmark-scenario.md#6-walidacja-local-first-w-warunkach-offline-h6) przechodzi do dokumentów testów. Jej testy spełniają definicję testu poprawności, ale opisy nie mają dziś modelu awarii ani kontroli negatywnej.

## Źródła

[S8], [B1]; [benchmark-scenario.md](../research/benchmark-scenario.md), [tests-methodology.md](../research/tests-methodology.md).
