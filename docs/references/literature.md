# Flat-SQL na tle literatury

Dokument zestawia założenia i wyniki Flat-SQL z pracami z [indeksu literatury](README.md). Części 1–5 dotyczą tez, które sprawdziły pomiary i testy: Benchmark 1 (prototyp), Benchmark 2 (wersja po optymalizacjach), pomiary pojedynczych optymalizacji i testy poprawności. Dla każdej pracy podają tezę, która dotyczy Flat-SQL, miejsce, w którym ją sprawdzono, wynik i ocenę. Część 6 porównuje z literaturą te własności projektu, których żaden pomiar nie obejmuje, a część 7 opisuje, co projekt wnosi.

Identyfikatory w nawiasach kwadratowych (np. [W1]) odsyłają do wpisów indeksu; prace oznaczone jako „poza rdzeniem” nie mają w nim wpisu. Źródła liczb: [benchmark-results.md](../research/benchmark-results.md), [optimization-results.md](../research/optimization-results.md) i [tests-results.md](../research/tests-results.md), do którego odsyłają numery testów (np. test 2.3). Hipotezy H1–H6 pochodzą z [benchmark-scenario.md](../research/benchmark-scenario.md), a ich związek z literaturą opisuje [benchmark-results.md §2.12](../research/benchmark-results.md#212-hipotezy-a-literatura).

## 1. Sposób oceny

Każda praca dostaje jedną z trzech ocen.

- Teza potwierdzona: benchmark albo test sprawdził tezę pracy wprost, a wynik mieści się w przewidzianym przez nią zakresie albo ma przewidziany kierunek.
- Teza niepotwierdzona: praca coś sugerowała, pomiar to sprawdził, a w tym silniku i środowisku wynik był inny.
- Praca zastosowana bez sprawdzenia: projekt przejął z pracy mechanizm, metodę albo uzasadnienie decyzji, ale żaden pomiar nie mógł jej tezy potwierdzić ani obalić, bo wymagałby wersji silnika, której nie zbudowano, albo teza nie jest mierzalna.

Ocena dotyczy tego przypadku. Wszystkie pomiary wykonano na jednym laptopie z Windows 11, w silniku V8 (Node.js 24 i Edge 156), na syntetycznym obciążeniu jednej tabeli 100 000 wierszy ([benchmark-scenario.md §7](../research/benchmark-scenario.md#7-zagrożenia-dla-trafności)). Teza niepotwierdzona nie oznacza błędu w pracy, której autorzy mierzyli inne programy w innych latach i przeglądarkach. Oznacza, że przewidywanie z tej pracy nie przenosi się na Flat-SQL i w tekście pracy dyplomowej nie można go użyć jako opisu tego silnika.

## 2. Zestawienie

| Praca | Teza dotycząca Flat-SQL | Gdzie sprawdzona | Wynik | Ocena |
|---|---|---|---|---|
| [P2] Haas i in. | WebAssembly działa blisko szybkości kodu natywnego (prawie wszystkie programy poniżej 2×), kod jest zwarty, jedynym niedeterminizmem arytmetyki są bity NaN | H2, rozmiar modułu, test 3.1 | operacje wykonywane w silniku 0,82–1,13× czasu natywnego; moduł 287 KiB po kompresji, najmniejszy wśród silników SQL w WebAssembly; po kanonizacji NaN te same dane dają te same bajty | potwierdzona |
| [W1] Jangda i in. | WebAssembly jest średnio 1,45–1,55× wolniejszy od kodu natywnego, do 2,5×, a koszt wywołań obsługiwanych przez JavaScript jest pomijalny | H2 | w silniku 0,89–1,13× (Benchmark 1) i 0,82–0,93× (Benchmark 2); narzut tylko na granicy JS↔WASM, do 3,9× | niepotwierdzona |
| [W11] Yan i in. | optymalizacje LLVM nie są dostrojone do WebAssembly, moduł optymalizowany pod rozmiar nie musi być wolniejszy | pomiar siedmiu wariantów kompilacji | `opt-level = "s"` wolniejszy o 25–40%, `"z"` o 48–78%, `opt-level = 3` najszybszy | niepotwierdzona |
| [D1] Kohn i in. | wynik przekazany jako bufor binarny czytany przez JavaScript wprost z pamięci modułu omija koszt granicy; silnik kolumnowy wygrywa analitykę i przegrywa odczyty pojedynczych rekordów | binarny format wyniku, H4 | 10 000 wierszy w przeglądarce 14,0 → 1,59 ms; DuckDB-Wasm: pełny skan 0,59 ms, odczyt po kluczu 1,28 ms | potwierdzona |
| [D2] Gaffney i in. | SQLite zawdzięcza przewagę formatowi stronicowemu (snapshot jako kopia stron), indeksom pokrywającym i instrukcjom kompilowanym raz | H4, liczenie po indeksie, plan w instrukcji przygotowanej | `COUNT` po indeksie 6–8× szybszy w SQLite, po zmianie 33–36× wolniejszy; snapshot i odtworzenie nadal 15× i 39× szybsze w sql.js; odczyt po kluczu natywnie 2,71 → 1,27 µs | potwierdzona |
| [D4] Litt i in. | lokalna baza relacyjna w wątku interfejsu mieści się w budżecie jednej klatki (16,7 ms przy 60 Hz) | H1, pauza checkpointu | odczyty do 9,4 ms przy 100 000 wierszy; checkpoint prototypu 40 ms, po zmianie 2,6 ms | potwierdzona |
| [L1] Kleppmann i in. | dane lokalne dają odpowiedź bez czekania na sieć i pracę offline; magazyn przeglądarki bywa czyszczony | H1, H6, testy 1.6–1.7 | odczyt lokalny 14 µs wobec RTT; praca offline z przetrwaniem restartu | potwierdzona |
| [L2] Shapiro i in. | rejestry LWW scalane przez maksimum znacznika zbiegają się niezależnie od kolejności i powtórzeń | test 2.3, testy losowe | 1000 ziaren, wszystkie repliki równe wyroczni; bez znaczników usunięcia wiersz wraca | potwierdzona |
| [L27] Go i in. | aplikacje bez trwałego lokalnego logu synchronizacji tracą dane przy przerwie sieci i zamknięciu; wiersz wystarcza jako jednostka atomowości | H6, test 2.3 | 19 865 restartów i 12 035 odcięć bez utraty wysłanej zmiany | potwierdzona |
| [L28] Almeida | wysyłanie bieżącego stanu zmienionych elementów (delta-state) daje ten sam wynik co pełna historia zmian | test 2.4, outbox od włączenia synchronizacji | outbox zbudowany przy włączeniu równy outboxowi prowadzonemu od początku, po każdym rodzaju restartu | potwierdzona |
| [C2] Kulkarni i in. | znacznik zegara HLC dalszy niż ε przed zegarem odbiorcy trzeba odrzucić | testy 2.1 i 2.3 | bez granicy zegar serwera przesuwa się w przyszłość; samo odrzucenie nie wystarcza do zbieżności | potwierdzona, z rozszerzeniem |
| [C7] Helland | atomowość obejmuje jedną encję, a wiadomości muszą być idempotentne | test 2.3 | 3481 utraconych potwierdzeń i 5141 utraconych kursorów bez zmiany stanu końcowego | potwierdzona |
| [C9] Gray i in. | replikacja „nowszy znacznik wygrywa” zbiega się, ale gubi współbieżne aktualizacje | test 2.3 | repliki zbieżne, wyrocznia zachowuje tylko zmianę z największym znacznikiem | potwierdzona |
| [C10] Kistler, Satyanarayanan | trwały, skracany log zmian offline pozwala pracować bez połączenia i odtworzyć zmiany na serwerze | H6, test 2.4 | 51 niewysłanych zmian przetrwało restart i dotarło do serwera po jego ponownym starcie | potwierdzona |
| [S1] DeWitt i in. | jeden zapis logu dla grupy transakcji (*group commit*) obniża koszt zatwierdzenia | H5 | 364 µs na pojedyncze zatwierdzenie, 5,1 µs na zatwierdzenie w grupie 1000 | potwierdzona |
| [S4] Stonebraker i in. | wyspecjalizowany, jednowątkowy silnik w pamięci nie płaci za blokady i bufor stron, płaci za logowanie | H4, outbox od włączenia synchronizacji | odczyty 1,6–36× szybsze niż w SQLite, zapisy 1,28–1,75× wolniejsze; usunięcie wpisów outboxa skraca zapisy o 22–32% | potwierdzona |
| [S8] Pillai i in. | protokoły trwałości dojrzałych aplikacji mają błędy kolejności utrwalania, najczęściej brak `fsync` katalogu | testy 1.1–1.4 | oba magazyny plikowe Flat-SQL miały takie błędy; po poprawce wszystkie stany po awarii odtwarzają stan zatwierdzony | potwierdzona |
| [S10] Cao i in. | naiwny snapshot całego stanu daje skok latencji rosnący z rozmiarem; kopiowanie przy zmianie daje pauzę prawie stałą kosztem pamięci | pauza checkpointu, checkpoint krokami | 6,7 / 20,9 / 40,3 ms dla 10, 50 i 100 tys. wierszy; po zmianie 1,2–2,7 ms bez zależności od rozmiaru | potwierdzona |
| [M2] Armbrust i in. | ograniczony koszt mają tylko zapytania, których liczbę odczytów ogranicza ścieżka dostępu; sam `LIMIT` nie wystarcza | H4 (rodzaje zapytań) | po kluczu i indeksie 1,9–3,3 µs; top-k z `LIMIT 10` bez indeksu 5,3 ms, tyle co pełny skan | potwierdzona |
| [B1] Georges i in. | w środowisku z kompilatorem JIT pojedynczy przebieg prowadzi do błędnych wniosków | wyniki wstępne a Benchmark 1 | jeden przebieg dał narzut WebAssembly 1,1–1,8×, dziesięć procesów 0,89–1,13× | potwierdzona |
| [B3] Raasveldt i in. | sposób użycia porównywanego systemu potrafi zmienić wynik bardziej niż różnica silników | adapter RxDB | zakres klucza 2,9 s w pierwszej wersji adaptera, 30 ms po poprawce | potwierdzona |
| [P1] Kleppmann | podręcznikowe podstawy logu, snapshotu, replikacji i zegarów | – | podstawa projektu, bez tezy liczbowej | zastosowana |
| [W2] Lehmann i in. | pamięć liniowa WebAssembly nie chroni danych modułu przed jego własnymi błędami | – | uzasadnienie Rusta bez `unsafe` i opisu izolacji | zastosowana |
| [W6] Döllerer, Engelke | memory64 wymaga programowego sprawdzania granic (średnio ok. 110% narzutu) | – | uzasadnienie celu `wasm32`; wariantu memory64 nie zbudowano | zastosowana |
| [D5] Zhang | ten sam silnik Rust dla serwera i przeglądarki, odczyt po kluczu 3,2–3,7 µs | – | cel optymalizacji odczytu po kluczu (osiągnięto 3,28 µs); systemu nie da się zmierzyć, kod jest zamknięty | zastosowana |
| [C3] Terry i in. | lokalne zapisy i serwer ustalający kolejność logu | – | wzorzec protokołu synchronizacji | zastosowana |
| [S6] Malviya i in. | logowanie poleceń daje ok. 1,5× większą przepustowość od logowania fizycznego kosztem dłuższego odtwarzania | – | uzasadnienie fizycznego WAL; logowania poleceń nie zbudowano | zastosowana |
| [V1] Cavoukian | prywatność jako ustawienie domyślne | – | źródło założenia „dane domyślnie lokalne” | zastosowana |

## 3. Tezy potwierdzone

### 3.1 WebAssembly i granica z JavaScriptem

Haas i in. [P2] pokazali na jądrach PolyBenchC, że WebAssembly działa blisko szybkości kodu natywnego: 7 z 24 programów w granicy 10%, prawie wszystkie poniżej 2×. Flat-SQL mieści się w tym obrazie z zapasem, bo operacje wykonywane w silniku są w WebAssembly tak szybkie jak natywnie albo szybsze. Druga teza tej pracy, zwarta reprezentacja kodu, także się przenosi: moduł Flat-SQL ma po kompresji gzip 287 KiB i jest najmniejszy wśród silników SQL w WebAssembly (sql.js 331 KiB, SQLite WASM 583 KiB, wa-sqlite 821 KiB), a zimny start w przeglądarce trwa 49 ms. Trzecią tezę, że jedynym niedeterminizmem arytmetyki WebAssembly są wzorce bitów NaN, projekt zastosował: kodek zapisuje każdy NaN jednym wzorcem, a test 3.1 sprawdza, że te same dane dają te same bajty snapshotu, WAL i paczek zmian.

Kohn i in. [D1] wskazali izolację pamięci modułu od sterty JavaScriptu jako główną przeszkodę dla bazy w przeglądarce i przekazują wyniki jako strumienie Apache Arrow czytane przez JavaScript wprost z pamięci modułu. Flat-SQL po tej samej zmianie (binarny format wyniku kodowany wprost z wierszy tabeli) pobiera 10 000 wierszy w przeglądarce w 1,59 ms zamiast 14,0 ms, a udział dekodowania UTF-8 w profilu spadł z 36% do 0,8%. Pobranie 10 000 wierszy jest teraz w WebAssembly szybsze niż natywnie (0,54–0,58), bo moduł nie kopiuje wierszy do wyniku pośredniego. Drugą tezę tej pracy, że silnik kolumnowy służy analityce i słabo radzi sobie z odczytami pojedynczych rekordów, potwierdza Benchmark 1: DuckDB-Wasm wykonuje pełny skan w 0,59 ms, ale odczyt po kluczu w 1,28 ms, a 100 000 pojedynczych wstawień w 42 s.

### 3.2 Silniki odniesienia i model danych

Gaffney i in. [D2] opisują trzy źródła przewagi SQLite, które Benchmark 1 pokazał wprost. Indeks pokrywający odpowiadał na `COUNT(*)` po indeksie 6–8 razy szybciej, bo SQLite liczy wpisy indeksu bez czytania wierszy. Po zmianie, w której planer liczy wiersze długością zbioru kluczy w indeksie, Flat-SQL odpowiada w 1,90 µs, 33–36 razy szybciej od SQLite. Format stronicowy daje snapshot i odtworzenie jako kopię stron: sql.js zapisuje bazę w 1,54 ms i odtwarza w 1,19 ms, Flat-SQL po optymalizacjach w 22,6 ms i 46,4 ms. Ta przewaga pozostała, bo zniwelowałby ją dopiero format stronicowy. Instrukcja przygotowana kompilowana raz, wzorowana na SQLite, skróciła natywny odczyt po kluczu z 2,71 do 1,27 µs i liczbę alokacji z 26 do 6.

Stonebraker i in. [S4] twierdzą, że silnik w pamięci wykonujący transakcje w jednym wątku nie płaci za blokady, zatrzaski i bufor stron, a zostaje mu koszt logowania. Wyniki rozkładają się zgodnie z tym podziałem. Po optymalizacjach Flat-SQL wykonuje w przeglądarce wszystkie odczyty szybciej niż sql.js i SQLite WASM, od 1,6 raza w pełnym skanie do 33–36 razy w `COUNT` po indeksie. Wolniejszy jest w zapisach: wstawienie trwa 1,28 raza dłużej niż w sql.js, `UPDATE` 1,75 i `DELETE` 1,44 raza dłużej niż w SQLite WASM. Każdy zapis Flat-SQL tworzy wpis w dzienniku wycofania i w WAL, a przy synchronizacji także w kolejce zmian do wysłania (outboxie). Gdy outbox przestał przyjmować wiersze w bazie bez synchronizacji, natywne wstawienie skróciło się o 22%, `UPDATE` o 23%, a `DELETE` o 32%.

Armbrust i in. [M2] pokazali, że ograniczony koszt mają tylko zapytania, w których liczbę odczytów z magazynu ogranicza ścieżka dostępu, a model bez złączeń sam takiej gwarancji nie daje. Wyniki obciążenia potwierdzają ten podział. Odczyt po kluczu (3,28 µs) i `COUNT` po indeksie (1,90 µs) wymagają jednego wyszukania w drzewie albo odczytu długości zbioru kluczy, zakres 1000 kluczy (45,5 µs) zależy od szerokości zakresu, a pełny skan (4,40 ms) i grupowanie (9,38 ms) rosną z tabelą. Top-k z `LIMIT 10` po kolumnie bez indeksu trwa 5,33 ms, tyle co pełny skan, bo limit ogranicza wynik, ale nie liczbę czytanych wierszy. Teza o „przewidywalnym wykonaniu” Flat-SQL jest więc prawdziwa dla planu, który zawsze dotyczy jednej tabeli i jest widoczny w `EXPLAIN`, a dla kosztu tylko w zapytaniach po kluczu lub indeksie.

Litt i in. [D4] przyjmują dla lokalnej bazy w aplikacji budżet jednej klatki, 16,7 ms przy 60 Hz, i próg 100 ms, powyżej którego użytkownik zauważa opóźnienie. W przeglądarce, przy 100 000 wierszy, najdłuższy odczyt po optymalizacjach (grupowanie) trwa 9,38 ms, a ponowne otwarcie bazy z IndexedDB 81 ms. Jedynym wyjątkiem w prototypie był checkpoint, który blokował wątek główny na 40 ms, powyżej budżetu od około 40 000 wierszy. Po podziale kodowania na kroki najdłuższa pauza wynosi 2,6 ms (§4.3).

### 3.3 Trwałość i odtwarzanie po awarii

DeWitt i in. [S1] opisali zatwierdzanie grupowe: jeden zapis logu utrwala wiele transakcji. Trwałe zatwierdzenie jednego wiersza w IndexedDB kosztuje we Flat-SQL 364 µs, a 1000 zatwierdzeń z jednym oczekiwaniem na trwałość 5,07 ms, czyli 5,1 µs na transakcję. Koszt trwałości rozkłada się na grupę zgodnie z tą pracą. W silnikach, które utrwalają każdą transakcję osobno, 1000 zatwierdzeń trwa od 0,36 s (Dexie.js) do 57 s (PGlite).

Cao i in. [S10] porównali algorytmy punktów kontrolnych stanu w pamięci i pokazali, że naiwny snapshot, czyli synchroniczna kopia całego stanu, wstrzymuje aplikację na czas rosnący z rozmiarem stanu, a kopiowanie przy aktualizacji daje pauzę prawie stałą kosztem dodatkowej pamięci. Checkpoint prototypu był naiwnym snapshotem: 6,7 ms przy 10 000, 20,9 ms przy 50 000 i 40,3 ms przy 100 000 wierszy, liniowo około 0,4 ms na 1000 wierszy. Po zmianie snapshot jest kodowany krokami po 4096 wierszy, a wiersz zmieniany w trakcie, do którego kroki jeszcze nie doszły, jest wcześniej kopiowany. Najdłuższa pauza wynosi 1,2–2,7 ms, na poziomie sondy w bezczynności, i nie zależy od liczby wierszy. Koszt w pamięci, który przewiduje praca, też występuje: przy intensywnych zapisach w trakcie checkpointu rosną kopie wierszy.

Pillai i in. [S8] znaleźli w dojrzałych aplikacjach błędy protokołów trwałości wynikające z kolejności, w jakiej system plików utrwala operacje, najczęściej brak `fsync` katalogu po utworzeniu albo zmianie nazwy pliku. Oba magazyny plikowe Flat-SQL miały błędy tej klasy. W magazynie natywnym test 1.1 na kodzie sprzed zmiany utracił tabelę z potwierdzonej transakcji, bo wpis pliku WAL w katalogu nie był utrwalony, a po pierwszej poprawce utracił transakcje sprzed checkpointu, bo obcięcie WAL zostało utrwalone przed zmianą nazwy snapshotu. Magazyn Node.js nie wykonywał żadnego `fsync`. Po poprawkach test 1.4 sprawdza wszystkie stany dysku, które model systemu plików dopuszcza po awarii, w stylu narzędzia ALICE z tej pracy, i każdy z nich odtwarza stan zatwierdzony. Eksperyment 1.2 uzupełnił pracę o Windows, którego ona nie obejmuje: wpisy katalogu utrwala `FlushFileBuffers` na uchwycie katalogu otwartym z prawem zapisu i flagą `FILE_FLAG_BACKUP_SEMANTICS`. Zatwierdzenie z `fsync` kosztuje 0,49 ms w Windows i 2,05 ms w Linuksie, bez niego 6–9 µs.

### 3.4 Synchronizacja replik

Shapiro i in. [L2] dowodzą, że rejestr „ostatni zapis wygrywa” (ang. *Last-Writer-Wins*, LWW) jest bezkonfliktowym typem danych replikowanych (ang. *Conflict-free Replicated Data Type*, CRDT): scalanie przez wybór większego znacznika jest łączne, przemienne i idempotentne, więc repliki zbiegają się niezależnie od kolejności i powtórzeń dostarczenia. Test 2.3 sprawdził to w 1000 przebiegach z czterema urządzeniami: 199 795 zapisów, 61 505 usunięć i 58 461 rund synchronizacji, a po każdym przebiegu stan serwera i każdego urządzenia był równy wyroczni liczonej niezależnie od silnika. Kontrola negatywna potwierdziła warunek z tej samej pracy: bez porównania zmiany ze znacznikiem usunięcia (ang. *tombstone*) usunięty wiersz wracał.

Gray i in. [C9] pokazali, że replikacja, w której nowszy znacznik czasu wygrywa, zbiega się, ale gubi współbieżne aktualizacje. Test 2.3 potwierdza obie części: repliki są zbieżne, a wyrocznia zachowuje dla każdego klucza tylko zmianę o największym znaczniku, więc współbieżne zmiany innych urządzeń przepadają z założenia. To granica gwarancji Flat-SQL, którą trzeba opisać aplikacji; aktualizacji przemiennych, które według tej pracy problemu nie mają, silnik jeszcze nie ma.

Helland [C7] wymaga, żeby wiadomości między encjami były idempotentne, bo przy zawodnej sieci każda może dotrzeć więcej niż raz. W teście 2.3 odpowiedź serwera ginie w 10% wysłań, a zapis kursora w 10% pobrań: w 1000 przebiegach 3481 paczek wysłano ponownie, a zmiany po 5141 utraconych kursorach pobrano drugi raz, bez wpływu na stan końcowy.

Kulkarni i in. [C2] opisują hybrydowy zegar logiczny (ang. *Hybrid Logical Clock*, HLC), który łączy czas fizyczny z licznikiem, i zalecają ignorowanie znaczników, które odsunęłyby zegar zbyt daleko przed czas fizyczny. Test 2.3 potwierdził potrzebę tej reguły: bez granicy dryfu urządzenie z zegarem w przyszłości przesunęło zegar serwera przed czas rzeczywisty. Pokazał też, że reguła z pracy nie wystarcza. Odrzucona zmiana ma ten sam znacznik przy każdej próbie, więc bez wstrzymania synchronizacji i nadania nowych znaczników po poprawie zegara synchronizacja nie kończyła się nigdy (kontrola negatywna 4). W 1000 przebiegach 9591 zmian dostało nowe znaczniki, a 3347 rund zostało wstrzymanych.

Kistler i Satyanarayanan [C10] opisali w systemie Coda trwały log zmian wykonanych bez połączenia, skracany tak, że nowszy zapis unieważnia wcześniejsze wpisy, i odtwarzany na serwerze po połączeniu. Outbox Flat-SQL jest takim logiem: wpis odsyła do bieżącego stanu wiersza. W teście end-to-end 51 zmian wykonanych przy wyłączonym serwerze przetrwało zamknięcie aplikacji i dotarło do serwera po jego ponownym starcie, a repliki i serwer skończyły z identyczną tabelą. Go i in. [L27] znaleźli w aplikacjach mobilnych korzystających z usług synchronizacji utratę albo uszkodzenie danych przy przerwie w sieci i zamknięciu aplikacji, a za jednostkę spójności uznali wiersz. W teście 2.3 urządzenia restartowano 19 865 razy i odcinano od sieci 12 035 razy, a żadna zmiana wysłana w granicy dryfu nie zginęła.

Almeida [L28] opisuje synchronizację przez stan zmienionych elementów (ang. *delta-state*): wysłanie bieżącego stanu każdego zmienionego elementu daje ten sam wynik co przesłanie wszystkich operacji. Na tej własności opiera się outbox prowadzony dopiero od włączenia synchronizacji: przy włączeniu każdy wiersz i każdy znacznik usunięcia staje się oczekujący. Test 2.4 sprawdza, że taka baza wysyła ten sam zbiór zmian co baza, która prowadziła outbox od początku, także po restarcie z samego WAL, ze snapshotu sprzed włączenia i ze snapshotu po nim. Zmiana skróciła natywne odtworzenie bazy bez synchronizacji ze 111 do 54,7 ms, a pamięć po wczytaniu z 73,3 do 54,8 MiB.

Kleppmann i in. [L1] wskazują, że magazyn przeglądarki bywa czyszczony razem z danymi strony. Projekt zastosował to ostrzeżenie: baza otwarta bez wskazania magazynu używa IndexedDB zamiast pamięci (test 1.6), i prosi przeglądarkę o magazyn trwały (test 1.7). Zachowania samej przeglądarki przy braku miejsca pomiar nie obejmował.

### 3.5 Metodyka pomiarów

Georges i in. [B1] pokazali, że w środowiskach z kompilatorem w czasie wykonania (ang. *just-in-time*, JIT) pojedynczy przebieg albo najlepszy wynik prowadzą do błędnych wniosków. Projekt dostarczył przykładu. Wyniki wstępne z jednym procesem na silnik dały narzut WebAssembly w operacjach wykonywanych w silniku 1,1–1,8×, zgodny z literaturą, a Benchmark 1 z dziesięcioma procesami na silnik dał 0,89–1,13×. W pomiarach optymalizacji ta sama operacja zmieniała czas o 12–28% zależnie od chwili, w której V8 podmieniał funkcje WebAssembly na wersje zoptymalizowane, choć zmieniony kod jej nie wykonywał.

Raasveldt i in. [B3] wskazują, że autor benchmarku zna swój system lepiej niż porównywane, a sposób użycia cudzego API potrafi zmienić wynik bardziej niż różnica silników. Pierwsza wersja adaptera RxDB wymuszała ponowne wykonanie zapytania warunkiem na polu bez indeksu, przez co planer RxDB czytał całą kolekcję: zakres klucza trwał 2,9 s, po poprawce 30 ms. Sumy kontrolne wyników tego nie wykryły, bo odpowiedź była poprawna.

## 4. Tezy niepotwierdzone

### 4.1 Narzut WebAssembly względem kodu natywnego

Jangda i in. [W1] zmierzyli programy SPEC CPU skompilowane do WebAssembly i uruchomione przez Browsix-Wasm: średnio 1,55 raza wolniej w Chrome i 1,45 raza w Firefoksie, najwyżej 2,5 raza. Narzut przypisali większej liczbie dostępów do pamięci przy braku rejestrów, sprawdzaniu przepełnienia stosu przy każdym wywołaniu i większemu kodowi. Koszt wywołań obsługiwanych przez JavaScript uznali za pomijalny. Hipoteza H2 przyjęła ten przedział.

We Flat-SQL żadna z tych dwóch tez się nie sprawdziła. Operacje wykonywane w całości w silniku mają w Benchmarku 1 0,89–1,13 czasu natywnego, a po optymalizacjach 0,82–0,93, czyli WebAssembly jest szybszy niż natywnie. Cały mierzalny narzut leży na granicy z JavaScriptem, którą praca pomija: odczyt po kluczu był 3,9 raza wolniejszy niż natywnie, a profil przypisał 24% czasu dekodowaniu UTF-8 i 19% kodowi sklejającemu. Po przeniesieniu wyniku do jednego bufora operacje przekazujące dużo danych naraz mają narzut bliski zeru (wstawienie jednym wywołaniem 1,02–1,05) albo są szybsze niż natywnie (pobranie 10 000 wierszy 0,54–0,58), a odczyt po kluczu, który przekracza granicę przy każdym wywołaniu, pozostał 2,7–3,6 raza wolniejszy.

Różnica ma kilka możliwych przyczyn, których pomiar nie rozdziela. Praca dotyczy programów obliczeniowych z 2019 roku, kompilowanych z C przez Emscripten, w przeglądarkach z tamtego czasu; Flat-SQL to kod Rust, którego czas wyznaczają przejścia drzew i porównania wartości, mierzony w V8 z 2026 roku. Na `wasm32` wskaźniki mają 4 bajty, a wartość silnika zajmuje 16 bajtów zamiast 24, więc ten sam zbiór wierszy zajmuje mniej pamięci podręcznej procesora; tego wpływu nie mierzono osobno. Ocenę „niepotwierdzona” wzmacnia przebieg badań: wyniki wstępne z jednym procesem na silnik dały 1,1–1,8× i wyglądały na zgodne z tą pracą, a różnica zniknęła dopiero przy dziesięciu procesach (§3.5).

### 4.2 Poziomy optymalizacji kompilatora

Yan i in. [W11] porównali WebAssembly i JavaScript w trzech przeglądarkach przy różnych poziomach optymalizacji i doszli do wniosku, że optymalizacje LLVM nie są dostrojone do WebAssembly, więc wyższy poziom nie zawsze daje szybszy kod. Z tego wyszła hipoteza, że moduł optymalizowany pod rozmiar (`opt-level = "s"` albo `"z"`) może być mniejszy bez straty czasu. Pomiar siedmiu wariantów kompilacji ją obalił. `opt-level = "s"` zmniejszył moduł o 17%, ale wydłużył wstawianie o 25%, zakres klucza o 40% i odtworzenie o 36%. `opt-level = "z"` zmniejszył moduł o 25% i wydłużył zakres klucza o 78%, a pełny skan o 48%. `opt-level = 2` był wolniejszy we wstawianiu o 9,5% i szybszy tylko w snapshocie (14,6%), a `wasm-opt -O4` nie dał zysku. W tym silniku `opt-level = 3` jest najszybszy i konfiguracja kompilacji została bez zmian.

Druga teza tej pracy, że WebAssembly zużywa znacznie więcej pamięci niż JavaScript, pozostaje nierozstrzygnięta. Flat-SQL zajmuje po wczytaniu 100 000 wierszy 51,8 MiB pamięci liniowej, a ta sama tabela w czystym JavaScripcie (`Map`) 20,7 MiB sterty, ale SQLite WASM zajmuje 8,0 MiB, a sql.js 21,1 MiB. Zużycie wyznacza więc reprezentacja danych w silniku, której sama platforma nie narzuca. Pamięć liniowa nie maleje po zwolnieniu, co praca też opisuje, i tak ją mierzy benchmark. Zależności wyników od przeglądarki, trzeciej tezy pracy, nie sprawdzono, bo wszystkie pomiary wykonano w V8.

## 5. Prace zastosowane bez sprawdzenia pomiarem

| Praca | Co zastosowano | Dlaczego bez pomiaru |
|---|---|---|
| [P1] Kleppmann | log i snapshot, replikacja z wieloma liderami, zawodność zegarów, zgodność formatów przy dodawaniu pól (test 2.2) | podręcznik bez tez liczbowych |
| [W2] Lehmann i in. | izolacja WebAssembly chroni przeglądarkę przed modułem, ale danych modułu przed jego błędami już nie, stąd rdzeń w Rust bez `unsafe`, kluczy kryptograficznych nie przechowuje się w bazie | teza o bezpieczeństwie, której benchmark nie mierzy |
| [W6] Döllerer, Engelke | cel `wasm32` zamiast memory64, bo przy 64-bitowej pamięci przeglądarki sprawdzają granice programowo | wariantu memory64 nie zbudowano; baza 100 000 wierszy zajmuje 52 MiB z 4 GiB przestrzeni adresowej |
| [D5] Zhang | cel odczytu po kluczu poniżej 3,5 µs (osiągnięto 3,28 µs w przeglądarce) | kod zamknięty, brak modułu do pobrania; deklaracje autora są niepotwierdzone i zmierzone w innych warunkach |
| [C3] Terry i in. | lokalne zapisy i serwer ustalający kolejność logu, z którego repliki pobierają zmiany | zbieżność tego protokołu sprawdza test 2.3, ale tezy samej pracy (procedury scalania aplikacji, zapisy wstępne i zatwierdzone) Flat-SQL nie realizuje |
| [S6] Malviya i in. | fizyczny WAL zamiast logowania poleceń, bo te same wpisy służą synchronizacji, a odtworzenie nie zależy od zegara | logowania poleceń nie zbudowano, więc przewagi 1,5× nie da się porównać |
| [V1] Cavoukian | prywatność jako ustawienie domyślne: dane zostają na urządzeniu, dopóki aplikacja nie włączy synchronizacji | zasada projektowa, niemierzalna |

## 6. Własności bez pomiaru

Część założeń dotyczy gwarancji i braków, których benchmark nie mierzy. Ocena w tabelach: zgodne (rozwiązanie odpowiada temu, co literatura uznaje za poprawne), częściowo (kierunek zgodny, ale z brakującym elementem), luka (literatura opisuje problem, którego projekt nie rozwiązuje).

### 6.1 Izolacja i prywatność

Założenie „przetwarzanie danych w izolowanym środowisku przeglądarki bez wysyłania ich na serwer” obejmuje trzy własności, które literatura rozdziela.

| Własność | Literatura | Stan Flat-SQL | Ocena |
|---|---|---|---|
| Moduł nie zaszkodzi przeglądarce | [P2]: pamięć liniowa oddzielona od kodu i stosu środowiska uruchomieniowego | zapewnia przeglądarka | zgodne |
| Błąd w module nie zaszkodzi danym modułu | [W2]: w pamięci liniowej nie ma kanarków stosu, losowania adresów ani stron ochronnych | rdzeń i wiązania WebAssembly bez `unsafe`, rdzeń bez zależności, więc klasy ataków z tej pracy w większości nie dotyczą kodu | zgodne |
| Dane są chronione przed innymi | [L1] ideał 6: szyfrowanie end-to-end, serwer przechowuje tylko szyfrogram; [V1]: prywatność domyślnie i ochrona w całym cyklu życia danych | IndexedDB nie jest szyfrowana i jest dostępna dla każdego skryptu tej samej domeny; serwer synchronizacji dostaje dane jawnym tekstem i nie uwierzytelnia klientów | częściowo |

Prywatność oznacza więc dziś minimalizację przesyłanych danych: zapytania wykonują się lokalnie, a synchronizacja jest opcjonalna, co odpowiada zasadzie prywatności domyślnej z [V1]. Ochrony kryptograficznej projekt nie ma. Szyfrowanie end-to-end paczek zmian jest możliwe, jeśli serwer stanie się przekaźnikiem zmian, których nie odczytuje (ideał 6 z [L1]): do rozstrzygnięcia konfliktu wystarczy jawny znacznik czasu i skrót klucza, a wartości mogą być zaszyfrowane. Wzorem są szyfrowane typy replikowane synchronizowane przez niezaufanych pośredników (Kuessner i in., 2022, praca poza rdzeniem). Kosztem byłaby utrata zapytań na serwerze i walidacji po stronie serwera dla zaszyfrowanych tabel.

### 6.2 Ideały local-first

| Ideał [L1] | Flat-SQL | Ocena | Uwagi z literatury |
|---|---|---|---|
| 1. Brak oczekiwania na odpowiedź | odczyt po kluczu 3,28 µs w przeglądarce, synchronicznie; najdłuższa pauza checkpointu 2,6 ms | zgodne | [D4]: budżet jednej klatki 16,7 ms na interakcję |
| 2. Praca nie jest uwięziona na jednym urządzeniu | synchronizacja przez serwer, repliki zbieżne (test 2.3) | zgodne | [C9]: replikacja dwuwarstwowa, w której lokalna kopia zgłasza zmiany do węzła bazowego |
| 3. Sieć jest opcjonalna | Service Worker, IndexedDB, trwały outbox, prośba o trwały magazyn (test 1.7), scenariusz offline w przeglądarce (test 2.5) | zgodne | wzorzec pracy bez połączenia z [C10]; [L1] ostrzega, że magazyn przeglądarki bywa czyszczony |
| 4. Płynna współpraca | „ostatni zapis wygrywa” dla całego wiersza | częściowo | współbieżne zmiany różnych kolumn jednego wiersza: jedna ginie [C9]; scalanie na poziomie kolumn (Yu, Ignat, 2020, praca poza rdzeniem) |
| 5. Długowieczność danych | formaty plików i paczek z nagłówkiem i numerem wersji, wspólne dla Rust i Node.js; snapshot poprzedniego formatu czytany przez nową wersję (test 2.4) | częściowo | brak zasad zgodności wersji schematu dla klientów długo pracujących offline ([P1] rozdz. 4) |
| 6. Bezpieczeństwo i prywatność domyślnie | dane lokalne domyślnie | częściowo | §6.1 |
| 7. Pełna kontrola użytkownika | eksport stanu jako snapshot, przełącznik synchronizacji w demo | częściowo | brak eksportu do pliku w otwartym formacie |

### 6.3 Granice transakcji, usunięcia i unikalność klucza

Lokalnie transakcja jest atomowa: dziennik wycofania i jedna ramka WAL. W synchronizacji tej granicy się nie zachowuje. Paczka zawiera do 1000 wpisów outboxu w kolejności numerów, a wiersz zmieniony ponownie przechodzi na koniec kolejki, więc zmiany jednej transakcji mogą trafić do różnych paczek i inne repliki mogą przez chwilę widzieć stan pośredni. Bailis i in. (2013, praca poza rdzeniem) pokazali, że widoczność wszystkich albo żadnego zapisu transakcji (ang. *Monotonic Atomic View*) da się zapewnić bez koordynacji. Helland [C7] przyjmuje jednak atomowość jednej encji, a Simba [L27], system synchronizacji tabel dla aplikacji mobilnych, świadomie daje atomowość tylko pojedynczego wiersza, bo atomowa synchronizacja wielu wierszy okazała się dla większości aplikacji zbędna i kosztowna. Flat-SQL daje dokładnie gwarancję Simby: transakcje są atomowe lokalnie, a między replikami atomowa jest zmiana pojedynczego wiersza.

Znacznik usunięcia (ang. *tombstone*) chroni przed przywróceniem wiersza przez spóźnioną zmianę, zgodnie z [L2]; test 2.3 sprawdził, że bez niego usunięty wiersz wraca. Znaczników usunięcia silnik nie kasuje. Bezpiecznym kryterium jest stabilność przyczynowa, czyli chwila, w której wszystkie repliki potwierdziły znacznik [L28]; przy serwerze centralnym wystarczyłby najmniejszy potwierdzony kursor aktywnych klientów.

Unikalność klucza głównego jest lokalna. Dwa urządzenia offline, które wstawią wiersz o tym samym kluczu, nie dostaną błędu, a po synchronizacji jeden wiersz zastąpi drugi. Kleppmann [P1, rozdz. 9] zaznacza, że ograniczenia unikalności między replikami wymagają konsensusu. Simba [L27] zgłasza konflikty aplikacji, a Coda [C10] unikała kolizji tymczasowymi identyfikatorami zastępowanymi przy reintegracji. W Flat-SQL aplikacja zapisująca offline musi używać kluczy bez kolizji, np. losowych identyfikatorów albo identyfikatora urządzenia w kluczu.

### 6.4 Protokół synchronizacji

| Element Flat-SQL | Odpowiednik w literaturze | Ocena |
|---|---|---|
| zapisy lokalne, serwer porządkujący log numerami przyjęcia | Bayou [C3]: zapisy do dowolnej repliki, serwer główny ustala kolejność | zgodne („ostatni zapis wygrywa” zamiast procedur scalania aplikacji) |
| outbox w WAL przetrwa restart, kolejne zmiany wiersza scalane w jedną | Coda [C10]: trwały log zmian offline, w którym nowszy zapis unieważnia wcześniejsze wpisy; delta-state [L28] | zgodne |
| ponowne dostarczenie paczki nie zmienia stanu | idempotencja wiadomości [C7], argument end-to-end [P1, rozdz. 12] | zgodne (test 2.3) |
| kursor klienta, epoka serwera, pełny stan przy nieznanym kursorze | offsety konsumentów i strumień zmian ze snapshotem początkowym [P1, rozdz. 11] | zgodne |
| log serwera tylko w pamięci, epoka losowana przy starcie | trwały log z offsetami [P1, rozdz. 11] | luka: każdy restart serwera wysyła wszystkim klientom pełny stan |
| klient pyta serwer o zmiany w rosnących odstępach | [P1, rozdz. 12]: serwer sam powiadamia klientów o zmianach | częściowo |
| serwer przyjmuje każdą zmianę | Gray i in. [C9]: właściciel kopii głównej zatwierdza zmiany wstępne według kryterium akceptacji; Bayou [C3]: sprawdzanie zależności | luka prototypu |
| każdy klient dostaje całą bazę | Cimbiosys (Ramasubramanian i in., 2009, praca poza rdzeniem): replikacja częściowa z filtrem klienta | luka |

### 6.5 Ewolucja schematu

Kleppmann i in. [L1] opisują brak jednego obowiązującego schematu w replikach jako problem otwarty. W prototypie definicja tabeli otrzymana z innej repliki była pomijana, jeśli tabela o tej nazwie już istniała, więc dwie repliki, które offline utworzyły tę samą tabelę z innymi kolumnami, rozjeżdżały się trwale. Teraz odbiorca porównuje definicje i zgłasza różnicę jako jeden konflikt schematu z obiema definicjami (test 2.2), a wiersze drugiej definicji odrzuca. Dane urządzenia w konflikcie nie docierają do innych, dopóki aplikacja go nie rozwiąże. Tłumaczenie danych między wersjami schematu soczewkami (Cambria, Litt i in., 2021, praca poza rdzeniem) odrzucono na rzecz struktury tabel w kodzie aplikacji, w której zmiany polegają tylko na dokładaniu kolumn. Wersjonowanie schematu i migracje to kierunek rozwoju.

### 6.6 Model danych bez złączeń

Za płaskim modelem przemawiają trzy prace rdzenia. Helland [C7] uznaje encję z unikalnym kluczem za naturalny zakres serializowalności, poza który transakcje atomowe nie powinny sięgać. Armbrust i in. [M2] pokazują, że zapytania z limitem bez złączeń, albo ze złączeniem po unikalnym kluczu, mają koszt niezależny od rozmiaru bazy. Simba [L27] uznaje tabelę z wierszem jako jednostką synchronizacji za wystarczającą dla typowych aplikacji mobilnych. Przemysłowym odpowiednikiem jest język CQL w Cassandrze, czyli składnia SQL nad modelem klucz–wiersz (Stonebraker, Pavlo, 2024, praca poza rdzeniem).

Przeciw przemawiają dwie obserwacje. Ta sama praca Stonebrakera i Pavlo odnotowuje, że systemy NoSQL niemal powszechnie dodały SQL ze złączeniami, a [M2] krytykuje języki, które dla skalowalności po prostu je usuwają. Litt i in. [D4] pokazują, że w aplikacji local-first znaczna część kodu interfejsu to złączenia i grupowania, a ich szybkość decyduje o responsywności; relacje wiele-do-wielu bez złączeń przenoszą pracę do aplikacji [P1, rozdz. 2].

Granulacja replikacji nie zależy od języka zapytań: systemy replikujące tabele relacyjne przesyłają wiersze albo kolumny, a trudność sprawiają więzy integralności przy współbieżnych usunięciach. Brak złączeń uzasadniają więc prostota i przewidywalność planu oraz brak więzów między tabelami. Przewidywalny jest plan, który zawsze dotyczy jednej tabeli i jest widoczny w `EXPLAIN`; ograniczony koszt mają tylko odczyty po kluczu lub indeksie (§3.2).

### 6.7 Przetwarzanie na brzegu sieci

Flat-SQL wykonuje zapytania na urządzeniu użytkownika, czyli w najdalszym od chmury miejscu, jakie obejmuje definicja przetwarzania na brzegu sieci (ang. *edge computing*): zasoby obliczeniowe na drodze między źródłem danych a chmurą (Shi i in., 2016, praca poza rdzeniem). Prototyp ma jeden centralny serwer, który jest jednocześnie węzłem synchronizacji i bazą główną; węzłów brzegowych synchronizowanych z bazą główną jeszcze nie ma. W literaturze replikacji taki układ to replikacja dwuwarstwowa Graya i in. [C9], z węzłami bazowymi jako warstwą zatwierdzającą zmiany.

Wynik H1 odpowiada temu, co przewiduje literatura: zapytanie przez sieć jest ograniczone od dołu czasem obiegu pakietu, więc przy RTT 20 ms już jedno zapytanie przekracza budżet klatki z [D4]. Pomiar pomija TLS, straty pakietów i zestawianie połączeń, więc jest dolnym ograniczeniem kosztu sieci. Zastrzeżenie w drugą stronę daje analiza wdrożeń: na wolnych urządzeniach lokalny odczyt z dysku bywa wolniejszy od sieci, dlatego Notion wysyła oba odczyty naraz i korzysta z szybszego ([use-cases.md](../research/use-cases.md)).

## 7. Co projekt wnosi względem literatury

Żaden pojedynczy mechanizm Flat-SQL nie jest nowy. Reguła „ostatni zapis wygrywa” na zegarze hybrydowym, WAL ze snapshotem, zatwierdzanie grupowe, outbox i log z kursorami mają dobrze opisane odpowiedniki, także w klasycznych pracach o pracy offline ([C3], [C9], [C10]). Wartość projektu leży w ich połączeniu i pomiarze:

1. jeden rdzeń bez zależności i bez operacji wejścia/wyjścia, uruchamiany w ten sam sposób w przeglądarce, na serwerze i natywnie, co realizuje cel przenośności z [P2]; podobną drogą idzie Zeta-Lite [D5], ale bez przyrostowej trwałości i synchronizacji, za to z pełnym SQL i modułem około dziesięciu razy większym;
2. pomiar kosztu granicy między JavaScriptem a WebAssembly dla silnika bazy danych, obszaru bez recenzowanych pomiarów, z wynikiem, że cały mierzalny narzut platformy leży na tej granicy (§4.1);
3. pomiar przyrostowej trwałości w IndexedDB z zatwierdzaniem grupowym wobec eksportu całej bazy i wobec silników zapisujących strony;
4. dialekt SQL ograniczony do encji pod kluczem, z synchronizacją wbudowaną w format WAL, w odróżnieniu od systemów, które dokładają warstwę replikacji do pełnej bazy relacyjnej (cr-sqlite, Teilen-SQL [X5]), i od Simby [L27], która jest usługą obok bazy.

Te systemy zachowują pełny SQL i dokładają synchronizację. Flat-SQL rezygnuje z części SQL w zamian za prostszy silnik, mniejszy moduł i synchronizację wbudowaną w format danych.
