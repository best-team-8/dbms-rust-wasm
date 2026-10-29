# ADR-0052: Rdzeń literatury według zastosowania i wyniku weryfikacji

| | |
|---|---|
| Status | przyjęta; odrzucenie usunięcia katalogu `_poza-rdzeniem/` zastąpione przez ADR-0057 |
| Data | 2026-10-28 |
| Rodzaj | proces |
| Powiązane | ADR-0021, ADR-0031 (zastępuje kryterium i skład rdzenia), ADR-0007, ADR-0050, ADR-0051; zlecenie autora po Benchmarku 2, bez zadania w todo.md (zadania literaturowe dotyczą pracy dyplomowej, [todo.md §5](../todo.md)); [literature.md](../references/literature.md) |

## Kontekst

ADR-0031 włączało do rdzenia każdą pracę z podrozdziałem w mapie rozdziałów pracy dyplomowej. Część z 33 prac miała tam tylko rolę pozycjonującą: definicję pojęcia ([E1]), krytykę modelu bez złączeń ([M1]), kierunki rozwoju ([L24], [C11]) albo zagrożenie dla trafności ([B5]). Po Benchmarku 1, Benchmarku 2 (ADR-0051), pomiarze wariantów kompilacji (ADR-0050) i testach poprawności można sprawdzić, które tezy literatury przeniosły się na Flat-SQL. Hipoteza H2 przyjęła przedział narzutu WebAssembly 1,2–3× z [P2] i [W1], a zadanie poziomów optymalizacji wyszło z tezy [W11]; obie przesłanki pomiar obalił. Autor poprosił o zestawienie hipotez z literaturą, ograniczenie rdzenia do prac faktycznie zastosowanych i podział reszty na potwierdzone i niepotwierdzone, w trzech katalogach.

## Decyzja

1. Praca jest w rdzeniu, jeśli projekt ją zastosował (przejął mechanizm, metodę pomiaru albo uzasadnienie decyzji) albo benchmark lub test sprawdził jej tezę. Warunek z ADR-0031, podrozdział w mapie rozdziałów, obowiązuje dalej jako plan cytowań.
2. Pliki rdzenia leżą w trzech katalogach `docs/references/papers/`: `potwierdzone/` (teza sprawdzona i potwierdzona, 19 prac: [P2], [D1], [D2], [D4], [L1], [L2], [L27], [L28], [C2], [C7], [C9], [C10], [S1], [S4], [S8], [S10], [M2], [B1], [B3]), `niepotwierdzone/` (praca coś sugerowała, pomiar to sprawdził i w tym silniku wykluczył: [W1], [W11]) i `zastosowane/` (zastosowana, ale bez tezy sprawdzalnej pomiarem: [P1], [W2], [W6], [D5], [C3], [S6], [V1]). Praca zastosowana i sprawdzona trafia do katalogu według wyniku.
3. [L24], [C11], [M1], [E1] i [B5] przechodzą do `papers/_poza-rdzeniem/` z wpisem w §13 indeksu. Rdzeń liczy 28 prac.
4. Ocena każdej pracy, z miejscem sprawdzenia i liczbami, jest w [literature.md](../references/literature.md). Dokument jest materiałem do rozdziałów 6–8 pracy, więc nie odwołuje się do ADR ani do zadań.
5. Dokumenty badań (`docs/research/`) przywołują prace spoza rdzenia opisowo. Analizy w `references/` (porównanie z literaturą, wnioski), listy zadań i dokumenty planowania zachowują identyfikatory, które rozwiązuje tabela §13 indeksu.

## Uzasadnienie

- **Przewidywania, które się nie sprawdziły.** [W1]: średnio 1,45–1,55× wolniej niż natywnie, koszt wywołań obsługiwanych przez JavaScript pomijalny. Flat-SQL: operacje wykonywane w silniku 0,89–1,13× (Benchmark 1) i 0,82–0,93× (Benchmark 2), a narzut tylko na granicy JS↔WASM (odczyt po kluczu 3,9×, po zmianach 2,7–3,6×). Wyniki wstępne z jednym procesem na silnik (1,1–1,8×) wyglądały na zgodne z [W1]; różnica zniknęła przy dziesięciu procesach, co potwierdza tezę [B1]. [W11]: moduł optymalizowany pod rozmiar nie musi być wolniejszy. Pomiar wariantów (ADR-0050): `opt-level = "s"` wolniejszy o 25–40%, `"z"` o 48–78%, `opt-level = 3` najszybszy.
- **Potwierdzenia, które wnoszą liczby do pracy.** Między innymi: naiwny snapshot [S10] (pauza 6,7 / 20,9 / 40,3 ms rosnąca z rozmiarem, po kodowaniu krokami 1,2–2,7 ms); błędy kolejności utrwalania [S8] znalezione w obu magazynach plikowych Flat-SQL; zbieżność rejestrów LWW [L2] w 1000 przebiegach testu z przesuniętymi zegarami i utratą wiadomości, z utratą współbieżnych zmian opisaną przez [C9]; binarny bufor wyniku [D1] (10 000 wierszy 14,0 → 1,59 ms); przewagi SQLite z [D2] (indeks pokrywający usunięty optymalizacją, format stronicowy w snapshocie i odtworzeniu pozostał); adapter RxDB zmieniający wynik stukrotnie [B3]. [C2] potwierdzone z rozszerzeniem: odrzucenie znacznika spoza granicy dryfu jest konieczne, ale bez nowych znaczników po poprawie zegara synchronizacja nie kończy się (ADR-0035).
- **Prace zastosowane bez pomiaru.** [W6] i [S6] uzasadniają decyzje, których alternatyw (memory64, logowanie poleceń) nie zbudowano; [D5] dał cel odczytu po kluczu (osiągnięto 3,28 µs), ale ma zamknięty kod; [P1], [C3], [W2], [V1] dały mechanizmy i zasady bez tez liczbowych.
- **Prace wyłączone.** Żadna z pięciu nie dała mechanizmu ani metody, którą projekt wykonał, i żadnej tezy nie dało się sprawdzić pomiarem. Interpretację H1 dają [L1] i [D4] zamiast [E1]; przewidywalność kosztu w modelu bez złączeń – [M2], którego teza została sprawdzona, zamiast [M1]; kierunki LWW per kolumna i replikacji częściowej mają w tekście jedno zdanie ([C9] wystarcza dla pierwszego). Rdzeń 28 prac spełnia wymóg co najmniej 15 publikacji z zapasem.
- **Identyfikatory w analizach.** Porównanie z literaturą i wnioski w `references/` to analizy z 20–22.10.2026; przepisanie kilkudziesięciu odwołań na opisowe nie zmieniłoby treści, a tabela §13 rozwiązuje każdy identyfikator. Opisowo cytują dokumenty badań, bo z nich powstaje tekst pracy z własną numeracją literatury.

## Odrzucone alternatywy

- **Prace zastosowane i sprawdzone w katalogu `zastosowane/`** – prawie każda potwierdzona praca była też zastosowana (np. [S1] group commit, [S8] protokół trwałości), więc katalogi potwierdzonych i niepotwierdzonych byłyby prawie puste, a podział nie pokazywałby wyniku badań.
- **Usunięcie katalogu `_poza-rdzeniem/`** – zawiera 55 plików z kilku rewizji, które można przywrócić (ADR-0021); usunięcie zgubiłoby materiał bez zysku.
- **Ocena [D5] jako niepotwierdzonej** – deklaracja autora (5–15% narzutu w przeglądarce) różni się od wyniku Flat-SQL dla odczytu po kluczu, ale warunki pomiaru autora są nieznane, a systemu nie da się uruchomić; porównanie byłoby „jabłkami i pomarańczami” [B3].
- **Dodanie do rdzenia prac Boncza i in. (MonetDB/X100) i Kerstena i in. (PVLDB 2018)** – są cytowane w opisie kompilacji wyrażeń, a ich teza o koszcie interpretacji zgadza się z rozbiciem kosztów (65% czasu skanu w interpreterze). Nie mają plików ani wpisów; decyzję o włączeniu według kryteriów ADR-0021 zostawiono autorowi, z uwagą w indeksie.

## Konsekwencje

- Indeks ([references/README.md](../references/README.md)) ma sekcję „Podział rdzenia według wyniku weryfikacji”, zaktualizowane ścieżki plików, wyniki przy [W1] i [W11], mapy bez prac wyłączonych i pięć nowych wierszy w §13. [search-methodology.md §4.2](../references/search-methodology.md#42-rewizja-4--zastosowanie-i-wynik-weryfikacji) opisuje rewizję.
- [literature.md](../references/literature.md) dostał uwagę, że ocena H2 i H3 opiera się na wynikach wstępnych; tabeli z 20.10.2026 nie przepisano.
- [benchmark-scenario.md](../research/benchmark-scenario.md) i [wasm-engines-comparison.md](../research/wasm-engines-comparison.md) przywołują YCSB i relacje replikowane bez konfliktów opisowo.
- W pracy dyplomowej H2 opisuje się wobec [P2] (zgodne) i [W1] (niezgodne), z wyjaśnieniem różnicy i lekcją metodyczną z wyników wstępnych.
- Katalog pliku zmienia się, gdy nowy pomiar zmieni ocenę pracy; nowa praca trafia do rdzenia z katalogiem i podrozdziałem w mapie.
- ADR-0031 ma status „przyjęta; kryterium i skład rdzenia zastąpione przez ADR-0052”.

## Źródła

[literature.md](../references/literature.md), [benchmark-results.md §1.10, §3.11](../research/benchmark-results.md), [optimization-results.md §12, §15, §16](../research/optimization-results.md), [tests-results.md](../research/tests-results.md); [P2], [W1], [W11], [B1], [B3], [S8], [S10], [L2], [C2], [C9], [D1], [D2].
