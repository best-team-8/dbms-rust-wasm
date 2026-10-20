# ADR-0023: Konfigurowalne tryby serwera synchronizacji

| | |
|---|---|
| Status | przyjęta |
| Data | 2026-10-20 |
| Rodzaj | architektura |
| Powiązane | ADR-0002, ADR-0003, ADR-0015, ADR-0017 (zastępuje jej pkt 2); future 2.1, 3.1, 4.3, 7.2, 10.1, 12.4 |

## Kontekst

Prototypowy serwer ([server/server.js](../../server/server.js)) trzyma jedną bazę Flat-SQL wspólną dla wszystkich klientów, z tym samym silnikiem co przeglądarka i z logiem paczek tylko w pamięci (ADR-0017 pkt 2). Każdy klient dostaje zmiany wszystkich, a serwer przyjmuje każdą poprawną zmianę.

Analiza wdrożeń ([real-world-usage.md](../research/real-world-usage.md)) pokazała dwie grupy zastosowań o sprzecznych potrzebach:

- **dane jednego użytkownika lub przestrzeni** – aplikacje osobiste (Actual Budget, Dexie Cloud), zapisy gier w chmurze, szyfrowane pokoje (Excalidraw): serwer ma tylko przechować i rozesłać zmiany, najlepiej tanio i bez wglądu w treść;
- **dane wspólne i reguły serwera** – kasy (zatwierdzanie zamówień, ceny), pamięć podręczna danych serwera w SaaS (Notion, Linear), zbieranie danych w terenie z podziałem na regiony (CHT, DHIS2), rankingi w grach: serwer musi rozumieć wiersze, sprawdzać je i być źródłem prawdy (ADR-0002 pkt 3).

Jeden sposób działania serwera nie obsłuży obu grup.

## Decyzja

Serwer synchronizacji to jeden program z dwoma trybami pracy, wybieranymi **dla każdej przestrzeni** (użytkownika, zespołu, rozgrywki) osobno:

1. **Magazyn paczek – tryb domyślny.** Serwer dopisuje przyjęte paczki do trwałego logu przestrzeni i oddaje je od kursora; nie uruchamia silnika i nie czyta treści paczek (poza nagłówkiem, sumą kontrolną i znacznikami czasu). Scalanie wykonują urządzenia.
2. **Folder z silnikiem – opcja.** Przestrzeń ma na serwerze bazę Flat-SQL (snapshot + WAL we wspólnym formacie plików, ADR-0011), ładowaną do pamięci na żądanie i zwalnianą po okresie bezczynności. Serwer stosuje paczki w silniku i może je walidować, odrzucać i kompensować, prowadzić tabele, filtrować dane dla klienta i wykonywać zapytania.

**Jeden zapisujący** nie jest trzecim trybem, tylko właściwością tabeli: tabelę zmienia wyłącznie serwer (future 2.1), a na węzłach edge zapis trafia do bazy głównej (future 7.2). Wymaga trybu z silnikiem.

Tryb z silnikiem jest nadbudową magazynu paczek: oba korzystają z tych samych przestrzeni, trwałego logu, kursorów i epok. Protokół `/sync/push` i `/sync/pull` (ADR-0017 pkt 1, 3, 4) jest w obu trybach ten sam, więc klient nie wie, w jakim trybie działa przestrzeń.

## Uzasadnienie

- **Dwie grupy zastosowań, dwa tryby.** Replikacja dwuwarstwowa Graya i in. [C9] rozdziela dane, które urządzenie zmienia samodzielnie, od danych z kopią główną zatwierdzającą zmiany. Magazyn paczek obsługuje pierwszą grupę, folder z silnikiem – drugą. Wymóg z ADR-0002 pkt 3 (serwer jako źródło prawdy dla danych wspólnych) spełnia tryb z silnikiem.
- **Magazyn paczek jako domyślny.** Najmocniej potwierdzone zastosowania z danymi jednej osoby (aplikacje osobiste, zapisy gier) nie potrzebują reguł serwera. Przestrzeń bez aktywnych klientów nie zajmuje pamięci, więc serwer skaluje się jak przechowywanie plików. Tylko w tym trybie możliwe jest szyfrowanie end-to-end (future 4.3), bo do rozstrzygnięcia LWW wystarczają jawne znaczniki [L11], [L1] – co przybliża spełnienie „Privacy by Design” (ADR-0003).
- **Silnik jako opcja.** Filtrowanie danych według treści wiersza (replikacja częściowa, [C11]), walidacja i zamiary zatwierdzane przez serwer ([C9], [C3]) wymagają rozumienia wierszy – serwer z tym samym silnikiem nie wymaga osobnej implementacji scalania (uzasadnienie z ADR-0017).
- **Wybór per przestrzeń.** Jedna aplikacja potrzebuje często obu trybów: w grze zapis gracza to magazyn paczek, a ranking – przestrzeń z silnikiem.
- **Mało dodatkowej pracy.** Prototyp już ma serwer z silnikiem; tryb z silnikiem to jego przebudowa na bazę per przestrzeń, a wspólny fundament (przestrzenie, trwały log) i tak jest potrzebny (future 12.4).

## Odrzucone alternatywy

- **Tylko magazyn paczek** – serwer niczego nie sprawdzi; odpadają kasy, pamięć podręczna danych serwera, teren z podziałem na regiony i rankingi, czyli część najmocniej potwierdzonych zastosowań.
- **Tylko silnik na serwerze (rozwinięcie prototypu, dawne future 3.1)** – każda aktywna przestrzeń to baza w pamięci; nie da się szyfrować danych end-to-end; nadmiarowe dla danych jednej osoby.
- **Tryb wybierany dla całego serwera** – jedna aplikacja musiałaby uruchamiać dwa serwery dla danych prywatnych i wspólnych.
- **Dwa osobne programy serwera** – zdublowany fundament (przestrzenie, log, kursory, uwierzytelnianie) przy identycznym protokole.

## Konsekwencje

- Pkt 2 ADR-0017 („serwer utrzymuje własną replikę z tym samym silnikiem”) przestaje obowiązywać; obowiązuje tylko w trybie z silnikiem.
- Kolejność prac: najpierw wspólny fundament – przestrzenie i trwały log per przestrzeń (future 12.4 jako pierwszy krok), potem magazyn paczek, potem silnik per przestrzeń (future 3.1, 10.1), a na końcu reguły i tabele serwera (future 2.1–2.4) oraz tryb jednego zapisującego (future 7.2).
- Każda przestrzeń ma zapisany tryb; zmiana trybu istniejącej przestrzeni wymaga odtworzenia bazy z logu paczek (z magazynu na silnik) albo eksportu stanu (z silnika na magazyn).
- W magazynie paczek kompaktowanie logu bez silnika wymaga, żeby klient wysłał punkt startowy (pełny stan); w przestrzeniach współdzielonych bezpieczniej kompaktuje tryb z silnikiem.
- Uwierzytelnianie i wybór przestrzeni na podstawie tożsamości pozostają osobną, nierozstrzygniętą decyzją (future 4.1).
- Do dokumentacji: [architecture.md §5](../architecture.md) i opis serwera w README po wdrożeniu trybów.

## Źródła

[C3], [C9], [C11], [L1], [L11]; [real-world-usage.md](../research/real-world-usage.md) §1–4.
