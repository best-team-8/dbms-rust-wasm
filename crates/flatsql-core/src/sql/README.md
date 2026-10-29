# sql – język Flat-SQL

Flat-SQL to dialekt SQL zaprojektowany dla silnika działającego po stronie klienta. **Składnia jest SQL-owa, model danych – NoSQL-owy**: każda tabela to niezależna, płaska kolekcja wierszy adresowanych kluczem głównym. Nie ma relacji między tabelami, a każde zapytanie dotyczy dokładnie jednej tabeli. Dzięki temu wykonanie zapytań jest proste i przewidywalne, a replikacja może działać na poziomie pojedynczego klucza.

Moduł `sql/` zamienia tekst zapytania na drzewo składni; dalsze etapy (wiązanie nazw, plan, wykonanie) opisuje [README rdzenia](../../README.md#droga-zapytania). Ten plik jest specyfikacją języka w wersji 0.1.

| Plik | Rola |
|---|---|
| `lexer.rs` | podział tekstu na tokeny: słowa kluczowe, identyfikatory (także w cudzysłowach, z Unicode), literały, operatory i komentarze |
| `parser.rs` | parser rekurencyjny zstępujący z pierwszeństwem operatorów; konstrukcje relacyjne (`JOIN`, `FOREIGN KEY`, `REFERENCES`, podzapytania, `UNION`) kończą się błędem `unsupported` |
| `ast.rs` | drzewo składni: instrukcje, wyrażenia, definicje tabel |
| `mod.rs` | wejście modułu: `parse`, `parse_one`, `param_count` |

## 1. Zasady modelu

1. Każda tabela **musi** mieć `PRIMARY KEY` (jedna kolumna). Klucz jest zawsze `NOT NULL`.
2. Zapytanie odwołuje się do jednej tabeli. `JOIN`, lista tabel w `FROM`, podzapytania, `UNION`, `FOREIGN KEY` i `REFERENCES` zwracają błąd `unsupported`.
3. Indeksy wtórne są jednokolumnowe i nieunikalne.
4. Nazwy tabel, kolumn i indeksów nie rozróżniają wielkości liter; w wynikach zachowywana jest pisownia z definicji. Identyfikatory w `"…"` lub `` `…` `` mogą zawierać spacje i słowa kluczowe.

## 2. Typy danych

| Typ | Aliasy | Wartości |
|---|---|---|
| `INT` | `INTEGER`, `BIGINT`, `SMALLINT`, `TINYINT` | liczby całkowite 64-bitowe |
| `FLOAT` | `REAL`, `DOUBLE`, `NUMERIC`, `DECIMAL` | liczby zmiennoprzecinkowe 64-bitowe |
| `TEXT` | `STRING`, `VARCHAR`, `CHAR` | tekst UTF-8 |
| `BOOL` | `BOOLEAN` | `TRUE`, `FALSE` |

Każda kolumna przyjmuje też `NULL` (chyba że jest `NOT NULL`). Długość/precyzja (`VARCHAR(255)`, `DECIMAL(10,2)`) jest akceptowana i ignorowana.

**Koercja przy zapisie** jest bezstratna: `INT` → `FLOAT` zawsze; `FLOAT` → `INT` tylko dla wartości całkowitych (`2.0`); `0`/`1` → `BOOL`. Inne kombinacje (np. tekst do kolumny `INT`) to błąd `type`.

**Porównania**: liczby porównuje się numerycznie niezależnie od typu (`1 = 1.0`), `BOOL` z liczbą jak `0`/`1`; porównanie tekstu z liczbą jest błędem `type`.

## 3. Instrukcje

### Definicja danych

```sql
CREATE TABLE [IF NOT EXISTS] nazwa (
  kolumna TYP [PRIMARY KEY] [NOT NULL | NULL],
  ...
  [, PRIMARY KEY (kolumna)]
);
DROP TABLE [IF EXISTS] nazwa;

CREATE INDEX [IF NOT EXISTS] nazwa_indeksu ON tabela (kolumna);
DROP INDEX [IF EXISTS] nazwa_indeksu [ON tabela];

SHOW TABLES;            -- tabela, liczba wierszy, kolumn, indeksów, klucz główny
DESCRIBE tabela;        -- kolumny, typy, NULL-owalność, klucze i indeksy (także DESC)
```

### Modyfikacja danych

```sql
INSERT [OR REPLACE | OR IGNORE] INTO tabela [(kolumny)] VALUES (…), (…);
REPLACE INTO tabela [(kolumny)] VALUES (…);      -- = INSERT OR REPLACE
UPDATE tabela SET kolumna = wyrażenie [, …] [WHERE warunek];
DELETE FROM tabela [WHERE warunek];
```

- Pominięte kolumny przyjmują `NULL`.
- Duplikat klucza: `INSERT` → błąd `constraint`; `OR IGNORE` → wiersz pominięty; `OR REPLACE` → wiersz zastąpiony.
- `UPDATE` może zmienić klucz główny (sprawdzana jest unikalność nowego klucza).
- Unikalność klucza głównego jest sprawdzana tylko w lokalnej replice. Dwa urządzenia, które offline wstawią wiersz o tym samym kluczu, po synchronizacji nie dostają błędu: zostaje wiersz z późniejszym znacznikiem czasu (ostatni zapis wygrywa), a drugi znika bez śladu. Tabele zapisywane offline powinny mieć klucze bez kolizji między urządzeniami: losowy identyfikator wygenerowany w aplikacji (UUID, ULID) w kolumnie `TEXT` albo identyfikator urządzenia w kluczu (np. `'tel-a:17'`). Licznik 1, 2, 3 prowadzony osobno na każdym urządzeniu prowadzi do kolizji.
- Wynik zawiera `rowsAffected`.

### Zapytania

```sql
SELECT [DISTINCT] * | wyrażenie [[AS] alias], …
[FROM tabela [alias]]
[WHERE warunek]
[GROUP BY wyrażenie | alias | pozycja, …]
[HAVING warunek]
[ORDER BY wyrażenie | alias | pozycja [ASC | DESC], …]
[LIMIT n [OFFSET m]];
```

- `SELECT` bez `FROM` oblicza wyrażenia (`SELECT 1 + 1`).
- `ORDER BY` może odwoływać się do aliasów i pozycji kolumn wyniku (`ORDER BY 2 DESC`). `NULL` jest najmniejszy (pierwszy przy `ASC`).
- Zapytanie z agregatami bez `GROUP BY` zwraca jeden wiersz (także dla pustej tabeli: `COUNT(*) = 0`, `SUM = NULL`); z `GROUP BY` na pustym zbiorze – zero wierszy.
- W zapytaniu z grupowaniem kolumny spoza `GROUP BY` przyjmują wartość z pierwszego wiersza grupy (jak w SQLite).
- Wynik zawiera `rowsScanned` – liczbę wierszy odwiedzonych przez ścieżkę dostępu (pokazuje, czy użyto indeksu).

### Transakcje

```sql
BEGIN [TRANSACTION];   -- także START TRANSACTION
COMMIT [TRANSACTION];  -- także END
ROLLBACK [TRANSACTION];
```

- Bez `BEGIN` każda instrukcja jest osobną transakcją.
- Każda instrukcja jest atomowa: błąd cofa całą instrukcję (np. wszystkie wiersze wielowierszowego `INSERT`), także wewnątrz jawnej transakcji – transakcja pozostaje otwarta.
- Zmiany transakcji trafiają do WAL i do synchronizacji dopiero po `COMMIT`.

### Plan zapytania

```sql
EXPLAIN SELECT … | UPDATE … | DELETE …;
```

Zwraca ścieżkę dostępu (`PRIMARY KEY LOOKUP`, `PRIMARY KEY MULTI-LOOKUP`, `PRIMARY KEY RANGE SCAN`, `INDEX LOOKUP`, `INDEX MULTI-LOOKUP`, `INDEX RANGE SCAN`, `FULL TABLE SCAN`, `EMPTY`), filtr, grupowanie, sortowanie (z informacją o selekcji top-k lub o wykorzystaniu porządku klucza) i limit.

### Parametry

Znak `?` oznacza parametr pozycyjny przygotowanej instrukcji (`db.prepare(sql)`, `db.run(stmt, [wartości])`). Parametry mogą wystąpić w każdym miejscu, w którym dozwolony jest literał.

## 4. Wyrażenia

| Kategoria | Składnia |
|---|---|
| Literały | `42`, `-7`, `3.14`, `1e3`, `'tekst'` (`''` w środku = apostrof), `TRUE`, `FALSE`, `NULL` |
| Arytmetyka | `+ - * / %` (dzielenie liczb całkowitych jest całkowite; dzielenie przez zero i przepełnienie to błąd `eval`) |
| Tekst | `a \|\| b` (konkatenacja), `LIKE` (wielkość liter ma znaczenie), `ILIKE` (bez rozróżniania), wzorce `%` i `_` |
| Porównania | `= == != <> < <= > >=`, `BETWEEN a AND b`, `IN (…)`, `IS [NOT] NULL`, negacje `NOT IN`, `NOT BETWEEN`, `NOT LIKE` |
| Logika | `AND`, `OR`, `NOT` – logika trójwartościowa SQL (`NULL AND FALSE = FALSE`, `NULL OR TRUE = TRUE`) |
| Kolumny | `kolumna`, `tabela.kolumna` |

Pierwszeństwo operatorów (od najniższego): `OR`, `AND`, `NOT`, porównania / `IS` / `IN` / `BETWEEN` / `LIKE`, `||`, `+ -`, `* / %`, jednoargumentowy `-`.

Warunek `WHERE` musi mieć wartość logiczną (liczby: 0 = fałsz); wiersze, dla których warunek jest `NULL`, nie są zwracane.

## 5. Funkcje

| Funkcja | Opis |
|---|---|
| `COUNT(*)`, `COUNT(x)`, `COUNT(DISTINCT x)` | liczba wierszy / wartości różnych od `NULL` / unikalnych |
| `SUM(x)`, `AVG(x)` (także z `DISTINCT`) | suma (całkowita, jeśli wszystkie wartości są `INT`) i średnia (`FLOAT`) |
| `MIN(x)`, `MAX(x)` | najmniejsza / największa wartość |
| `LOWER(s)`, `UPPER(s)` | zmiana wielkości liter (Unicode) |
| `LENGTH(s)` | długość w znakach |
| `SUBSTR(s, start[, długość])` | podciąg (indeksy od 1) |
| `ABS(x)`, `ROUND(x[, cyfry])` | wartość bezwzględna, zaokrąglenie |
| `COALESCE(a, b, …)`, `IFNULL(a, b)` | pierwsza wartość różna od `NULL` |
| `TYPEOF(x)` | `int`, `float`, `text`, `bool` albo `null` |

Agregaty są dozwolone w `SELECT`, `HAVING` i `ORDER BY`, nie mogą być zagnieżdżone ani użyte w `WHERE`/`GROUP BY`.

## 6. Błędy

Każdy błąd ma kategorię (`error.kind` w JavaScript):

| Kategoria | Przykład |
|---|---|
| `lex`, `parse` | `syntax error at position 7: expected an expression, found 'FROM'` |
| `schema` | brak tabeli/kolumny, brak `PRIMARY KEY`, agregat w `WHERE` |
| `constraint` | duplikat klucza, naruszenie `NOT NULL` |
| `type` | tekst w kolumnie `INT`, porównanie tekstu z liczbą |
| `eval` | dzielenie przez zero, przepełnienie liczby całkowitej |
| `transaction` | `COMMIT` bez `BEGIN`, snapshot w trakcie transakcji |
| `storage` | uszkodzony snapshot lub paczka zmian |
| `unsupported` | `JOIN`, podzapytania, `FOREIGN KEY` – konstrukcje spoza modelu Flat-SQL |

## 7. Przykład

```sql
CREATE TABLE inventory (
  id INT PRIMARY KEY,
  player TEXT NOT NULL,
  item TEXT NOT NULL,
  rarity TEXT,
  qty INT,
  power FLOAT
);
CREATE INDEX inventory_player ON inventory (player);

INSERT INTO inventory VALUES
  (1, 'ayla', 'sword', 'epic', 1, 87.5),
  (2, 'ayla', 'health potion', 'common', 5, NULL),
  (3, 'borin', 'shield', 'rare', 1, 60.0);

BEGIN;
UPDATE inventory SET player = 'borin' WHERE id = 1;
UPDATE inventory SET qty = qty - 1 WHERE id = 2;
COMMIT;

SELECT rarity, COUNT(*) AS items, ROUND(AVG(power), 1) AS avg_power
FROM inventory
WHERE player = 'borin'
GROUP BY rarity
ORDER BY avg_power DESC;

EXPLAIN SELECT * FROM inventory WHERE player = 'ayla' AND qty > 1;
```

## 8. Czego język nie ma

`CASE`, podzapytań, złączeń, indeksów złożonych i unikalnych. Złączenia i więzy między tabelami są poza modelem z definicji; `CASE` i pozostałe rozszerzenia opisuje [docs/future/README.md §1.1](../../../../docs/future/README.md#11-rdzeń-silnika).