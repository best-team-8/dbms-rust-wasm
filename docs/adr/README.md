# Decyzje projektowe

Każdy plik opisuje jedną obowiązującą decyzję: co postanowiono, dlaczego i jakie warianty odrzucono. Gdy decyzja się zmienia, poprawia się jej plik i dopisuje linijkę w sekcji „Historia”. Pełną historię zmian ma git.

Decyzje 0001–0007 to założenia projektu. Zmiana któregoś z nich zmienia, czym Flat-SQL jest.

## Spis

| Nr | Decyzja | Status |
|---|---|---|
| | **Założenia** | |
| [0001](0001-zapytania-lokalne.md) | Zapytania wykonywane lokalnie, bez serwera | obowiązuje |
| [0002](0002-local-first-z-serwerem.md) | Praca offline, synchronizacja w tle przez serwer | obowiązuje |
| [0003](0003-dane-domyslnie-lokalne.md) | Dane domyślnie lokalne | obowiązuje |
| [0004](0004-dialekt-bez-zlaczen.md) | Dialekt Flat-SQL: jedna tabela w zapytaniu, obowiązkowy klucz, bez złączeń | obowiązuje |
| | **Rdzeń i platforma** | |
| [0005](0005-rdzen-rust-bez-io.md) | Jeden rdzeń w Rust, bez operacji wejścia/wyjścia i bez zależności | obowiązuje |
| [0006](0006-dane-w-pamieci.md) | Cała baza w pamięci, moduł 32-bitowy | obowiązuje |
| [0007](0007-watek-glowny-domyslnie.md) | Silnik w głównym wątku strony, Web Worker jako opcja | obowiązuje |
| [0008](0008-magazyn-domyslny.md) | IndexedDB jako magazyn domyślny w przeglądarce, jawny wybór poza nią | obowiązuje |
| [0009](0009-ustawienia-kompilacji.md) | Kompilacja nastawiona na szybkość, bez zmian względem standardu | obowiązuje |
| | **Zapytania i wykonanie** | |
| [0010](0010-system-typow.md) | Cztery typy, konwersja tylko bez utraty danych, pełny porządek wartości | obowiązuje |
| [0011](0011-transakcje-szeregowe.md) | Transakcje wykonywane po kolei, z dziennikiem wycofania | obowiązuje |
| [0012](0012-plan-kompilowany-w-instrukcji.md) | Plan i warunki kompilowane raz, przy przygotowaniu instrukcji | obowiązuje |
| [0013](0013-sciezka-zapisu.md) | UPDATE w miejscu, jeden odczyt zegara na instrukcję, wiele wierszy jednym wywołaniem | obowiązuje |
| [0014](0014-krotkie-teksty-w-wartosci.md) | Krótkie teksty przechowywane w samej wartości | obowiązuje |
| [0015](0015-wynik-binarny-dla-js.md) | Wynik przekazywany do JavaScriptu w jednym buforze binarnym | obowiązuje |
| | **Trwałość** | |
| [0016](0016-wal-i-snapshot.md) | Trwałość przez dziennik zapisu i snapshot we własnym, przenośnym formacie | obowiązuje |
| [0017](0017-zatwierdzanie-grupowe.md) | Zatwierdzenia z jednej chwili zapisywane na dysk jednym zapisem | obowiązuje |
| [0018](0018-checkpoint.md) | Checkpoint kodowany krokami, a na plikach z utrwaleniem katalogu | obowiązuje |
| | **Synchronizacja** | |
| [0019](0019-ostatni-zapis-wygrywa.md) | Konflikty rozstrzyga „ostatni zapis wygrywa” dla całego wiersza | obowiązuje |
| [0020](0020-granica-dryfu-zegara.md) | Granica dryfu zegara: odrzucenie, wstrzymanie synchronizacji i nowe znaczniki | obowiązuje |
| [0021](0021-kolejka-zmian.md) | Kolejka zmian do wysłania odbudowywana z WAL i prowadzona od włączenia synchronizacji | obowiązuje |
| [0022](0022-protokol-synchronizacji.md) | Protokół wysyłania i pobierania zmian z kursorem nadawanym przez serwer | obowiązuje |
| [0023](0023-struktura-tabel-w-kodzie.md) | Struktura tabel w kodzie aplikacji; dziś jawne konflikty schematu | częściowo proponowana |
| [0024](0024-serwer-synchronizacji.md) | Docelowy serwer synchronizacji: dwa tryby, za bramką backendu, reguły w backendzie | proponowana |
| [0025](0025-galezie-bazy.md) | Pamięć agentów przez gałęzie bazy ze scalaniem warunkowym | proponowana |
| | **Badania** | |
| [0026](0026-metodyka-pomiarow.md) | Metodyka pomiarów: powtórzenia procesu, sumy kontrolne, grupy warunków, dwie wersje silnika | obowiązuje |

## Zasady

- Nowa decyzja dostaje kolejny numer i plik `NNNN-krotki-opis.md`.
- Decyzja „proponowana” opisuje kierunek, którego kod jeszcze nie realizuje.
- Gdy decyzja się zmienia, poprawia się jej treść tak, żeby opisywała stan obecny, i dopisuje linijkę w „Historii”: data, co się zmieniło i dlaczego.
- Decyzja, która przestała obowiązywać, jest usuwana. Gdy przeczy jej inna, informację przenosi się do „Odrzuconych wariantów” tej drugiej.
- Literaturę cytuje się jako „Autor (rok)” według [spisu literatury](../references/README.md).

## Wzór

```markdown
# NNNN. Decyzja jako wybór

- Status: obowiązuje | proponowana
- Ostatnia zmiana: RRRR-MM-DD

## Kontekst
Jaki problem wymagał decyzji.

## Decyzja
Co postanowiono.

## Uzasadnienie
Dlaczego: pomiary, literatura, praktyka innych projektów.

## Odrzucone warianty
Co rozważano i dlaczego nie.

## Konsekwencje
Co z tego wynika, dobrego i złego.

## Historia
- RRRR-MM-DD: co się zmieniło i dlaczego.
```
