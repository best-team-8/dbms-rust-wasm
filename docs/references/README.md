# Baza wiedzy – literatura do projektu Flat-SQL

Rdzeń literatury projektu: silnik SQL w Rust kompilowany do WebAssembly, płaski model tabel adresowanych kluczem, dane w pamięci z WAL i snapshotem, synchronizacja Local-First (HLC + LWW) oraz metodyka benchmarków. Stan po rewizji z **20 października 2026**.

| Plik / katalog | Zawartość |
|---|---|
| [conclusions.md](conclusions.md) | wnioski z analizy prototypu i literatury: pozycja na tle innych rozwiązań, kierunek rozwoju, słowniczek |
| [literature-comparison.md](literature-comparison.md) | porównanie implementacji z wynikami badań, obszar po obszarze, z rekomendacjami |
| [search-methodology.md](search-methodology.md) | jak szukano, audyt pokrycia atutów projektu, kryteria doboru rdzenia, ograniczenia |
| [notes/haas-2017-webassembly.md](notes/haas-2017-webassembly.md) | notatka do pracy poleconej przez promotora [P2] |
| [notes/ddia-kleppmann.md](notes/ddia-kleppmann.md) | notatka do książki poleconej przez promotora [P1], z mapą rozdziałów na moduły |
| `papers/` | 38 plików PDF rdzenia (lokalnie, wykluczone z gita w `.git/info/exclude`) |
| `papers/_poza-rdzeniem/` | 46 plików PDF odrzuconych w rewizji (§13) – zachowane, nie usunięte |

> **Prawa autorskie.** Pliki w `papers/` służą do własnej nauki. Wiele z nich to wersje autorskie z dopiskiem „for personal use, not for redistribution” albo kopie z serwisów kursów uniwersyteckich, dlatego katalog jest wykluczony z repozytorium i nie należy go publikować. W pracy dyplomowej cytuje się wersję wydawniczą (DOI), nie lokalną kopię.

## Jak czytać wpisy

Każdy wpis ma identyfikator używany w pozostałych dokumentach, opis bibliograficzny, plik, streszczenie i odpowiedź na pytanie, **co z pracy wynika dla Flat-SQL**, oraz zadania, które się na nią powołują ([todo.md](../todo.md) – optymalizacja prototypu, [todo_future.md](../todo_future.md) – rozszerzenia). Hipotezy H1–H6 pochodzą z [benchmark-scenario.md](../research/benchmark-scenario.md).

**Identyfikatory zachowano z poprzedniej wersji bazy**, żeby nie unieważniać odwołań; luki w numeracji to pozycje przeniesione poza rdzeń (§13). Nowe prace z rewizji: [C9]–[C11], [L27]–[L28], [M2]–[M3], [S10].

Kategorie: [P] polecone przez promotora · [W] WebAssembly · [D] bazy w przeglądarce i porównywane silniki · [L] Local-First, CRDT, synchronizacja · [C] replikacja, zegary, praca offline · [S] składowanie i odtwarzanie · [M] model danych i zapytania · [E] edge computing · [B] metodyka pomiarów · [V] prywatność · [X] do pobrania · [Q] dokumentacja platform i źródła branżowe.

## Mapa: założenia projektu → literatura

| Założenie lub atut Flat-SQL | Prace rdzenia |
|---|---|
| Płaski model, obowiązkowy klucz, brak JOIN, przewidywalne wykonanie | [C7] Helland, [M2] PIQL, [M1] Stonebraker i Pavlo (krytyka), [D4] Riffle (krytyka), [L8] Antidote SQL |
| Dane w pamięci, wykonanie jednowątkowe | [S4] H-Store, [S1] DeWitt, [P1] rozdz. 3 i 7, [D5] Zeta-Lite |
| WAL fizyczny + snapshot, group commit | [S1] DeWitt, [S6] Malviya, [S10] Cao, [P1] rozdz. 3 |
| Odporność na awarie zapisu | [S8] Pillai, [Q1] trwałość IndexedDB |
| Zegar HLC, porządek znaczników | [C2] Kulkarni, [P1] rozdz. 8–9 |
| LWW per wiersz, tombstone'y, zbieżność | [L2] Shapiro, [L28] Almeida, [C9] Gray, [L24] CRR |
| Outbox przetrwały restart, praca offline | [C10] Coda, [L27] Simba, [C3] Bayou |
| Protokół push/pull z kursorami, log serwera | [C3] Bayou, [P1] rozdz. 11–12, [C7] Helland (idempotencja) |
| Atomowość: transakcja lokalnie, wiersz między replikami | [C8] Bailis (MAV), [L27] Simba, [C7] Helland |
| Ewolucja schematu w replikach | [L26] Cambria |
| Mały moduł WASM, ten sam rdzeń wszędzie | [P2] Haas, [W1] Jangda, [W11] Yan |
| Granica JS↔WASM (H3) | [D1] DuckDB-Wasm (Arrow IPC); brak badań recenzowanych – wkład własny |
| Lokalne zapytania zamiast sieci (H1) | [L1] Local-first, [D4] Riffle (budżet 16 ms), [E1] Shi |
| Hybryda: lokalny silnik + serwer koordynujący | [L20] Niinikoski, [C9] Gray (two-tier), [C3] Bayou |
| Privacy by Design | [V1] Cavoukian, [L1], [W2] Lehmann, [L11] Kuessner |
| Rozszerzenia: replikacja częściowa, reguły serwera, zapytania reaktywne | [C11] Cimbiosys, [C9] Gray, [M3] Gupta i Mumick, [D4] Riffle |
| Metodyka benchmarków | [B1] Georges, [B3] Raasveldt, [B5] YCSB |

---

## 1. Prace polecone przez promotora

- **[P1]** Kleppmann M. *Designing Data-Intensive Applications: The Big Ideas Behind Reliable, Scalable, and Maintainable Systems.* O'Reilly Media, 2017. ISBN 978-1-449-37332-0.
  Wydanie polskie: *Przetwarzanie danych w dużej skali. Niezawodność, skalowalność i łatwość konserwacji systemów*, Helion, Gliwice 2018 (ISBN wg Goodreads 978-83-283-4065-7, wg katalogu TUM 978-83-283-4066-4 – sprawdzić na egzemplarzu). Wydanie 2: Kleppmann M., Riccomini C., O'Reilly, 2026.
  **Plik:** brak (książka komercyjna). Bezpłatne uzupełnienie autora: notatki do wykładu *Distributed Systems*, University of Cambridge 2021/22 – [papers/kleppmann-distributed-systems-lecture-notes.pdf](papers/kleppmann-distributed-systems-lecture-notes.pdf) (zegary, rozgłaszanie, replikacja, CRDT). Bibliografie rozdziałów: [github.com/ept/ddia-references](https://github.com/ept/ddia-references). Notatka: [notes/ddia-kleppmann.md](notes/ddia-kleppmann.md).
  — Podręcznik systemów danych: bazy w pamięci, log i snapshot (rozdz. 3), kodowanie i ewolucja formatów (4), klienci offline jako replikacja multi-leader i LWW (5), wykonanie szeregowe (7), zawodność zegarów (8), granice spójności ostatecznej (9), CDC i kompaktowanie logu (11), klienci ze stanem i model push (12). **Dla projektu:** teoretyczna podstawa niemal każdej decyzji z [architecture.md](../architecture.md). **Zadania:** todo 0.2, 1.1, 2.1; future 6.1, 6.2, §7, 11.2, 12.1, 12.4.

- **[P2]** Haas A., Rossberg A., Schuff D. L., Titzer B. L., Holman M., Gohman D., Wagner L., Zakai A., Bastien J. F. *Bringing the Web up to Speed with WebAssembly.* PLDI 2017, ACM, s. 185–200. DOI: [10.1145/3062341.3062363](https://doi.org/10.1145/3062341.3062363). Wersja skrócona: Rossberg A. i in., Communications of the ACM 61(12), 2018, s. 107–115, DOI: [10.1145/3282510](https://doi.org/10.1145/3282510).
  **Plik:** [papers/2017-haas-bringing-the-web-up-to-speed-with-webassembly-pldi.pdf](papers/2017-haas-bringing-the-web-up-to-speed-with-webassembly-pldi.pdf); wersja CACM: [papers/2018-rossberg-bringing-the-web-up-to-speed-with-webassembly-cacm.pdf](papers/2018-rossberg-bringing-the-web-up-to-speed-with-webassembly-cacm.pdf). Notatka: [notes/haas-2017-webassembly.md](notes/haas-2017-webassembly.md).
  — Artykuł wprowadzający WebAssembly: bezpieczny, szybki, przenośny i zwarty kod; pamięć liniowa izolowana od kodu i stosu silnika; formalna semantyka. W PolyBenchC 7 z 24 programów w granicy 10% kodu natywnego, prawie wszystkie poniżej 2×; kod o 33,7% szybszy od asm.js przy 62,5% jego rozmiaru. **Dla projektu:** punkt odniesienia dla H2, rozmiaru modułu i argumentu „izolowanej piaskownicy”; artykuł nie mierzy kosztu granicy JS↔WASM (H3). **Zadania:** todo 1.8.

## 2. WebAssembly – wydajność i bezpieczeństwo

- **[W1]** Jangda A., Powers B., Berger E. D., Guha A. *Not So Fast: Analyzing the Performance of WebAssembly vs. Native Code.* USENIX ATC 2019.
  **Plik:** [papers/2019-jangda-not-so-fast-webassembly-vs-native.pdf](papers/2019-jangda-not-so-fast-webassembly-vs-native.pdf).
  — SPEC CPU przez Browsix-Wasm: średnio 1,55× wolniej w Chrome i 1,45× w Firefoksie, maksymalnie 2,5×. Przyczyny: więcej dostępów do pamięci (presja na rejestry), sprawdzanie stosu przy każdym wywołaniu, większy kod. **Dla projektu:** główne odniesienie dla H2; tłumaczy, dlaczego spłaszczenie rekurencyjnego `eval` dało w WASM tak duży zysk ([benchmark-results.md §5](../research/benchmark-results.md)).

- **[W2]** Lehmann D., Kinder J., Pradel M. *Everything Old is New Again: Binary Security of WebAssembly.* USENIX Security 2020.
  **Plik:** [papers/2020-lehmann-binary-security-of-webassembly.pdf](papers/2020-lehmann-binary-security-of-webassembly.pdf).
  — W pamięci liniowej brakuje kanarków stosu, ASLR, stron ochronnych i danych tylko do odczytu, więc błędy pamięci z C/C++ są w WASM znów wykorzystywalne. **Dla projektu:** precyzuje, co daje „izolowana piaskownica” z README – chroni hosta przed modułem, nie dane modułu przed jego błędami ani przed innymi skryptami strony; argument za Rustem bez `unsafe` i przeciw trzymaniu kluczy kryptograficznych w bazie. **Zadania:** todo 0.1, 0.2.

- **[W11]** Yan Y., Tu T., Zhao L., Zhou Y., Wang W. *Understanding the Performance of WebAssembly Applications.* IMC 2021. DOI: [10.1145/3487552.3487827](https://doi.org/10.1145/3487552.3487827).
  **Plik:** [papers/2021-yan-understanding-performance-webassembly-applications.pdf](papers/2021-yan-understanding-performance-webassembly-applications.pdf).
  — WASM vs JS w Chrome, Firefoksie i Edge przy różnych poziomach optymalizacji: optymalizacje LLVM nie są dostrojone do WASM i bywają szkodliwe, wyniki silnie zależą od przeglądarki, WASM zużywa znacznie więcej pamięci niż JS. **Dla projektu:** podstawa porównania poziomów optymalizacji i pomiaru pamięci. **Zadania:** todo 2.5, 2.7, 4.4.

## 3. Bazy danych w przeglądarce i porównywane silniki

- **[D1]** Kohn A., Moritz D., Raasveldt M., Mühleisen H., Neumann T. *DuckDB-Wasm: Fast Analytical Processing for the Web.* PVLDB 15(12), 2022, s. 3574–3577. DOI: [10.14778/3554821.3554847](https://doi.org/10.14778/3554821.3554847).
  **Plik:** [papers/2022-kohn-duckdb-wasm.pdf](papers/2022-kohn-duckdb-wasm.pdf).
  — DuckDB w WASM: zapytania w Web Workerze, czytanie plików lokalnych i zdalnych fragmentami, wyniki serializowane jako strumienie Arrow IPC czytane przez JS wprost ze sterty WASM. **Dla projektu:** właściwe narzędzie dla scenariusza „CSV 500 MB” (inna klasa systemu) i wzór binarnego przekazywania wyników. **Zadania:** todo 0.2, 2.4.

- **[D2]** Gaffney K. P., Prammer M., Brasfield L., Hipp D. R., Kennedy D., Patel J. M. *SQLite: Past, Present, and Future.* PVLDB 15(12), 2022, s. 3535–3547. DOI: [10.14778/3554821.3554842](https://doi.org/10.14778/3554821.3554842).
  **Plik:** [papers/2022-gaffney-sqlite-past-present-future.pdf](papers/2022-gaffney-sqlite-past-present-future.pdf).
  — Architektura SQLite (format stronicowy, B-drzewa, OLTP), porównanie z DuckDB, optymalizacje analityczne. **Dla projektu:** wyjaśnia przewagi SQLite w H4 (snapshot = kopia pliku, indeksy pokrywające). **Zadania:** todo 2.1.

- **[D4]** Litt G., Schiefer N., Schickling J., Jackson D. *Riffle: Reactive Relational State for Local-First Applications.* UIST 2023. DOI: [10.1145/3586183.3606801](https://doi.org/10.1145/3586183.3606801).
  **Plik:** [papers/2023-litt-riffle-reactive-relational-state.pdf](papers/2023-litt-riffle-reactive-relational-state.pdf).
  — Cały stan aplikacji w lokalnej relacyjnej bazie (SKDB) z reaktywnymi zapytaniami i synchronicznymi transakcjami w wątku UI; cel: odpowiedź w 16 ms (jedna klatka), próg Nielsena 100 ms. Znaczna część kodu UI to złączenia i grupowania. **Dla projektu:** najbliższa praca naukowa (lokalna baza SQL + Local-First); budżet 16 ms do interpretacji H1 i pauzy checkpointu; argument za zapytaniami reaktywnymi i przeciw całkowitej rezygnacji ze złączeń. **Zadania:** todo 4.10; future 5.1, 5.2.

- **[D5]** Zhang G. *Zeta-Lite: A Concurrent, Branchable In-Browser SQL Database for Agentic Memory.* arXiv:2609.01818, wrzesień 2026 (preprint bez recenzji).
  **Plik:** [papers/2026-zeta-lite-in-browser-sql-database.pdf](papers/2026-zeta-lite-in-browser-sql-database.pdf).
  — Serwerowy silnik Zeta skompilowany do `wasm32-unknown-unknown` z `wasm-bindgen` (2,87 MB gzip): MVCC w jednym wątku, rozgałęzianie bazy, dialekt PostgreSQL; 268–315 tys. odczytów punktowych/s (ok. 3,2–3,7 µs), w przeglądarce 5–15% poniżej natywnej. Trwałość tylko przez snapshot do OPFS na żądanie – autor przyznaje, że zmiany po ostatnim snapshocie giną. Kod zamknięty, jeden autor. **Dla projektu:** najnowszy system o tej samej architekturze „jeden rdzeń dla serwera i przeglądarki”; kontrast dla przyrostowego WAL (H5) i cel dla odczytu po kluczu. Cytować ostrożnie. **Zadania:** todo 2.4, 4.1.

## 4. Local-First, CRDT i synchronizacja

- **[L1]** Kleppmann M., Wiggins A., van Hardenberg P., McGranaghan M. *Local-First Software: You Own Your Data, in spite of the Cloud.* Onward! 2019, s. 154–178. DOI: [10.1145/3359591.3359737](https://doi.org/10.1145/3359591.3359737).
  **Plik:** [papers/2019-kleppmann-local-first-software.pdf](papers/2019-kleppmann-local-first-software.pdf).
  — Siedem ideałów oprogramowania local-first (brak spinnerów, wiele urządzeń, sieć opcjonalna, współpraca, długowieczność danych, prywatność, kontrola użytkownika); CRDT jako fundament; magazyn przeglądarki bywa czyszczony razem z ciasteczkami. **Dla projektu:** główna rama oceny architektury ([literature-comparison.md §3](literature-comparison.md#3-ideały-local-first)). **Zadania:** todo 1.5; future 1.3, 4.3, 6.3.

- **[L2]** Shapiro M., Preguiça N., Baquero C., Zawirski M. *A comprehensive study of Convergent and Commutative Replicated Data Types.* INRIA, RR-7506, 2011.
  **Plik:** [papers/2011-shapiro-comprehensive-study-crdt.pdf](papers/2011-shapiro-comprehensive-study-crdt.pdf) (kopia z serwisu kursu IISc; oryginał: HAL inria-00555588).
  — Formalne podstawy CRDT i katalog typów, m.in. rejestr LWW, liczniki PN, zbiory z tombstone'ami. **Dla projektu:** model replikacji Flat-SQL to mapa rejestrów LWW indeksowana kluczem – tu jest jego formalna definicja i dowód zbieżności. **Zadania:** todo 1.9; future 2.5.

- **[L8]** Lopes P., Sousa J., Balegas V., Ferreira C., Duarte S., Bieniusa A., Rodrigues R., Preguiça N. *Antidote SQL: Relaxed When Possible, Strict When Necessary.* arXiv:1902.03576, 2019.
  **Plik:** [papers/2019-lopes-antidote-sql.pdf](papers/2019-lopes-antidote-sql.pdf).
  — SQL nad bazą słabo spójną: semantyka update-wins / delete-wins deklarowana w schemacie, kolumny addytywne na licznikach ograniczonych (metoda escrow), klucze obce bez koordynacji. **Dla projektu:** wzór deklarowania scalania w SQL (`MERGE ADD`) i argument, że trudność replikacji leży w więzach integralności, nie w samych złączeniach. **Zadania:** future 2.2, 2.5, 5.2.

- **[L11]** Kuessner C., Mogk R., Wickert A.-K., Mezini M. *Encrypted Replicated Data Types within REScala.* PLF 2022 (preprint).
  **Plik:** [papers/2022-kuessner-encrypted-replicated-data-types.pdf](papers/2022-kuessner-encrypted-replicated-data-types.pdf).
  — Replikowane typy danych z automatycznym szyfrowaniem i uwierzytelnianiem, synchronizowane przez niezaufanych pośredników. **Dla projektu:** wzór szyfrowania end-to-end przy zachowaniu serwera synchronizacji jako przekaźnika. **Zadania:** future 4.3.

- **[L20]** Niinikoski J.-K. *Local-First Architecture: Investigating the Suitability for Modern Browser Applications.* Praca magisterska, Aalto University, 2025.
  **Plik:** [papers/2025-niinikoski-local-first-architecture-thesis.pdf](papers/2025-niinikoski-local-first-architecture-thesis.pdf).
  — Przegląd literatury i wywiady z twórcami aplikacji webowych: local-first poprawia responsywność i pracę offline, ale przenosi na klienta odpowiedzialności serwera; w pełni bezserwerowe systemy rzadko są praktyczne, realne są modele hybrydowe z minimalną koordynacją serwera. **Dla projektu:** akademickie potwierdzenie hybrydowej architektury Flat-SQL, zgodne z [analizą wdrożeń](../research/real-world-usage.md). **Zadania:** future 3.2.

- **[L24]** Yu W., Ignat C.-L. *Conflict-Free Replicated Relations for Multi-Synchronous Database Management at Edge.* IEEE SMDS 2020. HAL: [hal-02983557](https://hal.science/hal-02983557).
  **Plik:** [papers/2020-yu-ignat-conflict-free-replicated-relations.pdf](papers/2020-yu-ignat-conflict-free-replicated-relations.pdf).
  — CRDT dla relacyjnej bazy na urządzeniach brzegowych: każdy atrybut to rejestr LWW z własnym znacznikiem, wiersz ma „długość przyczynową” (licznik wstawień i usunięć zamiast tombstone'a), kolumny liczbowe mogą być licznikami CRDT; naruszenia więzów (unikalność, klucze obce) wykrywane przy scalaniu i naprawiane cofnięciem przegrywającej zmiany. **Dla projektu:** najbliższa praca naukowa do synchronizacji Flat-SQL – wzór LWW per kolumna, raportu konfliktów klucza i kompensacji. **Zadania:** future 2.3, §7, 12.3, 12.5.

- **[L26]** Litt G., van Hardenberg P., Henry O. *Cambria: Schema Evolution in Distributed Systems with Edit Lenses.* PaPoC 2021. DOI: [10.1145/3447865.3457963](https://doi.org/10.1145/3447865.3457963).
  **Plik:** [papers/2021-litt-cambria-schema-evolution.pdf](papers/2021-litt-cambria-schema-evolution.pdf) (CC BY 4.0).
  — Warstwa tłumacząca dane między wersjami schematu „w locie” dwukierunkowymi soczewkami, zintegrowana z Automerge. **Dla projektu:** repliki z różnymi wersjami schematu tabel – dziś konflikt schematu jest cichy. **Zadania:** todo 1.7; future 6.1.

- **[L27]** Go Y., Agrawal N., Aranya A., Ungureanu C. *Reliable, Consistent, and Efficient Data Sync for Mobile Apps.* USENIX FAST 2015, s. 359–372. *(nowa)*
  **Plik:** [papers/2015-go-simba-data-sync-mobile-apps.pdf](papers/2015-go-simba-data-sync-mobile-apps.pdf) (otwarty dostęp USENIX).
  — Badanie popularnych aplikacji mobilnych z usługami synchronizacji (Dropbox, Parse, Kinvey): przy przerwie w sieci, zamknięciu aplikacji lub zaniku zasilania aplikacje tracą lub psują dane. Simba: tabela (`sTable`) z wierszem jako jednostką spójności i atomowości – lokalnie i w synchronizacji; polityka synchronizacji per tabela; konflikty zgłaszane aplikacji. Autorzy świadomie **nie** dają atomowej synchronizacji wielu wierszy, uznając ją dla większości aplikacji za zbędną i kosztowną. **Dla projektu:** empiryczne uzasadnienie wbudowanej synchronizacji z trwałym outboksem oraz gwarancji „atomowy wiersz między replikami”. **Zadania:** todo 0.1; future 12.5, 12.7.

- **[L28]** Almeida P. S. *Approaches to Conflict-free Replicated Data Types.* ACM Computing Surveys 57(2), art. 51, 2024. DOI: [10.1145/3695249](https://doi.org/10.1145/3695249); arXiv:2310.18220. *(nowa)*
  **Plik:** [papers/2024-almeida-approaches-to-crdts.pdf](papers/2024-almeida-approaches-to-crdts.pdf).
  — Aktualny przegląd podejść do CRDT: oparte na stanie, na operacjach, delta-state i „czysto operacyjne”, z kompaktowaniem po osiągnięciu stabilności przyczynowej. **Dla projektu:** jedno recenzowane źródło dla trzech tez: outbox wysyłający bieżący stan zmienionych wierszy to synchronizacja delta-state; tombstone można usunąć po stabilności przyczynowej; zbieżność wynika z łączności, przemienności i idempotencji scalania. Zastępuje cztery wcześniejsze pozycje (§13). **Zadania:** future 6.3, 12.6.

## 5. Replikacja, zegary i praca offline

- **[C2]** Kulkarni S. S., Demirbas M., Madeppa D., Avva B., Leone M. *Logical Physical Clocks and Consistent Snapshots in Globally Distributed Databases.* OPODIS 2014 (LNCS 8878).
  **Plik:** [papers/2014-kulkarni-hybrid-logical-clocks.pdf](papers/2014-kulkarni-hybrid-logical-clocks.pdf).
  — Zegar HLC: przyczynowość jak zegar logiczny, wartość blisko czasu fizycznego, postać 64-bitowa (48 bitów czasu + 16 bitów licznika). Sekcja 4: wiadomości odsuwające zegar logiczny zbyt daleko od fizycznego są **ignorowane**, a incydent logowany i zgłaszany. **Dla projektu:** `sync.rs` realizuje postać zwartą bez tej reguły. **Zadania:** todo 1.1, 1.9; future 11.4.

- **[C3]** Terry D. B., Theimer M. M., Petersen K., Demers A. J., Spreitzer M. J., Hauser C. H. *Managing Update Conflicts in Bayou, a Weakly Connected Replicated Storage System.* SOSP 1995, s. 172–182.
  **Plik:** [papers/1995-terry-bayou-update-conflicts.pdf](papers/1995-terry-bayou-update-conflicts.pdf).
  — Replikacja dla słabo połączonych urządzeń: zapisy do dowolnej repliki, zapisy wstępne i zatwierdzone, serwer główny ustalający kolejność, anti-entropy, sprawdzanie zależności i procedury scalania aplikacji. **Dla projektu:** protoplasta architektury „lokalne zapisy + serwer porządkujący log” i wzorzec zamiarów zatwierdzanych przez serwer. **Zadania:** future §2.

- **[C7]** Helland P. *Life beyond Distributed Transactions: an Apostate's Opinion.* CIDR 2007.
  **Plik:** [papers/2007-helland-life-beyond-distributed-transactions.pdf](papers/2007-helland-life-beyond-distributed-transactions.pdf).
  — Encja z unikalnym kluczem jako zakres serializowalności; transakcje atomowe nie obejmują wielu encji; wiadomości muszą być idempotentne. **Dla projektu:** najmocniejsze uzasadnienie płaskiego modelu i idempotentnych rund synchronizacji. **Zadania:** todo 0.1.

- **[C8]** Bailis P., Davidson A., Fekete A., Ghodsi A., Hellerstein J. M., Stoica I. *Highly Available Transactions: Virtues and Limitations.* PVLDB 7(3), 2013, s. 181–192.
  **Plik:** [papers/2014-bailis-highly-available-transactions.pdf](papers/2014-bailis-highly-available-transactions.pdf).
  — Które gwarancje da się zapewnić bez koordynacji: m.in. Read Committed i Monotonic Atomic View (wszystkie albo żadne zapisy transakcji) tak, zapobieganie utraconym aktualizacjom – nie. **Dla projektu:** język do opisu gwarancji po synchronizacji; MAV jest osiągalne, gdyby atomowość per wiersz okazała się za słaba. **Zadania:** future 12.7.

- **[C9]** Gray J., Helland P., O'Neil P., Shasha D. *The Dangers of Replication and a Solution.* ACM SIGMOD 1996, s. 173–182. DOI: [10.1145/233269.233330](https://doi.org/10.1145/233269.233330). *(nowa)*
  **Plik:** [papers/1996-gray-dangers-of-replication.pdf](papers/1996-gray-dangers-of-replication.pdf) (kopia z grupy czytelniczej Boston University).
  — Replikacja „zapis wszędzie” jest niestabilna przy wzroście skali (10× więcej węzłów i ruchu daje 1000× więcej zakleszczeń lub uzgodnień). Schemat ze znacznikami czasu („nowszy znacznik wygrywa”) zbiega się, ale **gubi aktualizacje**; aktualizacje przemienne (np. dodawanie stałej do liczby) tego problemu nie mają. Rozwiązanie: replikacja dwuwarstwowa – węzły mobilne wykonują transakcje wstępne na lokalnej kopii, a węzły bazowe, właściciele kopii głównych, wykonują je ponownie jako transakcje bazowe z kryterium akceptacji i informują o odrzuceniu. **Dla projektu:** w jednej pracy uzasadnienie i granica LWW (utracone aktualizacje), argument za licznikami `MERGE ADD`, model tabel prowadzonych przez serwer, zamiarów zatwierdzanych przez serwer i węzłów edge jako warstwy bazowej. **Zadania:** future 2.1–2.5, §7, 7.2, 12.3.

- **[C10]** Kistler J. J., Satyanarayanan M. *Disconnected Operation in the Coda File System.* ACM TOCS 10(1), 1992, s. 3–25. DOI: [10.1145/146941.146942](https://doi.org/10.1145/146941.146942) (wersja konferencyjna: SOSP 1991). *(nowa)*
  **Plik:** [papers/1991-kistler-satyanarayanan-coda-disconnected-operation.pdf](papers/1991-kistler-satyanarayanan-coda-disconnected-operation.pdf) (wersja SOSP, kopia z kursu University of Michigan).
  — Klasyczna praca o pracy bez połączenia: klient gromadzi zawczasu potrzebne dane (hoarding, z priorytetami i wymianą w pamięci podręcznej), offline zapisuje zmiany w trwałym logu (w pamięci odtwarzalnej z WAL), log skraca – nowszy zapis pliku unieważnia wcześniejsze wpisy – a po połączeniu odtwarza go na serwerze w jednej transakcji z wykrywaniem konfliktów; tymczasowe identyfikatory obiektów zastępowane są przy reintegracji stałymi. W praktyce konfliktów prawie nie było. **Dla projektu:** akademicki odpowiednik outboksu (scalanie kolejnych zmian wiersza, trwałość po restarcie, idempotentna reintegracja), wzór trybu pamięci podręcznej i obsługi kluczy tworzonych offline. **Zadania:** future 1.2, 7.1, 12.1, 12.4.

- **[C11]** Ramasubramanian V., Rodeheffer T. L., Terry D. B., Walraed-Sullivan M., Wobber T., Marshall C. C., Vahdat A. *Cimbiosys: A Platform for Content-based Partial Replication.* USENIX NSDI 2009. *(nowa)*
  **Plik:** [papers/2009-ramasubramanian-cimbiosys-partial-replication.pdf](papers/2009-ramasubramanian-cimbiosys-partial-replication.pdf) (Microsoft Research).
  — Replikacja częściowa z filtrem treści zdefiniowanym przez każde urządzenie. Gwarancja „ostatecznej spójności filtra”: urządzenie w końcu przechowuje dokładnie te elementy, których najnowsza wersja pasuje do filtra; elementy, które przestały pasować, usuwa po komunikacie *move-out*. Metadane synchronizacji rosną z liczbą urządzeń, nie elementów. **Dla projektu:** wzór replikacji częściowej, którą analiza wdrożeń uznała za obowiązkową przy wzroście danych – w tym zasady, że wiersz wychodzący z zakresu klienta jest usuwany tylko lokalnie. **Zadania:** future 3.2, 3.4.

## 6. Składowanie, WAL i odtwarzanie po awarii

- **[S1]** DeWitt D. J., Katz R. H., Olken F., Shapiro L. D., Stonebraker M. R., Wood D. *Implementation Techniques for Main Memory Database Systems.* SIGMOD 1984, s. 1–8.
  **Plik:** [papers/1984-dewitt-main-memory-database-systems.pdf](papers/1984-dewitt-main-memory-database-systems.pdf).
  — Struktury i odtwarzanie dla baz w pamięci; pierwszy drukowany opis **group commit** (jeden zapis logu dla grupy transakcji). **Dla projektu:** źródło group commitu w `FlatDB.flush()` i modelu „log + checkpoint”.

- **[S4]** Stonebraker M., Madden S., Abadi D. J., Harizopoulos S., Hachem N., Helland P. *The End of an Architectural Era (It's Time for a Complete Rewrite).* VLDB 2007, s. 1150–1160.
  **Plik:** [papers/2007-stonebraker-end-of-an-architectural-era.pdf](papers/2007-stonebraker-end-of-an-architectural-era.pdf).
  — H-Store: dane w pamięci, wykonanie jednowątkowe, specjalizacja zamiast „jednego rozmiaru dla wszystkich”; logowanie, blokady, latche i bufor stron to większość pracy klasycznego silnika. **Dla projektu:** uzasadnienie wąsko wyspecjalizowanego, jednowątkowego silnika w pamięci; Flat-SQL nie płaci za blokady i bufor stron, płaci za logowanie (undo log + WAL + outbox – H4).

- **[S6]** Malviya N., Weisberg A., Madden S., Stonebraker M. *Rethinking Main Memory OLTP Recovery.* ICDE 2014, s. 604–615.
  **Plik:** [papers/2014-malviya-rethinking-main-memory-oltp-recovery.pdf](papers/2014-malviya-rethinking-main-memory-oltp-recovery.pdf).
  — W VoltDB logowanie poleceń daje ok. 1,5× większą przepustowość niż logowanie fizjologiczne w stylu ARIES, kosztem dłuższego odtwarzania i wymogu determinizmu. **Dla projektu:** uzasadnienie logu fizycznego – te same wpisy WAL służą synchronizacji, a odtworzenie nie zależy od zegara ani wyników zapytań.

- **[S8]** Pillai T. S., Chidambaram V., Alagappan R., Al-Kiswany S., Arpaci-Dusseau A. C., Arpaci-Dusseau R. H. *All File Systems Are Not Created Equal: On the Complexity of Crafting Crash-Consistent Applications.* OSDI 2014, s. 433–448.
  **Plik:** [papers/2014-pillai-all-file-systems-are-not-created-equal.pdf](papers/2014-pillai-all-file-systems-are-not-created-equal.pdf).
  — Narzędzia BOB i ALICE: poprawność protokołów zapisu zależy od subtelnych właściwości systemu plików; 60 podatności w 11 dojrzałych aplikacjach, w tym brak `fsync` katalogu nadrzędnego po utworzeniu lub zmianie nazwy pliku. **Dla projektu:** dotyczy sekwencji `snapshot.tmp → rename → obcięcie WAL` w `FileStorage` i braku `fsync` w `FsStore`. **Zadania:** todo 1.2, 1.3, 1.6.

- **[S10]** Cao T., Vaz Salles M., Sowell B., Yue Y., Demers A., Gehrke J., White W. *Fast Checkpoint Recovery Algorithms for Frequently Consistent Applications.* ACM SIGMOD 2011, s. 265–276. DOI: [10.1145/1989323.1989352](https://doi.org/10.1145/1989323.1989352). *(nowa)*
  **Plik:** [papers/2011-cao-fast-checkpoint-recovery.pdf](papers/2011-cao-fast-checkpoint-recovery.pdf) (kopia z kursu Georgia Tech).
  — Checkpointy aplikacji w pamięci o częstych punktach spójności – na przykładzie gier MMO (stan spójny na granicy „ticku”) i H-Store. Porównuje „naiwny snapshot” (synchroniczna kopia całego stanu – skok latencji), kopiowanie przy aktualizacji i dwa nowe algorytmy bez blokad i kopii masowych, które kosztem dodatkowej pamięci dają niemal stałą latencję. **Dla projektu:** checkpoint Flat-SQL to dziś naiwny snapshot (35–49 ms, czyli 2–3 klatki), a gry są pierwszym zastosowaniem z prezentacji – praca daje gotowe kierunki i metrykę (najdłuższa pauza). **Zadania:** todo 2.6, 4.10.

## 7. Model danych i zapytania

- **[M1]** Stonebraker M., Pavlo A. *What Goes Around Comes Around... And Around...* ACM SIGMOD Record 53(2), 2024, s. 21–37.
  **Plik:** [papers/2024-stonebraker-pavlo-what-goes-around-comes-around.pdf](papers/2024-stonebraker-pavlo-what-goes-around-comes-around.pdf).
  — 20 lat modeli danych: systemy NoSQL niemal powszechnie dodały interfejs SQL (CQL w Cassandrze, PartiQL w DynamoDB, SQL++), model relacyjny pozostał dominujący. **Dla projektu:** Flat-SQL wpisuje się w trend „składnia SQL nad modelem klucz–wiersz” (najbliższy odpowiednik: CQL) – i podlega tej samej krytyce braku złączeń. **Zadania:** future 5.2.

- **[M2]** Armbrust M., Curtis K., Kraska T., Fox A., Franklin M. J., Patterson D. A. *PIQL: Success-Tolerant Query Processing in the Cloud.* PVLDB 5(3), 2011, s. 181–192. DOI: [10.14778/2078331.2078334](https://doi.org/10.14778/2078331.2078334); arXiv:1111.7166. *(nowa)*
  **Plik:** [papers/2011-armbrust-piql-scale-independence.pdf](papers/2011-armbrust-piql-scale-independence.pdf).
  — Deklaratywny język, który dla każdego zapytania wylicza górne ograniczenie liczby operacji na magazynie klucz–wartość („niezależność od skali”); zapytania bez ograniczenia są odrzucane przy kompilacji. Klasa zapytań o stałym koszcie to zapytania z `LIMIT` bez złączeń albo ze złączeniem tylko po unikalnym kluczu głównym. Autorzy krytykują języki, które dla skalowalności po prostu usuwają złączenia (np. GQL). **Dla projektu:** precyzuje tezę o „przewidywalnym wykonaniu” – płaski model daje przewidywalny plan, ale ograniczony koszt mają tylko zapytania po kluczu lub indeksie z limitem; pokazuje też, że złączenie po kluczu głównym nie psuje przewidywalności. **Zadania:** todo 0.1; future 5.2.

- **[M3]** Gupta A., Mumick I. S. *Maintenance of Materialized Views: Problems, Techniques, and Applications.* IEEE Data Engineering Bulletin 18(2), 1995, s. 3–18. *(nowa)*
  **Plik:** brak – bezpłatny numer biuletynu na stronie IEEE TCDE ([sites.computer.org/debull](http://sites.computer.org/debull/95JUN-CD.pdf)).
  — Taksonomia i techniki przyrostowego utrzymania widoków: według klasy widoku, dostępnych informacji i rodzaju modyfikacji; agregaty `COUNT`/`SUM` dają się aktualizować przyrostowo, `MIN`/`MAX` przy usunięciu wymagają ponownego przeliczenia. **Dla projektu:** podstawa projektowa zapytań reaktywnych w modelu jednotabelowym (Riffle [D4] pokazuje potrzebę, ta praca – technikę). **Zadania:** future 5.1.

## 8. Edge computing

- **[E1]** Shi W., Cao J., Zhang Q., Li Y., Xu L. *Edge Computing: Vision and Challenges.* IEEE Internet of Things Journal 3(5), 2016, s. 637–646. DOI: [10.1109/JIOT.2016.2579198](https://doi.org/10.1109/JIOT.2016.2579198).
  **Plik:** [papers/2016-shi-edge-computing-vision-and-challenges.pdf](papers/2016-shi-edge-computing-vision-and-challenges.pdf) (kopia z serwisu kursu).
  — Definicja: „edge” to dowolne zasoby obliczeniowe i sieciowe na drodze między źródłem danych a chmurą – także smartfon. **Dla projektu:** uzasadnia ujęcie z README, w którym urządzenie użytkownika jest brzegiem sieci, i odróżnia je od węzłów edge synchronizowanych z bazą główną. **Zadania:** future 7.3.

## 9. Metodyka pomiarów

- **[B1]** Georges A., Buytaert D., Eeckhout L. *Statistically Rigorous Java Performance Evaluation.* OOPSLA 2007, s. 57–76.
  **Plik:** [papers/2007-georges-statistically-rigorous-performance-evaluation.pdf](papers/2007-georges-statistically-rigorous-performance-evaluation.pdf).
  — Najlepszy wynik lub pojedynczy przebieg prowadzą w środowiskach z JIT do błędnych wniosków; zalecane wielokrotne uruchomienia maszyny wirtualnej i przedziały ufności. **Dla projektu:** jak wzmocnić H2–H5 (szum ±30% opisany w scenariuszu). **Zadania:** todo 4.2.

- **[B3]** Raasveldt M., Holanda P., Gubner T., Mühleisen H. *Fair Benchmarking Considered Difficult: Common Pitfalls In Database Performance Testing.* DBTest 2018. DOI: [10.1145/3209950.3209955](https://doi.org/10.1145/3209950.3209955).
  **Plik:** [papers/2018-raasveldt-fair-benchmarking-considered-difficult.pdf](papers/2018-raasveldt-fair-benchmarking-considered-difficult.pdf).
  — Pułapki: brak powtarzalności, nieoptymalny system porównywany, „jabłka i pomarańcze”, nadmierne strojenie, przebiegi zimne i gorące, niepoprawne wyniki. **Dla projektu:** lista kontrolna dla [benchmark-scenario.md](../research/benchmark-scenario.md) ([porównanie §8](literature-comparison.md#8-metodyka-benchmarków)). **Zadania:** todo 0.1, 4.1, 4.3, 4.6.

- **[B5]** Cooper B. F., Silberstein A., Tam E., Ramakrishnan R., Sears R. *Benchmarking Cloud Serving Systems with YCSB.* SoCC 2010, s. 143–154.
  **Plik:** [papers/2010-cooper-ycsb.pdf](papers/2010-cooper-ycsb.pdf).
  — Standardowe obciążenia dla magazynów adresowanych kluczem (proporcje odczyt/zapis, rozkład Zipfa). **Dla projektu:** Flat-SQL jest właśnie takim magazynem – obecny benchmark losuje klucze równomiernie. **Zadania:** future 12.8.

## 10. Prywatność

- **[V1]** Cavoukian A. *Privacy by Design: The 7 Foundational Principles – Implementation and Mapping of Fair Information Practices.* Information and Privacy Commissioner of Ontario.
  **Plik:** [papers/2009-cavoukian-privacy-by-design-principles.pdf](papers/2009-cavoukian-privacy-by-design-principles.pdf) (kopia z serwisu kursu University of Waterloo).
  — Siedem zasad Privacy by Design (prywatność domyślnie, ochrona w całym cyklu życia, przejrzystość) z mapowaniem na zasady FIP. Uzupełnienie prawne: art. 25 RODO. **Dla projektu:** źródło pojęcia z README; pozwala ocenić, które zasady prototyp spełnia ([porównanie §2.4](literature-comparison.md#24-izolacja-bezpieczeństwo-i-privacy-by-design)). **Zadania:** todo 0.1; future 4.4, 5.6.

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
- Źródła o wdrożeniach (Notion, Actual Budget, Linear, Community Health Toolkit, PowerSync, LiteFS, Turso i in.): [research/real-world-usage.md §5](../research/real-world-usage.md#5-źródła).

## 13. Pozycje poza rdzeniem

Pliki przeniesiono do `papers/_poza-rdzeniem/`. Kryteria wyłączenia: [search-methodology.md §4](search-methodology.md#4-dobór-rdzenia). Pozycje można przywrócić, jeśli praca dyplomowa obejmie dany wątek.

| ID | Praca | Powód wyłączenia |
|---|---|---|
| [P2a] | Rossberg i in., CACM 2018 | wersja [P2] – dołączona do jego wpisu (plik w `papers/`) |
| [L23] | Kleppmann, notatki do wykładu | dołączone do wpisu [P1] (plik w `papers/`) |
| [W3] | Hilbig i in., binaria WASM, WWW 2021 | kontekst zastosowań WASM, bez wpływu na decyzje |
| [W4] | Bosamiya i in., dowodliwa piaskownica, USENIX Sec 2022 | dotyczy implementacji środowisk uruchomieniowych, nie aplikacji |
| [W5] | Zhang i in., przegląd środowisk WASM, 2024 | przegląd ogólny; [P2], [W1] wystarczają |
| [W6] | Döllerer, Engelke, memory64, VMIL 2024 | wniosek (nie przechodzić na memory64) wystarcza w [Q6] |
| [W7] | Wasm-R3, OOPSLA 2024 | argument za realistycznym obciążeniem pokrywa [B5] |
| [W8] | He i in., WASM w przemyśle, 2025 | kontekst |
| [W9] | Corrias i in., podatności webowe w WASM, 2026 | preprint; tezę pokrywa recenzowane [W2] |
| [W10] | Specyfikacja WebAssembly 3.0 | po usunięciu zadania o JS String Builtins nie jest potrzebna; stan standardu – [Q6] |
| [D3] | Haffner, Dittrich, kompilacja SQL do WASM, 2021 | JIT zapytań nie opłaca się przy zapytaniach trwających mikrosekundy; brak zadania |
| [L3] | Preguiça, przegląd CRDT, 2018 | zastąpiony aktualnym przeglądem [L28] |
| [L4] | Gomes i in., weryfikacja SEC, OOPSLA 2017 | formalna weryfikacja poza zakresem pracy |
| [L5] | Kleppmann, Beresford, JSON CRDT, 2017 | model dokumentowy, nie płaskie tabele |
| [L6] | Almeida i in., delta-state, 2016 | pokryty przez [L28] (ten sam autor) |
| [L7] | Baquero i in., pure op-based, 2017 | kryterium stabilności przyczynowej pokryte przez [L28] |
| [L9] | Edwards i in., zmiany schematu na żywo, 2023 | problem otwarty opisany też w [L26], które daje rozwiązanie |
| [L10] | Edwards, Petricek, Baseline, 2025 | preprint; wersjonowanie danych poza planami |
| [L12] | Kleppmann, Howard, BEC, 2020 | dotyczy sieci bez zaufanego serwera; Flat-SQL ma serwer jako autorytet |
| [L13] | Merkle-CRDTs, 2020 | synchronizacja bez serwera – poza architekturą |
| [L14] | Meyer, uzgadnianie zakresów, 2023 | zadanie przeniesione do przyszłych i uzależnione od trwałego logu (future 3.7) |
| [L15] | Collabs, 2022 | biblioteka współpracy, bez wpływu na projekt |
| [L16] | Gentle, Kleppmann, Eg-walker, 2025 | edycja tekstu poza silnikiem; wystarcza [L1] |
| [L17] | Sal, 2026 | formalna weryfikacja, preprint |
| [L18] | Jacob i in., kontrola dostępu local-first, 2026 | kontrola dostępu bez zaufanego serwera; preprint |
| [L19] | Datalog dla CRDT, 2026 | formalna specyfikacja, preprint |
| [L21] | Niinikoski, Fagerholm, materiały uzupełniające | sam artykuł niedostępny – [X6] |
| [L22] | Otsamo, praca licencjacka, 2025 | słabsze źródło tez obecnych w [L1] i [C9] |
| [L25] | Tomter, praca magisterska, 2021 | powiela [L24] (wersja na SQLite) |
| [C1] | Lamport 1978 | porządek znaczników opisany w [C2] i [P1] rozdz. 9 |
| [C4] | Dynamo, SOSP 2007 | argument za dostępem po kluczu pokrywa [C7]; LWW – [C9] |
| [C5] | Cassandra, 2010 | artykuł nie opisuje CQL; analogię z CQL daje [M1] |
| [C6] | Bigtable, OSDI 2006 | atomowość per wiersz pokrywają [C7] i [L27] |
| [S2] | ARIES, TODS 1992 | WAL opisany w [P1] rozdz. 3 i 7; Flat-SQL ma log tylko-redo |
| [S3] | LSM-tree, 1996 | kierunek dla checkpointu lepiej pokrywa [S10] |
| [S5] | Harizopoulos i in., OLTP, SIGMOD 2008 | wniosek zawarty w [S4] |
| [S7] | SiloR, OSDI 2014 | równoległe odtwarzanie wymaga wątków, czyli COOP/COEP – sprzeczne z atutem |
| [S9] | Zheng i in., awarie zasilania, OSDI 2014 | model awarii pokrywa [S8] |
| [E2] | Satyanarayanan, 2017 | definicję edge daje [E1] |
| [E3] | Hall, Ramachandran, 2019 | funkcje serverless, nie bazy |
| [E4] | Sledge, 2020 | środowisko uruchomieniowe, nie bazy |
| [E5] | Faasm, 2020 | „snapshot jako obraz pamięci” łamie wspólny format plików Rust/WASM – zadanie usunięte |
| [E6] | Ménétrey i in., 2022 | artykuł pozycyjny; teza zweryfikowana przez sam prototyp |
| [E7] | Colosi i in., 2025 | warsztat; wniosek pokrywają [D1] i H1 |
| [E8] | Lumos, 2025 | WASM poza przeglądarką, serializacja I/O kontenerów |
| [B2] | Barrett i in., rozgrzewka VM, 2017 | uzupełnia [B1]; wystarcza jedno źródło |
| [B4] | Taipalus, przegląd porównań DBMS, 2023 | wniosek pokrywa [B3] |
| [B6] | Schwarz i in., zegary w JS, 2017 | rozdzielczość zegarów opisuje [Q5] |
| [Q9], [Q10] | wystąpienia Kleppmanna; FOSDEM i Smashing Magazine | bez bezpośredniego zastosowania; LWW per pole pokrywa [L24] |
