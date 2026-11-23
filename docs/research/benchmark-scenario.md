# Metodyka benchmarków

Dokument opisuje, co i jak mierzono: pytania badawcze, sprzęt, obciążenie, porównywane silniki i procedurę. Ta sama metodyka obowiązuje we wszystkich porównywanych pomiarach. Benchmark 1 zmierzył prototyp i inne silniki, a Benchmark 2 prototyp i wersję po optymalizacjach tym samym narzędziem. Wyniki: [wyniki benchmarków](benchmark-results.md). Polecenia do powtórzenia pomiarów: [bench/README.md](../../bench/README.md).

## Pytania badawcze

| Pytanie | Hipoteza przed pomiarem |
|---|---|
| Lokalnie a przez serwer: o ile szybsze jest zapytanie wykonane na urządzeniu? | Lokalny odczyt po kluczu trwa mikrosekundy, a przez sieć co najmniej tyle, ile opóźnienie sieci, czyli 2–4 rzędy wielkości dłużej. |
| Ile kosztuje WebAssembly wobec kodu natywnego? | Ten sam kod Rust w WebAssembly jest 1,2–3 razy wolniejszy. |
| Ile kosztuje granica między JavaScriptem a WebAssembly? | Przy małych wynikach dominuje przekazanie danych; wynik jako JSON jest szybszy od budowania obiektów z Rusta. |
| Jak Flat-SQL wypada na tle dojrzałych silników? | Konkurencyjny w odczytach, słabszy tam, gdzie SQLite kopiuje strony pliku (snapshot, odtworzenie) i korzysta z indeksów pokrywających. |
| Ile kosztuje trwałe zatwierdzenie w przeglądarce? | Przyrostowy WAL kosztuje tyle, ile zmiana, a eksport całej bazy rośnie z jej rozmiarem. |
| Czy aplikacja działa offline i czy urządzenia dochodzą do tego samego stanu? | Tak. To pytanie sprawdzają [testy](../testing/offline-work.md), a nie benchmark. |

Pomiar odpowiada też na pytania pomocnicze: rozmiar modułu i czas zimnego startu, pamięć zajęta przez 100 000 wierszy, długość pauzy przy checkpoincie, koszt Web Workera, koszt trybu `strict` w IndexedDB i koszt `fsync` przy zapisie plików.

## Środowisko

| Element | Wartość |
|---|---|
| Sprzęt | laptop, AMD Ryzen 5 4600H (6 rdzeni, 12 wątków), 16 GB RAM, dysk SSD NVMe |
| System | Windows 11 Home 10.0.26200, plan zasilania „Zrównoważony”, zasilanie sieciowe |
| Node.js | 24.21.0 |
| Przeglądarka | Microsoft Edge 156 (Chromium, silnik V8), bez okna, sterowana przez playwright-core 1.64.1 |
| Rust | 1.99.0, `opt-level = 3`, optymalizacja całego programu, `wasm-opt -O3` |
| Silniki odniesienia | sql.js 1.14.2, SQLite WASM 3.54.1, wa-sqlite 2.0.8, PGlite 0.5.11, DuckDB-Wasm 1.32.0, AlaSQL 4.21.0, Dexie.js 4.4.7, PouchDB 9.0.0, RxDB 17.6.0 |

Każdy pomiar zapisuje w pliku `manifest.json` wersje wszystkich składników, procesor, system i mierzoną wersję Flat-SQL. Systemu nie strojono, bo wynik ma opisywać typowe urządzenie użytkownika.

## Obciążenie

Jedna tabela wspólna dla silników SQL:

```sql
CREATE TABLE items (id INTEGER PRIMARY KEY, category TEXT, price FLOAT, qty INT, name TEXT);
CREATE INDEX items_category ON items (category);
```

Wiersz `i` to `(i, 'c' || (i % 100), ((i * 7919) % 10000) / 100.0, i % 50, 'item-' || i)`: 100 kategorii po 1000 wierszy, każda cena 10 razy. Domyślnie 100 000 wierszy. PGlite i DuckDB dostają te same typy w swoim dialekcie.

| Operacja | Zapytanie | Wykonań w uruchomieniu |
|---|---|---:|
| Wstawienie | 100 000 × przygotowany `INSERT` w jednej transakcji | 1 |
| Odczyt po kluczu | `SELECT * FROM items WHERE id = ?` | 10 000 |
| `COUNT` po indeksie | `SELECT COUNT(*) FROM items WHERE category = ?` | 500 |
| Zakres klucza | `SELECT SUM(price) FROM items WHERE id BETWEEN ? AND ?` (1000 kluczy) | 200 |
| Pełny przegląd z warunkiem | `SELECT COUNT(*) FROM items WHERE price > 50 AND qty < 10` | 20 |
| Grupowanie | `SELECT category, COUNT(*), AVG(price), MAX(qty) FROM items GROUP BY category` | 20 |
| Najwyższe ceny | `SELECT id, price FROM items ORDER BY price DESC, id ASC LIMIT 10` | 20 |
| Pobranie wierszy | `SELECT * FROM items WHERE id < 10000` (10 000 wierszy do JavaScriptu) | 20 |
| `UPDATE` 10% | `UPDATE items SET qty = qty + 1 WHERE id % 10 = 0` | 1 |
| `DELETE` 10% | `DELETE FROM items WHERE id % 10 = 1` | 1 |
| Snapshot | zapis całej bazy do bajtów | 1 |
| Odtworzenie | baza z tych bajtów | 1 |
| Wstawienie jednym wywołaniem | te same wiersze do drugiej tabeli; tylko Flat-SQL, od obciążenia `benchmark-2` | 1 |

Ostatnia operacja jest na końcu, więc wcześniejsze są takie same jak w Benchmarku 1. Zapytania są przygotowywane raz, a wyniki w całości trafiają do JavaScriptu jako tablice wierszy, tak jak w aplikacji. Parametry pochodzą z generatora o stałym ziarnie i są identyczne we wszystkich silnikach i uruchomieniach.

Silniki z trwałym magazynem mają dodatkowo fazę zapisu trwałego: 100 zatwierdzeń po jednym wierszu z czekaniem na trwałość, 1000 zatwierdzeń z jednym oczekiwaniem na końcu, zamknięcie bazy, ponowne otwarcie i pierwsze zapytanie. W tej grupie zapisy są mierzone do chwili, gdy zmiana jest trwała.

## Silniki i grupy warunków

Silniki porównuje się tylko w grupie o tych samych warunkach: tym samym wątku, rodzaju API i magazynie. Zestawienie silnika w pamięci z silnikiem, który utrwala każdy zapis, mierzyłoby różnicę warunków, a nie silników (Raasveldt i in., 2018).

| Grupa | Warunki | Konfiguracje |
|---|---|---|
| pamięć | dane w pamięci, silnik w wątku wywołującym; Node.js i przeglądarka | Flat-SQL (wynik jako obiekty i jako JSON), sql.js, SQLite WASM, PGlite, DuckDB-Wasm, AlaSQL, czysty JavaScript na `Map` |
| trwały zapis | IndexedDB, wątek główny przeglądarki | Flat-SQL (tryb domyślny i `strict`), sql.js z eksportem całej bazy, wa-sqlite (`synchronous=full` i `normal`), PGlite, Dexie.js (domyślny i `strict`), PouchDB, RxDB |
| Worker | silnik w Web Workerze; każde wywołanie to komunikat w obie strony | Flat-SQL w pamięci i z IndexedDB, SQLite WASM z OPFS (`opfs-sahpool`), PGlite z OPFS, DuckDB-Wasm |
| natywnie | ten sam rdzeń Rust bez WebAssembly | Flat-SQL (`flatsql bench`) |

Każdy silnik pracuje w konfiguracji domyślnej albo zalecanej w dokumentacji. Wyjątki są w nazwie konfiguracji. Bazy bez SQL (Dexie.js, PouchDB, RxDB) wykonują te same operacje przez własne API, a to, czego API nie wyraża (na przykład grupowanie), liczy JavaScript adaptera i wchodzi to do wyniku. Szczegóły odwzorowania są w kodzie adapterów w `bench/engines/`.

Nie mierzono Zeta-Lite (kod zamknięty, brak modułu), absurd-sql (projekt porzucony; jego podejście reprezentują wa-sqlite i `opfs-sahpool`) ani SQLite z VFS `opfs` (wymaga nagłówków izolacji, a dokumentacja SQLite wskazuje `opfs-sahpool` jako szybszy).

## Jak mierzymy

**Powtórzenia.** Jednostką powtórzenia jest proces. Każdy pomiar działa w nowym procesie Node.js albo w nowej przeglądarce z pustym profilem, co najmniej 10 razy, w losowej kolejności silników. Kompilator JIT i odśmiecanie pamięci zmieniają się między procesami bardziej niż w jednej pętli (Georges i in., 2007). Pomiar w jednym procesie dawał narzut WebAssembly 1,1–1,8×, a w dziesięciu procesach 0,89–1,13×.

**Rozgrzewka.** Przed pomiarem każdy silnik wykonuje skrócony przebieg na osobnej bazie (10 000 wierszy). Bez tego pierwsza operacja płaciłaby za kompilację kodu.

**Czas.** Czas operacji to czas pętli wszystkich wykonań podzielony przez ich liczbę. Strona pomiarowa w przeglądarce jest izolowana nagłówkami COOP i COEP tylko po to, żeby `performance.now()` miał dokładność 5 µs zamiast 100 µs. Żaden silnik nie korzysta z możliwości, które izolacja odblokowuje. Odczyt ma limit 20 s na uruchomienie, który dotyczy tylko najwolniejszych silników.

**Statystyka.** Z każdego uruchomienia jest jedna wartość. Wynik to mediana z pierwszym i trzecim kwartylem. Zmiana między wersjami jest wykazana, gdy 95% przedziały ufności średnich się nie nakładają. Mniejsze różnice opisuje się jako mieszczące się w rozrzucie.

**Poprawność.** Każda odpowiedź jest zapisywana w postaci kanonicznej (liczby × 10⁶ zaokrąglone, wiersze bez `ORDER BY` posortowane), a z niej liczona jest 64-bitowa suma kontrolna. Po każdej fazie zapisu silnik zwraca stan tabeli: `COUNT(*)`, `SUM(qty)`, `SUM(price)`. Suma 100 000 cen zależy od kolejności dodawania, więc stan tabeli porównuje się z tolerancją 10⁻⁹. Sama liczba wierszy nie wykryłaby złego wiersza zwróconego po kluczu.

**Zimny start.** Czas od importu silnika do wyniku pierwszego zapytania w nowym procesie, bez pamięci podręcznej. Pliki są serwowane z `localhost`, więc czas nie zawiera sieci. Rozmiar to suma wszystkich plików silnika po kompresji gzip, żeby dało się przeliczyć pobranie dla dowolnego łącza.

**Pamięć.** Rozmiar pamięci modułu WebAssembly po wczytaniu danych jedną transakcją. Pamięć modułu nie maleje (Yan i in., 2021), więc wynik obejmuje też szczyt z wczytywania. Dla silników w JavaScripcie mierzony jest przyrost sterty po pełnym odśmiecaniu.

**Trwały zapis.** Zatwierdzenie jest trwałe, gdy magazyn potwierdził zapis. Chromium domyślnie potwierdza zapis IndexedDB po przekazaniu danych systemowi (tryb `relaxed`), a tryb `strict` czeka na dysk. Konfiguracje `strict` mierzą koszt tej gwarancji.

**Pauza checkpointu.** Podczas checkpointu w wątku głównym działa sonda mierząca przerwy między obrotami pętli zdarzeń. Wynik to najdłuższa przerwa dla 10 000, 50 000 i 100 000 wierszy, na tle jednej klatki przy 60 Hz (16,7 ms).

**Koszt Workera.** Różnica median tej samej operacji w Workerze i w wątku głównym.

**Lokalnie a przez serwer.** Ten sam silnik i ta sama tabela 10 000 wierszy po obu stronach. Lokalnie zapytanie wykonuje się w procesie. Przez serwer idzie do `/api/query` z opóźnieniem sieci symulowanym po stronie serwera: 0, 20, 50 i 100 ms. Symulacja pomija TLS, utratę pakietów i nawiązywanie połączenia, więc koszt sieci jest zaniżony.

**Koszt utrwalania plików.** Faza zapisu trwałego na plikach: magazyn natywny i magazyn Node.js, każdy z `fsync` i bez, w Windows i w Linuksie (WSL 2).

**Wersje silnika.** Narzędzie zawsze pochodzi z bieżącego kodu, a mierzona wersja silnika z osobnej kopii roboczej. Dzięki temu prototyp (tag `v1-prototyp`) i wersję po optymalizacjach (tag `v2-optymalizacja`) mierzy to samo narzędzie. Surowy wynik każdego procesu zapisuje się w `data/<benchmark>/raw/`, a skrypt raportu tworzy z nich wszystkie tabele i wykresy.

## Ograniczenia trafności

- Wszystkie pomiary pochodzą z jednego laptopa i silnika V8. Wartości bezwzględne nie przenoszą się na inne urządzenia, zwłaszcza telefony. Przenoszą się proporcje między silnikami w grupie.
- Adaptery innych silników napisał autor Flat-SQL. Każdy używa API zalecanego w dokumentacji, a sumy kontrolne potwierdzają te same odpowiedzi, ale nie to, że wybrano najszybszy sposób. Pierwsza wersja adaptera RxDB była przez złe użycie indeksu sto razy wolniejsza (2,9 s zamiast 30 ms), co pokazuje skalę tego ryzyka.
- Wynik obejmuje przekazanie odpowiedzi jako tablic wierszy. Silnik, którego naturalnym formatem są kolumny (DuckDB-Wasm) albo obiekty, płaci za konwersję.
- Gwarancje trwałości różnią się między silnikami mimo wspólnej definicji, więc wyniki trwałego zapisu podaje się razem z trybem.
- Obciążenie jest syntetyczne: jedna tabela, równomierny rozkład kluczy. Rzeczywiste aplikacje mają rozkład skośny i wiele tabel.
- Przeglądarka działa bez okna, więc pauza checkpointu nie obejmuje rysowania klatek.
