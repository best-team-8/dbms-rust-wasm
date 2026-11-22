# 0012. Plan i warunki kompilowane raz, przy przygotowaniu instrukcji

- Status: obowiązuje
- Ostatnia zmiana: 2026-10-25

## Kontekst

Pierwsza wersja silnika przy każdym wykonaniu instrukcji przygotowanej od nowa rozwiązywała nazwy kolumn, wstawiała parametry, wybierała ścieżkę dostępu i kopiowała nazwy kolumn. Warunek `WHERE` był za każdym razem sprawdzany w całości, nawet gdy indeks już go gwarantował. Interpreter wyrażeń zajmował około dwóch trzecich czasu pełnego przeglądu tabeli. Rozbicie kosztów pokazało, że z 2,3 µs odczytu po kluczu samo wyszukanie w drzewie to 0,2 µs.

## Decyzja

1. `SELECT`, `INSERT`, `UPDATE` i `DELETE` są kompilowane przy przygotowaniu instrukcji. Nazwy są rozwiązane, parametry `?` zostają odwołaniami, a nazwy kolumn wyniku są liczone raz.
2. Ścieżka dostępu: warunek bez parametrów dostaje ją raz. Pojedynczy warunek na kluczu albo indeksie z parametrem (`col = ?`, `col < ?`, `BETWEEN ? AND ?`) dostaje szablon wypełniany przy wykonaniu. Inne warunki wybierają ścieżkę przy każdym wykonaniu.
3. Planer zwraca razem ze ścieżką listę części warunku, które ścieżka już gwarantuje. Wykonawca sprawdza tylko resztę. `COUNT(*)` z warunkiem w całości zagwarantowanym przez indeks liczy wiersze bez ich czytania.
4. Najczęstszy warunek, `kolumna op literał`, jest kompilowany do wariantu dopasowanego do typu kolumny. Warunki złożone są funkcjami składającymi takie warianty. Rzadkie przypadki ocenia dotychczasowy interpreter, który jest też wzorcem poprawności w testach.
5. Skompilowana postać jest ważna dla jednej wersji katalogu tabel. Każda zmiana schematu zmienia wersję i wymusza ponowną kompilację.
6. Most JavaScript pamięta ostatnio używane instrukcje przygotowane (domyślnie 64).

## Uzasadnienie

- Tak działa instrukcja przygotowana w SQLite: kompilowana raz, wykonywana wiele razy (Gaffney i in., 2022).
- Koszt interpretacji wiersz po wierszu dominuje silniki, które przetwarzają po jednym wierszu (Boncz i in., 2005). Kompilacja bez generowania kodu w czasie wykonania nie płaci kosztu kompilacji, który przy zapytaniach trwających mikrosekundy przekreśla generowanie kodu WebAssembly (Haffner i Dittrich, 2021).
- Zmierzone natywnie: odczyt po kluczu 2,71 → 1,27 µs, `COUNT(*)` po indeksie 486 → 1,66 µs, zakres 1000 kluczy 96,8 → 56,1 µs, pełny przegląd z warunkiem 7,4 → 5,4 ms. W moście `query` z parametrem 24,5 → 8,5 µs.

## Odrzucone warianty

- Same funkcje zamiast wariantów. Zmierzone o około 30% wolniejsze przy ocenie warunku.
- Generowanie kodu WebAssembly w czasie wykonania. Koszt kompilacji i zależności.
- Szablon ścieżki dla dowolnego warunku z parametrami. Duża złożoność, mały zysk.
- Pamięć instrukcji w rdzeniu dla zapytań z tekstu SQL. Zapytania z wartościami wpisanymi w tekst nigdy się nie powtarzają.

## Konsekwencje

- Przygotowanie instrukcji jest droższe (natywnie około 3 µs), wykonanie tańsze.
- Każda nowa ścieżka dostępu musi określić, które części warunku gwarantuje. Sprawdza to test porównujący odpowiedzi z pełnym przeglądem tabeli.

## Historia

- 2026-10-25: kompilacja instrukcji, części warunku gwarantowane przez ścieżkę, warianty typowane. Wcześniej warunek był interpretowany przy każdym wierszu.
