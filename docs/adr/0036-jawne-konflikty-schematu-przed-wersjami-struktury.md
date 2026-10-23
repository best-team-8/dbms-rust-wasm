# ADR-0036: Jawne konflikty schematu przed wersjami struktury

| | |
|---|---|
| Status | proponowana |
| Data | 2026-10-23 |
| Rodzaj | architektura |
| Powiązane | ADR-0015, ADR-0016, ADR-0027 (docelowy model wersji); todo 1.7; future 2.3, 2.6, 6.1 |

## Kontekst

Struktura tabel jest dziś rozsyłana jak dane (ADR-0016 pkt 1, docelowo zmienia to ADR-0027). Gdy replika dostawała `CREATE TABLE` tabeli, którą miała już z inną definicją, ignorowała go jak duplikat, a potem odrzucała każdy wiersz tej tabeli osobnym błędem „has N columns but M values”. Klient nie czytał odpowiedzi `/sync/push`, więc dla aplikacji było to ciche odrzucenie. ADR-0027 zawęża docelowo konflikt do zmian niedozwolonych (inny typ, inny klucz główny), bo różnicę „jedna wersja ma więcej kolumn” rozwiąże przechowywanie nieznanych kolumn i wersje struktury (future 6.1). Zadanie 1.7 wymagało ustalenia, co da się zrobić przed 6.1.

## Decyzja

1. `CREATE TABLE` innej repliki dla istniejącej tabeli jest porównywany z lokalną definicją (nazwy kolumn bez względu na wielkość liter, typy, `NOT NULL`, klucz główny, kolejność). Ta sama definicja – zmiana pominięta; inna – **konflikt schematu**: zmiana nie jest stosowana, a raport (`ApplyReport.conflicts`) dostaje jedną pozycję na tabelę z obiema definicjami i liczbą odrzuconych zmian wierszy.
2. Konflikt jest klasyfikowany według ADR-0027: **dokładanie** (`additive`: jedna definicja dodaje na końcu tylko kolumny dopuszczające `NULL`) albo **niezgodność** (typ, klucz główny, kolejność, nazwy, kolumna `NOT NULL`). Przed 6.1 oba rodzaje są konfliktem.
3. Wiersze tabeli w konflikcie są odrzucane także wtedy, gdy przypadkiem pasują do lokalnej definicji (w tej samej paczce co definicja). Wiersze, które przychodzą bez definicji i nie pasują, tworzą konflikt z nieznaną drugą definicją.
4. Serwer zwraca konflikty w odpowiedzi `/sync/push` i zapisuje je w dzienniku; `SyncClient` zgłasza zdarzenie `conflict` (kierunek `push` albo `pull`). Odrzucone zmiany opuszczają outbox jak inne odrzucone zmiany: ponowne wysłanie niczego nie zmieni.
5. Indeksy nie są porównywane: różne indeksy nie zmieniają danych.

## Uzasadnienie

- Przyjęcie wierszy innej definicji przed 6.1 nie jest bezpieczne. Wiersz starszej wersji uzupełniony wartościami `NULL` przy LWW per wiersz skasowałby w nowszej wersji wartości kolumn, których starsza nie zna (uzasadnienie pkt 4 ADR-0027), a obcięcie wiersza nowszej wersji zgubiłoby jej dane. Bez przechowywania nieznanych kolumn pozostaje jawne zgłoszenie.
- Klasyfikacja już teraz pokazuje, które konflikty znikną po wprowadzeniu wersji struktury, a które zostaną; [L1] wskazuje brak autorytatywnego schematu jako problem otwarty aplikacji local-first.
- Jedna pozycja na tabelę zamiast błędu na wiersz: test (`crates/flatsql-core/tests/sync.rs`, [tests-results.md §2.2](../research/tests-results.md#22-konflikt-definicji-tabeli)) pokazał na kodzie sprzed zmiany pominięty `CREATE TABLE` i trzy osobne błędy wierszy.

## Odrzucone alternatywy

- **Uzupełnianie brakujących kolumn `NULL` już teraz** – kasuje dane nowszej wersji przy pierwszej zmianie wiersza w starszej (LWW per wiersz).
- **Ponawianie odrzuconych zmian** – konflikt nie zniknie sam, więc wysyłanie trwałoby bez końca.
- **Zastąpienie lokalnej definicji zdalną** – jedno urządzenie zmieniałoby strukturę wszystkim; tego ADR-0027 właśnie zakazuje.

## Konsekwencje

- Urządzenie w konflikcie pracuje dalej lokalnie, ale jego wiersze tej tabeli nie docierają do innych (i odwrotnie), dopóki aplikacja nie rozwiąże konfliktu; zdarzenie `conflict` podaje obie definicje.
- `ApplyReport` i odpowiedź `/sync/push` mają nowe pole `conflicts`; błędy wierszy tabeli w konflikcie nie są już wypisywane pojedynczo.

## Otwarte kwestie

- **Konflikty typu „dokładanie”.** Nieznane: dokładna postać wersji struktury i przechowywania nieznanych kolumn (future 6.1). Przyjęto: każdy taki przypadek jest konfliktem oznaczonym `additive: true`. Rozstrzygnie: future 6.1. Zmieni się: różnica „jedna wersja ma więcej kolumn” przestanie być konfliktem (ADR-0027 pkt 4), a konfliktem zostanie tylko niezgodność.
- **Wiersze bez definicji, które pasują.** Nieznane: definicja tabeli u nadawcy wierszy, które przyszły w osobnej paczce. Przyjęto: wiersz pasujący do lokalnej definicji jest stosowany, niepasujący tworzy konflikt z nieznaną drugą definicją. Rozstrzygnie: wersja struktury w paczkach (future 6.1). Zmieni się: odbiorca rozpozna wiersze z niezgodnej definicji także wtedy, gdy pasują.
- **Co z danymi urządzenia w konflikcie.** Nieznane: czy aplikacje potrzebują automatycznego rozwiązania (np. odrzucenia lokalnej tabeli i pobrania stanu serwera). Przyjęto: tylko zgłoszenie. Rozstrzygnie: future 6.1 (polecenia struktury nie są rozsyłane) i future 2.3 (kompensacja odrzuconych zmian). Zmieni się: konflikt niezgodny będzie zgłaszany przy otwarciu bazy w nowej wersji, zanim powstaną dane.

## Źródła

[L1], [P1] rozdz. 4; [tests-results.md §2.2](../research/tests-results.md#22-konflikt-definicji-tabeli).
