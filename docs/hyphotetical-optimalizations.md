# Hipotetyczne optymalizacje z Benchmarku 1

Propozycje wynikające z Benchmarku 1 ([benchmark-results.md](research/benchmark-results.md), dane i skrypty: [research/data/benchmark-1/](research/data/benchmark-1/)), które nie były objęte zadaniami 2.1–2.8 z [todo.md](todo.md). Każda pozycja to hipoteza: objaw zmierzony w Benchmarku 1, podejrzana przyczyna z dowodem (profil, `EXPLAIN`, analiza kodu), proponowana zmiana i pomiar, który ją potwierdzi albo odrzuci. Żadnej nie wykonano; przed wpisaniem do listy zadań każdą trzeba potwierdzić pomiarem według metodyki Benchmarku 1 (ta sama wersja narzędzia, prototyp mierzony obok nowej wersji).

Wartości to mediany z 10 uruchomień, 100 000 wierszy, przeglądarka Edge 156 albo Node.js 24, jeśli nie podano inaczej.

Stan po rozbiciu kosztów silnika ([benchmark-results.md §1.12](research/benchmark-results.md#112-rozbicie-kosztów-silnika-pomiar-pomocniczy)): część hipotez potwierdził pomiar natywny i trafiły do listy zadań jako wymagane.

| Hipoteza | Stan |
|---|---|
| O1 | zadanie 2.13 |
| O2, O3 | część zadania 2.9 |
| O4 | zadanie 2.14 |
| O7 | zadanie 2.11 (outbox to około połowy czasu odtworzenia) |
| O9 | budowa indeksów: zadanie 2.12; outbox: 2.11; dekodowanie wierszy: 2.7 |
| O10 | zadanie 2.16 |
| O11 | zadanie 2.15 |
| O13 | przyczyna poprawiona, zob. niżej; zadanie 2.10 przed 2.7 |
| O5, O6, O8, O12 | hipotezy, bez zadania |

## Granica JS↔WASM i most

### O1. Stałe nazwy właściwości i kolumn w wyniku (przed 2.4)

- Objaw: odczyt po kluczu z wynikiem jako obiekty JS trwa 9,15 µs w przeglądarce i 9,32 µs w Node.js, wynik jako JSON 5,6 µs, natywnie 2,38 µs.
- Dowód: profil V8 (`analysis/profile.txt`) przypisuje dekodowaniu UTF-8 (`decodeUTF8`, `decodeText`, `decode`) około 24% próbek ścieżki „obiekty”. `result_to_js` w `crates/flatsql-wasm/src/lib.rs` przy każdym wyniku tworzy z UTF-8 pięć kluczy obiektu (`columns`, `rows`, `rowsAffected`, `rowsScanned`, `message`), wszystkie nazwy kolumn i tekst `message`, czyli przy odczycie jednego wiersza 11 łańcuchów niezależnych od danych.
- Zmiana: klucze jako łańcuchy internowane po stronie JS (np. funkcja `intern` wasm-bindgen albo obiekt budowany w kodzie sklejającym), nazwy kolumn zapamiętane przy przygotowanej instrukcji i zwracane z JS, `message` tylko dla instrukcji zmieniających dane.
- Pomiar: `point_lookup` i `materialize_10k` w grupie „pamięć”, profil z `analysis/profile.mjs`. Spodziewany wynik: ścieżka „obiekty” bliżej JSON (około 6–7 µs). To mniejszy krok niż binarny format z 2.4 i może go poprzedzić.

### O2. Odczyty w `FlatDB` bez sprawdzania WAL

- Objaw: odczyt po kluczu przez `FlatDB` (grupa „trwały zapis”) trwa 9,95 µs wobec 9,15 µs bezpośrednio na `Database` (+0,8 µs, 9%).
- Dowód: `FlatDB.run` po każdym wywołaniu woła `#afterWrite()`, które przechodzi przez granicę (`walPending()`), także po `SELECT`.
- Zmiana: flaga „tylko odczyt” przygotowanej instrukcji (planer ją zna) albo sprawdzanie WAL tylko po instrukcjach, które mogą pisać.
- Pomiar: `point_lookup` w grupie „trwały zapis” wobec grupy „pamięć”.

### O3. Pamięć podręczna przygotowanych instrukcji dla `exec(sql)` i `FlatDB.query`

- Objaw: lokalne zapytanie po kluczu z tekstem SQL (`db.exec`, tabela 10 000 wierszy) trwa 14,8 µs w Node.js, a przygotowane raz zapytanie z tym samym sposobem przekazania wyniku (obiekty JS, tabela 100 000 wierszy) 9,3 µs; parsowanie i planowanie kosztuje więc około 5 µs na wywołanie.
- Dowód: `FlatDB.query(sql, params)` przy każdym wywołaniu przygotowuje instrukcję od nowa (`this.prepare(sql)`), a zwolnienie zostawia odśmiecaczowi (`FinalizationRegistry`).
- Zmiana: mała pamięć podręczna LRU przygotowanych instrukcji według tekstu SQL w moście, z jawnym zwalnianiem wypieranych.
- Pomiar: H1 („lokalnie”) i nowa operacja `query(sql, params)` w obciążeniu; sprawdzić także pamięć liniową przy wielu różnych zapytaniach.

### O4. Wstawianie wielu wierszy jednym wywołaniem

- Objaw: 100 000 przygotowanych `INSERT` w jednej transakcji trwa 343–391 ms w WASM, 214 ms natywnie (1,6–1,8×), a w sql.js 208–223 ms.
- Dowód: narzut WASM jest bliski zeru w operacjach liczonych w całości w silniku (pełny skan 0,98×, top-k 0,95×), więc różnica pochodzi z 100 000 przejść przez granicę z pięcioma parametrami, w tym dwoma łańcuchami kodowanymi do UTF-8 przy każdym wierszu.
- Zmiana: `executeMany(statement, rows)` albo wsad kolumnowy (tablice typowane dla liczb, jeden bufor dla tekstów); 2.4 obejmuje typowane parametry pojedynczego wywołania, nie wsad.
- Pomiar: `insert` w Node.js i przeglądarce wobec natywnego.

### O5. Grupowanie wywołań w trybie Worker

- Objaw: odczyt po kluczu w Workerze trwa 65,4 µs wobec 9,15 µs w wątku głównym; pobranie 10 000 wierszy 23,1 ms wobec 12,0 ms.
- Dowód: różnica median 56 µs to koszt komunikatu w obie strony; przy 10 000 wierszy dochodzi kopiowanie tablic przez *structured clone* (11 ms).
- Zmiana: kilka instrukcji w jednym komunikacie (np. transakcja odczytu wysyłana jako wsad) i przekazywanie wyniku w buforze jako obiekt przenoszony (`transfer`), gdy powstanie binarny format z 2.4.
- Pomiar: grupa „Worker”, `point_lookup` i `materialize_10k`; nowa operacja „10 odczytów w jednym komunikacie”.

### O6. Mniej plików mostu przy starcie

- Objaw: zimny start w przeglądarce 47,9 ms, z czego import i kompilacja 38 ms; przeglądarka pobiera 8 plików (7 modułów JS mostu i moduł WASM), także `sync.js` i `indexeddb.js`, gdy aplikacja ich nie używa.
- Dowód: lista zasobów zimnego startu (`raw/browser/cold/flatsql/*.json`). Na `localhost` liczba żądań prawie nie kosztuje; w sieci z opóźnieniem każde zależne żądanie dokłada czas przynajmniej jednego RTT.
- Zmiana: jeden plik ESM mostu (wydanie zbudowane bundlerem) albo dynamiczny import modułów synchronizacji i IndexedDB.
- Pomiar: zimny start z symulowanym opóźnieniem sieci (nowa opcja serwera pomiarowego), porównanie liczby żądań i czasu do pierwszego zapytania.

## Pamięć

### O7. Outbox bez wpisu na każdy wiersz

- Objaw: baza 100 000 wierszy zajmuje 42,3 MiB pamięci liniowej przy wstawianiu w transakcjach po 1000 wierszy, około 8 razy więcej niż jej snapshot (5,3 MiB); pamięć liniowa sql.js po wczytaniu tych samych danych ma 21 MiB, SQLite WASM 8 MiB.
- Dowód: po wstawieniu 100 000 wierszy outbox ma 100 002 wpisy (`analysis/memory.txt`), także gdy aplikacja w ogóle nie synchronizuje. Każdy wpis to element `BTreeMap` z numerem sekwencyjnym, nazwą tabeli i kopią klucza; outbox trafia też do snapshotu.
- Zmiana: zwarty zapis outboxa (zakresy numerów sekwencyjnych z odwołaniem do WAL albo zbiór kluczy na tabelę bez powtarzania nazwy tabeli) albo outbox włączany dopiero przy konfiguracji synchronizacji. Zmiana dotyka formatu snapshotu i rekonstrukcji outboxa z WAL, więc wymaga decyzji projektowej.
- Pomiar: `analysis/memory.mjs` z rozbiciem na tabelę, indeks i outbox; rozmiar snapshotu; `snapshot`/`restore` w obciążeniu.

### O8. Szczyt pamięci przy dużej transakcji

- Objaw: wstawienie 100 000 wierszy w jednej transakcji zostawia 73,3 MiB pamięci liniowej, w transakcjach po 1000 wierszy 42,3 MiB. Pamięć liniowa nie maleje, więc szczyt z jednej transakcji zostaje do końca życia strony.
- Dowód: transakcja trzyma w pamięci trzy struktury proporcjonalne do liczby zmian (`Txn` w `db.rs`): dziennik wycofania (dla wstawienia wpis `Undo::Row` z kluczem, nazwą tabeli i pustym poprzednim stanem), zakodowany WAL transakcji (`ops`, 5,4 MB dla 100 000 wierszy) i wpisy outboxa transakcji. Wszystkie trzy to wektory rosnące przez podwajanie, więc tuż przed zatwierdzeniem mogą zajmować do dwóch razy więcej niż ich zawartość. Ten sam zbiór wczytany w transakcjach po 1000 wierszy nie przekracza 42,3 MiB (`analysis/memory.txt`).
- Zmiana: dla wierszy wstawianych do pustych kluczy bez osobnego wpisu wycofania (wycofanie usuwa klucze z zakresu numerów sekwencyjnych transakcji), a w moście wczytywanie dużych zbiorów w transakcjach ograniczonej wielkości; atomowość całego wczytania wymagałaby wtedy znacznika obejmującego wiele ramek WAL.
- Pomiar: `analysis/memory.mjs`, wariant jednej transakcji.

## Trwałość i start z magazynu

### O9. Szybsze ponowne otwarcie bazy

- Objaw: ponowne otwarcie bazy 100 000 wierszy z IndexedDB i pierwsze zapytanie trwa 174 ms; sql.js 10,7 ms, Dexie.js 2,8 ms (Dexie nie wczytuje danych do pamięci), wa-sqlite 7,7 ms. Na wolnym telefonie to kilkaset milisekund przed pierwszym zapytaniem.
- Dowód: odtworzenie snapshotu dekoduje każdą wartość osobno, alokuje wiersz i klucz, a indeksy pomocnicze buduje od nowa (`storage/snapshot.rs`, „indexes are rebuilt from rows instead of being stored”); `restore` w obciążeniu kosztuje 116 ms w WASM i 114 ms natywnie, czyli sam silnik.
- Zmiana: indeksy zapisane w snapshocie (kosztem rozmiaru) albo budowane leniwie przy pierwszym użyciu; niezależnie od 2.7 (zwarte wiersze skróciłyby dekodowanie wierszy).
- Pomiar: `reopen` w grupie „trwały zapis”, `restore`, rozmiar snapshotu.

### O10. `FsStore` z otwartym plikiem WAL

- Objaw: trwałe zatwierdzenie w `FsStore` (Node.js) bez `fsync` trwa 0,60 ms w Windows i 0,40 ms w Linuksie, a w `FileStorage` (CLI) 10 µs; z `fsync` 1,34 ms i 2,91 ms wobec 0,58 ms i 2,15 ms w CLI.
- Dowód: `FsStore.appendWal` przy każdym dopisaniu otwiera i zamyka plik (`fsp.open(this.#wal, 'a')`); CLI trzyma uchwyt otwarty.
- Zmiana: jeden uchwyt pliku WAL na czas życia magazynu (zamykany przy checkpoincie i `close`), `datasync` zamiast `sync` tam, gdzie metadane pliku nie są potrzebne do odtworzenia (do sprawdzenia z modelem awarii z testów trwałości).
- Pomiar: `node/fsync` w Windows i Linuksie, runda `/sync/push` serwera z katalogiem danych; testy trwałości `FsStore` bez zmian w wynikach.

## Zapytania

### O11. Grupowanie bez alokacji klucza na wiersz

- Objaw: `GROUP BY` po 100 kategoriach trwa 15,6 ms; DuckDB-Wasm 3,6 ms, czysty JS 1,7 ms.
- Dowód: wykonawca (`exec.rs`, typ `Groups`) dla każdego wiersza buduje klucz `Vec<Cow<Value>>` i szuka go w `BTreeMap` porównaniami łańcuchów.
- Zmiana: ścieżka dla jednej kolumny grupującej bez wektora klucza, tablica mieszająca z kluczem tekstowym, a przy indeksie na kolumnie grupującej przejście po indeksie (grupy już rozdzielone).
- Pomiar: `group_by` natywnie i w WASM (narzut WASM wynosi tu 1,05–1,13×, więc zysk będzie widoczny w obu).

### O12. Odczyt wyłącznie z indeksu dla innych zapytań niż `COUNT(*)`

- Objaw: `COUNT(*)` po indeksie trwa 438–457 µs wobec 63–73 µs w SQLite; `EXPLAIN` pokazuje odczyt każdego wiersza i ponowne sprawdzenie filtra, a SQLite używa indeksu pokrywającego (`analysis/explain.txt`).
- Dowód: zadanie 2.1 obejmuje `COUNT(*)`. Ten sam mechanizm (indeks zawiera wartość kolumny i klucz) wystarczy dla `SELECT id FROM … WHERE category = ?`, `COUNT(DISTINCT category)` i `GROUP BY category` z `COUNT(*)`.
- Zmiana: ogólna ścieżka „tylko indeks”, gdy wszystkie kolumny zapytania są w indeksie.
- Pomiar: nowe zapytania w obciążeniu (wymagają nowego identyfikatora obciążenia, więc dopiero w kolejnej wersji benchmarku).

### O13. Koszt pełnego przeglądu tabeli

- Objaw: pełny skan z filtrem trwa 8,0 ms (80 ns na wiersz), czysty JS 0,6 ms, DuckDB-Wasm 0,6 ms; natywnie Flat-SQL 8,1 ms, więc to koszt samego silnika.
- Dowód: wiersze są osobnymi alokacjami `Vec<Value>` w węzłach `BTreeMap`; każdy wiersz to kilka odwołań do pamięci rozrzuconej po stercie, a `Value` ma 32 bajty niezależnie od typu.
- Zmiana: 2.7 (zwarte wiersze) jest głównym kierunkiem; dodatkowo profil natywny (Windows Performance Toolkit jest zainstalowany) przed 2.7, żeby oddzielić koszt przechodzenia drzewa od ewaluacji predykatu.
- Pomiar: `full_scan`, `top_k`, `group_by` natywnie z profilem.
- Wynik rozbicia kosztów: ręczna pętla po tym samym drzewie z tym samym warunkiem trwa 2,9 ms wobec 8,2 ms silnika, a samo przejście drzewa 1,6 ms. Układ wierszy odpowiada więc za około jedną trzecią czasu, interpreter wyrażeń za dwie trzecie. Głównym kierunkiem jest kompilacja wyrażeń przy przygotowaniu (2.10), zwarte wiersze (2.7) w drugiej kolejności.
