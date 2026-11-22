# 0004. Dialekt Flat-SQL: jedna tabela w zapytaniu, obowiązkowy klucz, bez złączeń

- Status: obowiązuje
- Ostatnia zmiana: 2026-10-21

## Kontekst

Silnik działający u klienta ma być mały, przewidywalny i łatwy do replikacji. Pełny SQL (złączenia, podzapytania, klucze obce) powiększa silnik, komplikuje planer i dodaje więzy między tabelami, które trzeba utrzymać przy zmianach na wielu urządzeniach naraz.

## Decyzja

Składnia SQL nad modelem „klucz → wiersz”:

1. każda tabela ma jednokolumnowy klucz główny, który nie może być pusty;
2. zapytanie dotyczy dokładnie jednej tabeli; złączenia, podzapytania, `UNION` i klucze obce zwracają błąd `unsupported`;
3. indeksy dodatkowe są jednokolumnowe i nieunikalne.

Złączeń nie planuje się także w wariancie tylko do odczytu. Jednotabelowość jest częścią definicji języka, a nie brakiem do uzupełnienia.

## Uzasadnienie

- Encja z unikalnym kluczem jest naturalną granicą atomowości. Transakcje nie powinny obejmować wielu encji (Helland, 2007).
- Zapytanie o jedną tabelę ma jeden plan, widoczny w `EXPLAIN`. Zapytania po kluczu albo indeksie z limitem mają koszt niezależny od rozmiaru bazy (Armbrust i in., 2011).
- Bez więzów między tabelami replikacja jest prostsza. Trudność replikacji leży w więzach, a nie w języku zapytań.
- Przemysłowym odpowiednikiem jest język CQL w Cassandrze, czyli składnia SQL nad modelem klucz–wiersz (Stonebraker i Pavlo, 2024).

## Odrzucone warianty

- Pełny SQL albo zgodność z PostgreSQL lub SQLite. Moduł urósłby kilkanaście razy (PGlite i Zeta-Lite mają po około 3 MB po kompresji), a tę niszę zajmują już inne silniki.
- Złączenia tylko do odczytu po kluczu głównym. Nie psułyby przewidywalności ani replikacji, ale zmieniają tożsamość języka.
- Model dokumentowy bez SQL. To inna klasa systemów.

## Konsekwencje

- Relacje wiele-do-wielu składa aplikacja. Brak złączeń to najczęstszy zarzut wobec takich modeli (Litt i in., 2023).
- „Przewidywalne wykonanie” znaczy: plan zawsze dotyczy jednej tabeli i jest widoczny. Ograniczony koszt mają tylko odczyty po kluczu lub indeksie. Pełny przegląd i `GROUP BY` rosną z tabelą.
- To, co ma zmieniać się atomowo między urządzeniami, modeluje się jako jeden wiersz (decyzja 0019).

## Historia

- 2026-10-06: dialekt z pierwszej wersji silnika.
- 2026-10-21: złączenia tylko do odczytu wykreślono z planów rozwoju.
