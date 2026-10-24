# ADR-0039: Zadania optymalizacji z rozbicia kosztów silnika jako wymagane

| | |
|---|---|
| Status | przyjęta |
| Data | 2026-10-24 |
| Rodzaj | proces |
| Powiązane | ADR-0016, ADR-0019, ADR-0029, ADR-0037; zadania todo 2.4, 2.5, 2.7, 2.9–2.16 |

## Kontekst

Benchmark 1 ([benchmark-results.md §1](../research/benchmark-results.md#1-benchmark-1-prototyp-przed-optymalizacjami)) dał listę słabych stron prototypu S1–S11, a [hyphotetical-optimalizations.md](../hyphotetical-optimalizations.md) – hipotezy O1–O13 spoza zadań 2.1–2.8. Hipotezy miały trafić do listy zadań dopiero po potwierdzeniu pomiarem. Pomiar pomocniczy rozbicia kosztów silnika ([benchmark-results.md §1.12](../research/benchmark-results.md#112-rozbicie-kosztów-silnika-pomiar-pomocniczy)) pokazał trzy przyczyny, których plan nie obejmował:

- instrukcja przygotowana przechowuje tylko drzewo składni, więc każde wykonanie od nowa wiąże nazwy, wybiera ścieżkę dostępu i kopiuje nazwy kolumn; wyszukanie wiersza to 9% czasu odczytu po kluczu (2,3 µs natywnie);
- interpreter wyrażeń zajmuje około 64% czasu pełnego skanu (8,2 ms; ręczna pętla po tym samym drzewie 2,9 ms), więc hipoteza O13, według której głównym kierunkiem jest zwarta reprezentacja wierszy (2.7), była błędna;
- outbox z wpisem dla każdego lokalnie wstawionego wiersza, także bez synchronizacji, to około połowy czasu odtworzenia, jednej trzeciej czasu snapshotu i jednej czwartej jego rozmiaru.

## Decyzja

1. Do sekcji 2 [todo.md](../todo.md) dochodzą zadania 2.9–2.16 jako wymagane, bez dopisku „dobrze mieć”: 2.9 plan zapamiętany w instrukcji przygotowanej (z hipotezami O2, O3), 2.10 wyrażenia kompilowane przy przygotowaniu, 2.11 outbox tylko przy synchronizacji i w zwartej postaci (O7), 2.12 hurtowa budowa indeksów przy odtwarzaniu (część O9), 2.13 internowanie stałych łańcuchów wyniku (O1), 2.14 wstawianie wielu wierszy jednym wywołaniem (O4), 2.15 grupowanie bez alokacji klucza na wiersz (O11), 2.16 `FsStore` z otwartym plikiem WAL (O10).
2. Zadania 2.4, 2.5 i 2.7 dostają dopiski: 2.4 koduje wynik wprost z wierszy tabeli i osiąga cel dopiero z 2.9 i 2.13; 2.5 mierzy też alokator (dlmalloc wobec talc) i flagę `+simd128`; 2.7 idzie po 2.10.
3. Kolejność sekcji 2: 2.1, 2.2 → 2.9, 2.10, 2.11 → 2.8 → 2.12, 2.15, 2.16 → 2.3, 2.13 → 2.4 → 2.14 → 2.5. Zadania 2.6 i 2.7 pozostają „dobrze mieć”.
4. Hipotezy O5, O6, O8 i O12 pozostają hipotezami bez zadania.
5. Każde nowe zadanie jest oceniane jak pozostałe zadania sekcji 2: pomiar narzędziem z §4 razem z tagiem `prototyp` (ADR-0037).

## Uzasadnienie

- **Zakres pracy** (ADR-0029): zadania poprawiają wynik Flat-SQL w metrykach jego niszy (odczyt po kluczu, odczyt pojedynczych rekordów, zapisy, otwarcie bazy, pamięć) albo usuwają koszt, który obecny plan przypisywał niewłaściwej przyczynie.
- **Cel 2.4 jest nieosiągalny bez 2.9.** Sam silnik potrzebuje 2,4 µs na odczyt po kluczu, a celem 2.4 jest 3,5 µs łącznie z granicą JS↔WASM. Instrukcja przygotowana w SQLite jest kompilowana raz i wykonywana wielokrotnie [D2].
- **Interpretacja wiersz po wierszu dominuje wykonanie.** Boncz, Zukowski i Nes (MonetDB/X100, CIDR 2005) pokazali to dla silników przetwarzających krotka po krotce; Kersten i in. (PVLDB 11(13), 2018) porównują kompilację i wektoryzację jako sposoby usunięcia tego kosztu. Obie prace są kandydatami do rdzenia literatury przy rozdziale o optymalizacji. Domknięcia budowane przy przygotowaniu nie generują kodu w czasie wykonania, więc nie dotyczy ich argument, który wyłączył z rdzenia kompilację zapytań do WASM ([D3]: kompilacja nie opłaca się przy zapytaniach trwających mikrosekundy).
- **Outbox** jest kosztem płaconym przez każdą aplikację, także bez synchronizacji, w trzech metrykach naraz (snapshot, odtworzenie, pamięć); log skracany i wysyłanie stanu zmienionych wierszy opisują [C10] i [L28].
- **Pozostałe zadania** (2.12–2.16) mają mały nakład i wskazany pomiar, który je potwierdzi albo odrzuci.

## Odrzucone alternatywy

- **Nowe zadania jako „dobrze mieć”** – 2.9 jest warunkiem celu 2.4, a 2.10 i 2.11 dotyczą operacji, w których Flat-SQL przegrywa z SQLite (UPDATE, snapshot, odtworzenie); bez nich ponowny pomiar oceniałby głównie zmiany drugorzędne.
- **Zwarte wiersze (2.7) przed kompilacją wyrażeń** – układ wierszy odpowiada za około jedną trzecią czasu skanu, a 2.7 ma duży nakład.
- **Kompilacja zapytań do kodu WASM w czasie wykonania** – koszt kompilacji przy zapytaniach trwających mikrosekundy; domknięcia dają większość zysku bez generowania kodu.
- **Wpisanie wszystkich hipotez O1–O13** – O5, O6, O8 i O12 nie mają jeszcze potwierdzenia albo wymagają nowej wersji obciążenia (O12).

## Konsekwencje

- Sekcja 2 rośnie o osiem zadań; szacunkowo 2000–3400 linii kodu z testami, z czego około 1100–1700 w kodzie produkcyjnym rdzenia.
- 2.11 zmienia format snapshotu i decyzję ADR-0016; przy jego wykonaniu powstaje osobny ADR, a odczyt starego formatu zostaje.
- 2.14 wymaga nowej operacji obciążenia i nowego identyfikatora obciążenia; adapter wykrywa metodę, której prototyp nie ma (ADR-0037).
- Pomiar rozbicia kosztów trzeba powtórzyć na nieobciążonej maszynie przed użyciem liczb bezwzględnych w pracy.

## Źródła

[D2], [D3] (poza rdzeniem), [C10], [L28]; Boncz P., Zukowski M., Nes N., *MonetDB/X100: Hyper-Pipelining Query Execution*, CIDR 2005; Kersten T., Leis V., Kemper A., Neumann T., Pavlo A., Boncz P., *Everything You Always Wanted to Know About Compiled and Vectorized Queries But Were Afraid to Ask*, PVLDB 11(13), 2018, DOI 10.14778/3275366.3275370; [benchmark-results.md §1.12](../research/benchmark-results.md#112-rozbicie-kosztów-silnika-pomiar-pomocniczy).
