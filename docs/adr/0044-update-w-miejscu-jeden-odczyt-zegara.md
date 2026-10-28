# ADR-0044: UPDATE zmienia wiersz w miejscu, jeden odczyt zegara na instrukcję

| | |
|---|---|
| Status | przyjęta |
| Data | 2026-10-25 |
| Rodzaj | architektura |
| Powiązane | ADR-0008, ADR-0009, ADR-0015, ADR-0039; zadanie todo 2.8 |

## Kontekst

`UPDATE` 10% ze 100 000 wierszy trwał w Benchmarku 1 31 ms w przeglądarce i 32 ms natywnie, wobec 8,8–13,7 ms w SQLite. Każdy zmieniany wiersz był kopiowany w całości (także kolumny tekstowe) i walidowany jako cały wiersz, zapisywany ogólną ścieżką `write_put`, która drugi raz szukała klucza w drzewie, usuwała i wstawiała klucz w każdym indeksie, także gdy kolumna indeksu się nie zmieniła, i trzymała w dzienniku wycofania cały poprzedni wiersz. Znacznik HLC odczytywał zegar systemowy (w przeglądarce `Date.now()` przez granicę WASM↔JS) dla każdego wiersza osobno (ADR-0015).

## Decyzja

1. `UPDATE`, który nie przypisuje klucza głównego, zbiera dla każdego wiersza tylko wartości przypisanych kolumn, wyliczone w kolejności `SET` i sprawdzone (koercja, `NOT NULL`) w kolejności kolumn, tak jak cały wiersz. Zapis (`Database::write_update`) szuka wiersza raz, zamienia w nim przypisane kolumny, aktualizuje tylko indeksy, których wartość kolumny się zmieniła, i zapisuje w dzienniku wycofania tylko poprzednie wartości tych kolumn i poprzedni znacznik (`Undo::Update`).
2. WAL dostaje cały nowy wiersz, kodowany wprost z wiersza w tabeli, więc format WAL, odtwarzanie i synchronizacja się nie zmieniają.
3. `UPDATE` zmieniający klucz główny zostaje przy dotychczasowej ścieżce (usunięcie i zapis pod nowym kluczem).
4. Instrukcja zmieniająca dane (`INSERT`, `UPDATE`, `DELETE`) odczytuje zegar raz, a każdy jej wiersz dostaje znacznik z tej samej części fizycznej i kolejnym licznikiem logicznym (`Hlc::tick_at`). Znaczniki pozostają unikalne i rosnące.

## Uzasadnienie

- Rozbicie kosztów starej ścieżki dla 10 000 wierszy natywnie: kopia z walidacją 5,8–6,3 ms, utrzymanie indeksu niezmienionej kolumny około 5,5 ms, drugie wyszukanie 2,3–3,1 ms, odczyty zegara 0,33 ms (w JavaScripcie `Date.now()` około 81 ns na wywołanie). Pomiar po zmianie: [optimization-results.md §6](../research/optimization-results.md).
- Dojrzałe silniki aktualizują tylko indeksy zmienionych kolumn: SQLite pomija indeksy niezmienionych kolumn, PostgreSQL ma aktualizacje HOT (bez nowych wpisów indeksów, gdy zmienione kolumny nie są indeksowane). Wiersz zmieniany w miejscu z dziennikiem wycofania kolumn jest zgodny z modelem transakcji z dziennikiem wycofania i wykonaniem szeregowym (ADR-0008).
- HLC wymaga tylko, żeby znaczniki były monotoniczne i wiązały się z czasem fizycznym [C2]; jeden odczyt na instrukcję spełnia oba warunki, a instrukcja trwa milisekundy.

## Odrzucone alternatywy

- **Pozostawienie kopii całego wiersza i tylko pominięcie niezmienionych indeksów** – kopia z walidacją to najdroższa część po indeksach.
- **Dziennik wycofania z całym poprzednim wierszem przy zmianie w miejscu** – wymaga kopii, której zmiana ma uniknąć.
- **Zapis w WAL tylko zmienionych kolumn** – zmienia format WAL i paczek zmian (ADR-0009, ADR-0011) oraz semantykę LWW per wiersz (ADR-0015); osobna decyzja z LWW per kolumna ([todo_future.md](../todo_future.md) 12.3).

## Konsekwencje

- Mniej alokacji i mniejsza pamięć dużej transakcji `UPDATE` (dziennik wycofania trzyma tylko zmienione kolumny).
- Wiersze jednej instrukcji mają znaczniki z tą samą częścią fizyczną; przy więcej niż 65 536 wierszach w jednej milisekundzie licznik logiczny przechodzi na część fizyczną, tak jak dotąd przy szybkich zapisach.
- Poprawność sprawdza test losowych aktualizacji wobec tabeli bez indeksów, z wycofaniem w transakcji i odtworzeniem z WAL ([tests-results.md §3.5](../research/tests-results.md)).

## Źródła

[C2]; [optimization-results.md §6](../research/optimization-results.md).
