# Baza wiedzy – literatura do projektu Flat-SQL

Rdzeń literatury projektu: silnik SQL w Rust kompilowany do WebAssembly, płaski model tabel adresowanych kluczem, dane w pamięci z WAL i snapshotem, synchronizacja Local-First (HLC + LWW) oraz metodyka benchmarków. Stan po rewizji z **28 października 2026**: w rdzeniu są tylko prace, które projekt zastosował albo których tezy sprawdziły benchmarki i testy; 28 prac podzielono na trzy katalogi według wyniku tej weryfikacji ([podział rdzenia](#podział-rdzenia-według-wyniku-weryfikacji), [ADR-0052](../adr/0052-rdzen-literatury-wedlug-zastosowania-i-weryfikacji.md)). Każda praca rdzenia ma też podrozdział w [mapie rozdziałów](#mapa-rozdziały-pracy-dyplomowej--literatura) ([ADR-0031](../adr/0031-rdzen-literatury-wedlug-cytowan-w-pracy.md)).

| Plik / katalog | Zawartość |
|---|---|
| [literature.md](literature.md) | Flat-SQL na tle literatury: tezy prac sprawdzone pomiarami i testami, własności bez pomiaru, wkład projektu |
| [search-methodology.md](search-methodology.md) | jak szukano, audyt pokrycia atutów projektu, kryteria doboru rdzenia, ograniczenia |
| `papers/zastosowane/` | 7 plików PDF: prace zastosowane w projekcie, których tez pomiar nie sprawdzał |
| `papers/potwierdzone/` | 20 plików PDF: prace, których tezy potwierdziły benchmarki lub testy |
| `papers/niepotwierdzone/` | 2 pliki PDF: prace, których sugestie pomiar sprawdził i w tym silniku się nie sprawdziły |
| `papers/_poza-rdzeniem/` | prace odrzucone w rewizjach; liczby i kryteria: [search-methodology.md §4](search-methodology.md#4-dobór-rdzenia) |

> **Prawa autorskie.** Pliki w `papers/` służą do własnej nauki. Wiele z nich to wersje autorskie z dopiskiem „for personal use, not for redistribution” albo kopie z serwisów kursów uniwersyteckich, dlatego katalog jest wykluczony z repozytorium i nie należy go publikować. W pracy dyplomowej cytuje się wersję wydawniczą (DOI), nie lokalną kopię.

## Jak czytać wpisy

Każdy wpis ma identyfikator używany w pozostałych dokumentach, opis bibliograficzny, plik, streszczenie i odpowiedź na pytanie, **co z pracy wynika dla Flat-SQL**, oraz zadania, które się na nią powołują ([todo.md](../todo.md) – optymalizacja prototypu, [future/todo.md](../future/todo.md) – rozszerzenia). Hipotezy H1–H6 pochodzą z [benchmark-scenario.md](../research/benchmark-scenario.md).

**Identyfikatory zachowano z poprzedniej wersji bazy**, żeby nie unieważniać odwołań; luki w numeracji to pozycje przeniesione poza rdzeń ([search-methodology.md §4](search-methodology.md#4-dobór-rdzenia)). Nowe prace z rewizji: [C9]–[C11], [L27]–[L28], [M2]–[M3], [S10]. Rewizja z 22.10.2026 ([ADR-0031](../adr/0031-rdzen-literatury-wedlug-cytowan-w-pracy.md)): do rdzenia wróciła [W6], a [C8], [L8], [L11], [L20], [L26] i [M3] przeszły poza rdzeń. Rewizja z 28.10.2026 ([ADR-0052](../adr/0052-rdzen-literatury-wedlug-zastosowania-i-weryfikacji.md)): [L24], [C11], [M1], [E1] i [B5] przeszły poza rdzeń, bo projekt ich nie zastosował, a pomiar nie sprawdzał ich tez.

Kategorie: [P] polecone przez promotora · [W] WebAssembly · [D] bazy w przeglądarce i porównywane silniki · [L] Local-First, CRDT, synchronizacja · [C] replikacja, zegary, praca offline · [S] składowanie i odtwarzanie · [M] model danych i zapytania · [E] edge computing · [B] metodyka pomiarów · [V] prywatność · [X] do pobrania · [Q] dokumentacja platform i źródła branżowe.

## Mapa: założenia projektu → literatura

| Założenie lub atut Flat-SQL | Prace rdzenia |
|---|---|
| Płaski model, obowiązkowy klucz, brak JOIN, przewidywalne wykonanie | [C7] Helland, [M2] PIQL, [D4] Riffle (krytyka) |
| Dane w pamięci, wykonanie jednowątkowe | [S4] H-Store, [S1] DeWitt, [P1] rozdz. 3 i 7, [D5] Zeta-Lite |
| WAL fizyczny + snapshot, group commit | [S1] DeWitt, [S6] Malviya, [S10] Cao, [P1] rozdz. 3 |
| Odporność na awarie zapisu | [S8] Pillai, [Q1] trwałość IndexedDB |
| Zegar HLC, porządek znaczników | [C2] Kulkarni, [P1] rozdz. 8–9 |
| LWW per wiersz, tombstone'y, zbieżność | [L2] Shapiro, [L28] Almeida, [C9] Gray |
| Outbox przetrwały restart, praca offline | [C10] Coda, [L27] Simba, [C3] Bayou |
| Protokół push/pull z kursorami, log serwera | [C3] Bayou, [P1] rozdz. 11–12, [C7] Helland (idempotencja) |
| Atomowość: transakcja lokalnie, wiersz między replikami | [L27] Simba, [C7] Helland |
| Ewolucja schematu w replikach (ograniczenie) | [L1] (problem otwarty), [P1] rozdz. 4 |
| Mały moduł WASM, ten sam rdzeń wszędzie | [P2] Haas, [W1] Jangda, [W11] Yan |
| Cel `wasm32` bez memory64 | [W6] Döllerer i Engelke, [W11] Yan |
| Granica JS↔WASM (H3) | [D1] DuckDB-Wasm (Arrow IPC); brak badań recenzowanych – wkład własny |
| Lokalne zapytania zamiast sieci (H1) | [L1] Local-first, [D4] Riffle (budżet 16 ms) |
| Hybryda: lokalny silnik + serwer koordynujący | [C9] Gray (two-tier), [C3] Bayou |
| Privacy by Design | [V1] Cavoukian, [L1], [W2] Lehmann |
| Rozszerzenia: replikacja częściowa, reguły serwera, zapytania reaktywne | [C9] Gray, [D4] Riffle |
| Metodyka benchmarków | [B1] Georges, [B3] Raasveldt |

## Podział rdzenia według wyniku weryfikacji

Kryterium rdzenia od 28.10.2026 ([ADR-0052](../adr/0052-rdzen-literatury-wedlug-zastosowania-i-weryfikacji.md)): projekt zastosował pracę (mechanizm, metodę albo uzasadnienie decyzji) albo benchmark lub test sprawdził jej tezę. Katalog w `papers/` wynika z oceny w [literature-verification.md](literature.md): praca, której tezę sprawdzono, trafia do katalogu według wyniku, nawet jeśli projekt ją też zastosował.

| Katalog | Prace | Kryterium |
|---|---|---|
| `papers/potwierdzone/` | [P2], [D1], [D2], [D4], [L1], [L2], [L27], [L28], [C2], [C7], [C9], [C10], [S1], [S4], [S8], [S10], [M2], [B1], [B3] | teza sprawdzona benchmarkiem lub testem i potwierdzona ([literature.md §3](literature.md#3-tezy-potwierdzone)); [C2] z rozszerzeniem: samo odrzucanie znaczników nie wystarcza |
| `papers/niepotwierdzone/` | [W1], [W11] | praca sugerowała wynik, który pomiar sprawdził i w tym silniku wykluczył ([literature.md §4](literature.md#4-tezy-niepotwierdzone)) |
| `papers/zastosowane/` | [P1] (notatki do wykładu), [W2], [W6], [D5], [C3], [S6], [V1] | projekt przejął z pracy mechanizm, metodę albo uzasadnienie, ale pomiar nie mógł sprawdzić tezy ([literature.md §5](literature.md#5-prace-zastosowane-bez-sprawdzenia-pomiarem)) |

Prace cytowane w opisie optymalizacji wyrażeń bez identyfikatora (Boncz i in., MonetDB/X100; Kersten i in., PVLDB 2018) nie są w indeksie; ich teza o koszcie interpretacji wiersz po wierszu zgadza się z rozbiciem kosztów pełnego skanu (65% czasu w interpreterze).

## Mapa: rozdziały pracy dyplomowej → literatura

Kryterium rdzenia ([ADR-0031](../adr/0031-rdzen-literatury-wedlug-cytowan-w-pracy.md)): każda praca ma tu co najmniej jeden podrozdział; praca bez podrozdziału wychodzi poza rdzeń. Źródła [Q] to dokumentacja platform – w pracy cytowane jako strony WWW, nie liczą się do wymaganych 15 publikacji.

| Podrozdział pracy | Prace rdzenia | Dokumentacja [Q] |
|---|---|---|
| 1.1 Problematyka i motywacja | [L1], [D4] | – |
| 1.4 Założenia projektowe | ADR-0001: [D4], [L1]; ADR-0002: [L1], [C3], [C9], [C10], [L27], [P1] rozdz. 9; ADR-0003: [V1], [P2], [W2], [L1]; ADR-0004: [C7], [M2], [D4] | – |
| 2.1 WebAssembly jako platforma wykonawcza | [P2], [W1], [W2], [W11], [W6] | [Q6], [Q13] |
| 2.2 Przechowywanie danych w przeglądarce | [L1] (czyszczenie magazynu przeglądarki) | [Q1]–[Q4], [Q7], [Q8], [Q12] |
| 2.3 Trwałość i odtwarzanie po awarii | [P1] rozdz. 3, [S1], [S4], [S6], [S8], [S10] | – |
| 2.4 Local-First i synchronizacja replik | [L1], [L2], [L28], [C2], [C3], [C7], [C9], [C10], [L27], [P1] rozdz. 5, 8, 9, 11 | – |
| 2.5 Analiza porównawcza silników WASM | [D1], [D2], [D4], [D5], [L27] | [Q7], [Q11] |
| 2.6 Wnioski z przeglądu | [P2], [W1] (nie mierzą granicy JS↔WASM), [D1] (najbliższy precedens) | – |
| 3.1 Scenariusze zastosowań | [D1] (CSV 500 MB), [W2] (klucze portfeli), [C9], [P1] rozdz. 9 (płatności), [C3] (zamiary) | [Q2] |
| 3.3 Dialekt Flat-SQL i model danych | [C7], [M2], [D4] | – |
| 3.5 Decyzje projektowe (tabela 3.2) | ADR-0006: [P1] rozdz. 3, [S1], [S4]; ADR-0007: [W6], [W11], [D1]; ADR-0009: [S6], [S1], [S8], [S10], [D5]; ADR-0012: [S8]; ADR-0013: [D4]; ADR-0015: [C2], [L2], [L28], [C9]; ADR-0019: [D1], [D5] | ADR-0007: [Q6]; ADR-0012: [Q1]–[Q4], [Q7], [Q8]; ADR-0013: [Q5], [Q12]; ADR-0019: [Q13] |
| 4.2 Transakcje | [S4], [P1] rozdz. 7 | – |
| 4.3 Warstwa lokalnego składowania | [S1] (group commit), [S8], [P1] rozdz. 3 | [Q1] |
| 4.4 Synchronizacja Local-First | [C2], [C3], [C7], [C10], [L27], [L28], [P1] rozdz. 11–12 | – |
| 4.5 Most WebAssembly i interfejs JavaScript | [D1] | [Q13] |
| 5. Metodyka badań | [B1], [B3] | [Q5], [Q7] |
| 6.1–6.3 Wyniki H1–H3 | H1: [L1], [D4]; H2: [P2], [W1] (niepotwierdzona); H3: [D1] | – |
| 6.4–6.6 Wyniki H4–H5, zimny start i pamięć | H4: [D2], [S4], [D5]; H5: [S1], [D5]; pamięć i rozmiar: [W11], [P2], [B3] | [Q1] |
| 7. Optymalizacja silnika | 7.2 wykonanie zapytań: [D2]; 7.3 zapis i aktualizacja: [S4], [S6], [C10], [L28]; 7.4 granica JS↔WASM: [D1], [D5]; 7.5 składowanie i odtwarzanie: [S10], [D4] (checkpoint wobec 16 ms), [W11] (pamięć); 7.6 konfiguracja kompilacji: [W11] (niepotwierdzona); 7.7 zestawienie: [W1] (narzut WebAssembly po optymalizacjach) | – |
| 8.1–8.2 Awarie zapisu, zbieżność replik | [S8]; [L2], [L28], [C2] | – |
| 8.4–8.5 Scenariusz offline, granice gwarancji | [L1], [C10]; [C7], [L27], [C9] | [Q1]–[Q4] |
| 9.4 Ograniczenia | [C9] (utracone aktualizacje), [L1] (ewolucja schematu), [P1] rozdz. 9 (unikalność między replikami) | – |
| 9.5 Kierunki dalszego rozwoju | węzły edge: [C9], [P1] rozdz. 5; LWW per kolumna: [C9]; trwały log serwera: [P1] rozdz. 11, [C10]; checkpoint bez przestoju: [S10]; zwarta reprezentacja wierszy: [W11]; reguły serwera i zamiary: [C9], [C3]; aktualizacje przemienne: [C9], [L2]; szyfrowanie end-to-end: [L1], [V1]; zapytania reaktywne: [D4]; usuwanie tombstone'ów: [L28] | wiele kart: [Q12] |

---

## 1. Prace polecone przez promotora

- **[P1]** Kleppmann M. *Designing Data-Intensive Applications: The Big Ideas Behind Reliable, Scalable, and Maintainable Systems.* O'Reilly Media, 2017. ISBN 978-1-449-37332-0.
  Wydanie polskie: *Przetwarzanie danych w dużej skali. Niezawodność, skalowalność i łatwość konserwacji systemów*, Helion, Gliwice 2018 (ISBN wg Goodreads 978-83-283-4065-7, wg katalogu TUM 978-83-283-4066-4 – sprawdzić na egzemplarzu). Wydanie 2: Kleppmann M., Riccomini C., O'Reilly, 2026.
  — Podręcznik systemów danych: bazy w pamięci, log i snapshot (rozdz. 3), kodowanie i ewolucja formatów (4), klienci offline jako replikacja multi-leader i LWW (5), wykonanie szeregowe (7), zawodność zegarów (8), granice spójności ostatecznej (9), CDC i kompaktowanie logu (11), klienci ze stanem i model push (12). **Dla projektu:** teoretyczna podstawa niemal każdej decyzji z [flatsql-core/README.md](../../crates/flatsql-core/README.md). **Zadania:** todo 0.2, 1.1, 2.1; future 6.1, 6.2, §7, 12.1, 12.4.

- **[P2]** Haas A., Rossberg A., Schuff D. L., Titzer B. L., Holman M., Gohman D., Wagner L., Zakai A., Bastien J. F. *Bringing the Web up to Speed with WebAssembly.* PLDI 2017, ACM, s. 185–200. DOI: [10.1145/3062341.3062363](https://doi.org/10.1145/3062341.3062363). Wersja skrócona: Rossberg A. i in., Communications of the ACM 61(12), 2018, s. 107–115, DOI: [10.1145/3282510](https://doi.org/10.1145/3282510).
  — Artykuł wprowadzający WebAssembly: bezpieczny, szybki, przenośny i zwarty kod; pamięć liniowa izolowana od kodu i stosu silnika; formalna semantyka. W PolyBenchC 7 z 24 programów w granicy 10% kodu natywnego, prawie wszystkie poniżej 2×; kod o 33,7% szybszy od asm.js przy 62,5% jego rozmiaru. **Dla projektu:** punkt odniesienia dla H2, rozmiaru modułu i argumentu „izolowanej piaskownicy”; artykuł nie mierzy kosztu granicy JS↔WASM (H3). **Zadania:** todo 1.8.

## 2. WebAssembly – wydajność i bezpieczeństwo

- **[W1]** Jangda A., Powers B., Berger E. D., Guha A. *Not So Fast: Analyzing the Performance of WebAssembly vs. Native Code.* USENIX ATC 2019.
  **Plik:** [papers/niepotwierdzone/2019-jangda-not-so-fast-webassembly-vs-native.pdf](papers/niepotwierdzone/2019-jangda-not-so-fast-webassembly-vs-native.pdf).
  — SPEC CPU przez Browsix-Wasm: średnio 1,55× wolniej w Chrome i 1,45× w Firefoksie, maksymalnie 2,5×. Przyczyny: więcej dostępów do pamięci (presja na rejestry), sprawdzanie stosu przy każdym wywołaniu, większy kod. **Dla projektu:** podstawa przedziału 1,2–3× w H2. **Wynik:** teza niepotwierdzona – operacje wykonywane w silniku mają w WebAssembly 0,82–1,13 czasu natywnego, a narzut leży na granicy JS↔WASM, którą praca uznaje za pomijalną ([literature.md §4.1](literature.md#41-narzut-webassembly-względem-kodu-natywnego)).

- **[W2]** Lehmann D., Kinder J., Pradel M. *Everything Old is New Again: Binary Security of WebAssembly.* USENIX Security 2020.
  **Plik:** [papers/zastosowane/2020-lehmann-binary-security-of-webassembly.pdf](papers/zastosowane/2020-lehmann-binary-security-of-webassembly.pdf).
  — W pamięci liniowej brakuje kanarków stosu, ASLR, stron ochronnych i danych tylko do odczytu, więc błędy pamięci z C/C++ są w WASM znów wykorzystywalne. **Dla projektu:** precyzuje, co daje „izolowana piaskownica” z README – chroni hosta przed modułem, nie dane modułu przed jego błędami ani przed innymi skryptami strony; argument za Rustem bez `unsafe` i przeciw trzymaniu kluczy kryptograficznych w bazie. **Zadania:** todo 0.1, 0.2.

- **[W6]** Döllerer L., Engelke A. *Performant Bounds Checking for 64-Bit WebAssembly.* VMIL 2024 (16th ACM SIGPLAN International Workshop on Virtual Machines and Intermediate Languages), s. 23–31. DOI: [10.1145/3689490.3690400](https://doi.org/10.1145/3689490.3690400). *(przywrócona do rdzenia 22.10.2026)*
  **Plik:** [papers/zastosowane/2024-dollerer-bounds-checking-64bit-webassembly.pdf](papers/zastosowane/2024-dollerer-bounds-checking-64bit-webassembly.pdf) (CC BY 4.0).
  — Przy pamięci 32-bitowej środowiska sprawdzają granice „za darmo”: rezerwują całą przestrzeń adresową ze stronami ochronnymi i wychwytują wyjątek sprzętowy. Przy memory64 to nie działa, więc środowiska z obsługą memory64, w tym V8 i SpiderMonkey, wstawiają programowe sprawdzenie przed każdym dostępem – w SPEC CPU2017 narzut średnio ok. 110% (do 242%) względem wykonania bez sprawdzeń. Proponowane dwupoziomowe strony ochronne obniżają go do 12,7%, ale wymagają zmian w środowisku uruchomieniowym; pomiar we własnym środowisku autorów (Linux, x86-64). **Dla projektu:** recenzowane uzasadnienie pozostania przy `wasm32` ([ADR-0007](../adr/0007-wasm32-bez-memory64.md)); [Q6] to wpisy z blogów. **Zadania:** todo 2.7.

- **[W11]** Yan Y., Tu T., Zhao L., Zhou Y., Wang W. *Understanding the Performance of WebAssembly Applications.* IMC 2021. DOI: [10.1145/3487552.3487827](https://doi.org/10.1145/3487552.3487827).
  **Plik:** [papers/niepotwierdzone/2021-yan-understanding-performance-webassembly-applications.pdf](papers/niepotwierdzone/2021-yan-understanding-performance-webassembly-applications.pdf).
  — WASM vs JS w Chrome, Firefoksie i Edge przy różnych poziomach optymalizacji: optymalizacje LLVM nie są dostrojone do WASM i bywają szkodliwe, wyniki silnie zależą od przeglądarki, WASM zużywa znacznie więcej pamięci niż JS. **Dla projektu:** podstawa porównania poziomów optymalizacji i pomiaru pamięci. **Wynik:** teza niepotwierdzona – `opt-level = "s"` i `"z"` wolniejsze o 25–78%, `opt-level = 3` najszybszy; teza o pamięci nierozstrzygnięta ([literature.md §4.2](literature.md#42-poziomy-optymalizacji-kompilatora)). **Zadania:** todo 2.5, 2.7, 4.4.

## 3. Bazy danych w przeglądarce i porównywane silniki

- **[D1]** Kohn A., Moritz D., Raasveldt M., Mühleisen H., Neumann T. *DuckDB-Wasm: Fast Analytical Processing for the Web.* PVLDB 15(12), 2022, s. 3574–3577. DOI: [10.14778/3554821.3554847](https://doi.org/10.14778/3554821.3554847).
  **Plik:** [papers/potwierdzone/2022-kohn-duckdb-wasm.pdf](papers/potwierdzone/2022-kohn-duckdb-wasm.pdf).
  — DuckDB w WASM: zapytania w Web Workerze, czytanie plików lokalnych i zdalnych fragmentami, wyniki serializowane jako strumienie Arrow IPC czytane przez JS wprost ze sterty WASM. **Dla projektu:** właściwe narzędzie dla scenariusza „CSV 500 MB” (inna klasa systemu) i wzór binarnego przekazywania wyników. **Zadania:** todo 0.2, 2.4.

- **[D2]** Gaffney K. P., Prammer M., Brasfield L., Hipp D. R., Kennedy D., Patel J. M. *SQLite: Past, Present, and Future.* PVLDB 15(12), 2022, s. 3535–3547. DOI: [10.14778/3554821.3554842](https://doi.org/10.14778/3554821.3554842).
  **Plik:** [papers/potwierdzone/2022-gaffney-sqlite-past-present-future.pdf](papers/potwierdzone/2022-gaffney-sqlite-past-present-future.pdf).
  — Architektura SQLite (format stronicowy, B-drzewa, OLTP), porównanie z DuckDB, optymalizacje analityczne. **Dla projektu:** wyjaśnia przewagi SQLite w H4 (snapshot = kopia pliku, indeksy pokrywające). **Zadania:** todo 2.1.

- **[D4]** Litt G., Schiefer N., Schickling J., Jackson D. *Riffle: Reactive Relational State for Local-First Applications.* UIST 2023. DOI: [10.1145/3586183.3606801](https://doi.org/10.1145/3586183.3606801).
  **Plik:** [papers/potwierdzone/2023-litt-riffle-reactive-relational-state.pdf](papers/potwierdzone/2023-litt-riffle-reactive-relational-state.pdf).
  — Cały stan aplikacji w lokalnej relacyjnej bazie (SKDB) z reaktywnymi zapytaniami i synchronicznymi transakcjami w wątku UI; cel: odpowiedź w 16 ms (jedna klatka), próg Nielsena 100 ms. Znaczna część kodu UI to złączenia i grupowania. **Dla projektu:** najbliższa praca naukowa (lokalna baza SQL + Local-First); budżet 16 ms do interpretacji H1 i pauzy checkpointu; argument za zapytaniami reaktywnymi i przeciw całkowitej rezygnacji ze złączeń. **Zadania:** todo 4.10; future 5.1.

- **[D5]** Zhang G. *Zeta-Lite: A Concurrent, Branchable In-Browser SQL Database for Agentic Memory.* arXiv:2609.01818, wrzesień 2026 (preprint bez recenzji).
  **Plik:** [papers/zastosowane/2026-zeta-lite-in-browser-sql-database.pdf](papers/zastosowane/2026-zeta-lite-in-browser-sql-database.pdf).
  — Serwerowy silnik Zeta skompilowany do `wasm32-unknown-unknown` z `wasm-bindgen` (2,87 MB gzip): MVCC w jednym wątku, rozgałęzianie bazy, dialekt PostgreSQL; 268–315 tys. odczytów punktowych/s (ok. 3,2–3,7 µs), w przeglądarce 5–15% poniżej natywnej. Trwałość tylko przez snapshot do OPFS na żądanie – autor przyznaje, że zmiany po ostatnim snapshocie giną. Kod zamknięty, jeden autor. **Dla projektu:** najnowszy system o tej samej architekturze „jeden rdzeń dla serwera i przeglądarki”; kontrast dla przyrostowego WAL (H5) i cel dla odczytu po kluczu. Cytować ostrożnie. **Zadania:** todo 2.4, 4.1.

## 4. Local-First, CRDT i synchronizacja

- **[L1]** Kleppmann M., Wiggins A., van Hardenberg P., McGranaghan M. *Local-First Software: You Own Your Data, in spite of the Cloud.* Onward! 2019, s. 154–178. DOI: [10.1145/3359591.3359737](https://doi.org/10.1145/3359591.3359737).
  **Plik:** [papers/potwierdzone/2019-kleppmann-local-first-software.pdf](papers/potwierdzone/2019-kleppmann-local-first-software.pdf).
  — Siedem ideałów oprogramowania local-first (brak spinnerów, wiele urządzeń, sieć opcjonalna, współpraca, długowieczność danych, prywatność, kontrola użytkownika); CRDT jako fundament; magazyn przeglądarki bywa czyszczony razem z ciasteczkami. **Dla projektu:** główna rama oceny architektury ([literature.md](literature.md)). **Zadania:** todo 1.5; future 1.3, 4.3.

- **[L2]** Shapiro M., Preguiça N., Baquero C., Zawirski M. *A comprehensive study of Convergent and Commutative Replicated Data Types.* INRIA, RR-7506, 2011.
  **Plik:** [papers/potwierdzone/2011-shapiro-comprehensive-study-crdt.pdf](papers/potwierdzone/2011-shapiro-comprehensive-study-crdt.pdf) (kopia z serwisu kursu IISc; oryginał: HAL inria-00555588).
  — Formalne podstawy CRDT i katalog typów, m.in. rejestr LWW, liczniki PN, zbiory z tombstone'ami. **Dla projektu:** model replikacji Flat-SQL to mapa rejestrów LWW indeksowana kluczem – tu jest jego formalna definicja i dowód zbieżności. **Zadania:** todo 1.9; future 2.5.


- **[L27]** Go Y., Agrawal N., Aranya A., Ungureanu C. *Reliable, Consistent, and Efficient Data Sync for Mobile Apps.* USENIX FAST 2015, s. 359–372. *(nowa)*
  **Plik:** [papers/potwierdzone/2015-go-simba-data-sync-mobile-apps.pdf](papers/potwierdzone/2015-go-simba-data-sync-mobile-apps.pdf) (otwarty dostęp USENIX).
  — Badanie popularnych aplikacji mobilnych z usługami synchronizacji (Dropbox, Parse, Kinvey): przy przerwie w sieci, zamknięciu aplikacji lub zaniku zasilania aplikacje tracą lub psują dane. Simba: tabela (`sTable`) z wierszem jako jednostką spójności i atomowości – lokalnie i w synchronizacji; polityka synchronizacji per tabela; konflikty zgłaszane aplikacji. Autorzy świadomie **nie** dają atomowej synchronizacji wielu wierszy, uznając ją dla większości aplikacji za zbędną i kosztowną. **Dla projektu:** empiryczne uzasadnienie wbudowanej synchronizacji z trwałym outboksem oraz gwarancji „atomowy wiersz między replikami”. **Zadania:** todo 0.1; future 12.5, 12.7.

- **[L28]** Almeida P. S. *Approaches to Conflict-free Replicated Data Types.* ACM Computing Surveys 57(2), art. 51, 2024. DOI: [10.1145/3695249](https://doi.org/10.1145/3695249); arXiv:2310.18220. *(nowa)*
  **Plik:** [papers/potwierdzone/2024-almeida-approaches-to-crdts.pdf](papers/potwierdzone/2024-almeida-approaches-to-crdts.pdf).
  — Aktualny przegląd podejść do CRDT: oparte na stanie, na operacjach, delta-state i „czysto operacyjne”, z kompaktowaniem po osiągnięciu stabilności przyczynowej. **Dla projektu:** jedno recenzowane źródło dla trzech tez: outbox wysyłający bieżący stan zmienionych wierszy to synchronizacja delta-state; tombstone można usunąć po stabilności przyczynowej; zbieżność wynika z łączności, przemienności i idempotencji scalania. Zastępuje cztery wcześniejsze pozycje (§13). **Zadania:** future 12.6.

## 5. Replikacja, zegary i praca offline

- **[C2]** Kulkarni S. S., Demirbas M., Madeppa D., Avva B., Leone M. *Logical Physical Clocks and Consistent Snapshots in Globally Distributed Databases.* OPODIS 2014 (LNCS 8878).
  **Plik:** [papers/potwierdzone/2014-kulkarni-hybrid-logical-clocks.pdf](papers/potwierdzone/2014-kulkarni-hybrid-logical-clocks.pdf).
  — Zegar HLC: przyczynowość jak zegar logiczny, wartość blisko czasu fizycznego, postać 64-bitowa (48 bitów czasu + 16 bitów licznika). Sekcja 4: wiadomości odsuwające zegar logiczny zbyt daleko od fizycznego są **ignorowane**, a incydent logowany i zgłaszany. **Dla projektu:** `sync.rs` realizuje postać zwartą bez tej reguły. **Zadania:** todo 1.1, 1.9.

- **[C3]** Terry D. B., Theimer M. M., Petersen K., Demers A. J., Spreitzer M. J., Hauser C. H. *Managing Update Conflicts in Bayou, a Weakly Connected Replicated Storage System.* SOSP 1995, s. 172–182.
  **Plik:** [papers/zastosowane/1995-terry-bayou-update-conflicts.pdf](papers/zastosowane/1995-terry-bayou-update-conflicts.pdf).
  — Replikacja dla słabo połączonych urządzeń: zapisy do dowolnej repliki, zapisy wstępne i zatwierdzone, serwer główny ustalający kolejność, anti-entropy, sprawdzanie zależności i procedury scalania aplikacji. **Dla projektu:** protoplasta architektury „lokalne zapisy + serwer porządkujący log” i wzorzec zamiarów zatwierdzanych przez serwer. **Zadania:** future §2.

- **[C7]** Helland P. *Life beyond Distributed Transactions: an Apostate's Opinion.* CIDR 2007.
  **Plik:** [papers/potwierdzone/2007-helland-life-beyond-distributed-transactions.pdf](papers/potwierdzone/2007-helland-life-beyond-distributed-transactions.pdf).
  — Encja z unikalnym kluczem jako zakres serializowalności; transakcje atomowe nie obejmują wielu encji; wiadomości muszą być idempotentne. **Dla projektu:** najmocniejsze uzasadnienie płaskiego modelu i idempotentnych rund synchronizacji. **Zadania:** todo 0.1.

- **[C9]** Gray J., Helland P., O'Neil P., Shasha D. *The Dangers of Replication and a Solution.* ACM SIGMOD 1996, s. 173–182. DOI: [10.1145/233269.233330](https://doi.org/10.1145/233269.233330). *(nowa)*
  **Plik:** [papers/potwierdzone/1996-gray-dangers-of-replication.pdf](papers/potwierdzone/1996-gray-dangers-of-replication.pdf) (kopia z grupy czytelniczej Boston University).
  — Replikacja „zapis wszędzie” jest niestabilna przy wzroście skali (10× więcej węzłów i ruchu daje 1000× więcej zakleszczeń lub uzgodnień). Schemat ze znacznikami czasu („nowszy znacznik wygrywa”) zbiega się, ale **gubi aktualizacje**; aktualizacje przemienne (np. dodawanie stałej do liczby) tego problemu nie mają. Rozwiązanie: replikacja dwuwarstwowa – węzły mobilne wykonują transakcje wstępne na lokalnej kopii, a węzły bazowe, właściciele kopii głównych, wykonują je ponownie jako transakcje bazowe z kryterium akceptacji i informują o odrzuceniu. **Dla projektu:** w jednej pracy uzasadnienie i granica LWW (utracone aktualizacje), argument za licznikami `MERGE ADD`, model tabel prowadzonych przez serwer, zamiarów zatwierdzanych przez serwer i węzłów edge jako warstwy bazowej. **Zadania:** future 2.1–2.5, §7, 7.2, 12.3.

- **[C10]** Kistler J. J., Satyanarayanan M. *Disconnected Operation in the Coda File System.* ACM TOCS 10(1), 1992, s. 3–25. DOI: [10.1145/146941.146942](https://doi.org/10.1145/146941.146942) (wersja konferencyjna: SOSP 1991). *(nowa)*
  **Plik:** [papers/potwierdzone/1991-kistler-satyanarayanan-coda-disconnected-operation.pdf](papers/potwierdzone/1991-kistler-satyanarayanan-coda-disconnected-operation.pdf) (wersja SOSP, kopia z kursu University of Michigan).
  — Klasyczna praca o pracy bez połączenia: klient gromadzi zawczasu potrzebne dane (hoarding, z priorytetami i wymianą w pamięci podręcznej), offline zapisuje zmiany w trwałym logu (w pamięci odtwarzalnej z WAL), log skraca – nowszy zapis pliku unieważnia wcześniejsze wpisy – a po połączeniu odtwarza go na serwerze w jednej transakcji z wykrywaniem konfliktów; tymczasowe identyfikatory obiektów zastępowane są przy reintegracji stałymi. W praktyce konfliktów prawie nie było. **Dla projektu:** akademicki odpowiednik outboksu (scalanie kolejnych zmian wiersza, trwałość po restarcie, idempotentna reintegracja), wzór trybu pamięci podręcznej i obsługi kluczy tworzonych offline. **Zadania:** future 1.2, 7.1, 12.1, 12.4.


## 6. Składowanie, WAL i odtwarzanie po awarii

- **[S1]** DeWitt D. J., Katz R. H., Olken F., Shapiro L. D., Stonebraker M. R., Wood D. *Implementation Techniques for Main Memory Database Systems.* SIGMOD 1984, s. 1–8.
  **Plik:** [papers/potwierdzone/1984-dewitt-main-memory-database-systems.pdf](papers/potwierdzone/1984-dewitt-main-memory-database-systems.pdf).
  — Struktury i odtwarzanie dla baz w pamięci; pierwszy drukowany opis **group commit** (jeden zapis logu dla grupy transakcji). **Dla projektu:** źródło group commitu w `FlatDB.flush()` i modelu „log + checkpoint”.

- **[S4]** Stonebraker M., Madden S., Abadi D. J., Harizopoulos S., Hachem N., Helland P. *The End of an Architectural Era (It's Time for a Complete Rewrite).* VLDB 2007, s. 1150–1160.
  **Plik:** [papers/potwierdzone/2007-stonebraker-end-of-an-architectural-era.pdf](papers/potwierdzone/2007-stonebraker-end-of-an-architectural-era.pdf).
  — H-Store: dane w pamięci, wykonanie jednowątkowe, specjalizacja zamiast „jednego rozmiaru dla wszystkich”; logowanie, blokady, latche i bufor stron to większość pracy klasycznego silnika. **Dla projektu:** uzasadnienie wąsko wyspecjalizowanego, jednowątkowego silnika w pamięci; Flat-SQL nie płaci za blokady i bufor stron, płaci za logowanie (undo log + WAL + outbox – H4).

- **[S6]** Malviya N., Weisberg A., Madden S., Stonebraker M. *Rethinking Main Memory OLTP Recovery.* ICDE 2014, s. 604–615.
  **Plik:** [papers/zastosowane/2014-malviya-rethinking-main-memory-oltp-recovery.pdf](papers/zastosowane/2014-malviya-rethinking-main-memory-oltp-recovery.pdf).
  — W VoltDB logowanie poleceń daje ok. 1,5× większą przepustowość niż logowanie fizjologiczne w stylu ARIES, kosztem dłuższego odtwarzania i wymogu determinizmu. **Dla projektu:** uzasadnienie logu fizycznego – te same wpisy WAL służą synchronizacji, a odtworzenie nie zależy od zegara ani wyników zapytań.

- **[S8]** Pillai T. S., Chidambaram V., Alagappan R., Al-Kiswany S., Arpaci-Dusseau A. C., Arpaci-Dusseau R. H. *All File Systems Are Not Created Equal: On the Complexity of Crafting Crash-Consistent Applications.* OSDI 2014, s. 433–448.
  **Plik:** [papers/potwierdzone/2014-pillai-all-file-systems-are-not-created-equal.pdf](papers/potwierdzone/2014-pillai-all-file-systems-are-not-created-equal.pdf).
  — Narzędzia BOB i ALICE: poprawność protokołów zapisu zależy od subtelnych właściwości systemu plików; 60 podatności w 11 dojrzałych aplikacjach, w tym brak `fsync` katalogu nadrzędnego po utworzeniu lub zmianie nazwy pliku. **Dla projektu:** dotyczy sekwencji `snapshot.tmp → rename → obcięcie WAL` w `FileStorage` i braku `fsync` w `FsStore`. **Zadania:** todo 1.2, 1.3, 1.6.

- **[S10]** Cao T., Vaz Salles M., Sowell B., Yue Y., Demers A., Gehrke J., White W. *Fast Checkpoint Recovery Algorithms for Frequently Consistent Applications.* ACM SIGMOD 2011, s. 265–276. DOI: [10.1145/1989323.1989352](https://doi.org/10.1145/1989323.1989352). *(nowa)*
  **Plik:** [papers/potwierdzone/2011-cao-fast-checkpoint-recovery.pdf](papers/potwierdzone/2011-cao-fast-checkpoint-recovery.pdf) (kopia z kursu Georgia Tech).
  — Checkpointy aplikacji w pamięci o częstych punktach spójności – na przykładzie gier MMO (stan spójny na granicy „ticku”) i H-Store. Porównuje „naiwny snapshot” (synchroniczna kopia całego stanu – skok latencji), kopiowanie przy aktualizacji i dwa nowe algorytmy bez blokad i kopii masowych, które kosztem dodatkowej pamięci dają niemal stałą latencję. **Dla projektu:** checkpoint Flat-SQL to dziś naiwny snapshot (35–49 ms, czyli 2–3 klatki), a gry są pierwszym zastosowaniem z prezentacji – praca daje gotowe kierunki i metrykę (najdłuższa pauza). **Zadania:** todo 2.6, 4.10.

## 7. Model danych i zapytania


- **[M2]** Armbrust M., Curtis K., Kraska T., Fox A., Franklin M. J., Patterson D. A. *PIQL: Success-Tolerant Query Processing in the Cloud.* PVLDB 5(3), 2011, s. 181–192. DOI: [10.14778/2078331.2078334](https://doi.org/10.14778/2078331.2078334); arXiv:1111.7166. *(nowa)*
  **Plik:** [papers/potwierdzone/2011-armbrust-piql-scale-independence.pdf](papers/potwierdzone/2011-armbrust-piql-scale-independence.pdf).
  — Deklaratywny język, który dla każdego zapytania wylicza górne ograniczenie liczby operacji na magazynie klucz–wartość („niezależność od skali”); zapytania bez ograniczenia są odrzucane przy kompilacji. Klasa zapytań o stałym koszcie to zapytania z `LIMIT` bez złączeń albo ze złączeniem tylko po unikalnym kluczu głównym. Autorzy krytykują języki, które dla skalowalności po prostu usuwają złączenia (np. GQL). **Dla projektu:** precyzuje tezę o „przewidywalnym wykonaniu” – płaski model daje przewidywalny plan, ale ograniczony koszt mają tylko zapytania po kluczu lub indeksie z limitem; pokazuje też, że złączenie po kluczu głównym nie psuje przewidywalności. **Zadania:** todo 0.1.

## 8. Edge computing

Jedyna praca tej kategorii, [E1], przeszła 28.10.2026 poza rdzeń.

## 9. Metodyka pomiarów

- **[B1]** Georges A., Buytaert D., Eeckhout L. *Statistically Rigorous Java Performance Evaluation.* OOPSLA 2007, s. 57–76.
  **Plik:** [papers/potwierdzone/2007-georges-statistically-rigorous-performance-evaluation.pdf](papers/potwierdzone/2007-georges-statistically-rigorous-performance-evaluation.pdf).
  — Najlepszy wynik lub pojedynczy przebieg prowadzą w środowiskach z JIT do błędnych wniosków; zalecane wielokrotne uruchomienia maszyny wirtualnej i przedziały ufności. **Dla projektu:** jak wzmocnić H2–H5 (szum ±30% opisany w scenariuszu). **Zadania:** todo 4.2.

- **[B3]** Raasveldt M., Holanda P., Gubner T., Mühleisen H. *Fair Benchmarking Considered Difficult: Common Pitfalls In Database Performance Testing.* DBTest 2018. DOI: [10.1145/3209950.3209955](https://doi.org/10.1145/3209950.3209955).
  **Plik:** [papers/potwierdzone/2018-raasveldt-fair-benchmarking-considered-difficult.pdf](papers/potwierdzone/2018-raasveldt-fair-benchmarking-considered-difficult.pdf).
  — Pułapki: brak powtarzalności, nieoptymalny system porównywany, „jabłka i pomarańcze”, nadmierne strojenie, przebiegi zimne i gorące, niepoprawne wyniki. **Dla projektu:** lista kontrolna dla [benchmark-scenario.md](../research/benchmark-scenario.md) ([porównanie §8](literature.md)). **Zadania:** todo 0.1, 4.1, 4.3, 4.6.


## 10. Prywatność

- **[V1]** Cavoukian A. *Privacy by Design: The 7 Foundational Principles – Implementation and Mapping of Fair Information Practices.* Information and Privacy Commissioner of Ontario.
  **Plik:** [papers/zastosowane/2009-cavoukian-privacy-by-design-principles.pdf](papers/zastosowane/2009-cavoukian-privacy-by-design-principles.pdf) (kopia z serwisu kursu University of Waterloo).
  — Siedem zasad Privacy by Design (prywatność domyślnie, ochrona w całym cyklu życia, przejrzystość) z mapowaniem na zasady FIP. Uzupełnienie prawne: art. 25 RODO. **Dla projektu:** źródło pojęcia z README; pozwala ocenić, które zasady prototyp spełnia ([porównanie §2.4](literature.md)). **Zadania:** todo 0.1; future 4.4, 5.6.

---

## 11. Do sprawdzenia i pobrania

| ID | Publikacja | Stan | Gdzie szukać |
|---|---|---|---|
| **[X4]** | Kleppmann M. *Designing Data-Intensive Applications* (wyd. 1 2017, wyd. 2 2026) | książka komercyjna | biblioteka uczelni, wydanie polskie Helion 2018 |
| **[X5]** | Saadbye J., Gondelman L., Albano M. *Teilen-SQL: A library for building local-first software with SQLite and version control.* PLF+PLAID @ ECOOP 2025 | tylko streszczenie wystąpienia – SQLite w przeglądarce, komórki jako rejestry LWW | [strona wystąpienia](https://2025.ecoop.org/details/plf-plaid-2025-papers/7/Teilen-SQL-A-library-for-building-local-first-software-with-SQLite-and-version-contr); zapytać autorów |
| **[X6]** | Niinikoski J., Fagerholm F. *On the Suitability of Local-First Architecture From Web Application Developers' Perspective* (2026) | nie znaleziono tekstu; są tylko materiały uzupełniające (Zenodo, DOI [10.5281/zenodo.20121788](https://doi.org/10.5281/zenodo.20121788)) | ACM DL / Google Scholar po publikacji; rozwinięcie [L20] |

## 12. Dokumentacja platform i źródła branżowe (nierecenzowane)

Nie są to publikacje naukowe, ale opisują zachowanie platform, od którego zależy poprawność projektu – w pracy cytuje się je jako dokumentację techniczną.

- **[Q1]** Chrome for Developers: [A change to the default durability mode in IndexedDB](https://developer.chrome.com/blog/indexeddb-durability-mode-now-defaults-to-relaxed) – od Chrome 121 domyślna trwałość transakcji IndexedDB to `relaxed` (zdarzenie `complete` po przekazaniu danych do bufora systemu operacyjnego).
- **[Q2]** WebKit: [Full Third-Party Cookie Blocking and More](https://webkit.org/blog/10218/full-third-party-cookie-blocking-and-more/) (2020) – Safari usuwa m.in. IndexedDB po 7 dniach używania przeglądarki bez interakcji z witryną; aplikacje dodane do ekranu głównego mają własny licznik.
- **[Q3]** WebKit: [Updates to Storage Policy](https://webkit.org/blog/14403/updates-to-storage-policy/) (Safari 17, 2023) – limity miejsca i zasady usuwania; tryb trwały przez `navigator.storage.persist()`.
- **[Q4]** MDN: [Storage quotas and eviction criteria](https://developer.mozilla.org/docs/Web/API/Storage_API/Storage_quotas_and_eviction_criteria) – magazyn „best-effort” vs „persistent”.
- **[Q5]** web.dev: [Making your website cross-origin isolated using COOP and COEP](https://web.dev/articles/coop-coep) – rozdzielczość `performance.now()`: 100 µs domyślnie, 5 µs w kontekście izolowanym.
- **[Q6]** webassembly.org: [Wasm 3.0 Completed](https://webassembly.org/news/2025-09-17-wasm-3.0) (2025) – memory64, GC, wyjątki; SpiderMonkey: [Is Memory64 actually worth using?](https://spidermonkey.dev/blog/2025/01/15/is-memory64-actually-worth-using.html) – 64-bitowa pamięć jest wolniejsza, warto ją włączać tylko przy potrzebie > 4 GiB.
- **[Q7]** SQLite: [persistence options for the WASM build (OPFS, opfs-sahpool, kvvfs)](https://sqlite.org/wasm/doc/tip/persistence.md).
- **[Q8]** RxDB: [LocalStorage vs. IndexedDB vs. Cookies vs. OPFS vs. WASM-SQLite](https://rxdb.info/articles/localstorage-indexeddb-cookies-opfs-sqlite-wasm.html); PGlite: [benchmarks](https://pglite.dev/benchmarks) – pomiary producentów (konflikt interesów); jedyne dostępne dane o wydajności IndexedDB i OPFS.
- **[Q11]** Repozytoria systemów porównywanych w [wasm-engines-comparison.md](../research/wasm-engines-comparison.md): sql.js, wa-sqlite, PGlite, DuckDB-Wasm, GlueSQL, Turso, cr-sqlite, ElectricSQL, PowerSync.
- **[Q12]** Wiele kart: MDN [Web Locks API](https://developer.mozilla.org/docs/Web/API/Web_Locks_API) i [SharedWorker](https://developer.mozilla.org/docs/Web/API/SharedWorker) (Baseline 2026 – Chrome na Androidzie przywrócił `SharedWorker` w wersji 148, kwiecień 2026); wdrożenie Notion: [How we sped up Notion in the browser with WASM SQLite](https://www.notion.com/blog/how-we-sped-up-notion-in-the-browser-with-wasm-sqlite).
- **[Q13]** WebAssembly JS String Builtins: [MDN](https://developer.mozilla.org/docs/WebAssembly/JavaScript_builtins), [przegląd W3C TAG](https://tag-github-bot.w3.org/gh/w3ctag/design-reviews/940) – mechanizm dla języków, które chcą używać łańcuchów JS jako własnych (GC); Rust trzyma tekst w pamięci liniowej, więc kopiowania nie unika.
- Źródła o wdrożeniach (Notion, Actual Budget, Linear, Community Health Toolkit, PowerSync, LiteFS, Turso i in.): [research/use-cases.md](../research/use-cases.md#8-źródła).
