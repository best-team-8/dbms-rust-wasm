# 0013. UPDATE w miejscu, jeden odczyt zegara na instrukcję, wiele wierszy jednym wywołaniem

- Status: obowiązuje
- Ostatnia zmiana: 2026-10-25

## Kontekst

`UPDATE` 10% ze 100 000 wierszy trwał w prototypie 31 ms, wobec 9–14 ms w SQLite. Każdy zmieniany wiersz był kopiowany w całości, drugi raz wyszukiwany w drzewie i usuwany ze wszystkich indeksów, także gdy kolumna indeksu się nie zmieniła. Zegar był odczytywany osobno dla każdego wiersza. Wstawienie 100 000 wierszy z JavaScriptu trwało prawie dwa razy dłużej niż natywnie, bo każdy wiersz przekraczał granicę JavaScript–WebAssembly z pięcioma parametrami.

## Decyzja

1. `UPDATE`, który nie zmienia klucza głównego, szuka wiersza raz, zmienia tylko przypisane kolumny i aktualizuje tylko te indeksy, których kolumna się zmieniła. Dziennik wycofania trzyma tylko poprzednie wartości zmienionych kolumn.
2. WAL nadal dostaje cały nowy wiersz, więc format WAL, odtwarzanie i synchronizacja się nie zmieniają.
3. Instrukcja zmieniająca dane odczytuje zegar raz. Jej wiersze dostają znaczniki z tym samym czasem i kolejnym licznikiem, więc znaczniki dalej są unikalne i rosnące.
4. `executeMany(statement, rows)` wykonuje przygotowany `INSERT`, `UPDATE` albo `DELETE` dla wielu zestawów parametrów jako jedną instrukcję: wszystkie zmiany wchodzą razem albo wcale. JavaScript koduje parametry do bufora porcjami po około 64 KiB.

## Uzasadnienie

- Dojrzałe silniki aktualizują tylko indeksy zmienionych kolumn (SQLite, aktualizacje HOT w PostgreSQL).
- Zegar hybrydowy wymaga tylko znaczników rosnących i związanych z czasem fizycznym (Kulkarni i in., 2014). Jeden odczyt na instrukcję spełnia oba warunki.
- Zmierzone: `UPDATE` 10% w przeglądarce 30,9 → 15,4 ms. Wstawienie 100 000 wierszy w przeglądarce 273 → 151 ms, natywnie 145 ms, więc narzut WebAssembly spadł do kilku procent.

## Odrzucone warianty

- Zapis w WAL tylko zmienionych kolumn. Zmienia format paczek i regułę rozstrzygania konfliktów dla całego wiersza (decyzja 0019).
- Wsad kolumnowy (tablice liczb, jeden bufor tekstów). Wymaga od aplikacji przestawienia danych, a zysk byłby najwyżej kilkuprocentowy.
- Cały wsad w jednym buforze. Podwaja szczyt pamięci.

## Konsekwencje

- Mniej alokacji i mniej pamięci w dużej transakcji `UPDATE`.
- Obciążenie pomiarowe ma nową operację `insert_many` i identyfikator `benchmark-2`; wyniki starej operacji `insert` pozostały porównywalne z pierwszym pomiarem.

## Historia

- 2026-10-25: zmiana wiersza w miejscu, jeden odczyt zegara, `executeMany`.
