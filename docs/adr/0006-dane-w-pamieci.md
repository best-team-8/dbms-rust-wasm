# ADR-0006: Dane w całości w pamięci

| | |
|---|---|
| Status | przyjęta |
| Data | 2026-10-06 (silnik) |
| Rodzaj | architektura |
| Powiązane | ADR-0001, ADR-0007, ADR-0009, ADR-0013; todo 2.6, 2.7 |

## Kontekst

Zapytania mają wykonywać się lokalnie i bez opóźnień (ADR-0001), także w wątku głównym przeglądarki (ADR-0013), gdzie nie ma synchronicznego dostępu do plików.

## Decyzja

Cała baza jest w pamięci modułu: wiersze w `BTreeMap` uporządkowanej po kluczu głównym, indeksy wtórne jako mapy wartość → zbiór kluczy (nie są zapisywane, odbudowuje się je przy wczytaniu). Dysk służy wyłącznie trwałości (ADR-0009).

## Uzasadnienie

- Bazy w pamięci zawdzięczają szybkość głównie brakowi kodowania struktur do formatu dyskowego, a trwałość zapewniają logiem i snapshotami ([P1] rozdz. 3, [S1]).
- Jednowątkowy silnik w pamięci nie płaci za blokady, latche i bufor stron [S4].
- Zapytanie nie czeka na dysk, więc może być synchroniczne i działać w wątku głównym.

## Odrzucone alternatywy

- **Format stronicowy na dysku z buforem stron (jak SQLite)** – w przeglądarce wymaga synchronicznego dostępu do OPFS, czyli Web Workera; to inna architektura.
- **Stronicowanie na OPFS dla danych większych niż pamięć** – odrzucone jako tryb domyślny; zmieniałoby architekturę i wymagało Workera ([use-cases.md](../research/use-cases.md), granice koncepcji).

## Konsekwencje

- Rozmiar bazy ograniczają pamięć urządzenia i limit `wasm32` (ADR-0007); Flat-SQL celuje w dane operacyjne, nie w analitykę dużych plików (tę robi DuckDB-Wasm [D1]).
- Każdy wiersz to osobna alokacja: więcej pamięci, wolniejszy snapshot i odtwarzanie niż w SQLite (todo 2.7).
- Checkpoint koduje całą bazę – dziś synchronicznie, co powoduje przestój (todo 2.6, [S10]).

## Źródła

[P1] rozdz. 3, [S1], [S4], [D1], [S10]; [benchmark-results.md](../research/benchmark-results.md).
