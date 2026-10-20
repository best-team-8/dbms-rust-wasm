# ADR-0027: Struktura tabel w kodzie aplikacji, między wersjami tylko dokładanie

| | |
|---|---|
| Status | przyjęta |
| Data | 2026-10-21 |
| Rodzaj | architektura |
| Powiązane | ADR-0004, ADR-0014, ADR-0016 (zastępuje zdanie o zmianach schematu w pkt 1), ADR-0023; todo 1.7; future 2.6, 6.1 |

## Kontekst

W prototypie struktura tabel powstaje z instrukcji `CREATE TABLE` / `CREATE INDEX` wykonywanych w kodzie aplikacji (np. [web/app.js](../../web/app.js)), a zmiany struktury trafiają do outboksu i są rozsyłane jak dane (ADR-0016 pkt 1). Skutki:

- dowolne urządzenie może zmienić strukturę wszystkim w przestrzeni – `DROP TABLE` z jednego urządzenia usuwa tabelę na pozostałych;
- urządzenia nigdy nie aktualizują aplikacji jednocześnie: pracownik w terenie bywa tygodniami offline ze starą wersją, Service Worker trzyma starą wersję strony do przeładowania, telefon jest zaktualizowany, a laptop nie. Dwie wersje z różną definicją tej samej tabeli kończą się konfliktem, a wiersze „obcej” wersji są odrzucane ([literature-comparison.md §4.6](../references/literature-comparison.md#46-ewolucja-schematu), todo 1.7).

## Decyzja

1. **Struktura żyje w kodzie aplikacji, z numerami wersji.** Aplikacja deklaruje kolejne kroki budowy bazy (wersja 1, 2, …); lokalna baza pamięta, do której wersji doszła, i przy otwarciu wykonuje tylko brakujące kroki.
2. **Polecenia zmiany struktury nie są rozsyłane do innych replik jako polecenia do wykonania.** Każda replika buduje strukturę ze swojej wersji aplikacji; synchronizacja przenosi wiersze.
3. **Między wersjami wolno tylko dokładać:** nowe tabele, nowe kolumny dopuszczające `NULL`, nowe indeksy. Nie wolno usuwać ani zmieniać nazw tabel i kolumn ani zmieniać typów. Zmiana nazwy to dodanie nowej kolumny i porzucenie starej.
4. **Różne wersje współistnieją w jednej przestrzeni.** Wiersz z kolumną nieznaną starszej wersji jest przez nią przechowywany bez zmian (nie odrzucany i nie obcinany); wiersz bez kolumny dodanej w nowszej wersji ma w niej `NULL`.

## Uzasadnienie

- **Zgodność wstecz i w przód.** Kleppmann ([P1] rozdz. 4) opisuje ewolucję formatów danych przy współistnieniu starego i nowego kodu: bezpieczne jest dodawanie pól opcjonalnych, a usuwanie i zmiana znaczenia pól łamie zgodność – tak działają Protocol Buffers i Avro. Zasada „tylko dokładanie” przenosi to na tabele.
- **Problem znany z literatury local-first.** [L1] wskazuje brak autorytatywnego schematu jako problem otwarty; ideał 5 (długowieczność danych) jest dziś oceniony jako spełniony częściowo z powodu braku polityki zgodności wersji ([literature-comparison.md §3](../references/literature-comparison.md#3-ideały-local-first)).
- **Praktyka.** Dexie (`db.version(n).stores(...)` z funkcją migracji) i PowerSync (struktura lokalnej bazy w kodzie aplikacji klienckiej) trzymają strukturę w kodzie aplikacji, a synchronizują dane.
- **Zastosowania.** Zbieranie danych w terenie (CHT, DHIS2), aplikacje z pracą offline i gry na kilku urządzeniach ([real-world-usage.md](../research/real-world-usage.md)) wymagają, żeby dane ze starszej wersji aplikacji nie były odrzucane.
- **Przechowywanie nieznanych kolumn (pkt 4)** jest konieczne przy LWW per wiersz (ADR-0015): gdyby starsza wersja obcinała nieznaną kolumnę, jej `UPDATE` innej kolumny wysłałby wiersz bez tej wartości i skasował ją na wszystkich replikach.

## Odrzucone alternatywy

- **Struktura synchronizowana jak dane (stan prototypu)** – każde urządzenie może zmienić strukturę wszystkim; różne wersje aplikacji kończą się konfliktem.
- **Szablon struktury w konfiguracji serwera jako jedyne źródło** – struktura i tak musi odpowiadać kodowi aplikacji, a starsza wersja dostałaby tabele, których jej kod nie zna; działa tylko w trybie z silnikiem (ADR-0023).
- **Bez stałej struktury (wiersze jako dokumenty JSON)** – sprzeczne z typowanymi kolumnami (ADR-0014) i dialektem SQL (ADR-0004).
- **Tłumaczenie danych między wersjami** (soczewki Cambria [L26]) – pełna swoboda zmian kosztem dużej złożoności; poziom projektu badawczego.

## Konsekwencje

- Silnik potrzebuje `ALTER TABLE … ADD COLUMN`, przechowywania wartości nieznanych kolumn w wierszu i API wersji w `FlatDB.open` (future 6.1).
- Konflikt schematu (todo 1.7) zostaje tylko dla zmian niedozwolonych (inny typ, inny klucz główny); różnica „jedna wersja ma więcej kolumn” przestaje być konfliktem.
- Serwer odrzuca w paczkach polecenia `DROP TABLE` / `DROP INDEX` od klientów (future 2.6).
- **Do ustalenia w 6.1:** skąd serwer w trybie z silnikiem zna strukturę przestrzeni. Kandydaci: deklaracja struktury dołączana do paczek i scalana jako suma tabel i kolumn (bez konfliktów dzięki zasadzie dokładania – zbiór, który tylko rośnie) albo szablon w konfiguracji przestrzeni.
- Porzucone kolumny zostają w bazie na zawsze.

## Źródła

[P1] rozdz. 4, [L1], [L26]; [literature-comparison.md §4.6](../references/literature-comparison.md#46-ewolucja-schematu); [real-world-usage.md](../research/real-world-usage.md); dokumentacja Dexie (wersje schematu) i PowerSync (schemat po stronie klienta).
