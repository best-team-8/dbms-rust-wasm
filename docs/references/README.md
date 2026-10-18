# Baza wiedzy – literatura do projektu Flat-SQL

Zbiór publikacji naukowych, specyfikacji i źródeł technicznych związanych z projektem: silnik SQL w Rust kompilowany do WebAssembly, architektura Local-First, synchronizacja replik, warstwa składowania w przeglądarce oraz metodyka benchmarków. Stan na **18 października 2026**.

| Plik / katalog | Zawartość |
|---|---|
| [conclusions.md](conclusions.md) | wnioski z analizy projektu i przeglądu literatury: pozycja na tle innych rozwiązań, kierunek rozwoju, zastosowania, słowniczek |
| [literature-comparison.md](literature-comparison.md) | porównanie implementacji Flat-SQL z założeniami i wynikami badań, lista rekomendacji |
| [notes/haas-2017-webassembly.md](notes/haas-2017-webassembly.md) | notatka do pracy poleconej przez promotora: Haas i in. (2017) |
| [notes/ddia-kleppmann.md](notes/ddia-kleppmann.md) | notatka do książki poleconej przez promotora: Kleppmann (2017), z mapą rozdziałów na moduły projektu |
| [search-methodology.md](search-methodology.md) | jak przeprowadzono wyszukiwanie, kryteria doboru, ograniczenia i luki |
| `papers/` | 77 plików PDF (ok. 94 MB) – wersje otwarte, autorskie lub udostępnione przez uczelnie |
| `ddia-references/` | listy bibliografii do 12 rozdziałów DDIA (repozytorium autora, licencja CC BY-NC 4.0) |

> **Prawa autorskie.** Pliki w `papers/` służą do własnej nauki. Wiele z nich to wersje autorskie z dopiskiem „for personal use, not for redistribution” (np. ACM, IEEE) albo kopie z serwisów kursów uniwersyteckich. Nie należy publikować katalogu `papers/` w publicznym repozytorium – wystarczy ten indeks z odnośnikami (DOI / adres źródła). W pracy dyplomowej cytuje się wersję wydawniczą (DOI), nie lokalną kopię.

## Jak czytać wpisy

Każdy wpis ma identyfikator (np. **[W3]**) używany w [literature-comparison.md](literature-comparison.md), pełny opis bibliograficzny, link do lokalnej kopii (jeśli udało się ją pobrać), krótkie streszczenie i wskazanie, której części projektu dotyczy. Oznaczenia hipotez H1–H6 pochodzą z [docs/research/benchmark-scenario.md](../research/benchmark-scenario.md).

Kategorie: [P] polecone przez promotora · [W] WebAssembly · [D] bazy danych w przeglądarce · [L] Local-First i CRDT · [C] zegary i replikacja · [S] składowanie i odtwarzanie · [M] model danych · [E] edge computing · [B] metodyka pomiarów · [V] prywatność · [X] źródła niepobrane · [Q] dokumentacja platform i źródła branżowe.

---

## 1. Prace polecone przez promotora

- **[P1]** Kleppmann M. *Designing Data-Intensive Applications: The Big Ideas Behind Reliable, Scalable, and Maintainable Systems.* O'Reilly Media, 2017. ISBN 978-1-449-37332-0.
  Wydanie polskie: *Przetwarzanie danych w dużej skali. Niezawodność, skalowalność i łatwość konserwacji systemów*, Helion, Gliwice 2018 (ISBN 978-83-283-4065-7 wg Goodreads; katalog TUM podaje 978-83-283-4066-4 – sprawdzić na egzemplarzu).
  Wydanie 2: Kleppmann M., Riccomini C., O'Reilly, luty–marzec 2026.
  **Plik:** brak (książka komercyjna). Bibliografia rozdziałów: [ddia-references/](ddia-references/). Notatka z mapą rozdziałów: [notes/ddia-kleppmann.md](notes/ddia-kleppmann.md).
  — Podręcznik systemów danych: struktury składowania (log, LSM, B-drzewa, bazy w pamięci), kodowanie i ewolucja formatów, replikacja (w tym klienci offline jako replikacja multi-leader), rozwiązywanie konfliktów (LWW, CRDT), transakcje, zawodność zegarów, uporządkowanie zdarzeń, strumienie i log zmian. **Dla projektu:** teoretyczna podstawa niemal każdej decyzji z [architecture.md](../architecture.md) – WAL + snapshot, wykonanie szeregowe, LWW, outbox, log synchronizacji z kursorami.

- **[P2]** Haas A., Rossberg A., Schuff D. L., Titzer B. L., Holman M., Gohman D., Wagner L., Zakai A., Bastien J. F. *Bringing the Web up to Speed with WebAssembly.* PLDI 2017, ACM, s. 185–200. DOI: [10.1145/3062341.3062363](https://doi.org/10.1145/3062341.3062363).
  **Plik:** [papers/2017-haas-bringing-the-web-up-to-speed-with-webassembly-pldi.pdf](papers/2017-haas-bringing-the-web-up-to-speed-with-webassembly-pldi.pdf) (wersja autorska, MPI-SWS). Notatka: [notes/haas-2017-webassembly.md](notes/haas-2017-webassembly.md).
  — Artykuł wprowadzający WebAssembly: cele projektowe (bezpieczny, szybki, przenośny, zwarty kod), maszyna stosowa z typami sprawdzanymi przy walidacji, pamięć liniowa izolowana od kodu i stosu silnika, formalna semantyka. W pomiarach PolyBenchC 7 z 24 programów mieściło się w 10% czasu kodu natywnego, a prawie wszystkie w 2×; kod był średnio o 33,7% szybszy niż asm.js i miał 62,5% rozmiaru asm.js. **Dla projektu:** punkt odniesienia dla H2 (narzut WASM), rozmiaru modułu i argumentu „izolowanej piaskownicy” w Privacy by Design.

- **[P2a]** Rossberg A., Titzer B. L., Haas A., Schuff D. L., Gohman D., Wagner L., Zakai A., Bastien J. F., Holman M. *Bringing the Web up to Speed with WebAssembly.* Communications of the ACM 61(12), 2018, s. 107–115. DOI: [10.1145/3282510](https://doi.org/10.1145/3282510).
  **Plik:** [papers/2018-rossberg-bringing-the-web-up-to-speed-with-webassembly-cacm.pdf](papers/2018-rossberg-bringing-the-web-up-to-speed-with-webassembly-cacm.pdf).
  — Skrócona, przystępniejsza wersja [P2] dla CACM. Wygodna do cytowania w części wstępnej pracy.

## 2. WebAssembly – wydajność, bezpieczeństwo, ewolucja standardu

- **[W1]** Jangda A., Powers B., Berger E. D., Guha A. *Not So Fast: Analyzing the Performance of WebAssembly vs. Native Code.* USENIX ATC 2019.
  **Plik:** [papers/2019-jangda-not-so-fast-webassembly-vs-native.pdf](papers/2019-jangda-not-so-fast-webassembly-vs-native.pdf).
  — Pierwsze duże porównanie WASM z kodem natywnym na SPEC CPU (przez Browsix-Wasm): średnio 1,55× wolniej w Chrome i 1,45× w Firefoksie, maksymalnie 2,5× / 2,08×. Przyczyny: ok. 2× więcej odczytów i zapisów pamięci (presja na rejestry), więcej skoków (m.in. sprawdzanie przepełnienia stosu przy każdym wywołaniu, kontrola wywołań pośrednich), większy kod. Krytykuje [P2] za oparcie się na małych jądrach PolyBenchC. **Dla projektu:** najważniejsze odniesienie dla H2; wyjaśnia obserwację z [benchmark-results.md §5](../research/benchmark-results.md) (duża ramka stosu rekurencyjnego `eval`).

- **[W2]** Lehmann D., Kinder J., Pradel M. *Everything Old is New Again: Binary Security of WebAssembly.* USENIX Security 2020.
  **Plik:** [papers/2020-lehmann-binary-security-of-webassembly.pdf](papers/2020-lehmann-binary-security-of-webassembly.pdf).
  — Pokazuje, że wewnątrz pamięci liniowej brakuje zabezpieczeń znanych z kodu natywnego (kanarki stosu, ASLR, strony ochronne, ochrona metadanych sterty, dane tylko do odczytu), więc błędy z C/C++ są w WASM znów wykorzystywalne. **Dla projektu:** precyzuje, co naprawdę daje „izolowana piaskownica” – chroni hosta przed modułem, ale nie dane modułu przed błędami w nim samym; argument za Rustem bez `unsafe` (rdzeń i `flatsql-wasm` go nie używają).

- **[W3]** Hilbig A., Lehmann D., Pradel M. *An Empirical Study of Real-World WebAssembly Binaries: Security, Languages, Use Cases.* WWW 2021, s. 2696–2708. DOI: [10.1145/3442381.3450138](https://doi.org/10.1145/3442381.3450138).
  **Plik:** [papers/2021-hilbig-empirical-study-real-world-webassembly-binaries.pdf](papers/2021-hilbig-empirical-study-real-world-webassembly-binaries.pdf).
  — Analiza 8461 binariów WASM z repozytoriów, menedżerów pakietów i stron WWW: języki źródłowe, zastosowania, właściwości bezpieczeństwa. **Dla projektu:** kontekst, jak WASM jest używany w praktyce; uzupełnia [W2].

- **[W4]** Bosamiya J., Lim W. S., Parno B. *Provably-Safe Multilingual Software Sandboxing using WebAssembly.* USENIX Security 2022, s. 1975–1992.
  **Plik:** [papers/2022-bosamiya-provably-safe-sandboxing-webassembly.pdf](papers/2022-bosamiya-provably-safe-sandboxing-webassembly.pdf).
  — Dwie drogi do dowodliwie bezpiecznej piaskownicy WASM: kompilator z dowodem formalnym (vWasm) i tłumaczenie semantyki WASM do bezpiecznego Rusta (rWasm), przy wydajności zbliżonej do klasycznych podejść. **Dla projektu:** literatura do tezy, że bezpieczeństwo piaskownicy zależy od implementacji środowiska uruchomieniowego.

- **[W5]** Zhang Y., Liu M., Wang H., Ma Y., Huang G., Liu X. *Research on WebAssembly Runtimes: A Survey.* arXiv:2404.12621 (v2, 2024); wersja czasopismowa ACM TOSEM, 2025, DOI: [10.1145/3714465](https://doi.org/10.1145/3714465).
  **Plik:** [papers/2024-zhang-webassembly-runtimes-survey.pdf](papers/2024-zhang-webassembly-runtimes-survey.pdf).
  — Systematyczny przegląd ok. 100 prac o środowiskach uruchomieniowych WASM (projekt, testowanie, analiza, zastosowania, w tym SQLite w TEE). **Dla projektu:** gotowa mapa literatury do rozdziału o WebAssembly.

- **[W6]** Döllerer L., Engelke A. *Performant Bounds Checking for 64-Bit WebAssembly.* VMIL 2024. DOI: [10.1145/3689490.3690400](https://doi.org/10.1145/3689490.3690400).
  **Plik:** [papers/2024-dollerer-bounds-checking-64bit-webassembly.pdf](papers/2024-dollerer-bounds-checking-64bit-webassembly.pdf).
  — Programowe sprawdzanie granic przy 64-bitowej pamięci WASM (memory64) kosztuje ponad 100% czasu wykonania; dwupoziomowe strony ochronne obniżają narzut do 12,7%. **Dla projektu:** koszt wyjścia poza 4 GiB pamięci `wasm32` (dane w całości w pamięci).

- **[W7]** Baek D., Getz J., Sim Y., Lehmann D., Titzer B. L., Ryu S., Pradel M. *Wasm-R3: Record-Reduce-Replay for Realistic and Standalone WebAssembly Benchmarks.* PACMPL 8 (OOPSLA2), art. 347, 2024. DOI: [10.1145/3689787](https://doi.org/10.1145/3689787).
  **Plik:** [papers/2024-wasm-r3-realistic-webassembly-benchmarks.pdf](papers/2024-wasm-r3-realistic-webassembly-benchmarks.pdf).
  — Nagrywanie i odtwarzanie rzeczywistych aplikacji WASM jako samodzielnych benchmarków (zestaw 27 aplikacji); wskazuje, że PolyBenchC i libsodium nie są reprezentatywne. **Dla projektu:** argument za benchmarkiem opartym na realnym scenariuszu (demo), a nie tylko syntetycznym.

- **[W8]** He N., Cao S., Wang H., Guo Y., Luo X. *The Promise and Pitfalls of WebAssembly: Perspectives from the Industry.* arXiv:2503.21240, 2025.
  **Plik:** [papers/2025-promise-and-pitfalls-of-webassembly.pdf](papers/2025-promise-and-pitfalls-of-webassembly.pdf).
  — Największy dotąd zbiór binariów WASM używanych w przemyśle i ich charakterystyka; krytykuje dobór danych w [W3]. **Dla projektu:** aktualny (2025) obraz zastosowań WASM.

- **[W9]** Corrias L., Pisu L., Maiorca D., Giacinto G. *An Analysis of Modern Web Security Vulnerabilities Inside WebAssembly Applications.* arXiv:2603.09426, marzec 2026.
  **Plik:** [papers/2026-corrias-web-vulnerabilities-inside-webassembly.pdf](papers/2026-corrias-web-vulnerabilities-inside-webassembly.pdf).
  — Błędy binarne w module WASM (przepełnienie bufora, use-after-free) mogą prowadzić do klasycznych ataków webowych: SQL injection, XS-Leaks, SSTI. **Dla projektu:** najnowsze argumenty, że piaskownica nie zastępuje bezpiecznej implementacji; istotne dla silnika SQL wywoływanego z JS.

- **[W10]** WebAssembly Community Group (red. Rossberg A.). *WebAssembly Specification, Release 3.0.* Kompilacja z 13.10.2026. Wasm 3.0 ogłoszono 17.09.2025 ([webassembly.org](https://webassembly.org/news/2025-09-17-wasm-3.0)).
  **Plik:** [papers/webassembly-core-specification.pdf](papers/webassembly-core-specification.pdf).
  — Aktualna specyfikacja rdzenia: m.in. memory64, wiele pamięci, GC, obsługa wyjątków, wywołania ogonowe, relaxed SIMD; obok niej JS String Builtins w JS API. **Dla projektu:** stan standardu na czas pisania pracy (w [P2] opisano MVP z 2017 r.).

- **[W11]** Yan Y., Tu T., Zhao L., Zhou Y., Wang W. *Understanding the Performance of WebAssembly Applications.* IMC 2021. DOI: [10.1145/3487552.3487827](https://doi.org/10.1145/3487552.3487827).
  **Plik:** [papers/2021-yan-understanding-performance-webassembly-applications.pdf](papers/2021-yan-understanding-performance-webassembly-applications.pdf) (ACM DL, wersja ogólnodostępna).
  — Systematyczne porównanie WASM z JavaScriptem w Chrome, Firefoksie i Edge na trzech zbiorach programów i przy różnych poziomach optymalizacji (O1, O2, Oz, Ofast). Wnioski: optymalizacje LLVM nie są dostrojone do WASM i bywają nieskuteczne, a nawet szkodliwe; JIT wyraźnie przyspiesza JS, ale nie WASM; wyniki mocno zależą od środowiska; WASM zużywa znacznie więcej pamięci niż JS. **Dla projektu:** argument, by sprawdzić w benchmarku inne poziomy optymalizacji niż `opt-level = 3` + `wasm-opt -O3`, i tło dla zużycia pamięci przy danych w całości w pamięci.

## 3. Bazy danych w przeglądarce i SQL w WebAssembly

- **[D1]** Kohn A., Moritz D., Raasveldt M., Mühleisen H., Neumann T. *DuckDB-Wasm: Fast Analytical Processing for the Web.* PVLDB 15(12), 2022, s. 3574–3577. DOI: [10.14778/3554821.3554847](https://doi.org/10.14778/3554821.3554847).
  **Plik:** [papers/2022-kohn-duckdb-wasm.pdf](papers/2022-kohn-duckdb-wasm.pdf).
  — DuckDB w WASM: zapytania w wątkach w tle (Web Worker), funkcje użytkownika w JS, system plików czytający lokalne i zdalne pliki fragmentami; szybszy od wcześniejszych bibliotek webowych w TPC-H. **Dla projektu:** właściwy punkt odniesienia dla scenariusza „CSV 500 MB” z README – inna klasa systemu niż Flat-SQL.

- **[D2]** Gaffney K. P., Prammer M., Brasfield L., Hipp D. R., Kennedy D., Patel J. M. *SQLite: Past, Present, and Future.* PVLDB 15(12), 2022, s. 3535–3547. DOI: [10.14778/3554821.3554842](https://doi.org/10.14778/3554821.3554842).
  **Plik:** [papers/2022-gaffney-sqlite-past-present-future.pdf](papers/2022-gaffney-sqlite-past-present-future.pdf).
  — Architektura SQLite (wiersze, B-drzewa, OLTP), porównanie z DuckDB i optymalizacje analityczne (do 4,2× szybciej w SSB). **Dla projektu:** wyjaśnia przewagi SQLite widoczne w H4 (format stronicowy, indeksy pokrywające).

- **[D3]** Haffner I., Dittrich J. *Fast Compilation and Execution of SQL Queries with WebAssembly.* arXiv:2104.15098, 2021 (rozwinięcie: EDBT 2023).
  **Plik:** [papers/2021-haffner-fast-compilation-sql-queries-webassembly.pdf](papers/2021-haffner-fast-compilation-sql-queries-webassembly.pdf).
  — Kompilacja planów zapytań SQL do WASM i wykorzystanie wielopoziomowej kompilacji V8 (Liftoff → TurboFan) do uzyskania niskiej latencji i wysokiej przepustowości. **Dla projektu:** alternatywa dla interpretera wyrażeń z `expr`; kierunek dalszych prac (JIT zapytań).

- **[D4]** Litt G., Schiefer N., Schickling J., Jackson D. *Riffle: Reactive Relational State for Local-First Applications.* UIST 2023. DOI: [10.1145/3586183.3606801](https://doi.org/10.1145/3586183.3606801).
  **Plik:** [papers/2023-litt-riffle-reactive-relational-state.pdf](papers/2023-litt-riffle-reactive-relational-state.pdf).
  — Cały stan aplikacji webowej w trwałej, lokalnej relacyjnej bazie; reaktywne zapytania i synchroniczne transakcyjne aktualizacje w wątku UI; cel: odpowiedź w 16 ms (próg Nielsena: 100 ms). Złączenia w SQLite potrafiły trwać ponad 300 ms, inkrementalnie utrzymywane widoki – 1–10 ms. **Dla projektu:** najbliższa praca naukowa (lokalna baza SQL w przeglądarce + Local-First); argument za zapytaniami reaktywnymi i przeciw całkowitej rezygnacji ze złączeń.

- **[D5]** Zhang G. *Zeta-Lite: A Concurrent, Branchable In-Browser SQL Database for Agentic Memory.* arXiv:2609.01818, wrzesień 2026 (wersja v0.1, preprint bez recenzji).
  **Plik:** [papers/2026-zeta-lite-in-browser-sql-database.pdf](papers/2026-zeta-lite-in-browser-sql-database.pdf).
  — Przeglądarkowa wersja serwerowego silnika Zeta: ten sam parser, planer i wykonawca skompilowany do `wasm32-unknown-unknown` z `wasm-bindgen` (2,87 MB gzip). Transakcje z izolacją migawkową w jednym wątku (MVCC), rozgałęzianie całej bazy, pełny dialekt PostgreSQL; trwałość przez snapshot do OPFS zapisywany na żądanie – autor przyznaje, że zmiany po ostatnim snapshocie giną. Przepustowość w przeglądarce 5–15% poniżej natywnej. Kod silnika zamknięty, jeden autor. **Dla projektu:** najnowszy system o tej samej architekturze „jeden rdzeń dla serwera i przeglądarki” i tym samym toolchainie; jego zgrubna trwałość kontrastuje z przyrostowym WAL Flat-SQL (H5). Cytować ostrożnie (preprint).

## 4. Local-First, CRDT i synchronizacja

- **[L1]** Kleppmann M., Wiggins A., van Hardenberg P., McGranaghan M. *Local-First Software: You Own Your Data, in spite of the Cloud.* Onward! 2019, s. 154–178. DOI: [10.1145/3359591.3359737](https://doi.org/10.1145/3359591.3359737).
  **Plik:** [papers/2019-kleppmann-local-first-software.pdf](papers/2019-kleppmann-local-first-software.pdf) (wersja ACM, otwarty dostęp).
  — Siedem ideałów oprogramowania local-first: brak „spinnerów”, wiele urządzeń, sieć opcjonalna, współpraca, długowieczność danych, prywatność i bezpieczeństwo domyślnie, kontrola użytkownika; ocena istniejących technologii i CRDT jako fundamentu. Zwraca uwagę, że magazyn przeglądarki bywa czyszczony razem z ciasteczkami. **Dla projektu:** główna rama oceny architektury (tabela w [literature-comparison.md §3](literature-comparison.md#3-ideały-local-first)).

- **[L2]** Shapiro M., Preguiça N., Baquero C., Zawirski M. *A comprehensive study of Convergent and Commutative Replicated Data Types.* INRIA, Research Report RR-7506, 2011.
  **Plik:** [papers/2011-shapiro-comprehensive-study-crdt.pdf](papers/2011-shapiro-comprehensive-study-crdt.pdf) (kopia z serwisu kursu IISc; oryginał w HAL: inria-00555588).
  — Formalne podstawy CRDT (state-based / op-based) i katalog typów, m.in. rejestr LWW (specyfikacje 8–9), zbiory z tombstone'ami. **Dla projektu:** model Flat-SQL to mapa rejestrów LWW indeksowana kluczem głównym – tu jest jego formalna definicja.

- **[L3]** Preguiça N. *Conflict-free Replicated Data Types: An Overview.* arXiv:1806.10254, 2018.
  **Plik:** [papers/2018-preguica-crdt-overview.pdf](papers/2018-preguica-crdt-overview.pdf).
  — Zwięzły przegląd CRDT, wariantów synchronizacji i zastosowań. **Dla projektu:** łatwiejsze wejście w temat niż [L2].

- **[L4]** Gomes V. B. F., Kleppmann M., Mulligan D. P., Beresford A. R. *Verifying Strong Eventual Consistency in Distributed Systems.* PACMPL 1 (OOPSLA), art. 109, 2017. arXiv:1707.01747.
  **Plik:** [papers/2017-gomes-verifying-strong-eventual-consistency.pdf](papers/2017-gomes-verifying-strong-eventual-consistency.pdf).
  — Mechanicznie sprawdzony (Isabelle/HOL) dowód silnej spójności ostatecznej dla kilku CRDT w modelu sieci z utratą i zmianą kolejności wiadomości. **Dla projektu:** jak formalnie uzasadnić zbieżność, którą Flat-SQL sprawdza testem losowym.

- **[L5]** Kleppmann M., Beresford A. R. *A Conflict-Free Replicated JSON Datatype.* IEEE TPDS 28(10), 2017, s. 2733–2746. arXiv:1608.03960.
  **Plik:** [papers/2017-kleppmann-beresford-json-crdt.pdf](papers/2017-kleppmann-beresford-json-crdt.pdf).
  — CRDT dla zagnieżdżonych dokumentów JSON (podstawa biblioteki Automerge). **Dla projektu:** alternatywa dokumentowa wobec płaskich wierszy.

- **[L6]** Almeida P. S., Shoker A., Baquero C. *Delta State Replicated Data Types.* arXiv:1603.01529, 2016 (wersja czasopismowa: JPDC, 2018).
  **Plik:** [papers/2016-almeida-delta-state-crdt.pdf](papers/2016-almeida-delta-state-crdt.pdf).
  — Przesyłanie tylko zmienionych fragmentów stanu (delt) zamiast całego stanu. **Dla projektu:** outbox Flat-SQL (wysyłka bieżącego stanu zmienionych wierszy) jest w istocie synchronizacją delta-state.

- **[L7]** Baquero C., Almeida P. S., Shoker A. *Pure Operation-Based Replicated Data Types.* arXiv:1710.04469, 2017.
  **Plik:** [papers/2017-baquero-pure-operation-based-crdt.pdf](papers/2017-baquero-pure-operation-based-crdt.pdf).
  — CRDT oparte na częściowo uporządkowanym logu operacji z kompaktowaniem po osiągnięciu stabilności przyczynowej. **Dla projektu:** formalne kryterium, kiedy można usuwać tombstone'y (planowane w [architecture.md §8](../architecture.md)).

- **[L8]** Lopes P., Sousa J., Balegas V., Ferreira C., Duarte S., Bieniusa A., Rodrigues R., Preguiça N. *Antidote SQL: Relaxed When Possible, Strict When Necessary.* arXiv:1902.03576, 2019.
  **Plik:** [papers/2019-lopes-antidote-sql.pdf](papers/2019-lopes-antidote-sql.pdf).
  — Interfejs SQL nad bazą słabo spójną: semantyka update-wins / delete-wins deklarowana per tabela, klucze obce bez koordynacji, silna spójność tylko tam, gdzie konieczna. **Dla projektu:** pokazuje, że trudność replikacji leży w więzach integralności (klucze obce), a nie w samych złączeniach – ważne dla uzasadnienia modelu „bez JOIN”.

- **[L9]** Edwards J., Petricek T., van der Storm T. *Live & Local Schema Change: Challenge Problems.* arXiv:2309.11406, 2023.
  **Plik:** [papers/2023-live-local-schema-change.pdf](papers/2023-live-local-schema-change.pdf).
  — Zestaw problemów-wyzwań dotyczących zmian schematu w programowaniu na żywo i w oprogramowaniu local-first (rozjazd danych między replikami i wersjami kodu). **Dla projektu:** replikacja `CREATE/DROP TABLE` w Flat-SQL jest uproszczona – tu opisano, dlaczego to problem otwarty.

- **[L10]** Edwards J., Petricek T. *Baseline: Operation-Based Evolution and Versioning of Data.* arXiv:2512.09762 (v2: czerwiec 2026).
  **Plik:** [papers/2025-baseline-operation-based-evolution-of-data.pdf](papers/2025-baseline-operation-based-evolution-of-data.pdf).
  — Wersjonowanie danych oparte na operacjach (w tym refaktoryzacjach i zmianach schematu) z łączeniem gałęzi mimo zmian struktury. **Dla projektu:** najnowszy kierunek dla ewolucji schematu w replikach.

- **[L11]** Kuessner C., Mogk R., Wickert A.-K., Mezini M. *Encrypted Replicated Data Types within REScala.* PLF 2022 (preprint).
  **Plik:** [papers/2022-kuessner-encrypted-replicated-data-types.pdf](papers/2022-kuessner-encrypted-replicated-data-types.pdf).
  — Replikowane typy danych z automatycznym szyfrowaniem i uwierzytelnianiem, synchronizowane przez niezaufanych pośredników. **Dla projektu:** jak dodać szyfrowanie end-to-end bez rezygnacji z serwera synchronizacji.

- **[L12]** Kleppmann M., Howard H. *Byzantine Eventual Consistency and the Fundamental Limits of Peer-to-Peer Databases.* arXiv:2012.00472, 2020.
  **Plik:** [papers/2020-kleppmann-howard-byzantine-eventual-consistency.pdf](papers/2020-kleppmann-howard-byzantine-eventual-consistency.pdf).
  — Spójność ostateczna przy złośliwych uczestnikach i granice tego, co da się zagwarantować bez koordynacji. **Dla projektu:** serwer Flat-SQL przyjmuje dowolne zmiany i znaczniki – praca pokazuje, czego wymaga odporność na nieuczciwego klienta.

- **[L13]** Sanjuán H., Pöyhtäri S., Teixeira P., Psaras I. *Merkle-CRDTs: Merkle-DAGs meet CRDTs.* arXiv:2004.00107, 2020.
  **Plik:** [papers/2020-sanjuan-merkle-crdts.pdf](papers/2020-sanjuan-merkle-crdts.pdf).
  — Merkle-DAG jako warstwa transportu i trwałości CRDT, bez centralnego serwera. **Dla projektu:** alternatywa dla logu z kursorami po stronie serwera.

- **[L14]** Meyer A. *Range-Based Set Reconciliation.* arXiv:2212.13567, 2023.
  **Plik:** [papers/2023-meyer-range-based-set-reconciliation.pdf](papers/2023-meyer-range-based-set-reconciliation.pdf).
  — Wydajne uzgadnianie zbiorów przez rekurencyjne porównywanie odcisków zakresów. **Dla projektu:** sposób na synchronizację bez pełnego `exportState` po utracie kursora (nowy klient, nowa epoka serwera).

- **[L15]** Weidner M. i in. *Collabs: A Flexible and Performant CRDT Collaboration Framework.* arXiv:2212.02618 (v2, 2023).
  **Plik:** [papers/2022-weidner-collabs.pdf](papers/2022-weidner-collabs.pdf).
  — Biblioteka CRDT do budowy aplikacji współpracy, z oceną wydajności. **Dla projektu:** przykład dojrzałej warstwy CRDT w przeglądarce.

- **[L16]** Gentle J., Kleppmann M. *Collaborative Text Editing with Eg-walker: Better, Faster, Smaller.* EuroSys 2025. arXiv:2409.14252.
  **Plik:** [papers/2025-gentle-kleppmann-eg-walker.pdf](papers/2025-gentle-kleppmann-eg-walker.pdf).
  — Algorytm łączący zalety OT i CRDT: mniej pamięci i szybsze wczytywanie niż CRDT, szybkie scalanie długich gałęzi offline. **Dla projektu:** najnowsza praca Kleppmanna; kontekst dla danych tekstowych, których Flat-SQL nie scala (LWW na cały wiersz).

- **[L17]** Ramesh P., Soundarapandian V., Sivaramakrishnan K. C. *Sal: Multi-modal Verification of Replicated Data Types.* arXiv:2603.27202, marzec 2026.
  **Plik:** [papers/2026-sal-verification-replicated-data-types.pdf](papers/2026-sal-verification-replicated-data-types.pdf).
  — Weryfikacja CRDT i MRDT w Lean (automatyzacja z rekonstrukcją dowodu, SMT w razie potrzeby). **Dla projektu:** stan sztuki 2026 w formalnej weryfikacji typów replikowanych.

- **[L18]** Jacob F., Stuber J., Hartenstein H. *Towards System-Oriented Formal Verification of Local-First Access Control.* arXiv:2604.23560, kwiecień 2026.
  **Plik:** [papers/2026-local-first-access-control-verification.pdf](papers/2026-local-first-access-control-verification.pdf).
  — Formalnie weryfikowana kontrola dostępu dla systemów local-first odpornych na zachowania bizantyjskie (Matrix, Keyhive), specyfikacja w Ruście. **Dla projektu:** brak uwierzytelniania i autoryzacji w serwerze synchronizacji to znana luka – tu jej stan badań.

- **[L19]** Yanakieva E., Bieniusa A., Dumbrava S. *A Datalog Framework for Conflict-Free Replicated Data Types.* arXiv:2605.31569, maj 2026.
  **Plik:** [papers/2026-datalog-framework-crdts.pdf](papers/2026-datalog-framework-crdts.pdf).
  — Deklaratywna specyfikacja semantyki CRDT w Datalogu i testowanie własności implementacji. **Dla projektu:** sposób na precyzyjny opis semantyki LWW per wiersz z tombstone'ami.

- **[L20]** Niinikoski J.-K. *Local-First Architecture: Investigating the Suitability for Modern Browser Applications.* Praca magisterska, Aalto University, 2025.
  **Plik:** [papers/2025-niinikoski-local-first-architecture-thesis.pdf](papers/2025-niinikoski-local-first-architecture-thesis.pdf).
  — Przegląd literatury i wywiady z twórcami aplikacji webowych: local-first poprawia responsywność i pracę offline, ale przenosi na klienta odpowiedzialności serwera; w pełni bezserwerowe systemy rzadko są praktyczne, realne są modele hybrydowe z minimalną koordynacją serwera. **Dla projektu:** wspiera hybrydową architekturę Flat-SQL (lokalny silnik + serwer synchronizacji).

- **[L21]** Niinikoski J., Fagerholm F. *Supplementary Material for “On the Suitability of Local-First Architecture From Web Application Developers' Perspective”.* Zenodo, 2026. DOI: [10.5281/zenodo.20121788](https://doi.org/10.5281/zenodo.20121788).
  **Plik:** [papers/2026-niinikoski-fagerholm-local-first-suitability-supplementary.pdf](papers/2026-niinikoski-fagerholm-local-first-suitability-supplementary.pdf) (CC BY 4.0).
  — Materiały uzupełniające (protokół wywiadów, lista prac z przeglądu) do artykułu z 2026 r.; samego artykułu nie udało się zlokalizować. **Dla projektu:** gotowa lista literatury z przeglądu systematycznego.

- **[L22]** Otsamo O. *Local-First Software: Promises and Pitfalls.* Praca licencjacka, Aalto University, 2025.
  **Plik:** [papers/2025-otsamo-local-first-promises-and-pitfalls-thesis.pdf](papers/2025-otsamo-local-first-promises-and-pitfalls-thesis.pdf).
  — Local-first sprawdza się we współpracy asynchronicznej, ale nie zastąpi chmury m.in. przy transakcjach finansowych i dużych zbiorach danych. **Dla projektu:** krótkie, aktualne podsumowanie ograniczeń podejścia (do dyskusji w pracy).

- **[L23]** Kleppmann M. *Distributed Systems* – notatki do wykładu, University of Cambridge, 2021/22.
  **Plik:** [papers/kleppmann-distributed-systems-lecture-notes.pdf](papers/kleppmann-distributed-systems-lecture-notes.pdf).
  — Bezpłatne notatki autora DDIA: zegary fizyczne i logiczne, rozgłaszanie, replikacja, kworum, konsensus, CRDT. **Dla projektu:** legalnie dostępne uzupełnienie [P1] dla rozdziałów 5, 8 i 9.

- **[L24]** Yu W., Ignat C.-L. *Conflict-Free Replicated Relations for Multi-Synchronous Database Management at Edge.* IEEE International Conference on Smart Data Services (SMDS), 2020. HAL: [hal-02983557](https://hal.science/hal-02983557).
  **Plik:** [papers/2020-yu-ignat-conflict-free-replicated-relations.pdf](papers/2020-yu-ignat-conflict-free-replicated-relations.pdf) (wersja otwarta z HAL).
  — CRDT zastosowane do relacyjnej bazy danych na urządzeniach brzegowych. Dwie warstwy: aplikacja widzi zwykłe tabele, a pod spodem każda kolumna to rejestr LWW z własnym znacznikiem czasu, każdy wiersz ma „długość przyczynową” (licznik wstawień i usunięć zamiast tombstone'a), a kolumny liczbowe mogą być licznikami CRDT. Naruszenia więzów (unikalność, klucze obce) wykrywane są przy scalaniu i naprawiane przez cofnięcie przegrywającej zmiany. Prototyp na Ecto (Elixir) z PostgreSQL. **Dla projektu:** najbliższa praca naukowa do synchronizacji Flat-SQL – pokazuje, jak przejść z LWW na wiersz do LWW na kolumnę i jak obsłużyć konflikty klucza głównego (porównanie §4.2, §4.4).

- **[L25]** Tomter I. T. *Towards a General Database Management System of Conflict-Free Replicated Relations.* Praca magisterska, UiT – The Arctic University of Norway, 2021 (promotor: W. Yu).
  **Plik:** [papers/2021-tomter-dbms-conflict-free-replicated-relations-thesis.pdf](papers/2021-tomter-dbms-conflict-free-replicated-relations-thesis.pdf).
  — Przeniesienie CRR [L24] bezpośrednio do SQLite: dwie warstwy realizowane wyzwalaczami, scalanie wykonywane w całości skryptem SQL, istniejąca baza „rozszerzana” jednym poleceniem bez zmian w aplikacji. Ciekawy szczegół: SQLite daje czas z dokładnością do milisekundy, więc autor dokłada losowe cyfry, by uniknąć remisów znaczników. **Dla projektu:** porównanie z HLC we Flat-SQL (remisy rozstrzyga licznik i identyfikator węzła, nie losowość) i przykład pracy dyplomowej o podobnym zakresie.

- **[L26]** Litt G., van Hardenberg P., Henry O. *Cambria: Schema Evolution in Distributed Systems with Edit Lenses.* PaPoC 2021. DOI: [10.1145/3447865.3457963](https://doi.org/10.1145/3447865.3457963).
  **Plik:** [papers/2021-litt-cambria-schema-evolution.pdf](papers/2021-litt-cambria-schema-evolution.pdf) (wersja ACM, CC BY 4.0).
  — Osobna warstwa tłumacząca dokumenty między wersjami schematu „w locie” za pomocą dwukierunkowych soczewek (np. zmiana pola `complete: bool` na `status` z trzema wartościami); biblioteka w TypeScript zintegrowana z CRDT Automerge, sprawdzona na zdecentralizowanym systemie zgłoszeń. **Dla projektu:** gotowy wzorzec dla problemu z porównania §4.6 – repliki z różnymi wersjami schematu tabel.

## 5. Zegary, uporządkowanie i replikacja (prace klasyczne)

- **[C1]** Lamport L. *Time, Clocks, and the Ordering of Events in a Distributed System.* Communications of the ACM 21(7), 1978, s. 558–565.
  **Plik:** [papers/1978-lamport-time-clocks-ordering-of-events.pdf](papers/1978-lamport-time-clocks-ordering-of-events.pdf).
  — Relacja „zdarzyło się przed”, zegary logiczne i porządek całkowity z rozstrzyganiem remisów identyfikatorem procesu. **Dla projektu:** `Stamp { time, node }` to właśnie ten porządek.

- **[C2]** Kulkarni S. S., Demirbas M., Madeppa D., Avva B., Leone M. *Logical Physical Clocks and Consistent Snapshots in Globally Distributed Databases.* OPODIS 2014 (LNCS 8878); raport techniczny UB 2014-04.
  **Plik:** [papers/2014-kulkarni-hybrid-logical-clocks.pdf](papers/2014-kulkarni-hybrid-logical-clocks.pdf).
  — Zegar HLC: zachowuje przyczynowość jak zegar logiczny i pozostaje blisko czasu NTP; zwarta postać 64-bitowa (48 bitów czasu + 16 bitów licznika); reguły odporności – reset do czasu fizycznego i **ignorowanie wiadomości ze znacznikiem zbyt daleko w przyszłości**. **Dla projektu:** zegar w `sync.rs` realizuje postać zwartą, ale nie ma reguły odrzucania znaczników spoza granicy (zob. porównanie §4.1).

- **[C3]** Terry D. B., Theimer M. M., Petersen K., Demers A. J., Spreitzer M. J., Hauser C. H. *Managing Update Conflicts in Bayou, a Weakly Connected Replicated Storage System.* SOSP 1995, s. 172–182.
  **Plik:** [papers/1995-terry-bayou-update-conflicts.pdf](papers/1995-terry-bayou-update-conflicts.pdf).
  — Replikacja dla słabo połączonych urządzeń mobilnych: zapisy do dowolnej repliki, wykrywanie konfliktów (dependency checks) i procedury scalania dostarczane przez aplikację, anti-entropy, serwer główny ustalający kolejność zatwierdzeń. **Dla projektu:** protoplasta architektury „lokalne zapisy + serwer porządkujący log”.

- **[C4]** DeCandia G. i in. *Dynamo: Amazon's Highly Available Key-value Store.* SOSP 2007, s. 205–220.
  **Plik:** [papers/2007-decandia-dynamo.pdf](papers/2007-decandia-dynamo.pdf) (wersja autorska, „not for redistribution”).
  — Magazyn klucz-wartość z wektorami wersji i rozwiązywaniem konfliktów przez aplikację; podkreśla, że wiele usług potrzebuje wyłącznie dostępu po kluczu głównym, a magazyn sam może zastosować tylko proste reguły typu LWW. **Dla projektu:** uzasadnienie modelu adresowania kluczem i porównanie LWW z wektorami wersji.

- **[C5]** Lakshman A., Malik P. *Cassandra – A Decentralized Structured Storage System.* ACM SIGOPS Operating Systems Review 44(2), 2010, s. 35–40.
  **Plik:** [papers/2010-lakshman-malik-cassandra.pdf](papers/2010-lakshman-malik-cassandra.pdf).
  — Architektura Cassandry (model column-family, partycjonowanie, replikacja). **Dla projektu:** Cassandra i jej język CQL (składnia SQL, brak złączeń, obowiązkowy klucz główny, LWW ze znacznikami czasu) to najbliższy przemysłowy odpowiednik modelu Flat-SQL.

- **[C6]** Chang F. i in. *Bigtable: A Distributed Storage System for Structured Data.* OSDI 2006.
  **Plik:** [papers/2006-chang-bigtable.pdf](papers/2006-chang-bigtable.pdf).
  — Posortowana mapa (wiersz, kolumna, czas) → wartość; każdy odczyt i zapis w obrębie jednego wiersza jest atomowy, transakcje tylko jednowierszowe. **Dla projektu:** precedens dla atomowości „per wiersz” i adresowania kluczem.

- **[C7]** Helland P. *Life beyond Distributed Transactions: an Apostate's Opinion.* CIDR 2007.
  **Plik:** [papers/2007-helland-life-beyond-distributed-transactions.pdf](papers/2007-helland-life-beyond-distributed-transactions.pdf).
  — Encja z unikalnym kluczem jako zakres serializowalności; transakcje atomowe nie obejmują wielu encji; wiadomości muszą być idempotentne. **Dla projektu:** najmocniejsze uzasadnienie płaskiego modelu Flat-SQL i idempotentnych rund synchronizacji.

- **[C8]** Bailis P., Davidson A., Fekete A., Ghodsi A., Hellerstein J. M., Stoica I. *Highly Available Transactions: Virtues and Limitations.* PVLDB 7(3), 2013, s. 181–192.
  **Plik:** [papers/2014-bailis-highly-available-transactions.pdf](papers/2014-bailis-highly-available-transactions.pdf).
  — Które poziomy izolacji da się zapewnić bez koordynacji: m.in. Read Committed i Monotonic Atomic View tak, zapobieganie utraconym aktualizacjom i write skew – nie. **Dla projektu:** precyzyjny język do opisu gwarancji Flat-SQL po synchronizacji (transakcje atomowe lokalnie, ale bez MAV między replikami).

## 6. Składowanie, WAL i odtwarzanie po awarii

- **[S1]** DeWitt D. J., Katz R. H., Olken F., Shapiro L. D., Stonebraker M. R., Wood D. *Implementation Techniques for Main Memory Database Systems.* SIGMOD 1984, s. 1–8 (raport UCB/ERL M84/5).
  **Plik:** [papers/1984-dewitt-main-memory-database-systems.pdf](papers/1984-dewitt-main-memory-database-systems.pdf).
  — Struktury i odtwarzanie dla baz mieszczących się w pamięci; pierwszy drukowany opis **group commit** (autorzy piszą, że wcześniej był to „folklor” System R). **Dla projektu:** źródło group commitu w `FlatDB.flush()` i modelu „log + checkpoint”.

- **[S2]** Mohan C., Haderle D., Lindsay B., Pirahesh H., Schwarz P. *ARIES: A Transaction Recovery Method Supporting Fine-Granularity Locking and Partial Rollbacks Using Write-Ahead Logging.* ACM TODS 17(1), 1992, s. 94–162.
  **Plik:** [papers/1992-mohan-aries.pdf](papers/1992-mohan-aries.pdf).
  — Klasyczny WAL: numery LSN, powtarzanie historii, logowanie wycofań. **Dla projektu:** tło dla uproszczeń Flat-SQL (log tylko-redo, bo niezatwierdzone zmiany nigdy nie trafiają do WAL).

- **[S3]** O'Neil P., Cheng E., Gawlick D., O'Neil E. *The Log-Structured Merge-Tree (LSM-Tree).* Acta Informatica 33(4), 1996, s. 351–385.
  **Plik:** [papers/1996-oneil-lsm-tree.pdf](papers/1996-oneil-lsm-tree.pdf).
  — Struktura zoptymalizowana pod intensywne zapisy przez scalanie posortowanych przebiegów. **Dla projektu:** kierunek, gdyby pełny checkpoint (kopiowanie całej bazy) stał się wąskim gardłem.

- **[S4]** Stonebraker M., Madden S., Abadi D. J., Harizopoulos S., Hachem N., Helland P. *The End of an Architectural Era (It's Time for a Complete Rewrite).* VLDB 2007, s. 1150–1160.
  **Plik:** [papers/2007-stonebraker-end-of-an-architectural-era.pdf](papers/2007-stonebraker-end-of-an-architectural-era.pdf).
  — H-Store: dane w pamięci, wykonanie jednowątkowe, specjalizacja zamiast „jednego rozmiaru dla wszystkich”; 82× szybciej od komercyjnego RDBMS w TPC-C. **Dla projektu:** uzasadnienie wąsko wyspecjalizowanego, jednowątkowego silnika w pamięci.

- **[S5]** Harizopoulos S., Abadi D. J., Madden S., Stonebraker M. *OLTP Through the Looking Glass, and What We Found There.* SIGMOD 2008, s. 981–992.
  **Plik:** [papers/2008-harizopoulos-oltp-through-the-looking-glass.pdf](papers/2008-harizopoulos-oltp-through-the-looking-glass.pdf) (kopia ze strony D. Abadiego).
  — Rozkład instrukcji w klasycznym silniku OLTP: logowanie, blokady, latche i bufor stron pochłaniają większość pracy; „użyteczna praca” to niewielka część. **Dla projektu:** wyjaśnia, dlaczego Flat-SQL płaci za undo log + WAL + outbox w insertach (H4) i czego nie płaci (blokady, bufor stron).

- **[S6]** Malviya N., Weisberg A., Madden S., Stonebraker M. *Rethinking Main Memory OLTP Recovery.* ICDE 2014, s. 604–615.
  **Plik:** [papers/2014-malviya-rethinking-main-memory-oltp-recovery.pdf](papers/2014-malviya-rethinking-main-memory-oltp-recovery.pdf).
  — Logowanie poleceń (command logging) daje w VoltDB ok. 1,5× większą przepustowość niż logowanie fizjologiczne w stylu ARIES, kosztem dłuższego odtwarzania. **Dla projektu:** uzasadnienie wyboru logu fizycznego: te same wpisy WAL służą synchronizacji, a odtworzenie nie zależy od determinizmu zapytań.

- **[S7]** Zheng W., Tu S., Kohler E., Liskov B. *Fast Databases with Fast Durability and Recovery Through Multicore Parallelism.* OSDI 2014.
  **Plik:** [papers/2014-zheng-silor-fast-durability-and-recovery.pdf](papers/2014-zheng-silor-fast-durability-and-recovery.pdf).
  — SiloR: równoległe logowanie, checkpointy i odtwarzanie w bazie w pamięci. **Dla projektu:** kierunek optymalizacji odtwarzania snapshotu (200 ms dla 100 tys. wierszy w H4).

- **[S8]** Pillai T. S., Chidambaram V., Alagappan R., Al-Kiswany S., Arpaci-Dusseau A. C., Arpaci-Dusseau R. H. *All File Systems Are Not Created Equal: On the Complexity of Crafting Crash-Consistent Applications.* OSDI 2014, s. 433–448.
  **Plik:** [papers/2014-pillai-all-file-systems-are-not-created-equal.pdf](papers/2014-pillai-all-file-systems-are-not-created-equal.pdf).
  — Narzędzia BOB i ALICE: poprawność protokołów zapisu aplikacji zależy od subtelnych właściwości systemu plików (atomowość, kolejność operacji na różnych plikach); 60 podatności w 11 dojrzałych aplikacjach. **Dla projektu:** bezpośrednio dotyczy sekwencji `snapshot.tmp → rename → obcięcie WAL` w `FileStorage` (porównanie §6.4).

- **[S9]** Zheng M., Tucek J., Huang D., Qin F., Lillibridge M., Yang E. S., Zhao B. W., Singh S. *Torturing Databases for Fun and Profit.* OSDI 2014, s. 449–464.
  **Plik:** [papers/2014-zheng-torturing-databases.pdf](papers/2014-zheng-torturing-databases.pdf).
  — Wstrzykiwanie zaników zasilania: wszystkie 8 badanych baz (open source i komercyjne) zachowało się błędnie, 7 naruszyło ACID. Model awarii: utrwalony jest prefiks strumienia I/O. **Dla projektu:** test „ucięcie WAL w każdym bajcie” odpowiada temu modelowi awarii na poziomie pliku.

## 7. Model danych: SQL, NoSQL i „płaskie” tabele

- **[M1]** Stonebraker M., Pavlo A. *What Goes Around Comes Around... And Around...* ACM SIGMOD Record 53(2), 2024, s. 21–37.
  **Plik:** [papers/2024-stonebraker-pavlo-what-goes-around-comes-around.pdf](papers/2024-stonebraker-pavlo-what-goes-around-comes-around.pdf).
  — Przegląd 20 lat modeli danych: prawie wszystkie systemy NoSQL dodały interfejs SQL (CQL, PartiQL, SQL++), systemy column-family przejęły z Bigtable brak złączeń i indeksów wtórnych, model relacyjny pozostał dominujący. **Dla projektu:** Flat-SQL wpisuje się w ten trend (składnia SQL nad modelem klucz-wiersz) – i podlega tej samej krytyce braku złączeń.

## 8. Edge computing i WebAssembly poza przeglądarką

- **[E1]** Shi W., Cao J., Zhang Q., Li Y., Xu L. *Edge Computing: Vision and Challenges.* IEEE Internet of Things Journal 3(5), 2016, s. 637–646. DOI: [10.1109/JIOT.2016.2579198](https://doi.org/10.1109/JIOT.2016.2579198).
  **Plik:** [papers/2016-shi-edge-computing-vision-and-challenges.pdf](papers/2016-shi-edge-computing-vision-and-challenges.pdf) (kopia z serwisu kursu).
  — Definicja i wyzwania przetwarzania brzegowego. **Dla projektu:** cytowanie definicji „Edge Computing” z README.

- **[E2]** Satyanarayanan M. *The Emergence of Edge Computing.* IEEE Computer 50(1), 2017, s. 30–39. DOI: [10.1109/MC.2017.9](https://doi.org/10.1109/MC.2017.9).
  **Plik:** [papers/2017-satyanarayanan-emergence-of-edge-computing.pdf](papers/2017-satyanarayanan-emergence-of-edge-computing.pdf) (kopia ze strony autora).
  — Węzły obliczeniowe blisko urządzeń: niska latencja, egzekwowanie polityk prywatności, maskowanie awarii chmury. **Dla projektu:** motywacja trybu edge (serwer Node z tym samym `.wasm`).

- **[E3]** Hall A., Ramachandran U. *An Execution Model for Serverless Functions at the Edge.* IoTDI 2019. DOI: [10.1145/3302505.3310084](https://doi.org/10.1145/3302505.3310084).
  **Plik:** [papers/2019-hall-serverless-functions-at-the-edge.pdf](papers/2019-hall-serverless-functions-at-the-edge.pdf).
  — WebAssembly jako lżejsza od kontenerów izolacja funkcji serverless na brzegu sieci. **Dla projektu:** precedens uruchamiania tego samego modułu WASM na węźle brzegowym.

- **[E4]** Gadepalli P. K., McBride S., Peach G., Cherkasova L., Parmer G. *Sledge: a Serverless-first, Light-weight Wasm Runtime for the Edge.* Middleware 2020, s. 265–279.
  **Plik:** [papers/2020-gadepalli-sledge-wasm-runtime-edge.pdf](papers/2020-gadepalli-sledge-wasm-runtime-edge.pdf).
  — Środowisko WASM dla krótkotrwałych funkcji na urządzeniach brzegowych z izolacją programową. **Dla projektu:** jak wygląda „edge” z WASM w literaturze systemowej.

- **[E5]** Shillaker S., Pietzuch P. *Faasm: Lightweight Isolation for Efficient Stateful Serverless Computing.* USENIX ATC 2020. arXiv:2002.09344.
  **Plik:** [papers/2020-shillaker-faasm.pdf](papers/2020-shillaker-faasm.pdf).
  — Izolacja pamięci funkcji przez WASM i współdzielenie stanu w jednej przestrzeni adresowej; szybki start ze snapshotów. **Dla projektu:** stanowe obliczenia w WASM – analogia do silnika ze stanem w pamięci modułu.

- **[E6]** Ménétrey J., Pasin M., Felber P., Schiavoni V. *WebAssembly as a Common Layer for the Cloud-edge Continuum.* FRAME 2022. DOI: [10.1145/3526059.3533618](https://doi.org/10.1145/3526059.3533618).
  **Plik:** [papers/2022-menetrey-webassembly-cloud-edge-continuum.pdf](papers/2022-menetrey-webassembly-cloud-edge-continuum.pdf).
  — Artykuł pozycyjny: WASM jako wspólna warstwa wykonawcza od urządzeń IoT po chmurę. **Dla projektu:** wprost wspiera założenie „jeden rdzeń, wiele środowisk”.

- **[E7]** Colosi M., Farahani R., Lovén L., Prodan R., Villari M. *Serverless Everywhere: A Comparative Analysis of WebAssembly Workflows Across Browser, Edge, and Cloud.* arXiv:2512.04089 (IoT 2025 – warsztat ScaleSys).
  **Plik:** [papers/2025-colosi-serverless-everywhere-webassembly.pdf](papers/2025-colosi-serverless-everywhere-webassembly.pdf).
  — Ten sam przepływ WASM w przeglądarce, na brzegu i w chmurze: przy małych danych przeglądarka jest konkurencyjna dzięki wymianie danych w pamięci; przy dużych wygrywa kompilacja AOT na serwerach. **Dla projektu:** empiryczne wsparcie dla wykonywania małych zapytań operacyjnych lokalnie, a ciężkiej analityki gdzie indziej.

- **[E8]** Marcelino C., Krennmair N., Pusztai T., Nastic S. *Lumos: Performance Characterization of WebAssembly as a Serverless Runtime in the Edge-Cloud Continuum.* arXiv:2510.05118, 2025.
  **Plik:** [papers/2025-lumos-webassembly-serverless-edge-cloud.pdf](papers/2025-lumos-webassembly-serverless-edge-cloud.pdf).
  — Obrazy WASM kompilowane AOT są do 30× mniejsze od kontenerów i startują do 16% szybciej; WASM interpretowany ma 30–55× większą latencję i 7–10× większy narzut serializacji I/O. **Dla projektu:** pokazuje, że koszt przekazywania danych między WASM a otoczeniem bywa istotniejszy niż samo wykonanie – kontekst dla H3.

## 9. Metodyka pomiarów

- **[B1]** Georges A., Buytaert D., Eeckhout L. *Statistically Rigorous Java Performance Evaluation.* OOPSLA 2007, s. 57–76.
  **Plik:** [papers/2007-georges-statistically-rigorous-performance-evaluation.pdf](papers/2007-georges-statistically-rigorous-performance-evaluation.pdf).
  — Popularne metody raportowania (najlepszy wynik, pojedynczy przebieg) prowadzą do błędnych wniosków w środowiskach z JIT; zalecane wielokrotne uruchomienia maszyny wirtualnej i przedziały ufności. **Dla projektu:** jak wzmocnić wnioski H2–H4 (szum ±30% opisany w scenariuszu).

- **[B2]** Barrett E., Bolz-Tereick C. F., Killick R., Mount S., Tratt L. *Virtual Machine Warmup Blows Hot and Cold.* PACMPL 1 (OOPSLA), art. 52, 2017. arXiv:1602.00602.
  **Plik:** [papers/2017-barrett-vm-warmup-blows-hot-and-cold.pdf](papers/2017-barrett-vm-warmup-blows-hot-and-cold.pdf).
  — Wiele par (VM, benchmark) nigdy nie osiąga stabilnego stanu po rozgrzewce; metoda wykrywania punktów zmiany. **Dla projektu:** uzasadnia rozgrzewkę w benchmarku i pokazuje jej granice (Liftoff → TurboFan w V8).

- **[B3]** Raasveldt M., Holanda P., Gubner T., Mühleisen H. *Fair Benchmarking Considered Difficult: Common Pitfalls In Database Performance Testing.* DBTest 2018. DOI: [10.1145/3209950.3209955](https://doi.org/10.1145/3209950.3209955).
  **Plik:** [papers/2018-raasveldt-fair-benchmarking-considered-difficult.pdf](papers/2018-raasveldt-fair-benchmarking-considered-difficult.pdf).
  — Lista pułapek: brak powtarzalności, brak optymalizacji systemu porównywanego, „jabłka i pomarańcze”, nadmierne strojenie, przebiegi zimne/gorące/ciepłe, pomijanie przygotowania danych, niepoprawne wyniki. **Dla projektu:** lista kontrolna do recenzji [benchmark-scenario.md](../research/benchmark-scenario.md) (porównanie §8).

- **[B4]** Taipalus T. *Database management system performance comparisons: A systematic literature review.* arXiv:2301.01095 (v3, 2023).
  **Plik:** [papers/2023-dbms-performance-comparisons-review.pdf](papers/2023-dbms-performance-comparisons-review.pdf).
  — Większość porównań wydajności DBMS testuje scenariusze dalekie od rzeczywistych i opisuje je zbyt skąpo, by je powtórzyć. **Dla projektu:** argument za opisem środowiska i surowymi wynikami w JSON (już robione) oraz scenariuszem bliskim aplikacji.

- **[B5]** Cooper B. F., Silberstein A., Tam E., Ramakrishnan R., Sears R. *Benchmarking Cloud Serving Systems with YCSB.* SoCC 2010, s. 143–154.
  **Plik:** [papers/2010-cooper-ycsb.pdf](papers/2010-cooper-ycsb.pdf).
  — Standardowe obciążenia dla magazynów klucz-wartość (proporcje odczyt/zapis, rozkład Zipfa). **Dla projektu:** gotowe wzorce obciążeń dla operacji po kluczu; obecny benchmark używa rozkładu równomiernego.

- **[B6]** Schwarz M., Maurice C., Gruss D., Mangard S. *Fantastic Timers and Where to Find Them: High-Resolution Microarchitectural Attacks in JavaScript.* Financial Cryptography 2017, LNCS 10322.
  **Plik:** [papers/2017-schwarz-fantastic-timers.pdf](papers/2017-schwarz-fantastic-timers.pdf).
  — Dlaczego przeglądarki ograniczają rozdzielczość zegarów i jakie są alternatywne źródła czasu. **Dla projektu:** tło dla ograniczenia `performance.now()` do 100 µs w pomiarach przeglądarkowych.

## 10. Prywatność

- **[V1]** Cavoukian A. *Privacy by Design: The 7 Foundational Principles – Implementation and Mapping of Fair Information Practices.* Information and Privacy Commissioner of Ontario.
  **Plik:** [papers/2009-cavoukian-privacy-by-design-principles.pdf](papers/2009-cavoukian-privacy-by-design-principles.pdf) (kopia z serwisu kursu University of Waterloo).
  — Siedem zasad Privacy by Design (m.in. prywatność domyślnie, ochrona w całym cyklu życia danych, przejrzystość) z mapowaniem na zasady FIP. **Dla projektu:** źródło pojęcia „Privacy by Design” użytego w README; pozwala ocenić, które zasady projekt spełnia (porównanie §2.4). Uzupełnienie prawne: art. 25 RODO (ochrona danych w fazie projektowania i domyślna ochrona danych).

---

## 11. Źródła ważne, których nie pobrano – do pobrania ręcznie

| ID | Publikacja | Dlaczego brak pliku | Gdzie szukać |
|---|---|---|---|
| **[X4]** | Kleppmann M. *Designing Data-Intensive Applications* (wyd. 1 2017, wyd. 2 2026) | książka komercyjna | biblioteka uczelni, wydanie polskie Helion 2018, O'Reilly Learning |
| **[X5]** | Saadbye J., Gondelman L., Albano M. *Teilen-SQL: A library for building local-first software with SQLite and version control.* PLF+PLAID @ ECOOP 2025 | opublikowano tylko streszczenie wystąpienia | [strona wystąpienia](https://2025.ecoop.org/details/plf-plaid-2025-papers/7/Teilen-SQL-A-library-for-building-local-first-software-with-SQLite-and-version-contr) – SQLite w przeglądarce, komórki jako rejestry LWW |
| **[X6]** | Niinikoski J., Fagerholm F. *On the Suitability of Local-First Architecture From Web Application Developers' Perspective* (2026) | nie znaleziono tekstu artykułu, tylko materiały uzupełniające [L21] | wyszukać tytuł w ACM DL / Google Scholar po publikacji |

Pozycje dawniej oznaczone [X1], [X2], [X3] i [X7] pobrano ręcznie przez przeglądarkę – są teraz wpisami [L24], [L26], [W11] i [L25]. Dla [X5] i [X6] ponowne wyszukiwanie (18.10.2026) nie znalazło pełnego tekstu.

## 12. Dokumentacja platform i źródła branżowe (nierecenzowane)

Źródła te nie są publikacjami naukowymi, ale opisują zachowanie przeglądarek, od którego zależy poprawność projektu – w pracy warto je cytować jako dokumentację techniczną.

- **[Q1]** Chrome for Developers: [A change to the default durability mode in IndexedDB](https://developer.chrome.com/blog/indexeddb-durability-mode-now-defaults-to-relaxed) – od Chrome 121 domyślna trwałość transakcji IndexedDB to `relaxed` (zdarzenie `complete` po przekazaniu danych do bufora systemu operacyjnego).
- **[Q2]** WebKit: [Full Third-Party Cookie Blocking and More](https://webkit.org/blog/10218/full-third-party-cookie-blocking-and-more/) (24.03.2020) – Safari usuwa m.in. IndexedDB, LocalStorage i rejestracje Service Workera po 7 dniach używania Safari bez interakcji użytkownika z witryną; aplikacje dodane do ekranu głównego mają własny licznik.
- **[Q3]** WebKit: [Updates to Storage Policy](https://webkit.org/blog/14403/updates-to-storage-policy/) (Safari 17, 2023) – nowe limity miejsca dla źródła i zasady usuwania danych; tryb trwały przez `navigator.storage.persist()`.
- **[Q4]** MDN: [Storage quotas and eviction criteria](https://developer.mozilla.org/docs/Web/API/Storage_API/Storage_quotas_and_eviction_criteria) – magazyn „best-effort” vs „persistent” w przeglądarkach.
- **[Q5]** web.dev: [Making your website cross-origin isolated using COOP and COEP](https://web.dev/articles/coop-coep) – rozdzielczość `performance.now()`: 100 µs domyślnie, 5 µs w kontekście izolowanym.
- **[Q6]** webassembly.org: [Wasm 3.0 Completed](https://webassembly.org/news/2025-09-17-wasm-3.0) (17.09.2025); SpiderMonkey: [Is Memory64 actually worth using?](https://spidermonkey.dev/blog/2025/01/15/is-memory64-actually-worth-using.html) (15.01.2025).
- **[Q7]** SQLite: [persistence options for the WASM build (OPFS, opfs-sahpool, kvvfs)](https://sqlite.org/wasm/doc/tip/persistence.md).
- **[Q8]** RxDB: [LocalStorage vs. IndexedDB vs. Cookies vs. OPFS vs. WASM-SQLite](https://rxdb.info/articles/localstorage-indexeddb-cookies-opfs-sqlite-wasm.html); PGlite: [benchmarks](https://pglite.dev/benchmarks) – pomiary producentów (uwaga na konflikt interesów).
- **[Q9]** Kleppmann M.: keynote PaPoC 2025 „Byzantine eventual consistency and local-first access control” i SRDS 2026 (Rzym, 24.09.2026) – [martin.kleppmann.com](https://martin.kleppmann.com/); wywiad [SE Radio 716](https://se-radio.net/2026/04/se-radio-716-martin-kleppmann-local-first-software/) (04.2026).
- **[Q10]** [FOSDEM 2026 – ścieżka Local-First, sync engines, CRDTs](https://fosdem.org/2026/schedule/track/local-first/); [Smashing Magazine, „The Architecture Of Local-First Web Development”](https://www.smashingmagazine.com/2026/05/architecture-local-first-web-development/) (05.2026) – praktyk zaleca LWW na poziomie pola, nie rekordu.
- **[Q11]** Repozytoria systemów porównywanych w [wasm-engines-comparison.md](../research/wasm-engines-comparison.md): sql.js, wa-sqlite, PGlite, DuckDB-Wasm, GlueSQL, Turso, cr-sqlite, ElectricSQL, PowerSync.
