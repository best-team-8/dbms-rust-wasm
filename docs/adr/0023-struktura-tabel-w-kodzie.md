# 0023. Struktura tabel w kodzie aplikacji; dziś jawne konflikty schematu

- Status: punkt 1 obowiązuje, punkty 2–4 proponowane
- Ostatnia zmiana: 2026-10-23

## Kontekst

Dziś struktura tabel powstaje z `CREATE TABLE` w kodzie aplikacji i jest rozsyłana do innych urządzeń jak dane. Każde urządzenie może więc zmienić strukturę wszystkim, na przykład usunąć tabelę. Urządzenia nigdy nie aktualizują aplikacji jednocześnie: pracownik w terenie bywa tygodniami offline ze starą wersją. Dwie wersje z różną definicją tej samej tabeli kończyły się tym, że wiersze jednej były po cichu odrzucane przez drugą.

## Decyzja

1. Obecnie: definicja tabeli z innego urządzenia jest porównywana z lokalną. Gdy się różni, powstaje jeden konflikt schematu na tabelę z obiema definicjami, a wiersze tej tabeli są odrzucane. Konflikt dostaje rodzaj: „dokładanie” (jedna definicja ma tylko dodatkowe kolumny dopuszczające `NULL`) albo „niezgodność”. `SyncClient` zgłasza zdarzenie `conflict`.
2. Docelowo struktura żyje w kodzie aplikacji z numerami wersji. Baza pamięta, do której wersji doszła, i przy otwarciu wykonuje brakujące kroki. Polecenia zmiany struktury nie są rozsyłane.
3. Między wersjami wolno tylko dokładać: nowe tabele, nowe kolumny dopuszczające `NULL`, nowe indeksy. Nie wolno usuwać, zmieniać nazw ani typów.
4. Starsza wersja przechowuje wiersz z nieznaną kolumną bez zmian. Wtedy konfliktem zostaje tylko niezgodność.

## Uzasadnienie

- Przy współistnieniu starego i nowego kodu bezpieczne jest tylko dodawanie pól opcjonalnych (Kleppmann, 2017, rozdz. 4). Brak jednego obowiązującego schematu to otwarty problem aplikacji local-first (Kleppmann i in., 2019).
- Dexie i PowerSync też trzymają strukturę w kodzie aplikacji, a synchronizują dane.
- Przechowywanie nieznanych kolumn jest konieczne przy regule „ostatni zapis wygrywa” dla wiersza. Gdyby starsza wersja obcinała nieznaną kolumnę, jej zmiana innej kolumny skasowałaby tę wartość na wszystkich urządzeniach. Z tego samego powodu dziś wiersze innej definicji są odrzucane, a nie uzupełniane `NULL`.

## Odrzucone warianty

- Struktura rozsyłana jak dane (stan prototypu bez konfliktów).
- Struktura tylko w konfiguracji serwera. Musi i tak odpowiadać kodowi aplikacji.
- Tłumaczenie danych między wersjami (Litt i in., 2021). Duża złożoność.

## Konsekwencje

- Urządzenie w konflikcie pracuje dalej lokalnie, ale jego wiersze tej tabeli nie docierają do innych, dopóki aplikacja nie rozwiąże konfliktu.
- Punkty 2–4 wymagają `ALTER TABLE … ADD COLUMN`, przechowywania nieznanych kolumn i API wersji w `FlatDB.open`. Porzucone kolumny zostają w bazie na zawsze.

## Historia

- 2026-10-21: przyjęto kierunek „struktura w kodzie, tylko dokładanie”.
- 2026-10-23: jawne konflikty schematu jako krok przed wersjami struktury.
