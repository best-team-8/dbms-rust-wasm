# ADR-0054: Dokumentacja według modułów, jedno źródło każdej informacji i dwie wersje silnika

| | |
|---|---|
| Status | przyjęta |
| Data | 2026-10-29 |
| Rodzaj | proces |
| Powiązane | ADR-0021, ADR-0022, ADR-0029, ADR-0031, ADR-0032, ADR-0037, ADR-0039, ADR-0052; future 12.2 |

## Kontekst

Dokumentacja rosła razem z projektem i ta sama informacja była zapisywana w kilku miejscach, w różnych chwilach. Skutki sprawdzone 28.10.2026:

- tabela wyników w README głównym pochodziła z wyników wstępnych z 6.10.2026 (jedno uruchomienie na silnik), sprzed Benchmarku 1 i 2; liczby testów w README były nieaktualne;
- procedura pomiaru była opisana trzy razy i za każdym razem inaczej (`bench/README.md`, scenariusz badawczy §5.13, README danych Benchmarku 2), a Benchmark 2 wykonano trzecią z nich;
- przegląd literatury miał trzy dokumenty z trzech chwil (wnioski z 20.10, porównanie sprzed pomiarów, weryfikacja tez po pomiarach), dwa pierwsze z liczbami, które Benchmark 1 obalił;
- zastosowania opisywały dwa pliki, z których drugi porównywał się z pierwszym;
- kierunki rozwoju były w sześciu miejscach, a projekt docelowy miał tylko jeden komponent (`future/components/sync-server.md`);
- planowane README modułów powieliłyby `architecture.md` i `flat-sql.md`;
- 17 tagów wersji (`prototyp`, `po-2.1`–`po-2.16`) i 7 wariantów kompilacji, z wersją końcową nazwaną numerem zadania wykonanego jako ostatnie (`po-2.5`); żaden tag nie był na GitHubie, więc instrukcja odtworzenia nie działała na świeżym klonie.

Dokumentację ma czytać także recenzent albo promotor, który nie zajmuje się bazami w przeglądarce, więc wejście do repozytorium i instrukcja odtworzenia badań muszą dać się wykonać bez znajomości historii projektu.

## Decyzja

1. Każda informacja ma jedno miejsce, a pozostałe dokumenty do niego odsyłają. README główne podaje tylko wybrane wyniki Benchmarku 2 z datą i warunkami pomiaru.
2. Budowę systemu opisują README modułów: rdzeń (`crates/flatsql-core/`) z podmodułami `src/sql/` (specyfikacja języka), `src/storage/` (trwałość i formaty binarne) i `tests/`, wiązania WebAssembly, CLI, most JavaScript, serwer, demo i testy end-to-end. README ma stałą kolejność sekcji (odpowiedzialność i granice, zależności, struktura, interfejs, działanie, decyzje, budowanie i testy, ograniczenia), a sekcje, które nie dotyczą modułu, pomija. Pozostałe moduły Rust opisują komentarze dokumentacyjne (`//!`, `cargo doc`). `docs/architecture.md` i `docs/flat-sql.md` zostały rozłożone na te README i usunięte.
3. README główne jest po polsku i opisuje produkt i badanie: czym jest Flat-SQL, cele produktu, najważniejsze wyniki, architekturę całości z przepływami danych, mapę modułów, szybki start, mapę dokumentacji według pytań czytelnika, ograniczenia i słowniczek. Nie ma osobnych map `docs/README.md` ani `docs/research/README.md`.
4. Badania: `benchmark-results.md` zawiera Benchmark 1 i 2 (wyniki wstępne usunięte; wynik ręcznego scenariusza offline przeniesiony do `tests-results.md` §2.5, porównanie liczby uruchomień do scenariusza badawczego §5.1, związek hipotez z literaturą do §2.12), `optimization-results.md` 16 pojedynczych optymalizacji bez powtórzonego zestawienia Benchmarku 2, `use-cases.md` łączy katalog zastosowań z analizą wdrożeń.
5. Literatura: `references/literature.md` łączy porównanie z literaturą i weryfikację tez pomiarami; `conclusions.md` i notatki do prac usunięte. Lista prac spoza rdzenia nie jest już częścią indeksu; `search-methodology.md` podaje ich liczbę i kryteria wyłączenia, a dokumenty, które się na nie powołują, podają autora i rok z dopiskiem „praca poza rdzeniem”.
6. Kierunki rozwoju: `docs/future/README.md` (według komponentów, cele w metrykach niszy, zastosowania, projekt docelowego serwera synchronizacji). Lista rozszerzeń przeszła z `docs/todo_future.md` do `docs/future/todo.md`; zasada podziału list z ADR-0029 się nie zmienia. Niewykonane hipotezy z Benchmarku 1 (grupowanie wywołań w trybie Worker, mniej plików mostu przy starcie, szczyt pamięci przy dużej transakcji, odczyt wyłącznie z indeksu) są w części „Kierunki na później”, a plik z hipotezami O1–O13 został usunięty.
7. Wersje silnika: tylko adnotowane tagi `v1-prototyp` (dawny `prototyp`) i `v2-optymalizacja` (commit dawnego `po-2.5`), wypychane na GitHub. Tagi `po-2.N` usunięto; ich commity i tematy zawiera tabela w `docs/research/data/optimization/README.md`, a opisy optymalizacji podają wersję „przed” tematem commitu.
8. Odtworzenie badań opisuje tylko `bench/README.md`, w trzech poziomach: tabele i wykresy z zapisanych wyników, pomiar kontrolny, pełne powtórzenie; z wymaganiami, czasem trwania i oczekiwanym wynikiem każdego kroku. Scenariusz badawczy i README danych do niej odsyłają.
9. Repozytorium: sprawdzanie odnośników w plikach Markdown (`scripts/check-links.mjs`, osobne zadanie CI), wersja Rust przypięta w `rust-toolchain.toml`, usunięty pusty przebieg CI `ci.yml`. Materiały do pisania pracy (szablon, zasady uczelni, analiza innych prac) są w `docs/diploma/`, poza dokumentacją produktu; prezentacja produktu w `docs/presentation/v1.pdf`.

## Uzasadnienie

- Jedno źródło prawdy: wszystkie nieaktualne liczby w README i analizach literatury były kopiami. Odnośnik się nie starzeje, a sprawdzanie odnośników w CI wykrywa jego zerwanie po przeniesieniu pliku.
- Odtwarzalność w poziomach odpowiada praktyce oceny artefaktów badawczych na konferencjach ACM (artefakt dostępny, działający, odtworzony): czytelnik sprawdza najpierw, że tabele wynikają z danych, potem że narzędzie działa, a dopiero na końcu powtarza wielogodzinny pomiar. Poziom 1 sprawdzono: skrypty raportów odtwarzają opublikowane tabele, wartości i wykresy bez różnic w treści (około 3 s); poziom 2 trwa około minuty.
- Dwie wersje odpowiadają dwóm etapom badań (prototyp i wersja po optymalizacjach); wersje pośrednie są potrzebne tylko do pomiarów pojedynczych zmian, a manifesty tych pomiarów zapisują commit, więc usunięcie tagów nie traci informacji.
- Opis modułu obok kodu jest aktualizowany razem z kodem; opis architektury w osobnym pliku musiałby powtarzać to samo.

## Odrzucone alternatywy

- Mapy `docs/README.md` i `docs/research/README.md`: powtarzałyby mapę dokumentacji z README głównego.
- Osobny plik w `docs/future/components/` dla każdego komponentu: projekt docelowy ma tylko serwer synchronizacji, a kierunki pozostałych komponentów mieszczą się w sekcjach jednego pliku.
- Zachowanie `architecture.md` obok README modułów: dwa opisy tego samego.
- Połączenie `optimization-results.md` z `benchmark-results.md`: pierwszy opisuje 16 osobnych eksperymentów z hipotezą i pomiarem przed zmianą i po niej, drugi dwa pełne pomiary; wspólne było tylko zestawienie Benchmarku 2, które usunięto z pierwszego.
- Zostawienie tagów `po-2.N` obok nowych: za dużo wersji dla czytelnika spoza projektu.
- README w każdym katalogu `src/`: moduły Rust opisują komentarze dokumentacyjne, README mają tylko podmoduły, które czytelnik otwiera osobno (język, składowanie, testy).
- Plik licencji i plik z danymi do cytowania repozytorium: na razie nie, decyzja autora.

## Konsekwencje

- Treść wcześniejszych ADR się nie zmienia; odnośniki do usuniętych plików prowadzą do plików, które przejęły ich treść, a ADR-0039 i ADR-0041 odsyłają do pliku hipotez w wersji z commitu, w którym jeszcze istniał.
- Manifesty, raporty `compare.md` i część skryptów pomiarowych podają dawne nazwy tagów; przypisanie do commitów zawiera README danych optymalizacji.
- Zasady pracy w repozytorium wskazują `docs/future/todo.md` zamiast `docs/todo_future.md`.
- Zadanie future 12.2 (odnośniki w README) jest wykonane przez mapę dokumentacji w README głównym.
- Ręczny scenariusz offline w przeglądarce ma wynik tylko z wersji sprzed prototypu; powtórzenie go na `v2-optymalizacja` uaktualniłoby `tests-results.md` §2.5.

## Źródła

[bench/README.md](../../bench/README.md), [docs/future/README.md](../future/README.md), [references/literature.md](../references/literature.md), [research/use-cases.md](../research/use-cases.md), [data/optimization/README.md](../research/data/optimization/README.md#wersje-mierzone-w-czasie-optymalizacji).
