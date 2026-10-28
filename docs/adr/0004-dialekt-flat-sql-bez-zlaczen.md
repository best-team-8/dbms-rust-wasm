# ADR-0004: Dialekt Flat-SQL – jedna tabela, obowiązkowy klucz, bez złączeń

| | |
|---|---|
| Status | przyjęta |
| Data | 2026-10-06 (silnik, [flatsql-core/src/sql/README.md](../../crates/flatsql-core/src/sql/README.md)) |
| Rodzaj | założenie |
| Powiązane | ADR-0015, ADR-0018, ADR-0030; todo 0.1 |

## Kontekst

Silnik działający po stronie klienta ma być mały, przewidywalny i łatwy do replikacji. Pełny SQL relacyjny (złączenia, podzapytania, klucze obce) zwiększa rozmiar silnika, komplikuje planer i wprowadza więzy między tabelami, które trzeba utrzymać przy współbieżnych zmianach na wielu replikach.

## Decyzja

Składnia SQL, model danych klucz–wiersz:

1. każda tabela ma jednokolumnowy `PRIMARY KEY` (zawsze `NOT NULL`);
2. zapytanie dotyczy dokładnie jednej tabeli; `JOIN`, lista tabel w `FROM`, podzapytania, `UNION`, `FOREIGN KEY` i `REFERENCES` zwracają błąd `unsupported`;
3. indeksy wtórne są jednokolumnowe i nieunikalne.

## Uzasadnienie

- Encja z unikalnym kluczem jest naturalnym zakresem atomowości, a transakcje nie powinny obejmować wielu encji [C7].
- Zapytanie o jedną tabelę ma jeden plan dostępu widoczny w `EXPLAIN`; zapytania po kluczu lub indeksie z limitem mają koszt niezależny od rozmiaru bazy [M2].
- Brak więzów między tabelami upraszcza replikację – trudność replikacji leży w więzach (klucze obce, kaskady), nie w języku zapytań [L8].
- Przemysłowym odpowiednikiem jest CQL w Cassandrze [M1].

## Odrzucone alternatywy

- **Pełny SQL lub zgodność z PostgreSQL/SQLite** – moduł urósłby kilkanaście razy (PGlite, Zeta-Lite: ok. 3 MB gzip); tę niszę zajmują inne silniki.
- **Model dokumentowy (JSON CRDT)** – brak SQL, inna klasa systemów.

## Konsekwencje

- Relacje wiele-do-wielu składa aplikacja; brak złączeń to najczęstszy zarzut wobec takich modeli ([M1], [D4]).
- „Przewidywalne wykonanie” znaczy: plan jest zawsze jednotabelowy i widoczny; ograniczony koszt mają tylko odczyty po kluczu lub indeksie z limitem – pełny skan i `GROUP BY` rosną z tabelą (todo 0.1).
- To, co ma zmieniać się atomowo między replikami, modeluje się jako jeden wiersz (ADR-0018).
- Złączenia tylko do odczytu po kluczu głównym (dawne future 5.2, poza zakresem według ADR-0030) nie psułyby przewidywalności ani replikacji [M2], [L8], ale zmieniają tę decyzję – ich wprowadzenie wymaga nowego ADR.

## Źródła

[C7], [M1], [M2], [L8], [D4]; [flatsql-core/src/sql/README.md §1](../../crates/flatsql-core/src/sql/README.md).
