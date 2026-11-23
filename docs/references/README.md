# Spis literatury

Prace, na których opiera się Flat-SQL, pogrupowane według obszarów. Przy każdej pracy: o czym jest i do czego posłużyła w projekcie. Jak te prace łączą się w uzasadnienie budowy silnika, opisuje [literatura a budowa Flat-SQL](literature.md). Jak je wyszukano i wybrano: [metodyka przeglądu](search-methodology.md).

W dokumentach projektu prace cytuje się jako „Autor (rok)”. Pliki PDF są w `papers/`, poza repozytorium, bo wiele z nich to wersje autorskie nieprzeznaczone do rozpowszechniania. W pracy dyplomowej cytuje się wersje wydawnicze.

## Podstawy

- **Kleppmann (2017).** *Designing Data-Intensive Applications.* O'Reilly. Wydanie polskie: *Przetwarzanie danych w dużej skali*, Helion 2018.
  Podręcznik systemów danych: bazy w pamięci, log i snapshot, zgodność formatów, replikacja, zawodność zegarów, strumienie zmian. W projekcie podstawa niemal każdej decyzji, zwłaszcza trwałości (rozdz. 3), zmian struktury (rozdz. 4) i protokołu z kursorem (rozdz. 11).
- **Stonebraker i in. (2007).** *The End of an Architectural Era (It's Time for a Complete Rewrite).* VLDB 2007.
  W klasycznym silniku większość pracy to logowanie, blokady i bufor stron. Wyspecjalizowany silnik w pamięci, jednowątkowy, pozbywa się blokad i bufora. W projekcie uzasadnienie danych w pamięci i transakcji wykonywanych po kolei; wyjaśnia, dlaczego Flat-SQL czyta szybciej, a zapisuje wolniej niż SQLite.
- **DeWitt i in. (1984).** *Implementation Techniques for Main Memory Database Systems.* SIGMOD 1984.
  Struktury i odtwarzanie baz w pamięci; pierwszy opis grupowania zatwierdzeń (group commit). W projekcie źródło modelu „log i checkpoint” oraz zapisu wielu zatwierdzeń naraz.

## Praca offline i local-first

- **Kleppmann i in. (2019).** *Local-First Software: You Own Your Data, in spite of the Cloud.* Onward! 2019. DOI: [10.1145/3359591.3359737](https://doi.org/10.1145/3359591.3359737).
  Siedem ideałów oprogramowania, które trzyma dane na urządzeniu: brak czekania, wiele urządzeń, sieć opcjonalna, współpraca, długowieczność danych, prywatność, kontrola użytkownika. W projekcie główna rama oceny; ostrzega też, że przeglądarka czasem czyści dane strony.
- **Kistler i Satyanarayanan (1992).** *Disconnected Operation in the Coda File System.* ACM TOCS 10(1). DOI: [10.1145/146941.146942](https://doi.org/10.1145/146941.146942).
  Klasyczna praca o pracy bez połączenia: trwały log zmian offline, skracany tak, że nowszy zapis unieważnia starsze, i odtwarzany na serwerze po połączeniu. W projekcie wzór kolejki zmian; przyda się przy trybie pamięci podręcznej i trwałym logu serwera.
- **Terry i in. (1995).** *Managing Update Conflicts in Bayou, a Weakly Connected Replicated Storage System.* SOSP 1995.
  Zapisy do dowolnej repliki, serwer ustalający ich kolejność, zapisy wstępne i zatwierdzone. W projekcie wzór protokołu synchronizacji i zamiarów zatwierdzanych przez backend.
- **Go i in. (2015).** *Reliable, Consistent, and Efficient Data Sync for Mobile Apps.* USENIX FAST 2015.
  Aplikacje mobilne z gotowymi usługami synchronizacji tracą dane przy przerwie w sieci. System Simba synchronizuje tabele z wierszem jako jednostką atomowości. W projekcie uzasadnienie trwałej kolejki zmian i gwarancji „między urządzeniami atomowy jest wiersz”.
- **Litt i in. (2023).** *Riffle: Reactive Relational State for Local-First Applications.* UIST 2023. DOI: [10.1145/3586183.3606801](https://doi.org/10.1145/3586183.3606801).
  Cały stan aplikacji w lokalnej bazie relacyjnej z zapytaniami w wątku interfejsu, w budżecie jednej klatki (16 ms). W projekcie miara „bez czekania”, argument za zapytaniami reaktywnymi i za złączeniami.

## Trwałość zapisu

- **Malviya i in. (2014).** *Rethinking Main Memory OLTP Recovery.* ICDE 2014.
  Logowanie poleceń daje około 1,5 raza większą przepustowość niż logowanie wyników, kosztem dłuższego odtwarzania. W projekcie uzasadnienie WAL opisującego wynik, bo te same wpisy służą synchronizacji.
- **Pillai i in. (2014).** *All File Systems Are Not Created Equal: On the Complexity of Crafting Crash-Consistent Applications.* OSDI 2014.
  System plików utrwala operacje w innej kolejności, niż je wykonano; 60 błędów w 11 dojrzałych aplikacjach, najczęściej brak utrwalenia katalogu. W projekcie model awarii testów zapisu i poprawka checkpointu na plikach.
- **Cao i in. (2011).** *Fast Checkpoint Recovery Algorithms for Frequently Consistent Applications.* SIGMOD 2011. DOI: [10.1145/1989323.1989352](https://doi.org/10.1145/1989323.1989352).
  Checkpointy stanu w pamięci, na przykładzie gier. Naiwny snapshot daje pauzę rosnącą ze stanem, kopia przy zmianie pauzę prawie stałą. W projekcie metryka „najdłuższa pauza” i kierunek checkpointu krokami.

## Synchronizacja i konflikty

- **Shapiro i in. (2011).** *A comprehensive study of Convergent and Commutative Replicated Data Types.* INRIA RR-7506.
  Podstawy bezkonfliktowych typów replikowanych (CRDT), w tym rejestru „ostatni zapis wygrywa” i liczników. W projekcie formalne uzasadnienie zbieżności urządzeń; przyda się przy licznikach scalanych przez dodawanie.
- **Almeida (2024).** *Approaches to Conflict-free Replicated Data Types.* ACM Computing Surveys 57(2). DOI: [10.1145/3695249](https://doi.org/10.1145/3695249).
  Przegląd podejść do CRDT, w tym wysyłania stanu zmienionych elementów (delta-state) i usuwania znaczników usunięcia. W projekcie uzasadnienie kolejki zmian prowadzonej od włączenia synchronizacji; przyda się przy usuwaniu znaczników usunięcia.
- **Kulkarni i in. (2014).** *Logical Physical Clocks and Consistent Snapshots in Globally Distributed Databases.* OPODIS 2014.
  Zegar hybrydowy: kolejność przyczynowa jak zegar logiczny, wartość blisko czasu fizycznego, 64 bity. Zaleca odrzucanie znaczników zbyt daleko w przyszłości. W projekcie zegar znaczników i granica dryfu.
- **Gray i in. (1996).** *The Dangers of Replication and a Solution.* SIGMOD 1996. DOI: [10.1145/233269.233330](https://doi.org/10.1145/233269.233330).
  Replikacja „zapis wszędzie” traci stabilność ze skalą, a reguła „nowszy znacznik wygrywa” gubi współbieżne zmiany. Rozwiązanie: transakcje wstępne na urządzeniu zatwierdzane przez właściciela kopii głównej. W projekcie granica reguły konfliktów i podstawa projektu serwera, liczników i węzłów edge.
- **Helland (2007).** *Life beyond Distributed Transactions: an Apostate's Opinion.* CIDR 2007.
  Encja z unikalnym kluczem jako granica atomowości, wiadomości idempotentne. W projekcie najmocniejsze uzasadnienie płaskiego modelu i powtarzalnych rund synchronizacji.

## Model danych i zapytania

- **Armbrust i in. (2011).** *PIQL: Success-Tolerant Query Processing in the Cloud.* PVLDB 5(3). DOI: [10.14778/2078331.2078334](https://doi.org/10.14778/2078331.2078334).
  Język, który dla każdego zapytania wylicza górne ograniczenie liczby odczytów. Stały koszt mają zapytania z limitem bez złączeń albo ze złączeniem po kluczu. W projekcie precyzuje, co znaczy „przewidywalne wykonanie”.
- **Gaffney i in. (2022).** *SQLite: Past, Present, and Future.* PVLDB 15(12). DOI: [10.14778/3554821.3554842](https://doi.org/10.14778/3554821.3554842).
  Architektura SQLite: format stronicowy, B-drzewa, instrukcje kompilowane raz. W projekcie wyjaśnia przewagi SQLite w snapshocie i liczeniu po indeksie; wzór planu w instrukcji przygotowanej.

## WebAssembly i bazy w przeglądarce

- **Haas i in. (2017).** *Bringing the Web up to Speed with WebAssembly.* PLDI 2017. DOI: [10.1145/3062341.3062363](https://doi.org/10.1145/3062341.3062363).
  Artykuł wprowadzający WebAssembly: szybki, zwarty, przenośny kod w izolowanej pamięci. Jedynym niedeterminizmem arytmetyki są bity NaN. W projekcie punkt odniesienia dla kosztu WebAssembly; uzasadnienie jednego zapisu NaN.
- **Jangda i in. (2019).** *Not So Fast: Analyzing the Performance of WebAssembly vs. Native Code.* USENIX ATC 2019.
  Programy SPEC w WebAssembly średnio 1,45–1,55 raza wolniejsze niż natywnie. W projekcie źródło hipotezy o koszcie WebAssembly, którą pomiar obalił.
- **Yan i in. (2021).** *Understanding the Performance of WebAssembly Applications.* IMC 2021. DOI: [10.1145/3487552.3487827](https://doi.org/10.1145/3487552.3487827).
  Optymalizacje kompilatora bywają dla WebAssembly szkodliwe, a pamięć modułu nie maleje. W projekcie źródło hipotezy o kompilacji pod rozmiar, którą pomiar obalił, i metody pomiaru pamięci.
- **Döllerer i Engelke (2024).** *Performant Bounds Checking for 64-Bit WebAssembly.* VMIL 2024. DOI: [10.1145/3689490.3690400](https://doi.org/10.1145/3689490.3690400).
  Przy pamięci 64-bitowej przeglądarki sprawdzają granice programowo, co średnio podwaja koszt dostępu. W projekcie uzasadnienie pozostania przy module 32-bitowym.
- **Kohn i in. (2022).** *DuckDB-Wasm: Fast Analytical Processing for the Web.* PVLDB 15(12). DOI: [10.14778/3554821.3554847](https://doi.org/10.14778/3554821.3554847).
  Silnik analityczny w przeglądarce, wynik przekazywany jako bufor czytany wprost z pamięci modułu. W projekcie wzór binarnego formatu wyniku i granica zastosowań: analityka to inny silnik.
- **Zhang (2026).** *Zeta-Lite: A Concurrent, Branchable In-Browser SQL Database for Agentic Memory.* arXiv:2609.01818 (praca bez recenzji).
  Serwerowy silnik Rust w przeglądarce, z wieloma wersjami wierszy i gałęziami bazy, zapisujący snapshot tylko na żądanie. W projekcie najbliższy system i cel czasu odczytu po kluczu; kod zamknięty, więc niemierzony.
- **Lehmann i in. (2020).** *Everything Old is New Again: Binary Security of WebAssembly.* USENIX Security 2020.
  W pamięci WebAssembly nie ma zabezpieczeń znanych z systemów operacyjnych, więc błędy pamięci z C i C++ znów da się wykorzystać. W projekcie precyzuje, co daje izolacja; argument za Rustem bez `unsafe` i przeciw trzymaniu kluczy w bazie.

## Prywatność

- **Cavoukian (2009).** *Privacy by Design: The 7 Foundational Principles.* Information and Privacy Commissioner of Ontario.
  Siedem zasad, w tym prywatność jako ustawienie domyślne i ochrona w całym cyklu życia danych. W projekcie źródło hasła „Privacy by Design” i miara tego, co prototyp spełnia.

## Metodyka pomiarów

- **Georges i in. (2007).** *Statistically Rigorous Java Performance Evaluation.* OOPSLA 2007.
  W środowiskach z kompilatorem w trakcie działania pojedynczy przebieg prowadzi do błędnych wniosków. W projekcie powód powtarzania pomiaru w nowych procesach.
- **Raasveldt i in. (2018).** *Fair Benchmarking Considered Difficult: Common Pitfalls In Database Performance Testing.* DBTest 2018. DOI: [10.1145/3209950.3209955](https://doi.org/10.1145/3209950.3209955).
  Typowe błędy porównań baz: brak sprawdzenia wyników, różne warunki, nierówna znajomość systemów. W projekcie lista kontrolna metodyki: sumy kontrolne i grupy warunków.

## Prace pomocnicze

Przywoływane przy pojedynczych zadaniach albo odrzuconych wariantach, poza rdzeniem literatury.

- **Bailis i in. (2013).** *Highly Available Transactions: Virtues and Limitations.* PVLDB 7(3). Atomowa widoczność transakcji bez koordynacji; opcja dla granic transakcji w paczkach.
- **Boncz i in. (2005).** *MonetDB/X100: Hyper-Pipelining Query Execution.* CIDR 2005. Koszt interpretacji wiersz po wierszu; uzasadnienie kompilacji warunków.
- **Cooper i in. (2010).** *Benchmarking Cloud Serving Systems with YCSB.* SoCC 2010. Obciążenie z rozkładem Zipfa; zadanie realistycznego obciążenia.
- **Gupta i Mumick (1995).** *Maintenance of Materialized Views: Problems, Techniques, and Applications.* IEEE Data Engineering Bulletin 18(2). Przyrostowe utrzymanie wyników; zadanie zapytań reaktywnych.
- **Haffner i Dittrich (2021).** Kompilacja zapytań SQL do WebAssembly, arXiv. Koszt kompilacji przekreśla generowanie kodu przy zapytaniach trwających mikrosekundy.
- **Kuessner i in. (2022).** Szyfrowane typy replikowane, warsztat PLF 2022. Wzór szyfrowania end-to-end przy synchronizacji.
- **Litt i in. (2021).** *Cambria: Schema Evolution in Distributed Systems with Edit Lenses.* PaPoC 2021. Tłumaczenie danych między wersjami schematu; odrzucony wariant zmian struktury.
- **Lopes i in. (2019).** Antidote SQL, arXiv. Kolumny addytywne i liczniki ograniczone; zadanie liczników scalanych przez dodawanie.
- **Meyer (2023).** *Range-Based Set Reconciliation.* Uzgadnianie zbiorów przez odciski zakresów; zadanie po utracie kursora.
- **Niinikoski (2025).** Praca magisterska o architekturze local-first. Model hybrydowy z minimalną koordynacją serwera.
- **Ramasubramanian i in. (2009).** *Cimbiosys: A Platform for Content-based Partial Replication.* NSDI 2009. Replikacja częściowa z filtrem klienta.
- **Saadbye i in. (2025).** *Teilen-SQL: A library for building local-first software with SQLite and version control.* PLF 2025 (tylko streszczenie). SQLite w przeglądarce z komórkami jako rejestrami „ostatni zapis wygrywa”.
- **Stonebraker i Pavlo (2024).** *What Goes Around Comes Around... And Around...* SIGMOD Record 53(2). Język CQL jako odpowiednik płaskiego modelu i krytyka modeli bez złączeń.
- **Yu i Ignat (2020).** *Conflict-Free Replicated Relations for Multi-Synchronous Database Management at Edge.* IEEE SMDS 2020. Reguła „ostatni zapis wygrywa” dla kolumny i cofanie zmian naruszających więzy.
- **Zobel i Moffat (2006).** *Inverted Files for Text Search Engines.* ACM Computing Surveys 38(2). Indeks odwrócony; do zebrania przy wyszukiwaniu pełnotekstowym.

## Dokumentacja platform

Nie są to publikacje naukowe, ale opisują zachowanie przeglądarek, od którego zależy poprawność projektu.

- Chrome: [zmiana domyślnego trybu trwałości IndexedDB](https://developer.chrome.com/blog/indexeddb-durability-mode-now-defaults-to-relaxed). Od Chrome 121 zapis jest potwierdzany po przekazaniu danych systemowi.
- WebKit: [blokada ciasteczek stron trzecich](https://webkit.org/blog/10218/full-third-party-cookie-blocking-and-more/) i [zasady przechowywania danych](https://webkit.org/blog/14403/updates-to-storage-policy/). Safari usuwa dane strony po 7 dniach bez wizyty, chyba że strona dostała trwały magazyn.
- MDN: [limity i usuwanie danych w przeglądarce](https://developer.mozilla.org/docs/Web/API/Storage_API/Storage_quotas_and_eviction_criteria), [Web Locks](https://developer.mozilla.org/docs/Web/API/Web_Locks_API), [SharedWorker](https://developer.mozilla.org/docs/Web/API/SharedWorker), [JS String Builtins](https://developer.mozilla.org/docs/WebAssembly/JavaScript_builtins).
- web.dev: [izolacja strony nagłówkami COOP i COEP](https://web.dev/articles/coop-coep). Dokładność `performance.now()` 100 µs, w izolowanej stronie 5 µs.
- WebAssembly: [Wasm 3.0](https://webassembly.org/news/2025-09-17-wasm-3.0) i SpiderMonkey: [czy memory64 się opłaca](https://spidermonkey.dev/blog/2025/01/15/is-memory64-actually-worth-using.html).
- SQLite: [zapis danych w wersji WebAssembly](https://sqlite.org/wasm/doc/tip/persistence.md).
- RxDB: [porównanie magazynów przeglądarki](https://rxdb.info/articles/localstorage-indexeddb-cookies-opfs-sqlite-wasm.html) i PGlite: [pomiary](https://pglite.dev/benchmarks). Pomiary producentów, jedyne dostępne dane o wydajności IndexedDB i OPFS.
- Notion: [SQLite w przeglądarce](https://www.notion.com/blog/how-we-sped-up-notion-in-the-browser-with-wasm-sqlite). Wiele kart i ładowanie biblioteki w tle.
