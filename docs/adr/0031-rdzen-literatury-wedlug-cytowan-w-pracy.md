# ADR-0031: Rdzeń literatury według cytowań w pracy dyplomowej

| | |
|---|---|
| Status | przyjęta; kryterium i skład rdzenia zastąpione przez [ADR-0052](0052-rdzen-literatury-wedlug-zastosowania-i-weryfikacji.md) |
| Data | 2026-10-22 |
| Rodzaj | proces |
| Powiązane | ADR-0021 (zastępuje kryterium włączenia i skład rdzenia), ADR-0029, ADR-0007; zadania todo 1.7, 2.7, future 2.5, 3.2, 4.3, 5.1, 12.7 |

## Kontekst

ADR-0021 przyjęło do rdzenia każdą pracę, która uzasadnia decyzję lub atut projektu, jest punktem odniesienia dla hipotezy albo porównywanego systemu lub jest podstawą zadania – także zadania z [todo_future.md](../todo_future.md). Od tego czasu [todo.md](../todo.md) ograniczono do zakresu pracy inżynierskiej (ADR-0029). Rozszerzenia trafią do pracy tylko jako kierunki rozwoju w podrozdziale 9.5, więc część prac rdzenia nie miałaby w tekście żadnego miejsca. Wykaz literatury według ZR 45/2024 zawiera tylko pozycje wykorzystane, a od 2026 r. co najmniej 15 publikacji bez stron WWW – rdzeń ma być pulą, z której ten wykaz powstaje.

Przy przeglądzie wyszło też, że decyzja ADR-0007 (cel `wasm32`), prezentowana w tabeli 3.2 pracy, opiera się wyłącznie na wpisach z blogów [Q6], a recenzowana praca o koszcie memory64 leżała poza rdzeniem.

## Decyzja

Praca jest w rdzeniu, jeśli będzie cytowana w tekście pracy dyplomowej – ma podrozdział w [mapie rozdziałów](../references/README.md#mapa-rozdziały-pracy-dyplomowej--literatura) – i nie dubluje tam innej pozycji rdzenia. Kierunek rozwoju z todo_future.md liczy się, jeśli trafia do podrozdziału 9.5; samo zadanie w todo_future.md nie wystarcza. Praca, która miałaby w tekście tylko jedno zdanie, a tę tezę daje inna praca rdzenia, też jest poza rdzeniem. Rdzeń liczy 33 prace: [W6] wraca do rdzenia, a [C8], [L8], [L11], [L20], [L26] i [M3] przechodzą poza rdzeń. Stałe identyfikatory, kryteria wyłączenia i zachowanie plików w `papers/_poza-rdzeniem/` z ADR-0021 obowiązują dalej.

## Uzasadnienie

- **[W6] do rdzenia.** Jedyne recenzowane źródło dla ADR-0007: przy memory64 przeglądarki (V8, SpiderMonkey) sprawdzają granice programowo przed każdym dostępem, co w pomiarze autorów na SPEC CPU2017 daje średnio ok. 110% narzutu. Powód wyłączenia z rundy 2 („wniosek wystarcza w [Q6]”) nie uwzględniał, że [Q6] to strony WWW, które nie liczą się do wymaganych publikacji. Cytowana w podrozdziałach 2.1 i 3.5.
- **[L11] poza rdzeń.** Szyfrowanie end-to-end jest w pracy tylko kierunkiem rozwoju (podrozdział 9.5, uzupełnienie Privacy by Design z ADR-0003). Tezę „serwer przechowuje tylko szyfrogram” daje recenzowane [L1] (ideał 6), a [L11] to preprint warsztatowy – kryteria wyłączenia 1 (duplikat) i 5 (słabsze źródło) z ADR-0021.
- **[L26] poza rdzeń.** Problem ewolucji schematu w replikach opisują [L1] (brak autorytatywnego schematu) i [P1] rozdz. 4; soczewki Cambria są odrzuconą alternatywą w ADR-0027, decyzji dla etapu po pracy. Zadanie todo 1.7 zgłasza konflikt jawnie i z tej techniki nie korzysta.
- **[M3] poza rdzeń.** Jedyną rolą była technika dla future 5.1; w kierunkach rozwoju zapytania reaktywne uzasadnia [D4]. Pracę oceniono tylko na podstawie streszczenia, bez pliku.
- **[C8] poza rdzeń.** W pracy miałaby jedną uwagę w podrozdziale 8.5: Monotonic Atomic View jako opcja na później (future 12.7). Atomowość wiersza między replikami uzasadniają [C7] i [L27], a potrzebę koordynacji przy płatnościach i unikalności – [C9] i [P1] rozdz. 9.
- **[L8] poza rdzeń.** W pracy miałaby jedną uwagę w podrozdziale 3.3 (trudność replikacji leży w więzach, nie w złączeniach). Dialekt uzasadniają [C7], [M2] i [M1], aktualizacje przemienne w kierunkach rozwoju – [C9] i [L2]; [L8] to preprint.
- **[L20] poza rdzeń.** Model hybrydowy z serwerem koordynującym (ADR-0002) uzasadniają recenzowane [C9] (replikacja dwuwarstwowa) i [C3]; praca magisterska jest słabszym źródłem tej samej tezy – kryterium 5 z ADR-0021.
- **Pozostałe 29 prac** mają w mapie co najmniej jeden podrozdział z tezą, której nie daje inna praca rdzenia. Najsłabiej związane z tekstem zostają z konkretnego powodu: [C11] – replikacja częściowa, najważniejszy brak dojrzałości według analizy wdrożeń (9.4, 9.5); [B5] – zagrożenia dla trafności (rozkład dostępu w rozdziale 5); [L24] – porównanie z warstwami synchronizacji i LWW per kolumna (2.5, 9.5); [S10] – checkpoint w podrozdziale 2.3, nawet bez todo 2.6; [D5] – porównywany system i cel odczytu po kluczu (preprint, cytowany ostrożnie).

## Odrzucone alternatywy

- **Rdzeń jako zbiór wszystkich podstaw zadań (ADR-0021)** – zostawia prace cytowane tylko w dokumentach roboczych; wykaz literatury trzeba by filtrować dopiero przy pisaniu.
- **Przywrócenie [C1] Lamport (1978)** – HLC w podrozdziale 2.4 wystarczy opisać przez [C2] i [P1] rozdz. 8–9; do przywrócenia, jeśli tekst wyprowadzi HLC z zegarów logicznych.
- **Przywrócenie [S7] SiloR i [E5] Faasm** – odrzucone kierunki z [todo.md §5](../todo.md) opisuje się w podrozdziale 7.7 krótko, przez powód odrzucenia (wątki i COOP/COEP, obraz pamięci), bez omawiania tych systemów.
- **Zastąpienie [L2] przeglądem [L28]** – [L28] obejmuje rejestr LWW, ale [L2] to praca źródłowa CRDT, a formatka WETI zaleca w przeglądzie prace klasyczne obok prac z ostatnich pięciu lat.

## Konsekwencje

- Mapa rozdziałów w [references/README.md](../references/README.md#mapa-rozdziały-pracy-dyplomowej--literatura) jest kryterium rdzenia i zmienia się razem z układem pracy: praca bez podrozdziału wychodzi poza rdzeń, nowa trafia do rdzenia tylko z przypisanym podrozdziałem.
- Dokumenty robocze ([todo_future.md](../todo_future.md) 2.5, 4.3, 5.1 i 12.7, [sync-server.md](../future/components/sync-server.md), [use-cases.md](../use-cases.md), analizy w `references/`) przywołują prace spoza rdzenia opisowo; zapisy ADR-0002, ADR-0003, ADR-0004, ADR-0018, ADR-0021, ADR-0023, ADR-0027 i ADR-0030 zachowują identyfikatory, które rozwiązuje tabela §13 indeksu.
- Rdzeń spełnia wymóg co najmniej 15 publikacji, w tym 5 obcojęzycznych, z zapasem: 33 prace, wszystkie anglojęzyczne poza polskim wydaniem [P1].
- Mapa jest planem, nie dowodem: po napisaniu pracy rdzeń porównuje się z jej wykazem literatury, a prace niecytowane przechodzą poza rdzeń według tej samej reguły.
- ADR-0021 ma status „przyjęta; kryterium włączenia i skład rdzenia zastąpione przez ADR-0031”.

## Źródła

[references/README.md – mapa rozdziałów i §13](../references/README.md), [search-methodology.md §4.1](../references/search-methodology.md#41-rewizja-3--cytowania-w-pracy-dyplomowej); [W6], [L1], [P1] rozdz. 4, [D4].
