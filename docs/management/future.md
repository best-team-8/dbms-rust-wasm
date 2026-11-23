# Na później: rozwój Flat-SQL po pracy inżynierskiej

Lista zadań, które rozwijają Flat-SQL w dojrzały produkt. Wynikają z analizy wdrożeń innych baz ([zastosowania](../research/engines-and-use-cases.md)), z literatury i ze słabych stron, które zostały po optymalizacjach. Literaturę podaną przy zadaniu opisuje [spis literatury](../references/README.md).

Każde rozszerzenie musi zachować to, co wyróżnia Flat-SQL: mały moduł, pracę w głównym wątku strony bez nagłówków izolacji, zapytania lokalne w mikrosekundach, przyrostowy zapis na dysk, wbudowaną synchronizację, płaski model tabel i jeden rdzeń działający wszędzie. Rozszerzenie, które wymaga Web Workera albo dużej biblioteki, jest opcją, a nie trybem domyślnym.

Rozmiar zadania: `[S]` pojedyncze funkcje i testy, `[M]` kilka modułów, nowe testy i pomiary, `[L]` nowy podsystem. `Po:` znaczy, że zadanie wymaga wcześniej innego.

## Cele

| Metryka | Teraz | Cel |
|---|---|---|
| Ponowne otwarcie bazy 100 000 wierszy | 81 ms jednym blokiem | bez pauzy dłuższej niż jedna klatka (16,7 ms) |
| Pliki silnika pobierane przy starcie | 287 KiB po kompresji | nie rosnąć; dodatki jako osobne warianty modułu |
| Utracone zmiany przy konflikcie | „ostatni zapis wygrywa” dla całego wiersza | reguła dla kolumny, liczniki, raport konfliktów |
| Wiele kart przeglądarki | zapisuje tylko jedna karta | jedna instancja silnika dla wszystkich kart |
| Restart serwera | każdy klient pobiera pełny stan | trwały log, pobieranie tylko zmian |

## Co blokuje najmocniejsze zastosowania

| Zastosowanie | Potrzebne zadania |
|---|---|
| Zbieranie danych w terenie offline | uwierzytelnianie, przestrzenie, replikacja częściowa |
| Aplikacje osobiste na wielu urządzeniach | uwierzytelnianie, przestrzenie, szyfrowanie |
| Kopia danych serwera w aplikacjach SaaS | wiele kart, tryb pamięci podręcznej, tabele tylko do odczytu, replikacja częściowa |
| Kasy i sprzedaż offline | tabele tylko do odczytu, zamiary zatwierdzane przez backend, uwierzytelnianie |
| Konfiguracja i flagi na węzłach edge | węzły edge, tabele tylko do odczytu |
| Zapisy gier | przestrzenie, uwierzytelnianie, eksport bazy, punkty przywracania |
| Pamięć asystentów | wektory, wyszukiwanie pełnotekstowe, gałęzie bazy, szyfrowanie |

## Platforma przeglądarki

### [M] Wiele kart

Dziś aplikacja demonstracyjna trzyma blokadę Web Locks: pierwsza karta zapisuje do IndexedDB, kolejne pracują tylko w pamięci i ich zapisy giną po zamknięciu. Docelowo jedna instancja silnika obsługuje wszystkie karty przez `SharedWorker`, a karty dowiadują się o zmianach przez `BroadcastChannel`. Dla starszych przeglądarek wariant zapasowy: aktywna karta wybierana blokadą, pozostałe kierują do niej zapytania. Test: dwie karty zapisują jednocześnie, a po zamknięciu aktywnej druga przejmuje zapis bez utraty danych. Notion miał uszkodzone dane, gdy kilka kart zapisywało naraz.

### [M] Tryb pamięci podręcznej danych serwera

Klient trzyma kopię potrzebnych danych serwera, a serwer pozostaje źródłem prawdy. Aplikacja działa, zanim moduł się wczyta. Odczyt lokalny „ściga się” z zapytaniem do API, bo na wolnych urządzeniach dysk bywa wolniejszy od sieci. Tabele podręczne mają limit rozmiaru, a najdawniej używane wiersze są usuwane tylko lokalnie, bez wysyłania `DELETE`. Literatura: Kistler i Satyanarayanan (1992). Po: tabele tylko do odczytu, pomiar na telefonie.

### [S] Eksport i import bazy do pliku

Pobranie bazy jako pliku (snapshot i WAL albo czytelny JSON) i wczytanie go z powrotem. Kopia zapasowa na wypadek usunięcia danych przez przeglądarkę i pełna kontrola użytkownika nad danymi. Literatura: Kleppmann i in. (2019).

### [S] Przykładowe rozszerzenie przeglądarki

Rozszerzenie z `FlatDB` w workerze tła. Worker tła bywa zatrzymywany, więc potrzebny jest test odtworzenia bazy po wybudzeniu i pomiar jego czasu.

### [M] Magazyn OPFS w trybie Worker

Szybszy zapis dużych baz w prywatnym systemie plików przeglądarki. Synchroniczny dostęp istnieje tylko w Web Workerze, więc magazyn może być wyłącznie opcją.

## Rdzeń i składowanie

### [M] Odtwarzanie bazy krokami

Otwarcie bazy to dziś jedno wywołanie: 46 ms w wątku głównym przy 100 000 wierszy, więcej niż jedna klatka. Dekodowanie snapshotu, budowa indeksów i odtworzenie WAL porcjami, z oddaniem sterowania pętli zdarzeń między nimi, tak jak już działa checkpoint. Pomiar: najdłuższa pauza i cały czas otwarcia przy 10, 50 i 100 tysiącach wierszy. Zyskuje na znaczeniu przy wektorach.

### [S] Szybsza suma kontrolna CRC32

Suma jest dziś liczona bajt po bajcie i zajmuje około 9 ms snapshotu 100 000 wierszy i drugie tyle przy otwarciu. Wariant ośmiu tablic daje te same wartości bez zmiany formatu.

### [S] Ramka WAL kodowana od razu w buforze wyjściowym

Zatwierdzenie dużej transakcji przez chwilę trzyma trzy kopie operacji, a pamięć modułu nie maleje. Przy wczytaniu 100 000 wierszy jedną transakcją to około 10 MiB nadmiaru. Transakcja może kodować operacje od razu w buforze wyjściowym, a zatwierdzenie tylko uzupełniać nagłówek. Oczekiwany wynik: 51,8 → około 42 MiB. Pomiar: `docs/research/data/tests/memory-peak/memory-peak.mjs`.

### [M] Wstawienie bez wpisu w dzienniku wycofania

Dziennik wycofania ma wpis dla każdego wstawionego wiersza, także gdy nie ma czego przywracać. Przy 100 000 wierszy to do 10 MiB. Razem z poprzednim zadaniem wczytanie jedną transakcją zbliżyłoby się do wczytania w porcjach po 10 000 wierszy (25,4 MiB).

### [M] Wymienna warstwa wierszy

Interfejs w rdzeniu (odczyt po kluczu, zakres, przegląd, indeksy, zapis) z dzisiejszą implementacją jako jedynym wariantem. Kompilator usuwa abstrakcję, więc nie może być wolniej niż w ostatnim pomiarze. Podstawa dla drzewa trwałego i ewentualnych wielu wersji wierszy.

### [L] Drzewo trwałe (warunkowo)

Drzewo współdzielące niezmienione węzły, w którym gałąź bazy powstaje bez kopii. Tylko jako wariant wymiennej warstwy wierszy i tylko wtedy, gdy kopia bazy okaże się za droga dla pamięci asystenta, bo kopiowanie ścieżek spowalnia zapisy. Po: wymienna warstwa wierszy.

### [S] Warianty modułu

Flagi kompilacji dla wyszukiwania pełnotekstowego, wektorów i gałęzi, żeby moduł podstawowy nie rósł. Pomiar rozmiaru i zimnego startu obu wariantów.

## Zapytania

### [S] `CASE` w wyrażeniach

Najczęściej brakująca konstrukcja w obrębie jednej tabeli.

### [M] Zapytania reaktywne

Subskrypcja zapytania jednotabelowego. Po zatwierdzeniu sprawdzane są tylko zmienione klucze. `COUNT` i `SUM` aktualizowane przyrostowo, `MIN` i `MAX` po usunięciu ekstremum liczone od nowa. Cel: aktualizacja w mikrosekundach, w budżecie jednej klatki. Literatura: Litt i in. (2023); Gupta i Mumick (1995). Po: wiele kart.

### [M] Punkty przywracania i sloty

Nazwany punkt zapamiętujący pozycję w WAL. Przywrócenie stanu albo kopia bazy do osobnego slotu. Zastosowania: sloty zapisu w grach, „spróbuj i cofnij”.

### [M] Wyszukiwanie pełnotekstowe

Indeks słów dla kolumny `TEXT`, operator wyszukiwania, podział na słowa z obsługą Unicode, proste ustalanie trafności. Moduł może urosnąć najwyżej o kilkadziesiąt KiB. Literaturę trzeba zebrać przy rozpoczęciu, na przykład Zobel i Moffat (2006) o indeksie odwróconym.

### [L] Typ wektorowy i wyszukiwanie podobieństwa

`VECTOR(n)` przechowywany zwięźle, funkcje odległości, `ORDER BY odległość LIMIT k`. Najpierw porównanie ze wszystkimi wierszami (do kilkudziesięciu tysięcy wpisów), później indeks przybliżony, na przykład HNSW. Źródło każdego wpisu jest zapisywane i widoczne dla użytkownika.

### [M] Gałęzie bazy

Kopia bazy, na której agent albo użytkownik pracuje osobno, a potem ją scala albo porzuca. Scalanie warunkowe sprawdza, czy wiersze przeczytane i zmienione w gałęzi nie zmieniły się w bazie głównej. Opis: [decyzja 0025](../adr/0025-galezie-bazy.md). Testy: dwie gałęzie zmieniające ten sam wiersz, odrzucone scalenie, porzucenie gałęzi bez śladu w WAL. Po: punkty przywracania.

### [M] Przykład pamięci asystenta

Aplikacja z tabelą wpisów pamięci, źródłem każdego wpisu, podglądem i usuwaniem przez użytkownika. Treść z obcej strony zapisana jako wspomnienie może później sterować asystentem, więc użytkownik musi ją widzieć. Pomiar otwarcia bazy z 10 000 wektorów po 384 liczby. Po: wyszukiwanie pełnotekstowe, wektory, gałęzie.

## Konflikty i synchronizacja

### [M] „Ostatni zapis wygrywa” dla kolumny

Znacznik dla kolumny przechowywany tylko wtedy, gdy różni się od znacznika wiersza. Współbieżne zmiany różnych kolumn jednego wiersza przestają ginąć. Protokół się nie zmienia. Pomiar rozmiaru snapshotu i paczek oraz czasu wstawiania przed i po. Literatura: Gray i in. (1996); Yu i Ignat (2020).

### [M] Liczniki scalane przez dodawanie

Kolumna deklarowana w SQL, na przykład `qty INT MERGE ADD`, której zmiany są sumowane, a nie nadpisywane. Rozszerzenie: licznik z dolną granicą (saldo ≥ 0) przez przydział „do wydania offline” z góry. Literatura: Gray i in. (1996); Shapiro i in. (2011); Lopes i in. (2019). Po: „ostatni zapis wygrywa” dla kolumny.

### [M] Raport konfliktów klucza głównego

Dwa wstawienia tego samego klucza z różnych urządzeń są zgłaszane w raporcie synchronizacji zamiast cichego nadpisania. Literatura: Go i in. (2015).

### [M] Usuwanie znaczników usunięcia

Znacznik usunięcia można skasować, gdy wszystkie urządzenia go potwierdziły. Przy serwerze centralnym wystarczy najmniejszy potwierdzony kursor aktywnych klientów. Literatura: Almeida (2024). Po: trwały log serwera.

### [S] Granice transakcji w paczkach (warunkowo)

Kolejka zmian nie dzieli jednej transakcji między paczki, więc inne urządzenia nie widzą stanu pośredniego. Tylko jeśli aplikacja demonstracyjna pokaże potrzebę. Literatura: Bailis i in. (2013).

### [M] Wersje struktury tabel

Kroki budowy bazy z numerami wersji w `FlatDB.open`, `ALTER TABLE … ADD COLUMN`, przechowywanie wartości nieznanych kolumn w wierszu, polecenia zmiany struktury nierozsyłane do innych urządzeń, wersja formatu paczek. Do ustalenia, skąd serwer w trybie z silnikiem zna strukturę przestrzeni. Opis: [decyzja 0023](../adr/0023-struktura-tabel-w-kodzie.md). Literatura: Kleppmann (2017, rozdz. 4).

### [S] Serwer odrzuca zmiany struktury od klientów

Serwer odrzuca w paczkach polecenia `CREATE` i `DROP` od urządzeń. Po: wersje struktury tabel.

### [M] Powiadomienia z serwera

WebSocket albo SSE zamiast samego odpytywania co kilka sekund. Przydatne, gdy ktoś czeka na cudzą zmianę, na przykład ruch przeciwnika. Literatura: Kleppmann (2017, rozdz. 12).

### [L] Uzgadnianie zbiorów po utracie kursora (warunkowo)

Porównywanie odcisków zakresów kluczy zamiast wysyłania pełnego stanu. Tylko jeśli po trwałym logu pełne pobrania nadal będą problemem. Literatura: Meyer (2023). Po: trwały log serwera.

## Serwer synchronizacji

Docelowy serwer synchronizacji to osobna usługa obok backendu aplikacji. Przechowuje zmiany z lokalnych baz i oddaje je innym urządzeniom, które mają dostęp do tych samych danych. Nie zna użytkowników, nie zawiera logiki biznesowej i nie jest główną bazą aplikacji. Synchronizacja pozostaje opcją: `FlatDB` działa bez serwera. Uzasadnienie: [decyzja 0024](../adr/0024-serwer-synchronizacji.md).

Reguły biznesowe pisze programista w backendzie. Serwer daje mu trzy opcjonalne mechanizmy: tabele tylko do odczytu dla klientów, webhook sprawdzający zmiany przed przyjęciem oraz odczyt i zapis danych przestrzeni przez `/api/query`. Wszystkie wymagają trybu z silnikiem. Zadania są w kolejności wdrażania.

### [M] Trwały log synchronizacji

Dziś log jest tylko w pamięci, więc każdy restart serwera wysyła wszystkim pełny stan. Zapis logu i kursorów w katalogu danych, osobno dla każdej przestrzeni. Epoka zmienia się tylko po utracie logu. Skrócenie logu zastępuje stare paczki punktem startowym, czyli pełnym stanem bazy w formacie paczki. Urządzenie z kursorem sprzed skrócenia dostaje punkt startowy i resztę logu. Miejsce zapisu jest wymienne: dysk, magazyn obiektów (na przykład S3) albo tabela w bazie SQL backendu. Literatura: Kleppmann (2017, rozdz. 11); Kistler i Satyanarayanan (1992).

### [M] Przestrzenie

Przestrzeń to osobna baza z dowolną liczbą tabel: dane jednej osoby (`user-42`), zespołu (`team-7`), jednej rozgrywki (`game-1234`) albo dane wspólne dla wszystkich (`shop` z cennikiem). Powstaje przy pierwszej synchronizacji, jest ładowana przy pierwszym żądaniu i zwalniana po bezczynności. Wszystkie urządzenia jednego użytkownika używają tej samej przestrzeni, ale każde ma własną kopię bazy, kolejkę zmian i kursor. Członkostwo w przestrzeni współdzielonej sprawdza backend. Zapytanie działa w obrębie jednej przestrzeni. Dlatego dane porównywane między użytkownikami, na przykład ranking, trzyma baza backendu albo wspólna przestrzeń, do której pisze tylko backend. Po: trwały log.

### [M] Tryby pracy przestrzeni

Każda przestrzeń ma jeden z dwóch trybów. Tryb ustala konfiguracja serwera przy tworzeniu przestrzeni, na przykład według wzorca nazwy. Protokół jest w obu trybach ten sam. Dzisiejszy prototyp to tryb z silnikiem z jedną bazą dla wszystkich.

- Magazyn paczek, tryb domyślny. Serwer dopisuje paczki do logu i oddaje je od kursora, bez czytania treści. W pamięci trzyma prawie nic, więc skaluje się jak przechowywanie plików. Tylko w nim możliwe jest szyfrowanie end-to-end. Pasuje do danych jednej osoby: notatek albo zapisu gry jednoosobowej, w którym oszust oszukuje tylko siebie.
- Tryb z silnikiem, opcja. Przestrzeń ma na serwerze własną bazę Flat-SQL, do której serwer stosuje paczki. Otwarte bazy aktywnych przestrzeni zajmują pamięć. W zamian serwer rozumie wiersze, więc może zmiany sprawdzać, odrzucać i filtrować. Pasuje do danych wspólnych: cennika, kas, danych zespołu, gry wieloosobowej.

Magazyn paczek nie składa stanu z paczek, bo złożenie „stary stan + paczka” wymaga rozstrzygania konfliktów, czyli silnika. Z tego samego powodu urządzenia wysyłają zmiany, a nie całą bazę: stan wysłany później nadpisałby zmiany innego urządzenia. Zmiana trybu istniejącej przestrzeni wymaga przekształcenia danych. Do ustalenia: jak skracać log w magazynie paczek, skoro serwer nie złoży z niego punktu startowego. Literatura: Gray i in. (1996). Po: przestrzenie.

### [M] Uwierzytelnianie przez bramkę backendu

Serwer nie ma publicznego portu i słucha tylko w sieci wewnętrznej. Bramką jest backend aplikacji, napisany w dowolnym języku. Backend sprawdza sesję użytkownika, wybiera przestrzeń, do której użytkownik ma dostęp, i przekazuje paczkę bez czytania jej. Nazwę przestrzeni i klucz bramki podaje w nagłówkach. Kursor i epoka z odpowiedzi serwera wracają przez bramkę do urządzenia. Klucz bramki to losowy ciąg znany backendowi i serwerowi, który nigdy nie trafia do przeglądarki. Serwer odrzuca żądania bez niego, co chroni przed przypadkowym wystawieniem serwera. `/api/query` dostępne tylko z bramki, z możliwością wyłączenia. Dziś wykonuje dowolny SQL bez logowania i nie wolno go wystawiać publicznie. Cały ruch synchronizacji przechodzi przez backend, więc aplikacja bez backendu musi postawić bramkę. Po: przestrzenie.

### [M] Tabele tylko do odczytu dla klientów

Lista tabel w konfiguracji przestrzeni, na przykład cennik w przestrzeni `shop`. Serwer odrzuca paczki urządzeń, które je zmieniają, a lokalny zapis do takiej tabeli kończy się błędem. Zapisuje je tylko backend przez `/api/query`, a urządzenia czytają je lokalnie, także offline. Zamyka obejście, w którym użytkownik zmienia lokalną kopię w narzędziach przeglądarki. Podstawa dla cenników, katalogów, flag i rankingów. Po: tryby pracy przestrzeni.

### [M] Webhook do backendu

Przed przyjęciem paczki serwer pyta backend, które zmiany odrzucić. Wysyła identyfikator paczki i jej zmiany jako JSON: operację, tabelę, klucz oraz stan wiersza przed zmianą i po niej. Stan przed zmianą zna tylko baza serwera, dlatego webhook wymaga trybu z silnikiem. Backend odpowiada listą odrzuconych zmian z powodem, a pusta lista oznacza przyjęcie wszystkich. Odrzucenie dotyczy pojedynczych wierszy i odrzucone zmiany nie są zapisywane. Gdy backend nie odpowie w limicie czasu, paczka nie jest przyjęta i zostaje w kolejce urządzenia. Identyfikator paczki pozwala backendowi zwrócić tę samą decyzję przy ponowieniu. Przydaje się w kasach (stan magazynu, ceny), w danych zespołu (uprawnienia) i w grach wieloosobowych (oszustwa). W aplikacjach osobistych jest zbędny. Pomiar: liczba żądań do backendu i opóźnienie przyjęcia paczki przy wielu aktywnych urządzeniach. Literatura: Gray i in. (1996). Po: tryby pracy przestrzeni.

### [M] Kompensacja odrzuconych zmian

Serwer odsyła poprawną wartość z nowszym znacznikiem, żeby urządzenie wróciło do dozwolonego stanu. Literatura: Yu i Ignat (2020). Po: webhook.

### [M] Zamiary zatwierdzane przez backend

Klient dopisuje wiersz ze statusem „oczekujący” (zamówienie, ruch w grze, zakup za walutę z gry). Backend sprawdza go i zmienia status na „zatwierdzony” albo „odrzucony” z powodem. Interfejs pokazuje stan wstępny do czasu zatwierdzenia. Płatności prawdziwymi pieniędzmi zostają poza synchronizacją: obsługuje je backend, zawsze online. Literatura: Gray i in. (1996); Terry i in. (1995). Po: tabele tylko do odczytu, webhook.

### [L] Replikacja częściowa

Klient deklaruje swój zakres danych (przestrzeń roboczą, region, listę pacjentów). Serwer filtruje pobieranie i odrzuca zmiany spoza uprawnień. Wiersz, który przestał pasować do zakresu, klient usuwa tylko lokalnie. Pierwszy stan pobierany w częściach, z wznowieniem po przerwaniu. Testy: użytkownik A nigdy nie dostaje wierszy użytkownika B, także po skróceniu logu i zmianie zakresu. Pomiar pamięci serwera i czasu pierwszej synchronizacji przy wielu użytkownikach. Literatura: Ramasubramanian i in. (2009). Po: przestrzenie.

### [M] Skala serwera

Wszystkie otwarte bazy jednego procesu w trybie z silnikiem dzielą 4 GiB pamięci modułu WebAssembly, a pojedyncza przestrzeń musi zmieścić się w pamięci. Potrzebny jest limit baz w pamięci i zrzucanie nieużywanych, przypisanie przestrzeni do wielu instancji serwera oraz monitoring: liczba klientów, rozmiar logu, czas rund, liczba odrzuconych zmian i konfliktów. Dla dużych przestrzeni wspólnych potrzebna jest replikacja częściowa. Pomiar: pamięć serwera w obu trybach przy wielu przestrzeniach.

## Bezpieczeństwo

### [L] Szyfrowanie end-to-end

Paczki zmian szyfrowane kluczem użytkownika, a serwer jako przekaźnik szyfrogramu. Do rozstrzygnięcia konfliktu wystarcza jawny znacznik i skrót klucza. Działa tylko w magazynie paczek, bo serwer nie może czytać danych. Literatura: Kleppmann i in. (2019); Kuessner i in. (2022).

### [M] Szyfrowanie danych lokalnych

WAL i snapshot szyfrowane przez WebCrypto kluczem z hasła. Dopiero razem z szyfrowaniem end-to-end czyni „Privacy by Design” pełnym. Literatura: Cavoukian (2009).

## Węzły edge

### [L] Węzły edge zsynchronizowane z bazą główną

Węzeł blisko użytkowników trzyma własną kopię i log dla swoich klientów, a z bazą główną rozmawia tym samym protokołem co przeglądarki. Przekazuje w górę zmiany klientów i działa dalej, gdy baza główna jest niedostępna. Testy: dwóch klientów, dwa węzły i baza główna, odcięcie węzła, konflikt edycji, restart węzła. Pomiar opóźnienia do węzła wobec opóźnienia do bazy głównej. Literatura: Kleppmann (2017, rozdz. 5); Gray i in. (1996). Po: trwały log, uwierzytelnianie.

### [M] Jeden zapisujący dla wybranych tabel

Zapis na węźle przekazywany do bazy głównej i przez nią potwierdzany, zamiast rozstrzygania konfliktu na węźle. Dla danych, przy których spójność ostateczna jest ryzykowna, na przykład unieważniania sesji. Tak działa LiteFS. Literatura: Gray i in. (1996). Po: węzły edge.

### [M] Wdrożenie na platformie edge

Najprościej serwer Node.js w kilku regionach. Docelowo platforma typu Cloudflare Workers, gdzie jednowątkowy obiekt ze stanem (Durable Object) odpowiada modelowi wykonania Flat-SQL. Potrzebny magazyn dla tej platformy.

## Wdrożenie obok backendu

### [S] Obraz Docker

Obraz Node.js z modułem budowanym w osobnym etapie, wolumen na dane, sprawdzanie stanu przez `/api/status`. Serwer domyślnie słucha na `127.0.0.1`, więc w kontenerze trzeba podać `--host 0.0.0.0`.

### [S] Serwer bez aplikacji demonstracyjnej i z konfigurowalnym CORS

Opcja wyłączająca udostępnianie `web/` i lista dozwolonych domen zamiast `*`.

### [M] Przykład bramki i docker-compose

Mała bramka w Node.js z udawanym logowaniem dwóch użytkowników, z których każdy widzi tylko swoje dane. Serwer synchronizacji bez publicznego portu. Po: uwierzytelnianie przez bramkę.

### [S] Instrukcja wdrożenia

Uruchomienie, parametry, kopie zapasowe katalogu danych, aktualizacja obrazu.

## Interfejs C i aplikacje natywne

### [M] Interfejs C nad rdzeniem

Biblioteka z otwarciem bazy z katalogu, wykonaniem instrukcji, instrukcjami przygotowanymi, wymianą zmian i zwalnianiem pamięci.

### [M] Inne języki i aplikacje mobilne

Nakładki dla Go, C# i aplikacji mobilnych: Flat-SQL zamiast SQLite, z tym samym formatem plików i synchronizacją co w przeglądarce. Po: interfejs C.

### [L] Panel administracyjny WinUI 3

Dalszy etap z pierwotnego planu projektu. Wspólny format plików pozwala otworzyć bazę serwera. Literatury nie zebrano. Po: interfejs C.

## Badania

### [S] Pomiar na telefonie i w innych przeglądarkach

Wszystkie pomiary były w silniku V8 na jednym laptopie. Powtórzyć je na telefonie (tryb ręczny `bench/browser/serve.js` z `adb reverse`) oraz w Firefoksie i Safari, dla obu wersji silnika. Osobno czas otwarcia bazy, bo na wolnych urządzeniach dysk bywa wolniejszy od sieci.

### [M] Realistyczne obciążenie

Obciążenie YCSB z rozkładem Zipfa (na przykład 95% odczytów) i scenariusz z aplikacji demonstracyjnej. Wymaga nowego identyfikatora obciążenia i pomiaru obu wersji silnika. Literatura: Cooper i in. (2010).

## Pomysły bez zadań

Zgodne z koncepcją, ale bez potwierdzonej potrzeby albo pomiaru. Przed wpisaniem do listy każdy trzeba potwierdzić pomiarem.

- **Wiele wersji wierszy jako wariant kompilacji.** Dla wielu długich transakcji otwartych naraz, na przykład agentów czekających na odpowiedź modelu. Objęłoby około 25–35% rdzenia. Tylko jeśli gałęzie bazy nie wystarczą.
- **Weryfikacja przepustek JWT przez sam serwer.** Dla aplikacji bez backendu. Wymaga publicznego wystawienia serwera.
- **Serwer synchronizacji jako biblioteka w procesie backendu.** Przez interfejs C. Do tego czasu integracja tylko przez HTTP.
- **Kilka instrukcji w jednym komunikacie do Web Workera.** Każde wywołanie w Workerze kosztuje około 50 µs.
- **Mniej plików mostu przy starcie.** Jeden plik zamiast kilku modułów, bo w sieci z opóźnieniem każde zależne żądanie dokłada czas.
- **Odczyt wyłącznie z indeksu także dla innych zapytań niż `COUNT(*)`.** Wymaga nowych zapytań w obciążeniu.

## Poza zakresem

- **Złączenia, także tylko do odczytu.** Jedna tabela w zapytaniu to część definicji języka ([decyzja 0004](../adr/0004-dialekt-bez-zlaczen.md)).
- **Wspólna edycja dokumentów na żywo.** Takie edytory opierają się na CRDT tekstu (Yjs, Automerge), a Flat-SQL byłby dla nich tylko magazynem.
- **Synchronizacja z istniejącą bazą serwerową, na przykład PostgreSQL.** To zakres osobnych produktów (PowerSync, ElectricSQL). Serwer Flat-SQL jest własną bazą główną.
- **Dane większe niż pamięć, pełny SQL, analityka dużych plików, wątki wymagające nagłówków izolacji.** Zmieniłyby architekturę albo niszę projektu.
