# ADR-0042: Warunki i przypisania kompilowane przy kompilacji instrukcji: warianty typowane i domknięcia, interpreter jako rezerwa

| | |
|---|---|
| Status | przyjęta |
| Data | 2026-10-25 |
| Rodzaj | architektura |
| Powiązane | ADR-0005, ADR-0014, ADR-0039, ADR-0041; zadania todo 2.10, 2.15, 2.7 |

## Kontekst

Rozbicie kosztów Benchmarku 1 przypisało interpreterowi wyrażeń około dwóch trzecich czasu pełnego skanu z filtrem: silnik 8,1 ms, ręczna pętla po tym samym drzewie z tym samym warunkiem 2,8 ms (ADR-0039). Interpreter (`expr::eval_pred`) dla każdego wiersza schodzi rekurencyjnie po drzewie wyrażenia, zwraca `Result<Cow<Value>>` z każdego węzła i porównuje wartości przez ogólny porządek `Value`, także wtedy, gdy typ kolumny i literału jest znany przy kompilacji (`price > 50` przy kolumnie FLOAT). Od ADR-0041 każda instrukcja ma postać skompilowaną, w której można przechować wynik takiej specjalizacji.

## Decyzja

1. Przy kompilacji instrukcji każdy koniunkt warunku `WHERE` (SELECT, UPDATE, DELETE) i każde wyrażenie `SET` są kompilowane (`compile::predicate`, `compile::value`).
2. Koniunkt `kolumna op literał` jest wariantem wyliczenia `Pred` z literałem zamienionym na typ kolumny (np. `50` → `50.0` przy FLOAT), ocenianym jednym dopasowaniem wartości kolumny, bez wywołania przez wskaźnik. Węzły złożone (AND, OR, NOT, BETWEEN, porównania i arytmetyka kolumn INT i FLOAT) są domknięciami składającymi takie warianty i typowane wyrażenia liczbowe. Każdy inny węzeł jest oceniany przez interpreter wewnątrz domknięcia, więc kompiluje się każde wyrażenie.
3. Semantyka jest semantyką interpretera: logika trójwartościowa, porządek wartości (NaN za wszystkimi liczbami i równy sobie), sprawdzana arytmetyka całkowita, te same błędy. Wyrocznią jest interpreter: test różnicowy porównuje wynik i komunikat błędu dla losowych wyrażeń i wierszy.
4. Interpreter zostaje dla projekcji, `HAVING`, kluczy `ORDER BY` i `GROUP BY` (kolumna jako klucz jest już oceniana bez wywołania funkcji) i jako rezerwa wewnątrz domknięć.

## Uzasadnienie

- Pomiar przed i po ([optimization-results.md §4](../research/optimization-results.md)): pełny skan z filtrem natywnie 7,41 → 5,38 ms, w Node.js 8,30 → 5,20 ms; `DELETE` 10% −5 do −7%, `UPDATE` 10% −4,5% (pomiar natywny z 20 uruchomieniami).
- Eksperyment z tym samym warunkiem na 100 000 wierszy (pętla ręczna, bez wykonawcy): domknięcia z operatorem wybieranym w czasie wykonania 4,1 ms, domknięcia ze stałym operatorem 3,5 ms, płaska lista typowanych warunków wybieranych dopasowaniem 3,2 ms, kod pisany ręcznie 2,1 ms. Wywołanie przez wskaźnik kosztuje około 10 ns na koniunkt i wiersz, dlatego najczęstszy koniunkt jest wariantem, a domknięcia zostały dla węzłów rzadszych. Pierwsza wersja z samymi domknięciami dała silnikowi 5,6 ms zamiast 3,9 ms pętli z wariantami.
- Koszt interpretacji wiersz po wierszu dominuje wykonanie w silnikach przetwarzających krotka po krotce (Boncz, Zukowski, Nes, CIDR 2005); kompilacja i wektoryzacja to dwa sposoby jego usunięcia (Kersten i in., PVLDB 2018). Specjalizacja przy kompilacji instrukcji nie generuje kodu w czasie wykonania, więc nie płaci kosztu kompilacji, który przy zapytaniach trwających mikrosekundy przekreśla generowanie kodu WASM ([D3], poza rdzeniem).

## Odrzucone alternatywy

- **Same domknięcia (pierwotny plan zadania)** – zmierzone wolniejsze od wariantów o około 30% w samej ocenie warunku.
- **Generowanie kodu WebAssembly w czasie wykonania** – koszt kompilacji przy zapytaniach mikrosekundowych [D3]; rdzeń nie ma zależności (ADR-0005).
- **Wektoryzacja (ocena warunku na paczkach wierszy)** – wymaga kolumnowego albo paczkowego układu danych i przebudowy wykonawcy; wiersze są obecnie osobnymi wektorami wartości.
- **Funkcja wywoływana bez wstawiania (`#[inline(never)]`) dla reszty warunku** – sprawdzona, żeby pętle zapytań bez warunku były mniejsze: pełny skan wolniejszy (5,47 → 6,29 ms), a zapytania bez warunku bez zmiany.

## Konsekwencje

- Moduł WebAssembly rośnie o około 5,7 KiB po kompresji.
- Po zmianie warunek kosztuje mniej niż reszta wykonawcy: pętla ręczna z tymi samymi skompilowanymi warunkami trwa 3,8–3,9 ms wobec 5,4 ms silnika, więc kolejnym kosztem pełnego skanu jest przejście drzewa wierszy i obsługa wiersza w wykonawcy (zwarta reprezentacja wierszy, todo 2.7).
- Zapytania bez warunku (grupowanie, top-k) są natywnie o około 2% wolniejsze, choć ich kod się nie zmienił; dwie sprawdzone przyczyny (wstawianie porównania liczb zmiennoprzecinkowych, wstawianie oceny warunku do pętli) odrzucono pomiarem.

## Źródła

[D3] (poza rdzeniem); Boncz P., Zukowski M., Nes N., *MonetDB/X100: Hyper-Pipelining Query Execution*, CIDR 2005; Kersten T. i in., *Everything You Always Wanted to Know About Compiled and Vectorized Queries But Were Afraid to Ask*, PVLDB 11(13), 2018; [optimization-results.md §4](../research/optimization-results.md), [tests-results.md §3.4](../research/tests-results.md).
