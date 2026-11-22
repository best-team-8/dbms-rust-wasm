# Literatura a budowa Flat-SQL

Dokument opisuje, na czym opiera się budowa Flat-SQL: co w każdym obszarze ustaliła literatura, co z tego wybrał projekt i czego jeszcze nie rozwiązano. Prace są opisane w [spisie literatury](README.md), decyzje w [rejestrze decyzji](../adr/README.md), a to, co pomiary potwierdziły albo obaliły, we [wnioskach z badań](../research/conclusions.md).

## Zestawienie

| Obszar | Prace | Co projekt z nich wziął |
|---|---|---|
| Praca offline | Kleppmann i in. (2019), Kistler i Satyanarayanan (1992), Terry i in. (1995), Go i in. (2015) | lokalne zapisy, trwała kolejka zmian, serwer porządkujący log |
| Dane w pamięci i transakcje | Stonebraker i in. (2007), DeWitt i in. (1984), Kleppmann (2017) | cała baza w pamięci, transakcje po kolei w jednym wątku |
| Trwałość zapisu | DeWitt i in. (1984), Malviya i in. (2014), Pillai i in. (2014), Cao i in. (2011) | WAL opisujący wynik, grupowanie zatwierdzeń, utrwalanie katalogu, checkpoint krokami |
| Synchronizacja i konflikty | Shapiro i in. (2011), Almeida (2024), Kulkarni i in. (2014), Gray i in. (1996), Helland (2007) | „ostatni zapis wygrywa” na zegarze hybrydowym, wysyłanie stanu zmienionych wierszy, granica dryfu zegara |
| Model danych | Helland (2007), Armbrust i in. (2011), Litt i in. (2023) | jedna tabela w zapytaniu, wiersz jako jednostka atomowości |
| Zmiany struktury tabel | Kleppmann (2017), Kleppmann i in. (2019) | struktura w kodzie aplikacji, tylko dokładanie |
| Serwer i reguły | Gray i in. (1996), Terry i in. (1995) | tryby serwera, zapisy wstępne zatwierdzane przez backend |
| WebAssembly | Haas i in. (2017), Jangda i in. (2019), Yan i in. (2021), Döllerer i Engelke (2024), Kohn i in. (2022) | moduł 32-bitowy, wynik w jednym buforze |
| Prywatność | Cavoukian (2009), Lehmann i in. (2020) | dane domyślnie lokalne, rdzeń bez `unsafe` |
| Metodyka pomiarów | Georges i in. (2007), Raasveldt i in. (2018) | powtórzenia procesu, sumy kontrolne, grupy warunków |

## Praca offline

Oprogramowanie local-first trzyma dane na urządzeniu, działa bez sieci i synchronizuje się w tle. Kleppmann i in. (2019) opisują jego siedem ideałów, od braku czekania na odpowiedź po pełną kontrolę użytkownika nad danymi. Klasycznym wzorcem pracy bez połączenia jest system plików Coda: klient zapisuje zmiany w trwałym logu, skraca go tak, że nowszy zapis unieważnia starsze wpisy, a po połączeniu odtwarza go na serwerze (Kistler i Satyanarayanan, 1992). Bayou dodał serwer, który ustala kolejność zapisów przyjętych przez różne repliki (Terry i in., 1995). Go i in. (2015) pokazali, że aplikacje mobilne korzystające z gotowych usług synchronizacji tracą dane przy przerwie w sieci i zamknięciu aplikacji.

Flat-SQL łączy te wzorce. Zapytania i zapisy kończą się lokalnie ([decyzja 0001](../adr/0001-zapytania-lokalne.md)), zmiany czekają w trwałej kolejce odbudowywanej z WAL ([decyzja 0021](../adr/0021-kolejka-zmian.md)), a serwer nadaje im numery w kolejności przyjęcia ([decyzja 0022](../adr/0022-protokol-synchronizacji.md)). Kolejka trzyma odwołania do wierszy, więc wiele zmian jednego wiersza daje jedną zmianę do wysłania, tak jak skracany log w Coda.

Otwarte: każde urządzenie dostaje dziś całą bazę. Replikację częściową, w której klient ma tylko pasujące do filtra wiersze, opisali Ramasubramanian i in. (2009). Zadania: replikacja częściowa, wiele kart, tryb pamięci podręcznej danych serwera.

## Dane w pamięci i transakcje

Stonebraker i in. (2007) pokazali, że w klasycznym silniku większość pracy to logowanie, blokady i zarządzanie buforem stron. Wyspecjalizowany silnik, który trzyma dane w pamięci i wykonuje transakcje po kolei w jednym wątku, pozbywa się blokad i bufora, a płaci tylko za logowanie. Bazy w pamięci zapewniają trwałość logiem i snapshotami (DeWitt i in., 1984; Kleppmann, 2017, rozdz. 3).

Flat-SQL jest takim silnikiem ([decyzje 0006](../adr/0006-dane-w-pamieci.md) i [0011](../adr/0011-transakcje-szeregowe.md)). Zapytanie nie czeka na dysk, więc może być synchroniczne i działać w głównym wątku strony. Pomiary potwierdzają podział kosztów: odczyty są szybsze niż w SQLite, a zapisy wolniejsze, bo każdy tworzy wpis w dzienniku wycofania i w WAL.

Otwarte: transakcje trwające przez `await` blokują pozostałe. Dla pamięci agentów, gdzie to ma znaczenie, kierunkiem są gałęzie bazy ([decyzja 0025](../adr/0025-galezie-bazy.md)).

## Trwałość zapisu

Jeden zapis logu dla grupy transakcji obniża koszt zatwierdzenia (DeWitt i in., 1984). Logowanie poleceń daje około 1,5 raza większą przepustowość od logowania wyników, ale wymaga deterministycznego wykonania i dłużej odtwarza (Malviya i in., 2014). Pillai i in. (2014) znaleźli w dojrzałych aplikacjach błędy, przez które po awarii giną dane, bo system plików utrwala operacje w innej kolejności, niż je wykonano. Najczęstszy błąd to brak utrwalenia katalogu po zmianie nazwy pliku. Cao i in. (2011) porównali checkpointy stanu w pamięci: naiwny snapshot wstrzymuje aplikację na czas rosnący z rozmiarem stanu, a kopia wierszy zmienianych w trakcie daje pauzę prawie stałą.

Flat-SQL zapisuje w WAL wynik, a nie polecenie, bo te same wpisy służą synchronizacji, a odtworzenie nie zależy od zegara ([decyzja 0016](../adr/0016-wal-i-snapshot.md)). Zatwierdzenia z jednej chwili trafiają na dysk jednym zapisem ([decyzja 0017](../adr/0017-zatwierdzanie-grupowe.md)). Oba magazyny plikowe miały błąd opisany przez Pillai i in.; teraz utrwalają katalog, a test sprawdza każdy stan dysku dopuszczony po awarii ([decyzja 0018](../adr/0018-checkpoint.md), [test](../testing/crash-recovery.md)). Checkpoint jest kodowany krokami z kopią wierszy zmienianych w trakcie.

Otwarte: otwarcie bazy wciąż blokuje wątek na czas dekodowania całego snapshotu. Zadania: odtwarzanie bazy krokami, szybsza suma kontrolna.

## Synchronizacja i konflikty

Rejestr „ostatni zapis wygrywa” jest bezkonfliktowym typem replikowanym (CRDT): wybór większego znacznika jest łączny, przemienny i idempotentny, więc repliki zbiegają się niezależnie od kolejności i powtórzeń (Shapiro i in., 2011). Wysyłanie bieżącego stanu zmienionych elementów daje ten sam wynik co przesłanie wszystkich operacji, a znaczniki usunięcia można skasować, gdy wszystkie repliki je potwierdziły (Almeida, 2024). Zegar hybrydowy łączy czas fizyczny z licznikiem i zaleca odrzucanie znaczników zbyt daleko w przyszłości (Kulkarni i in., 2014). Helland (2007) wymaga idempotentnych wiadomości i atomowości ograniczonej do jednej encji. Gray i in. (1996) pokazali koszt reguły opartej na znacznikach: zbiega się, ale gubi współbieżne zmiany. Aktualizacje przemienne, na przykład dodawanie do licznika, tego problemu nie mają.

Flat-SQL stosuje regułę dla całego wiersza ([decyzja 0019](../adr/0019-ostatni-zapis-wygrywa.md)), z granicą dryfu zegara rozszerzoną o wstrzymanie synchronizacji i nowe znaczniki ([decyzja 0020](../adr/0020-granica-dryfu-zegara.md)). Między urządzeniami atomowy jest wiersz, tak jak w systemie Simba (Go i in., 2015). Silniejsza gwarancja, w której inne repliki widzą wszystkie zmiany transakcji albo żadnej, jest osiągalna bez koordynacji (Bailis i in., 2013), ale nie była potrzebna.

Otwarte: zadania „ostatni zapis wygrywa” dla kolumny (Yu i Ignat, 2020), liczniki scalane przez dodawanie (Gray i in., 1996), usuwanie znaczników usunięcia (Almeida, 2024), raport konfliktów klucza głównego.

## Model danych bez złączeń

Za płaskim modelem przemawiają trzy prace. Encja z unikalnym kluczem jest naturalną granicą atomowości (Helland, 2007). Zapytania z limitem bez złączeń albo ze złączeniem po kluczu głównym mają koszt niezależny od rozmiaru bazy (Armbrust i in., 2011). Simba uznaje tabelę z wierszem jako jednostką synchronizacji za wystarczającą dla typowych aplikacji mobilnych (Go i in., 2015). Przemysłowym odpowiednikiem jest język CQL w Cassandrze (Stonebraker i Pavlo, 2024).

Przeciw przemawiają dwie obserwacje. Systemy NoSQL prawie powszechnie dodały SQL ze złączeniami (Stonebraker i Pavlo, 2024), a Armbrust i in. krytykują języki, które po prostu je usuwają. W aplikacjach local-first znaczna część kodu interfejsu to złączenia i grupowania (Litt i in., 2023).

Flat-SQL zachowuje jedną tabelę w zapytaniu jako część definicji języka ([decyzja 0004](../adr/0004-dialekt-bez-zlaczen.md)). Przewidywalny jest plan, który zawsze dotyczy jednej tabeli. Ograniczony koszt mają tylko odczyty po kluczu lub indeksie; pełny przegląd i grupowanie rosną z tabelą, co potwierdził pomiar.

## Zmiany struktury tabel

Gdy stary i nowy kod współistnieją, bezpieczne jest tylko dodawanie pól opcjonalnych (Kleppmann, 2017, rozdz. 4). Brak jednego obowiązującego schematu w replikach to otwarty problem oprogramowania local-first (Kleppmann i in., 2019). Tłumaczenie danych między wersjami schematu jest możliwe, ale złożone (Litt i in., 2021).

Flat-SQL zgłasza dziś różne definicje tej samej tabeli jako jawny konflikt, a docelowo trzyma strukturę w kodzie aplikacji i pozwala tylko dokładać kolumny ([decyzja 0023](../adr/0023-struktura-tabel-w-kodzie.md)). Otwarte: zadanie wersji struktury tabel.

## Serwer i reguły

Replikacja dwuwarstwowa rozdziela dane, które urządzenie zmienia samo, od danych z kopią główną. Urządzenie wykonuje transakcje wstępne, a właściciel kopii głównej je zatwierdza albo odrzuca (Gray i in., 1996). Bayou dodał zapisy wstępne i zatwierdzone oraz sprawdzanie zależności przez aplikację (Terry i in., 1995).

Docelowy serwer synchronizacji odpowiada temu podziałowi: magazyn paczek dla danych jednej osoby, tryb z silnikiem i webhook backendu dla danych wspólnych ([decyzja 0024](../adr/0024-serwer-synchronizacji.md)). Otwarte: wszystkie zadania części „Serwer synchronizacji” w [liście na później](../management/future.md).

## WebAssembly

WebAssembly działa blisko szybkości kodu natywnego, a jego kod jest zwarty (Haas i in., 2017). Jangda i in. (2019) zmierzyli średni narzut 1,45–1,55 na programach obliczeniowych i uznali koszt wywołań przez JavaScript za pomijalny. Yan i in. (2021) sugerowali, że optymalizacje kompilatora nie są dostrojone do WebAssembly. Przy pamięci 64-bitowej przeglądarka musi sprawdzać granice każdego dostępu programowo, co średnio podwaja jego koszt (Döllerer i Engelke, 2024). DuckDB-Wasm przekazuje wyniki jako bufor czytany przez JavaScript wprost z pamięci modułu (Kohn i in., 2022).

Flat-SQL zostaje przy 32-bitowym WebAssembly ([decyzja 0006](../adr/0006-dane-w-pamieci.md)), przekazuje wynik w jednym buforze binarnym ([decyzja 0015](../adr/0015-wynik-binarny-dla-js.md)) i kompiluje moduł pod szybkość ([decyzja 0009](../adr/0009-ustawienia-kompilacji.md)). Pomiary potwierdziły tezy Haasa i in. i Kohna i in., a obaliły tezy Jangdy i in. oraz Yana i in. dla tego silnika ([wnioski](../research/conclusions.md)).

## Prywatność

Zasada prywatności jako ustawienia domyślnego wymaga, żeby ochrona danych działała bez starań użytkownika (Cavoukian, 2009). Izolacja WebAssembly chroni przeglądarkę przed modułem, ale nie dane modułu przed jego błędami pamięci (Lehmann i in., 2020). Szyfrowane typy replikowane pozwalają synchronizować dane przez serwer, który ich nie odczyta (Kuessner i in., 2022).

Flat-SQL realizuje dziś tylko minimalizację: dane zostają na urządzeniu, dopóki aplikacja nie włączy synchronizacji, a rdzeń jest napisany w bezpiecznym Ruście ([decyzja 0003](../adr/0003-dane-domyslnie-lokalne.md)). IndexedDB nie jest szyfrowana, a serwer dostaje dane jawnym tekstem. Otwarte: szyfrowanie end-to-end i szyfrowanie lokalne.

## Metodyka pomiarów

W środowiskach z kompilatorem w trakcie działania pojedynczy przebieg prowadzi do błędnych wniosków; potrzebne są powtórzenia maszyny wirtualnej i przedziały ufności (Georges i in., 2007). Raasveldt i in. (2018) wymieniają typowe błędy porównań baz: brak sprawdzenia poprawności wyników, porównywanie systemów w różnych warunkach i lepszą znajomość własnego systemu niż porównywanych. Metodyka Flat-SQL odpowiada na każdy z nich ([decyzja 0026](../adr/0026-metodyka-pomiarow.md)).

## Podobne systemy

| System | Co ma wspólnego z Flat-SQL | Czym się różni |
|---|---|---|
| Zeta-Lite (Zhang, 2026) | ten sam kod Rust na serwerze i w przeglądarce | pełny SQL, wiele wersji wierszy, moduł około dziesięć razy większy; bez przyrostowego zapisu i synchronizacji |
| Riffle (Litt i in., 2023) | lokalna baza relacyjna z zapytaniami w wątku interfejsu | pełny SQL ze złączeniami i zapytania reaktywne |
| Simba (Go i in., 2015) | tabela z wierszem jako jednostką atomowości synchronizacji | usługa obok bazy, a nie część silnika |
| cr-sqlite, Teilen-SQL | synchronizacja tabel SQLite przez rejestry „ostatni zapis wygrywa” | warstwa dokładana do pełnej bazy relacyjnej |
| Actual Budget | SQLite w przeglądarce, zegar hybrydowy, własny serwer synchronizacji | aplikacja, a nie silnik do ponownego użycia |

Te systemy zachowują pełny SQL i dokładają synchronizację. Flat-SQL rezygnuje z części SQL w zamian za prostszy silnik, mniejszy moduł i synchronizację wbudowaną w format danych.
