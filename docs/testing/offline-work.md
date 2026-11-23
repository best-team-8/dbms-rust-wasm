# Praca offline

Test sprawdza główną obietnicę projektu: aplikacja czyta i zapisuje bez serwera, niewysłane zmiany przetrwają zamknięcie aplikacji, a gdy serwer wróci, wszystkie urządzenia mają ten sam stan (Kleppmann i in., 2019). Ma dwie części: automatyczny test end-to-end z prawdziwym serwerem i ręczny scenariusz w przeglądarce.

## Test end-to-end

Plik `test/offline-sync.test.js`. Prawdziwy serwer HTTP i klienci `FlatDB` z `SyncClient`:

1. dwa urządzenia synchronizują dane przez serwer;
2. serwer zostaje wyłączony; 50 zapisów na urządzeniu A i edycja na urządzeniu B kończą się lokalnie, odczyty działają, a w kolejce A czeka 51 zmian;
3. aplikacja A zostaje zamknięta i otwarta ponownie offline; kolejka i dane są na miejscu;
4. serwer startuje ponownie jako nowy proces z tym samym katalogiem danych i nową epoką; po synchronizacji kolejki są puste, a A, B i serwer mają identyczną tabelę;
5. oba urządzenia offline edytują ten sam wiersz; wszędzie wygrywa późniejsza edycja, niezależnie od kolejności ponownego połączenia;
6. zapisy przez API serwera docierają do urządzeń.

```bash
npm run build && node --test test/offline-sync.test.js
```

## Scenariusz w przeglądarce

Node.js nie uruchamia Service Workera ani prawdziwej IndexedDB, dlatego scenariusz jest ręczny:

1. `npm run build && npm run start:persistent`, w przeglądarce bez danych strony otworzyć `http://127.0.0.1:8080`, wczytać dane i wykonać zapytania;
2. włączyć „Symuluj brak sieci” i wykonać kilka zapisów; licznik oczekujących zmian rośnie;
3. zatrzymać serwer, wykonać zapis i odświeżyć stronę; po starcie wykonać odczyt i zapis;
4. uruchomić serwer ponownie, bez odświeżania strony, i sprawdzić przez `/api/query`, że zmiany dotarły.

Test `the Service Worker caches at install every file the page loads` w `test/browser.test.js` sprawdza automatycznie, że Service Worker zapisuje przy instalacji wszystkie pliki ładowane przez stronę.

## Wynik

Scenariusz zaliczony 29.10.2026 w Chromium 152 na wersji `v2-optymalizacja` z poprawką Service Workera:

- 2000 przedmiotów wczytanych w 38,5 ms, zapytania 1,0–1,7 ms;
- zapisy offline 0,1–1,1 ms;
- po odświeżeniu bez serwera aplikacja uruchomiła się z pamięci podręcznej i odtworzyła bazę z IndexedDB w 4,4 ms, z 4 oczekującymi zmianami;
- około 7 s po starcie serwera klient sam wysłał 5 zmian, a serwer zwrócił te same wiersze co baza lokalna.

Strona nie była izolowana, więc czasy mają dokładność 0,1 ms.

## Kontrola negatywna

Pierwsze powtórzenie scenariusza na wersji `v2-optymalizacja` zawiodło w kroku 3. Po odświeżeniu bez serwera strona wczytała się z pamięci podręcznej, ale silnik się nie uruchomił, bo dwóch plików JavaScript wiązań WebAssembly nie było w pamięci podręcznej Service Workera. Pliki te pojawiły się w module razem z optymalizacją wyniku, a lista plików Service Workera się nie zmieniła. Dane nie zginęły: po powrocie serwera i odświeżeniu aplikacja odtworzyła bazę i wysłała zmiany. Test Service Workera w `test/browser.test.js` na kodzie sprzed poprawki wskazuje te same dwa pliki.

## Ograniczenia

- Jedna przeglądarka na jednym komputerze.
- Scenariusz nie obejmuje utraty bazy serwera. Serwer uruchomiony bez katalogu danych (`npm start`) nie zna tabel i odrzuca ich wiersze, a klient potwierdza paczkę bez zgłoszenia konfliktu, więc zmiany zostają tylko na urządzeniu. Praca offline wymaga serwera z katalogiem danych.
