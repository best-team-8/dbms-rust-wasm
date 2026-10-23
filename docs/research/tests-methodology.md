# Metodyka testów poprawności

Dokument określa, jak projektować, przeprowadzać i opisywać testy poprawności Flat-SQL – testy, które rozstrzygają, czy silnik dotrzymuje swoich gwarancji: trwałości zatwierdzonych danych, zbieżności replik, semantyki zapytań i stałego formatu plików. Ile te gwarancje kosztują, mierzą benchmarki według osobnej metodyki: [benchmark-scenario.md](benchmark-scenario.md). Opisy testów i ich wyniki: [tests-results.md](tests-results.md).

## 1. Test poprawności a benchmark

| | Test poprawności | Benchmark |
|---|---|---|
| Pytanie | czy gwarancja jest spełniona | ile kosztuje operacja |
| Wynik | zaliczony / niezaliczony, liczba sprawdzonych przypadków | rozkład czasu, pamięć, rozmiar |
| Powtarzalność | deterministyczna: każde uruchomienie daje ten sam wynik | statystyczna: wiele uruchomień, mediana i rozrzut |
| Zależność od maszyny | brak, poza jawnie opisanymi testami platformowymi | wynik dotyczy konkretnej maszyny |
| Uruchamianie | przy każdej zmianie, także w CI | na żądanie, w kontrolowanych warunkach |

Test, który mierzy czas, jest benchmarkiem. Jeśli poprawka ma koszt wydajnościowy (np. dodatkowe utrwalenie pliku), test sprawdza tylko jej poprawność, a koszt mierzy benchmark.

## 2. Poziomy testów

| Poziom | Gdzie | Zakres |
|---|---|---|
| Testy silnika | Rust, `crates/flatsql-core/tests/` | logika rdzenia: zapytania, WAL, snapshot, synchronizacja; magazyn w pamięci albo katalog tymczasowy, bez sieci i przeglądarki |
| Testy end-to-end | Node.js, `test/*.test.js` | moduł WASM, most JS, magazyn plikowy, prawdziwy serwer HTTP |
| Sprawdzenia w przeglądarce | ręczne, w aplikacji demo | zachowania zależne od przeglądarki (IndexedDB, trwały magazyn, Service Worker), których nie da się uruchomić w Node.js |
| Eksperymenty platformowe | jednorazowy program lub test | zachowanie systemu operacyjnego albo systemu plików, od którego zależy implementacja |

Test umieszcza się na najniższym poziomie, który obejmuje sprawdzany mechanizm. Sprawdzenie ręczne wymaga opisu krok po kroku, tak żeby inna osoba mogła je powtórzyć.

## 3. Zasady projektowania

1. **Jedno pytanie.** Test rozstrzyga jedną gwarancję, a jego nazwa ją wypowiada (np. `outbox_survives_restart_and_ack_is_durable`). Kilka testów odpowiadających na to samo pytanie tworzy grupę z jednym opisem.
2. **Jawny model awarii.** Przed napisaniem testu ustala się, co może się zepsuć (urwany zapis w dowolnym bajcie, utrata operacji nieutrwalonej przed awarią, zerwane połączenie, przesunięty zegar, różne wersje aplikacji) i co uznaje się za niezawodne. Test dowodzi tylko tego, co obejmuje model.
3. **Wyrocznia.** Kryterium poprawności jest zapisane przed uruchomieniem i nie zależy od testowanej implementacji: stan po ostatniej zatwierdzonej transakcji, prosty model referencyjny, stan innej repliki, identyczność bajtów.
4. **Determinizm.** Wynik nie zależy od zegara systemowego, kolejności wątków ani sieci: zegar i losowość są wstrzykiwane, ziarna stałe. Błąd odtwarza się jednym poleceniem.
5. **Pełne pokrycie, gdy jest możliwe.** Skończoną przestrzeń stanów (każdy prefiks pliku, każdy stan pośredni operacji na plikach) sprawdza się w całości. Gdy jest za duża, przypadki losuje się ze stałym ziarnem, a ich liczba jest częścią wyniku.
6. **Kontrola negatywna.** Test musi wykryć problem, który ma wykrywać: uruchomiony na kodzie sprzed poprawki albo z celowo wprowadzonym błędem (pominięte utrwalenie, odwrócona kolejność kroków, wyłączony warunek) kończy się niepowodzeniem. Test, który nigdy nie zawiódł, niczego nie dowodzi.
7. **Izolacja.** Każdy test pracuje na własnych danych (katalog tymczasowy, osobna baza IndexedDB, osobny proces serwera), sprząta po sobie i nie zależy od kolejności innych testów.
8. **Czas.** Testy uruchamiane przy każdej zmianie trwają najwyżej kilka sekund. Dłuższe przebiegi losowe mają parametr liczby przypadków: krótka wersja działa w CI, pełna jest uruchamiana ręcznie i opisana w wyniku.
9. **Platformy.** Wynik dotyczy tylko systemów, na których test uruchomiono. Zachowanie pozostałych platform opisuje się na podstawie literatury albo wpisuje do ograniczeń.

## 4. Techniki dla typowych obszarów

### 4.1 Trwałość i odtwarzanie po awarii

- **Urwany i uszkodzony zapis** – plik ucięty w każdym bajcie albo ze zmienionym bajtem. Oczekiwany wynik: odtworzenie do ostatniego zatwierdzonego stanu albo jawne odrzucenie uszkodzonego pliku, nigdy stan pośredni.
- **Stany pośrednie operacji na kilku plikach** – operację (np. zapis snapshotu, zmiana nazwy, obcięcie WAL) rozpisuje się na kroki. Dla każdego zbioru kroków, które system plików mógł utrwalić przed awarią, przy zachowaniu kolejności wymuszonej przez utrwalenie, odtworzenie musi dać stan zatwierdzony. Technika wzorowana na [S8].
- **Restart procesu** – zamknięcie i ponowne otwarcie bazy w trakcie pracy, także bez sieci.

Symulacja nie obejmuje rzeczywistej utraty zasilania ani błędów sprzętu i oprogramowania dysku; to ograniczenie każdego testu z tej grupy, nie jego wynik.

### 4.2 Synchronizacja replik

- **Repliki w jednym procesie** – test steruje wymianą paczek między replikami i serwerem bez sieci, więc ma pełną kontrolę nad kolejnością dostarczenia, podziałami sieci i powtórzeniami.
- **Scenariusze** – równoczesne edycje i usunięcia, podział sieci, ponowne wysłanie tej samej paczki, różne kolejności dostarczenia, przesunięte zegary urządzeń [C2].
- **Kryteria** – zbieżność: po pełnej wymianie wszystkie repliki i serwer mają identyczny stan [L2]; brak „wskrzeszeń” usuniętych wierszy; brak utraty zatwierdzonych zmian; ponowne przyjęcie tej samej paczki nie zmienia stanu.

### 4.3 Zapytania i format danych

- **Model referencyjny** – wynik zapytania porównywany z prostą implementacją w pamięci albo z innym silnikiem dla tych samych danych.
- **Deterministyczny format** – te same dane logiczne dają identyczne bajty snapshotu i paczek zmian, niezależnie od kolejności operacji i reprezentacji wartości w pamięci.
- **Zgodność formatu** – plik zapisany przez poprzednią wersję daje się odczytać, a plik niepoprawny jest odrzucany z błędem.

## 5. Opis testu

Każdy test albo grupa testów odpowiadająca na jedno pytanie ma opis w [tests-results.md](tests-results.md). Dokument zaczyna się środowiskiem (§6) i tabelą zbiorczą (test – pytanie – status), a opisy są pogrupowane według obszarów z §4.

| Element | Zawartość |
|---|---|
| Pytanie | jaką gwarancję test rozstrzyga – jednym zdaniem |
| Uzasadnienie | skąd potrzeba: problem w kodzie, praca z literatury, zastosowanie |
| Model awarii i założenia | co jest symulowane, co uznano za niezawodne, czego test świadomie nie obejmuje |
| Procedura | plik i nazwa testu, dane, ziarno i liczba przypadków, wyrocznia, polecenie; sprawdzenie ręczne krok po kroku |
| Kontrola negatywna | na jakim kodzie albo z jakim wprowadzonym błędem test zawiódł i z jakim komunikatem |
| Wynik | status, liczby (przypadki, czas przebiegu), data, środowisko, commit (przez temat) |
| Wpływ na projekt | co zmieniono w kodzie lub w dokumentacji w wyniku testu |
| Ograniczenia | czego wynik nie dowodzi i jaki test mógłby to uzupełnić |

**Status:** *zaliczony* – gwarancja potwierdzona w całym modelu awarii; *częściowy* – potwierdzona w części modelu, brakująca część jest w ograniczeniach; *niezaliczony* – test obalił gwarancję. Wynik negatywny zostaje w dokumencie razem z przyczyną i dalszym postępowaniem.

Opis ma się dać przenieść do pracy bez przeróbek: odwołuje się do literatury przez identyfikatory z [indeksu](../references/README.md), do kodu przez plik i nazwę testu, a nie odwołuje się do rejestru decyzji ani list zadań. Komentarz w kodzie testu jest krótki i opisuje scenariusz.

## 6. Środowisko i odtwarzalność

Na początku [tests-results.md](tests-results.md): system operacyjny i system plików, procesor, pamięć, wersje Rust, Node.js i przeglądarki, a także CI (`.github/workflows/build-test.yml`, `ubuntu-latest`). Wynik każdego testu podaje datę i commit, na którym go uzyskano.

```bash
cargo test --workspace                          # testy silnika
cargo test -p flatsql-core --test sync <nazwa>  # pojedynczy test silnika
npm run build && npm test                       # moduł WASM, most JS, testy end-to-end
node --test test/<plik>.test.js                 # pojedynczy plik testów end-to-end
```

## 7. Zagrożenia dla trafności

- Symulacja awarii przyjmuje model systemu plików; rzeczywiste systemy mogą utrwalać mniej albo w innej kolejności [S8].
- Przebieg losowy sprawdza tylko wylosowane przypadki – brak błędu nie dowodzi poprawności.
- Sprawdzenia w przeglądarce obejmują jedną przeglądarkę w jednej wersji.
- Test i implementację pisze ta sama osoba, więc mogą dzielić błędne założenie. Ryzyko ograniczają kontrola negatywna i wyrocznia niezależna od implementacji.
