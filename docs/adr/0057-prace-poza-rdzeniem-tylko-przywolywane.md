# ADR-0057: Z prac spoza rdzenia zostają tylko przywoływane w dokumentach projektu

| | |
|---|---|
| Status | przyjęta |
| Data | 2026-10-29 |
| Rodzaj | proces |
| Powiązane | ADR-0021, ADR-0031, ADR-0052 (zastępuje odrzucenie usunięcia katalogu `_poza-rdzeniem/`); zlecenie autora przy przygotowaniu pracy do oddania, bez zadania w todo.md |

## Kontekst

Po czterech rewizjach rdzenia w `docs/references/papers/_poza-rdzeniem/` leżało 55 plików PDF prac wyłączonych z rdzenia. ADR-0052 odrzuciło usunięcie katalogu, bo prace dało się przywrócić do rdzenia. Po Benchmarku 2 i testach poprawności skład rdzenia jest ustalony, a tekst pracy dyplomowej cytuje dokładnie 28 prac rdzenia i 25 źródeł nierecenzowanych z §12 indeksu. Większości plików spoza rdzenia nie przywołuje już żaden dokument projektu, więc katalog utrudnia znalezienie tych kilku prac, które są podstawą zadań po pracy.

## Decyzja

1. W `_poza-rdzeniem/` zostaje 18 plików prac, które przywołuje co najmniej jeden śledzony dokument: ADR, `todo.md`, `future/todo.md`, `future/README.md`, dokumenty badań albo `references/literature.md`. Są to: Cimbiosys [C11], YCSB [B5], Bailis i in. (HAT) [C8], SiloR [S7], Satyanarayanan [E2], Shi i in. [E1], Hall i Ramachandran [E3], Antidote SQL [L8], Faasm [E5], Yu i Ignat [L24], Haffner i Dittrich [D3], Cambria [L26], Kuessner i in. [L11], Meyer [L14], Stonebraker i Pavlo [M1], Niinikoski [L20], Merkle-CRDT [L13], Kleppmann i Howard [L12].
2. Pozostałe 37 plików usunięto. Wzmianka w ADR tylko o tym, że praca nie wraca do rdzenia (Lamport [C1] w ADR-0031), nie zatrzymuje pliku.
3. Identyfikatory usuniętych prac nie są używane ponownie, jak dotąd.

## Uzasadnienie

- Każdą z 55 prac sprawdzono wyszukiwaniem nazwiska autora, nazwy systemu i dawnego identyfikatora w śledzonych plikach Markdown, z pominięciem samego indeksu i opisu metody wyszukiwania, które wymieniają wszystkie pozycje.
- Pliki są poza repozytorium (`.git/info/exclude`), więc usunięcie nie zmienia historii ani załącznika do pracy, a jedynie porządkuje kopię lokalną. Opis bibliograficzny każdej pracy zostaje w historii indeksu.
- Rdzeń 28 prac spełnia wymóg co najmniej 15 publikacji z zapasem (ADR-0052), więc przywracanie usuniętych prac do rdzenia nie jest planowane.

## Odrzucone alternatywy

- **Zachowanie całego katalogu** (ADR-0052) – przy ustalonym rdzeniu 37 plików nie ma żadnego zastosowania.
- **Usunięcie całego katalogu** – 18 prac jest podstawą zadań w `future/todo.md` albo argumentem w ADR; trzeba by je pobierać ponownie przy pracy nad tymi zadaniami.

## Konsekwencje

- `_poza-rdzeniem/` ma 18 plików; [search-methodology.md §4.2](../references/search-methodology.md#42-rewizja-4--zastosowanie-i-wynik-weryfikacji) i opis katalogu w [indeksie](../references/README.md) podają nową liczbę.
- ADR-0052 ma status „przyjęta; odrzucenie usunięcia katalogu `_poza-rdzeniem/` zastąpione przez ADR-0057”.
- Praca nowa albo przywoływana ponownie trafia do katalogu według kryteriów ADR-0021 i ADR-0052.

## Źródła

[references/README.md](../references/README.md), [search-methodology.md §4](../references/search-methodology.md#4-dobór-rdzenia), [future/todo.md](../future/todo.md), [literature.md](../references/literature.md).
