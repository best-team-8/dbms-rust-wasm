# 0010. Cztery typy, konwersja tylko bez utraty danych, pełny porządek wartości

- Status: obowiązuje
- Ostatnia zmiana: 2026-10-06

## Kontekst

Dialekt ma być prosty (decyzja 0004). Wartości są kluczami w drzewach tabel i indeksów, więc muszą dawać się zawsze uporządkować. Muszą też kodować się identycznie na wszystkich urządzeniach.

## Decyzja

1. Typy: `INT` (64-bitowa liczba całkowita), `FLOAT` (64-bitowa zmiennoprzecinkowa), `TEXT` (UTF-8), `BOOL`, a w każdej kolumnie także `NULL`. Popularne aliasy (`INTEGER`, `VARCHAR`, `DECIMAL`…) są przyjmowane, a długość i precyzja ignorowane.
2. Przy zapisie wartość jest zamieniana na typ kolumny tylko bez utraty danych: `INT` na `FLOAT` zawsze, `FLOAT` na `INT` tylko dla liczb całkowitych, `0` i `1` na `BOOL`. Tekst w kolumnie liczbowej to błąd.
3. Wartości mają pełny porządek: liczby porównuje się po wartości niezależnie od typu, a NaN jest na końcu. Dzięki temu każda wartość może być kluczem.
4. Porównanie tekstu z liczbą to błąd, a nie ukryta konwersja.

## Uzasadnienie

- Pełny porządek jest warunkiem użycia drzewa uporządkowanego dla kluczy i indeksów.
- Jawne błędy typów chronią dane, które synchronizacja rozsyła na inne urządzenia. Błąd na jednym urządzeniu nie rozlewa się po cichu na inne.

## Odrzucone warianty

- Typ wartości niezależny od kolumny, jak w SQLite. Utrudnia porządek i porównania między urządzeniami.
- Typy z precyzją, daty i dane binarne. Poza zakresem obecnej wersji.

## Konsekwencje

- Wyniki funkcji mają przewidywalne typy: `SUM` daje liczbę całkowitą, gdy wszystkie wartości są całkowite, a `AVG` zawsze liczbę zmiennoprzecinkową.
- NaN ma wiele postaci bitowych. Kodek zapisuje każdy NaN jednym wzorcem, żeby te same dane dawały te same bajty na każdym urządzeniu (Haas i in., 2017).

## Historia

- 2026-10-06: system typów pierwszej wersji silnika.
- 2026-10-22: jeden zapis bitowy NaN w snapshocie, WAL i paczkach zmian.
