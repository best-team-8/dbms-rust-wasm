# Zbieżność urządzeń w trudnych warunkach

Test sprawdza, że wszystkie urządzenia i serwer kończą w tym samym stanie, wyznaczonym przez najnowsze wysłane zmiany. Warunki: zegary urządzeń są przesunięte, sieć odcina urządzenia, wiersze są usuwane w trakcie przerwy, odpowiedzi serwera giną, a aplikacje są restartowane.

## Dlaczego

Reguła „ostatni zapis wygrywa” zbiega się niezależnie od kolejności i powtórzeń dostarczenia (Shapiro i in., 2011). Prawdziwe urządzenia mają jednak rozjechane zegary, a urządzenie z zegarem w przyszłości wygrywałoby każdy konflikt (Kulkarni i in., 2014). Wcześniejszy test losowy używał zegara logicznego i obejmował tylko przerywaną łączność.

## Model

Cztery urządzenia i serwer, jedna tabela, 12 kluczy. Czas symulowany rośnie o 0–1,5 s na krok. Urządzenie 1 spieszy się o 40 s, urządzenie 2 spóźnia o 40 s (każde w granicy 60 s wobec serwera). Urządzenie 3 spieszy się o godzinę przez sto kroków, potem ma zegar poprawiony.

Kroki są losowe: zapis, zmiana, usunięcie, runda synchronizacji, utrata albo odzyskanie sieci, restart z WAL i snapshotu, checkpoint. Odpowiedź serwera na wysłanie ginie w 10% przypadków, a zapis kursora w 10% pobrań. W każdym przebiegu są zawsze dwa zdarzenia: urządzenie z przyszłości próbuje się synchronizować, a jedno urządzenie usuwa wiersz, który odcięte urządzenie właśnie zmieniło. Urządzenia stosują protokół klienta z [decyzji 0020](../adr/0020-granica-dryfu-zegara.md).

Model nie obejmuje restartu serwera, zmian struktury tabel ani prawdziwej sieci. Restart serwera z nową epoką sprawdza [test pracy offline](offline-work.md).

## Jak

Dla każdego ziarna 400 kroków, potem faza końcowa: wszystkie urządzenia w sieci z poprawnym zegarem, rundy aż kolejki będą puste. Wzorzec jest liczony z wysłanych paczek, niezależnie od silnika: dla każdego klucza zmiana o największym znaczniku spośród przyjętych przez serwer. Usunięcie daje brak wiersza, więc wzorzec wyklucza też przywrócenie usuniętego wiersza. Po każdej rundzie zegar serwera może być najwyżej 60 s przed czasem rzeczywistym.

```bash
cargo test -p flatsql-core --test sync_faults
```

Pełny przebieg: `FLATSQL_SYNC_SEEDS=1000 cargo test -p flatsql-core --release --test sync_faults`.

Granicę dryfu sprawdzają też osobne testy: dwa w `crates/flatsql-core/tests/sync.rs` i dwa w `test/clock.test.js` (prawdziwy serwer i `SyncClient`, zegar przesunięty o godzinę w przód i w tył). Ręcznie w przeglądarce: po przesunięciu `Date.now` o godzinę aplikacja pokazuje odchylenie „+3600,0 s (limit 60 s)” i wstrzymuje synchronizację, a po przywróceniu zegara nadaje zmianom nowe znaczniki i je wysyła.

## Wynik

| W przebiegu | 40 ziaren | 1000 ziaren |
|---|---:|---:|
| zapisy i zmiany | 7 994 | 199 795 |
| usunięcia (w tym w trakcie przerwy innego urządzenia) | 2 485 (2 336) | 61 505 (55 958) |
| rundy synchronizacji | 2 248 | 58 461 |
| rundy wstrzymane przez zegar | 133 | 3 347 |
| zmiany z nowymi znacznikami | 384 | 9 591 |
| utracone potwierdzenia / kursory | 156 / 196 | 3 481 / 5 141 |
| restarty / odcięcia od sieci | 737 / 493 | 19 865 / 12 035 |

We wszystkich przebiegach serwer i każde urządzenie po restarcie mają stan równy wzorcowi. Przebieg trwa 0,7 s dla 40 ziaren i 2,9 s dla 1000. Pomiar z 23.10.2026.

## Kontrola negatywna

Każdy z czterech celowo wprowadzonych błędów test wykrywa już przy 40 ziarnach:

- bez granicy dryfu zegar serwera przesuwa się w przyszłość;
- klient, który potwierdza paczkę odrzuconą za zegar, zostaje z wierszami, których nie ma nikt inny;
- zmiana zdalna bez porównania ze znacznikiem usunięcia przywraca usunięty wiersz;
- bez nowych znaczników po poprawie zegara synchronizacja nigdy się nie kończy.

## Skutek

Kod przeszedł bez zmian. Kontrola negatywna pokazała, że zbieżność wymaga czterech mechanizmów: granicy dryfu, niepotwierdzania odrzuconej paczki, nowych znaczników po poprawie zegara i znaczników usunięcia. Zgubienie współbieżnej zmiany z mniejszym znacznikiem jest zamierzonym skutkiem reguły, a nie błędem.
