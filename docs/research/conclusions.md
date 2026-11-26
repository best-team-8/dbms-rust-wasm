# Wnioski z badań

Dokument zbiera odpowiedzi na pytania badawcze z [metodyki](benchmark-scenario.md) i zestawia je z literaturą. Liczby pochodzą z [wyników benchmarków](benchmark-results.md), [wyników optymalizacji](optimization-results.md) i [testów](../testing/README.md). Opis prac: [spis literatury](../references/README.md).

Najkrócej: zapytanie lokalne jest o trzy do czterech rzędów wielkości szybsze niż przez sieć. WebAssembly prawie nic nie kosztuje, dopóki dane nie przekraczają granicy z JavaScriptem. Po optymalizacjach Flat-SQL czyta szybciej niż SQLite, ale wolniej zapisuje i dłużej otwiera bazę. Praca offline i zbieżność urządzeń działają, z ograniczeniem reguły „ostatni zapis wygrywa”.

## Lokalnie a przez serwer

Hipoteza się potwierdziła. Lokalny odczyt po kluczu z tekstem SQL trwa 10–14 µs. Przez serwer z opóźnieniem sieci 20–100 ms trwa tyle co opóźnienie, czyli 1000–7000 razy dłużej. Koszt wykonania na serwerze jest przy tym pomijalny.

Zgadza się to z tezą, że dane lokalne usuwają czekanie na sieć (Kleppmann i in., 2019). Nawet najdłuższy odczyt w obciążeniu (grupowanie 100 000 wierszy, 9,4 ms) mieści się w budżecie jednej klatki obrazu, który Litt i in. (2023) przyjmują dla lokalnej bazy w aplikacji. Symulacja sieci pomija szyfrowanie i nawiązywanie połączeń, więc przewaga jest zaniżona. Z drugiej strony na wolnych urządzeniach odczyt z dysku bywa wolniejszy od sieci, czego pomiar na laptopie nie pokazuje.

## Koszt WebAssembly

Hipoteza zakładała, że ten sam kod w WebAssembly jest 1,2–3 razy wolniejszy niż natywnie. Nie potwierdziła się. Operacje wykonywane w całości w silniku (zakres kluczy, pełny przegląd, grupowanie, odtworzenie) trwają w WebAssembly 0,89–1,13 czasu natywnego, a po optymalizacjach 0,82–0,93. Narzut zostaje tylko tam, gdzie dane przechodzą przez granicę z JavaScriptem: odczyt po kluczu jest 2,7–3,6 raza wolniejszy niż natywnie.

Wynik zgadza się z Haas i in. (2017), według których WebAssembly działa blisko szybkości kodu natywnego. Nie zgadza się z Jangda i in. (2019), którzy zmierzyli średnio 1,45–1,55 i uznali koszt wywołań przez JavaScript za pomijalny. Przyczyn różnicy pomiar nie rozdziela. Tamta praca mierzyła programy obliczeniowe w C z 2019 roku, a Flat-SQL to kod Rust z przechodzeniem drzew, mierzony w przeglądarce z 2026 roku. Czasy krótsze od natywnych tłumaczą najpewniej dwie cechy modułu, których pomiar nie rozdziela. W 32-bitowym WebAssembly wartość silnika zajmuje 16 bajtów zamiast 24, więc dane lepiej mieszczą się w pamięci podręcznej procesora. Pamięć przydziela też inny alokator: biblioteka standardowa Rust używa w WebAssembly alokatora dlmalloc, a natywnie w Windows alokatora systemowego.

Ostrzeżeniem jest przebieg badań. Pierwsze pomiary z jednym procesem na silnik dały 1,1–1,8 i wyglądały na zgodne z literaturą. Różnica zniknęła dopiero przy dziesięciu procesach.

## Koszt granicy z JavaScriptem

Hipoteza, że przy małych wynikach dominuje przekazanie danych, się potwierdziła. W prototypie wynik jako JSON był szybszy od obiektów budowanych z Rusta (5,6 wobec 9,2 µs), a profil przypisał dekodowaniu tekstów i kodowi sklejającemu po około jednej trzeciej czasu.

Optymalizacje poszły jednak w kierunku DuckDB-Wasm (Kohn i in., 2022): wynik w jednym buforze czytanym przez JavaScript wprost z pamięci modułu. Teraz wynik jako obiekty jest szybszy od JSON przy jednym wierszu (3,28 wobec 4,16 µs) i przy 10 000 wierszy (1,55 wobec 7,06 ms). Recenzowanych pomiarów tej granicy dla silników baz nie znaleziono, więc ten pomiar jest wkładem projektu.

## Na tle innych silników

Prototyp wygrywał z SQLite w grupowaniu, sortowaniu i pobieraniu wierszy, przegrywał w liczeniu po indeksie, zapisach, snapshocie i odtwarzaniu. Po optymalizacjach Flat-SQL jest szybszy od sql.js i SQLite WASM we wszystkich odczytach, od 1,6 raza w pełnym przeglądzie do 33–36 razy w `COUNT` po indeksie. Wolniejszy zostaje we wstawianiu (1,28 raza), w `UPDATE` (do 1,75 raza), w snapshocie (15 razy) i w odtworzeniu (39 razy).

Ten podział wyjaśnia literatura. Silnik w pamięci, wykonujący transakcje w jednym wątku, nie płaci za blokady i bufor stron, ale płaci za logowanie (Stonebraker i in., 2007). Każdy zapis Flat-SQL tworzy wpis w dzienniku wycofania i w WAL. SQLite zawdzięcza przewagę w snapshocie i odtwarzaniu formatowi stronicowemu, bo kopiuje gotowe strony pliku (Gaffney i in., 2022). Tej przewagi Flat-SQL nie zniweluje bez zmiany formatu.

DuckDB-Wasm wygrywa przegląd i grupowanie, ale odczyt po kluczu zajmuje mu 1,3 ms, a wstawienie 100 000 wierszy pojedynczo 42 s. To potwierdza granicę zastosowań: silnik kolumnowy do analityki, Flat-SQL do odczytów i zapisów pojedynczych rekordów.

## Koszt trwałego zatwierdzenia

Hipoteza się potwierdziła. Trwałe zatwierdzenie w IndexedDB kosztuje około 0,36 ms niezależnie od rozmiaru bazy. sql.js płaci 19,3 ms przy 100 000 wierszy, a jego eksport rośnie z rozmiarem bazy. Grupowanie zatwierdzeń (DeWitt i in., 1984) obniża koszt do 5 µs na transakcję. Silniki, które utrwalają każdą transakcję osobno, płacą od 0,56 ms (Dexie.js) do 54 ms (PGlite).

Koszt Flat-SQL przenosi się na checkpoint i otwarcie bazy. Checkpoint prototypu był naiwnym snapshotem, który wstrzymuje aplikację na czas rosnący z rozmiarem stanu (Cao i in., 2011): 40 ms przy 100 000 wierszy. Po podziale na kroki najdłuższa pauza wynosi 2,6 ms. Otwarcie bazy z IndexedDB (81 ms) pozostaje jednym z najdłuższych, bo trzeba zdekodować całą bazę.

## Praca offline i zbieżność urządzeń

Testy potwierdziły pracę offline i zbieżność. 51 zmian wykonanych bez serwera przetrwało zamknięcie aplikacji i dotarło do serwera po jego powrocie, a w 1000 losowych przebiegach z przesuniętymi zegarami, podziałami sieci i restartami wszystkie urządzenia skończyły w tym samym stanie. To odpowiada tezom o trwałym logu zmian offline (Kistler i Satyanarayanan, 1992; Go i in., 2015) i o zbieżności reguły „ostatni zapis wygrywa” (Shapiro i in., 2011).

Testy pokazały też granice. Współbieżna zmiana z mniejszym znacznikiem przepada, tak jak opisują Gray i in. (1996). Reguła Kulkarniego i in. (2014), żeby odrzucać znaczniki zbyt daleko w przyszłości, jest konieczna, ale nie wystarcza: bez wstrzymania synchronizacji i nowych znaczników po poprawie zegara synchronizacja nigdy się nie kończyła. To rozszerzenie wypracowane w projekcie.

## Inne ustalenia

- **Kompilacja pod rozmiar.** Yan i in. (2021) sugerowali, że optymalizacje kompilatora nie są dostrojone do WebAssembly, więc mniejszy moduł nie musi być wolniejszy. W tym silniku kompilacja pod rozmiar zmniejszyła moduł o 17–25%, ale spowolniła operacje nawet o 78%.
- **Pamięć.** Ta sama praca podaje, że WebAssembly zużywa więcej pamięci niż JavaScript. Tu zużycie wyznacza układ danych w silniku, a nie platforma: Flat-SQL zajmuje po wczytaniu 51,8 MiB, sql.js 21,1 MiB, a SQLite WASM 8,0 MiB, wszystkie w WebAssembly.
- **Sposób użycia cudzego silnika.** Raasveldt i in. (2018) ostrzegają, że autor benchmarku zna swój system lepiej niż porównywane. Pierwsza wersja adaptera RxDB była sto razy wolniejsza przez złe użycie indeksu, a sumy kontrolne tego nie wykryły, bo odpowiedź była poprawna.

## Co zostało słabe

| Słaba strona | Prototyp | Po optymalizacjach | Dlaczego zostało |
|---|---|---|---|
| Odczyt po kluczu wobec natywnego | 3,9 raza wolniej | 2,7–3,6 raza wolniej | każde wywołanie przekracza granicę z JavaScriptem |
| Zapisy wobec SQLite | `UPDATE` 3,5 raza wolniej | 1,75 raza wolniej | koszt dziennika wycofania i WAL |
| Snapshot i odtworzenie wobec SQLite | 20 i 100 razy wolniej | 15 i 39 razy wolniej | SQLite kopiuje strony pliku |
| Ponowne otwarcie z IndexedDB | 174 ms | 81 ms | cała baza dekodowana do pamięci |
| Pamięć po wczytaniu 100 000 wierszy | 73 MiB | 52 MiB | wiersz jako osobna alokacja, kopie w dużej transakcji |
| Wywołanie w Web Workerze | 56 µs | około 45 µs | koszt komunikatu między wątkami |
| Grupowanie i przegląd wobec DuckDB-Wasm | 4,4 i 13,5 raza wolniej | 2,6 i 7,5 raza wolniej | silnik wierszowy wobec kolumnowego |

Zadania, które mogą to poprawić, są w [liście na później](../management/future.md).

## Co projekt wnosi

Poszczególne mechanizmy Flat-SQL mają odpowiedniki w literaturze: reguła „ostatni zapis wygrywa” na zegarze hybrydowym, WAL ze snapshotem, zatwierdzanie grupowe, kolejka zmian i log z kursorami. Wkładem projektu jest ich połączenie i pomiar:

1. jeden rdzeń bez zależności i bez operacji wejścia/wyjścia, działający tak samo w przeglądarce, na serwerze i natywnie, z modułem około dziesięć razy mniejszym niż Zeta-Lite, który idzie podobną drogą;
2. pomiar kosztu granicy między JavaScriptem a WebAssembly dla silnika bazy danych, z wynikiem, że cały mierzalny narzut platformy leży na tej granicy;
3. pomiar przyrostowego zapisu w IndexedDB z grupowaniem zatwierdzeń wobec eksportu całej bazy i wobec silników zapisujących strony;
4. dialekt SQL ograniczony do encji pod kluczem, z synchronizacją wbudowaną w format WAL, zamiast warstwy replikacji dokładanej do pełnej bazy relacyjnej.
