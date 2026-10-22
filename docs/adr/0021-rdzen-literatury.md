# ADR-0021: Rdzeń literatury i kryteria doboru

| | |
|---|---|
| Status | przyjęta; kryterium włączenia i skład rdzenia zastąpione przez [ADR-0031](0031-rdzen-literatury-wedlug-cytowan-w-pracy.md) |
| Data | 2026-10-20 |
| Rodzaj | proces |
| Powiązane | ADR-0022; [search-methodology.md](../references/search-methodology.md) |

## Kontekst

Pierwsza runda przeglądu (18.10.2026) zebrała 77 publikacji według obszarów technologicznych (WebAssembly, CRDT, edge). Audyt pokazał nadmiar prac ogólnych (środowiska WASM, formalna weryfikacja CRDT, serverless na edge) i luki tam, gdzie leżą atuty i plany projektu: praca offline z trwałym logiem, replikacja częściowa, przewidywalność ograniczonego języka zapytań, checkpointy stanu w pamięci.

## Decyzja

Literatura projektu to **rdzeń 38 prac** ([references/README.md](../references/README.md)): 30 z pierwszej rundy i 8 dodanych w lukach ([C9], [C10], [C11], [L27], [L28], [M2], [M3], [S10]). Pozycja jest w rdzeniu, jeśli uzasadnia decyzję lub atut projektu, jest punktem odniesienia dla hipotezy H1–H6 albo porównywanego systemu lub jest podstawą zadania – i nie dubluje innej pozycji rdzenia. Identyfikatory są stałe; 46 pozostałych prac przeniesiono do `papers/_poza-rdzeniem/` z uzasadnieniem.

## Uzasadnienie

- Każda praca w rdzeniu odpowiada na pytanie „dlaczego Flat-SQL robi to tak”, a nie „co jeszcze istnieje w tej dziedzinie”.
- Kryteria wyłączenia: duplikat, kierunek sprzeczny z założeniami (np. wątki wymagające COOP/COEP – ADR-0013, obraz pamięci – ADR-0011), zakres poza pracą inżynierską (formalna weryfikacja), kontekst bez wpływu na decyzje, słabsze źródło tej samej tezy.
- Liczba ok. 30 była orientacyjna – prace dodano, bo wypełniają luki w obszarach atutów.

## Odrzucone alternatywy

- **Zachowanie wszystkich 77 prac** – rozmywa uzasadnienia; trudno wskazać, która praca za czym stoi.
- **Przegląd systematyczny (PRISMA)** – wymaga płatnych baz bibliograficznych; opisany jako sposób uzupełnienia ([search-methodology.md §6](../references/search-methodology.md)).

## Konsekwencje

- Nowa praca trafia do rdzenia tylko wtedy, gdy spełnia kryterium; wtedy dostaje kolejny identyfikator w swojej kategorii.
- Trzy obszary pozostają bez badań recenzowanych: wydajność IndexedDB i OPFS, koszt granicy JS↔WASM w silnikach baz, koordynacja wielu kart.

## Źródła

[search-methodology.md §3–§4](../references/search-methodology.md), [references/README.md §13](../references/README.md).
