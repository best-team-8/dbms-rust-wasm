# Wyniki optymalizacji silnika

Szesnaście zmian między prototypem a wersją `v2-optymalizacja`. Każda wychodzi od słabej strony zmierzonej w Benchmarku 1 i kończy się pomiarem przed zmianą i po niej. Trzynaście potwierdziło się, dwie częściowo, a zmiana ustawień kompilacji niczego nie dała. Wynik wszystkich zmian razem: [Benchmark 2](benchmark-results.md#benchmark-2-prototyp-i-wersja-po-optymalizacjach).

## Jak mierzono każdą zmianę

Skrypt `bench/compare.js` mierzy wersję przed zmianą i po niej tym samym narzędziem co benchmark, na przemian, każdą w 10 nowych procesach Node.js i 10 procesach natywnych. Zmiana jest wykazana, gdy 95% przedziały ufności się nie nakładają. Sumy kontrolne obu wersji muszą być równe, bo optymalizacja nie może zmienić żadnej odpowiedzi. Dodatkowo program `cost-split` rozbija czas operacji na części, a program `allocations` liczy alokacje pamięci, które nie zależą od obciążenia maszyny. Zmiany granicy z JavaScriptem mierzono też w przeglądarce.

Surowe wyniki, raporty `compare.md` i skrypty są w [data/optimization/](data/optimization/README.md), po jednym katalogu na zmianę. Pomiary wykonano 25–28 października 2026 na laptopie z benchmarków. Część z nich przy obciążeniu procesora przez inne aplikacje (10–22%), co zwiększa rozrzut.

Zmiana operacji, której zmieniony kod nie wykonuje, bywa widoczna przy rozłącznych przedziałach. Takie różnice mają zwykle przeciwny znak w różnych środowiskach. Przypisano je rozmieszczeniu kodu w module, stanowi sterty po wcześniejszych operacjach albo chwili, w której V8 podmienia funkcje WebAssembly na zoptymalizowane. Opisano je przy zmianie, której dotyczą.

## Zestawienie

| Zmiana | Operacja docelowa: przed → po | Wynik |
|---|---|---|
| [1. Liczenie wierszy po indeksie](#1-liczenie-wierszy-po-indeksie-bez-ich-czytania) | `COUNT(*)` po indeksie natywnie 486 → 1,66 µs | potwierdzona |
| [2. Pomijanie warunku gwarantowanego przez ścieżkę](#2-pomijanie-warunku-gwarantowanego-przez-ścieżkę-dostępu) | zakres 1000 kluczy natywnie 96,8 → 56,1 µs | potwierdzona |
| [3. Plan w instrukcji przygotowanej](#3-plan-zapamiętany-w-instrukcji-przygotowanej) | odczyt po kluczu natywnie 2,71 → 1,27 µs | potwierdzona |
| [4. Warunki kompilowane](#4-warunki-kompilowane-przy-przygotowaniu-instrukcji) | pełny przegląd natywnie 7,58 → 5,52 ms | częściowo |
| [5. Kolejka zmian od włączenia synchronizacji](#5-kolejka-zmian-od-włączenia-synchronizacji) | odtworzenie natywnie 111 → 54,7 ms | potwierdzona |
| [6. `UPDATE` w miejscu](#6-update-w-miejscu) | `UPDATE` 10% natywnie 23,1 → 15,7 ms | potwierdzona |
| [7. Hurtowa budowa indeksów](#7-hurtowa-budowa-indeksów) | budowa indeksu 21–22 → 11 ms | potwierdzona |
| [8. Grupowanie przez tablicę mieszającą](#8-grupowanie-przez-tablicę-mieszającą) | `GROUP BY` natywnie 16,7 → 11,0 ms | potwierdzona |
| [9. Otwarty plik WAL w magazynie Node.js](#9-otwarty-plik-wal-w-magazynie-nodejs) | zatwierdzenie bez `fsync` w Windows 595 → 85 µs | potwierdzona |
| [10. Profil granicy z JavaScriptem](#10-profil-granicy-z-javascriptem) | pomiar bez zmiany kodu | pomiar |
| [11. Obiekt wyniku tworzony w JavaScripcie](#11-obiekt-wyniku-tworzony-w-javascripcie) | odczyt po kluczu w przeglądarce 8,99 → 3,95 µs | potwierdzona |
| [12. Binarny format wyniku](#12-binarny-format-wyniku) | pobranie 10 000 wierszy w przeglądarce 14,0 → 1,59 ms | potwierdzona |
| [13. Wiele wierszy jednym wywołaniem](#13-wiele-wierszy-jednym-wywołaniem) | wstawienie 100 000 wierszy w przeglądarce 273 → 151 ms | potwierdzona |
| [14. Krótkie teksty w wartości](#14-krótkie-teksty-w-wartości) | pamięć po wczytaniu 54,8 → 51,8 MiB | częściowo |
| [15. Checkpoint krokami](#15-checkpoint-krokami) | najdłuższa pauza 27,8 → 1,70 ms | potwierdzona |
| [16. Ustawienia kompilacji](#16-ustawienia-kompilacji) | siedem wariantów, żaden lepszy | odrzucona |

## 1. Liczenie wierszy po indeksie bez ich czytania

**Problem.** `COUNT(*) WHERE category = ?` trwał 438 µs w przeglądarce, wobec 63–69 µs w SQLite. Natywnie tyle samo, więc to koszt silnika. Indeks zna zbiór kluczy każdej wartości, ale wykonawca czytał każdy z 1000 wierszy kategorii i ponownie sprawdzał warunek. SQLite odpowiada z samego indeksu.

**Zmiana.** Planer mówi, czy ścieżka dostępu gwarantuje cały warunek. Jeśli tak, a zapytanie liczy tylko `COUNT(*)`, wynik to długość zbioru kluczy albo liczba kluczy w zakresie. Zakres indeksu bez dolnej granicy zaczyna się za NULL, bo NULL nie spełnia żadnego porównania.

**Wynik.** Natywnie 486 → 1,66 µs, Node.js 498 → 9,47 µs. Odczytanych wierszy 1000 → 0. W Node.js pozostały czas to prawie w całości koszt granicy z JavaScriptem. Dane: `01-count-by-index/`.

## 2. Pomijanie warunku gwarantowanego przez ścieżkę dostępu

**Problem.** Każde zapytanie po kluczu albo indeksie sprawdzało w każdym wierszu warunek, który ścieżka już gwarantowała (`FILTER id BETWEEN …`). Rozbicie kosztów: to ponad połowa czasu zakresu 1000 kluczy.

**Zmiana.** Warunek jest listą części połączonych `AND`. Planer zwraca maskę części gwarantowanych przez ścieżkę, a wykonawca liczy tylko resztę. Pierwsza wersja spowalniała zapytania bez `WHERE` o 3–8%, więc wersja końcowa sprawdza raz na zapytanie, czy reszta w ogóle istnieje.

**Wynik.** Zakres 1000 kluczy natywnie 96,8 → 56,1 µs (−44% w pomiarze z 20 uruchomieniami). Grupowanie i sortowanie w rozrzucie. Dane: `02-guaranteed-filter/`.

## 3. Plan zapamiętany w instrukcji przygotowanej

**Problem.** Instrukcja przygotowana trzymała tylko drzewo składni, więc każde wykonanie od nowa wiązało nazwy, wybierało ścieżkę i kopiowało nazwy kolumn. Samo wyszukanie to 9% z 2,2 µs odczytu po kluczu. Most JavaScript przy każdym `query` przygotowywał instrukcję od nowa.

**Zmiana.** Instrukcje są kompilowane przy przygotowaniu, a ścieżka dostępu z parametrem jest szablonem wypełnianym przy wykonaniu. Most pamięta 64 ostatnio używane instrukcje i nie sprawdza WAL po odczytach.

**Wynik.** Natywnie odczyt po kluczu 2,71 → 1,27 µs, `COUNT(*)` 1,73 → 0,70 µs, alokacje odczytu 26 → 6. W moście `query` z parametrem 24,5 → 8,5 µs. Przygotowanie jest droższe (natywnie 1,6–1,9 → 2,9–3,2 µs). W Node.js przy wyniku jako obiekty zysk ginie w koszcie granicy, którym zajęły się zmiany 11 i 12. Dane: `03-prepared-plan/`.

## 4. Warunki kompilowane przy przygotowaniu instrukcji

**Problem.** Pełny przegląd z warunkiem trwał natywnie 8,1 ms, a ta sama pętla z warunkiem napisanym ręcznie 2,8 ms. Dwie trzecie czasu zajmował interpreter wyrażeń, który dla każdego wiersza schodził rekurencyjnie po drzewie.

**Zmiana.** Najczęstszy warunek `kolumna op literał` staje się wariantem dopasowanym do typu kolumny. Warunki złożone są funkcjami składającymi warianty. Pierwotny plan zakładał same funkcje, ale pomiar pokazał, że wywołanie przez wskaźnik kosztuje około 10 ns na warunek i wiersz, a warianty są o około 30% szybsze.

**Wynik.** Pełny przegląd natywnie 7,58 → 5,52 ms (−27%), Node.js 8,46 → 5,38 ms (−37%). `UPDATE` −5%, `DELETE` −8%. Spodziewano się około −65%. Reszta różnicy to praca wykonawcy wokół warunku (około 1,5 ms na 100 000 wierszy), a nie sam warunek. Grupowanie i sortowanie natywnie zwolniły o około 2% bez ustalonej przyczyny. Dane: `04-compiled-expressions/`.

## 5. Kolejka zmian od włączenia synchronizacji

**Problem.** Kolejka zmian miała wpis dla każdego wstawionego wiersza także w bazie bez synchronizacji: 100 002 wpisy po wstawieniu 100 000 wierszy. Odpowiadała za połowę czasu odtworzenia i jedną czwartą rozmiaru snapshotu.

**Zmiana.** Kolejka przyjmuje wiersze dopiero po włączeniu synchronizacji, a włączenie oznacza cały stan jako oczekujący. Snapshot zapisuje kolejkę zwięźle (format 2).

**Wynik.** Natywnie odtworzenie 111 → 54,7 ms, snapshot 25,7 → 18,2 ms i 5,03 → 3,71 MiB, wstawienie −22%, `UPDATE` −23%, `DELETE` −32%. Pamięć po wczytaniu 73,3 → 54,8 MiB. W Node.js odczyty wielu wierszy zwolniły o 7–10%, choć ich kod się nie zmienił. Przyczyny nie ustalono; prawdopodobnie inny stan sterty modułu po wstawianiu. Dane: `05-outbox-with-sync/`.

## 6. `UPDATE` w miejscu

**Problem.** `UPDATE` 10% trwał 31 ms wobec 9–14 ms w SQLite. Rozbicie kosztów dla 10 000 wierszy: kopia wiersza z walidacją 6 ms, indeks niezmienionej kolumny 5,5 ms, drugie wyszukanie 2,5 ms.

**Zmiana.** Wiersz jest zmieniany w miejscu, aktualizowane są tylko indeksy zmienionych kolumn, a zegar jest czytany raz na instrukcję.

**Wynik.** Natywnie 23,1 → 15,7 ms, Node.js 33,8 → 17,5 ms. Alokacje o połowę. Rozrzut w WebAssembly zmalał kilkukrotnie, bo zniknęło 10 000 odczytów zegara przez granicę. Dane: `06-update-in-place/`.

## 7. Hurtowa budowa indeksów

**Problem.** Odtworzenie bazy budowało indeks 100 000 pojedynczymi wstawieniami: 21 ms, jedna szósta czasu odtworzenia.

**Zmiana.** Wiersze przychodzą w kolejności kluczy, więc klucze są grupowane według wartości w tablicy mieszającej, a drzewa budowane hurtem z posortowanego wejścia. Pierwszy wariant, sortowanie wszystkich par (wartość, klucz), był ponad dwa razy wolniejszy. Budowę przy pierwszym użyciu indeksu odrzucono, bo przesuwa ten sam koszt na pierwsze zapytanie, czyli w nieprzewidywalne miejsce. Zapis indeksów w snapshocie odrzucono, bo powiększa snapshot, a odtworzenie i tak dekoduje wiersze.

**Wynik.** Budowa indeksu 21–22 → 11 ms, odtworzenie natywnie 53,9 → 46,2 ms. Dane: `07-bulk-index-build/`.

## 8. Grupowanie przez tablicę mieszającą

**Problem.** `GROUP BY` po 100 kategoriach trwał 15,7 ms wobec 3,6 ms w DuckDB-Wasm. Wykonawca szukał grupy każdego wiersza w drzewie porównaniami tekstów.

**Zmiana.** Przy jednej kolumnie grupującej grupa jest szukana w tablicy mieszającej, a wynik sortowany na końcu. Przejście po indeksie kolumny grupującej zmierzono jako wolniejsze (27–28 ms).

**Wynik.** Natywnie 16,7 → 11,0 ms, Node.js 19,2 → 11,7 ms. Dane: `08-group-by-hash/`.

## 9. Otwarty plik WAL w magazynie Node.js

**Problem.** Zatwierdzenie w magazynie plikowym Node.js trwało bez `fsync` 0,40–0,60 ms, wobec 10 µs w narzędziu natywnym, bo plik WAL był otwierany i zamykany przy każdym dopisaniu.

**Zmiana.** Jeden uchwyt pliku WAL między checkpointami. Kolejność utrwaleń i model awarii bez zmian.

**Wynik.** Bez `fsync`: Windows 595 → 85 µs, Linux 432 → 177 µs. Z `fsync`: 1,42 → 0,72 ms i 3,14 → 2,83 ms. Testy awarii zapisu dają te same wyniki. Dane: `09-fsstore-open-wal/`.

## 10. Profil granicy z JavaScriptem

**Pytanie.** Na co idzie czas odczytu po kluczu w WebAssembly, skoro sam silnik potrzebuje natywnie 1,2 µs? Recenzowanych pomiarów tej granicy dla silników baz nie znaleziono.

**Pomiar.** Odczyt wykonany w wariantach, które kończą się w różnych miejscach drogi przez granicę, i profil V8.

**Wynik.** Wykonanie z parametrem trwa w WebAssembly 1,18 µs, tyle co natywnie, więc przekazanie parametru jest pomijalne. Z 7,58 µs odczytu z wynikiem jako obiekty 6,4 µs to zamiana wyniku na obiekty JavaScriptu. W profilu dekodowanie UTF-8 i kod sklejający zajmowały po około 32% czasu. To wyznaczyło zmiany 11 i 12. Dane: `10-boundary-profile/`.

## 11. Obiekt wyniku tworzony w JavaScripcie

**Problem.** Każdy wynik budował z Rusta pięć kluczy obiektu, nazwy kolumn i komunikat, dekodując je z UTF-8 i ustawiając każdy osobnym wywołaniem przez kod sklejający.

**Zmiana.** Obiekt tworzy funkcja JavaScriptu dołączona do modułu, w której klucze są literałami. Nazwy kolumn instrukcji przygotowanej powstają raz na plan, a każdy wynik dostaje ich kopię. Internowanie tekstów przez wasm-bindgen odrzucono, bo zostawia osobne wywołanie na każdy klucz.

**Wynik.** Odczyt po kluczu: Node.js 8,20 → 3,04 µs, przeglądarka 8,99 → 3,95 µs. Wynik jako obiekty stał się szybszy od JSON przy jednym wierszu. Dane: `11-result-object/`.

## 12. Binarny format wyniku

**Problem.** Pobranie 10 000 wierszy trwało w przeglądarce 14,0 ms wobec 3,8 ms natywnie. Każda wartość przechodziła przez granicę osobno, a dekodowanie UTF-8 zajmowało 36% czasu.

**Zmiana.** Wynik jest kodowany wprost z tabeli do jednego bufora, a JavaScript dekoduje go jednym wywołaniem.

**Wynik.** Pobranie 10 000 wierszy: Node.js 10,8 → 1,93 ms, przeglądarka 14,0 → 1,59 ms, czyli szybciej niż natywnie z kopią wierszy. Odczyt po kluczu w przeglądarce 3,99 → 3,29 µs, poniżej celu 3,5 µs, który odpowiadał deklaracji autora Zeta-Lite. Operacje, które nie używają zmienionego kodu, przyspieszyły o 4–16%, prawdopodobnie przez mniej rozdrobnioną stertę modułu. Dane: `12-binary-result/`.

## 13. Wiele wierszy jednym wywołaniem

**Problem.** Wstawienie 100 000 wierszy trwało w WebAssembly 1,8 raza dłużej niż natywnie. Każdy wiersz przekraczał granicę z pięcioma parametrami, a połowę czasu zajmowało samo wywołanie.

**Zmiana.** `executeMany(statement, rows)` wykonuje instrukcję dla wielu zestawów parametrów jako jedną instrukcję. JavaScript koduje parametry do bufora porcjami po 64 KiB. Obciążenie pomiarowe dostało operację `insert_many`.

**Wynik.** Node.js 283 → 160 ms, przeglądarka 273 → 151 ms, natywnie 145 ms. Narzut WebAssembly spadł z 1,8 do 1,04–1,10. Dane: `13-insert-many/`.

## 14. Krótkie teksty w wartości

**Problem.** Baza zajmowała około ośmiu razy więcej pamięci niż jej snapshot. Plan zakładał wiersze w ciągłych blokach pamięci zamiast osobnych alokacji.

**Zmiana.** Doświadczenie z czterema układami wierszy pokazało, że koszt leży w osobnych alokacjach krótkich tekstów, a bloki ciągłe nie pomagają. Teksty do 14 bajtów są trzymane w samej wartości. Pierwsza wersja spowalniała wstawianie o 11–17%, póki porównanie tekstów było rozwijane w miejscu wywołania.

**Wynik.** Pamięć po wczytaniu 54,8 → 51,8 MiB, alokacje odtworzenia 319 540 → 119 440. W przeglądarce zakres −11%, grupowanie −14%, snapshot −10%, odtworzenie −6%. Natywnie sortowanie zwolniło o 18%, a grupowanie o 8%; przyczyny nie ustalono. Dane: `14-compact-text/`.

## 15. Checkpoint krokami

**Problem.** Checkpoint blokował wątek główny na 28 ms przy 100 000 wierszy. Kodowanie wierszy zajmowało około 9 ms, suma kontrolna około 9 ms, a zapis tablicy bajtów do IndexedDB 4,4 ms.

**Zmiana.** Snapshot jest kodowany porcjami po 4096 wierszy z kopią wierszy zmienianych w trakcie, a do IndexedDB trafia jako `Blob`.

**Wynik.** Najdłuższa pauza przy 100 000 wierszy 27,8 → 1,70 ms, na poziomie zakłóceń w bezczynności, niezależnie od liczby wierszy. Cały checkpoint trwa tyle samo. Ponowne otwarcie bazy z IndexedDB jest o 5–6% krótsze. Dane: `15-checkpoint-steps/`.

## 16. Ustawienia kompilacji

**Pytanie.** Czy moduł kompilowany pod rozmiar albo z innym alokatorem lub SIMD jest lepszy?

**Pomiar.** Siedem wariantów, każdy z jedną zmianą, w Node.js.

**Wynik.** Kompilacja pod rozmiar zmniejsza moduł o 17–25%, ale spowalnia operacje nawet o 78%. Pozostałe warianty nic nie dają. Konfiguracja została bez zmian. Dane: `16-build-options/`.
