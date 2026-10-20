# ADR-0007: Cel `wasm32`, bez memory64

| | |
|---|---|
| Status | przyjęta |
| Data | 2026-10-20 (rewizja literatury; `wasm32` od 2026-10-06) |
| Rodzaj | architektura |
| Powiązane | ADR-0006 |

## Kontekst

Moduł `wasm32` adresuje pamięć 32-bitowymi liczbami, więc może mieć najwyżej 4 GiB pamięci. Wasm 3.0 (2025) dodał memory64 – adresy 64-bitowe bez tego limitu. Przy danych w pamięci (ADR-0006) limit pamięci modułu jest limitem rozmiaru bazy.

## Decyzja

Silnik zostaje przy celu `wasm32-unknown-unknown`. Memory64 nie jest planowane.

## Uzasadnienie

- Przy adresach 32-bitowych przeglądarka sprawdza granice pamięci „za darmo” (rezerwuje przestrzeń adresową ze stronami ochronnymi). Przy 64-bitowych musi wstawić sprawdzenie przed każdym dostępem, więc memory64 jest wolniejsze i warto je włączać tylko przy faktycznej potrzebie powyżej 4 GiB [Q6].
- Dane docelowe są o rzędy wielkości mniejsze: ekwipunek gry to kilobajty, katalog 100 tys. produktów – 5 MB na dysku i kilkadziesiąt MB w pamięci. Na telefonie przeglądarka zamknie kartę, zanim zabraknie adresów – realnym limitem jest pamięć urządzenia.
- Zbiory powyżej 4 GiB to analityka dużych plików, którą obsługuje inny silnik (DuckDB-Wasm [D1]).

## Odrzucone alternatywy

- **Memory64** – spowolniłoby każde zapytanie u wszystkich użytkowników dla danych, których projekt nie obsługuje.

## Konsekwencje

- Limit 4 GiB pamięci modułu; scenariusz „CSV 500 MB” przypisano DuckDB-Wasm (todo 0.2).
- Zmniejszanie zużycia pamięci idzie przez zwartą reprezentację wierszy (todo 2.7), nie przez większą przestrzeń adresową.

## Źródła

[Q6], [D1], [W11]; praca o kosztach memory64 (Döllerer i Engelke 2024) – poza rdzeniem literatury.
