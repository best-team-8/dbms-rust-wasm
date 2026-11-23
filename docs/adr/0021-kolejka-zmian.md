# 0021. Kolejka zmian do wysłania odbudowywana z WAL i prowadzona od włączenia synchronizacji

- Status: obowiązuje
- Ostatnia zmiana: 2026-10-25

## Kontekst

Zmiany wykonane offline muszą czekać na wysłanie, przetrwać zamknięcie aplikacji i nie zostać wysłane dwa razy. Bez takiego mechanizmu aplikacje mobilne tracą zmiany przy przerwie w sieci (Go i in., 2015). W prototypie kolejka miała wpis dla każdego zmienionego wiersza także w aplikacji bez synchronizacji. Odpowiadała za około połowę czasu odtworzenia bazy i jedną czwartą rozmiaru snapshotu.

## Decyzja

1. Kolejka zmian (outbox) trzyma tylko odwołanie do wiersza: tabelę, klucz i numer kolejny. Przy wysyłce odczytywany jest bieżący stan wiersza albo znacznik usunięcia, więc wiele zmian jednego wiersza daje jedną zmianę do wysłania. Zmiany schematu są przechowywane w całości.
2. Każda lokalna operacja w WAL ma numer kolejny, a potwierdzenie serwera trafia do WAL. Kolejka jest odbudowywana przy odtwarzaniu bazy, więc nie potrzebuje osobnego magazynu.
3. Zmiany pobrane z serwera nie trafiają do kolejki, więc nie wracają na serwer.
4. Baza ma flagę synchronizacji zapisaną w WAL i snapshocie. Dopóki synchronizacja jest wyłączona, kolejka przyjmuje tylko zmiany schematu. Włączenie oznacza wszystkie wiersze i znaczniki usunięcia jako oczekujące, więc pierwsza wysyłka zawiera cały stan. `SyncClient` włącza synchronizację przy utworzeniu, serwer przy starcie.
5. Snapshot zapisuje kolejkę zwięźle: wiersze pogrupowane po tabeli, nazwa tabeli raz.

## Uzasadnienie

- To klasyczny wzorzec pracy bez połączenia: trwały log zmian, w którym nowszy zapis unieważnia wcześniejsze wpisy, odtwarzany po połączeniu (Kistler i Satyanarayanan, 1992).
- Wysyłanie bieżącego stanu zmienionych elementów daje ten sam wynik co przesłanie wszystkich operacji (synchronizacja delta-state, Almeida, 2024). Dla bazy, która jeszcze nie synchronizowała, cały stan jest dokładnie tym, co zawierałaby kolejka prowadzona od początku.
- Zmierzone natywnie: odtworzenie bazy bez synchronizacji 111 → 55 ms, snapshot 5,0 → 3,7 MiB, wstawienie −22%, `UPDATE` −23%, pamięć po wczytaniu 73 → 55 MiB.

## Odrzucone warianty

- Kolejka jako osobna tabela albo magazyn. Dodatkowy zapis przy każdej zmianie i ryzyko rozjazdu z danymi.
- Kolejka wszystkich operacji. Rośnie bez potrzeby i powtarza stany pośrednie.
- Odtworzenie zmian sprzed włączenia z WAL. WAL jest obcinany przy checkpoincie.

## Konsekwencje

- Niewysłane zmiany przetrwają restart aplikacji offline, ale nie usunięcie danych przez przeglądarkę.
- Wiersz zmieniony ponownie przechodzi na koniec kolejki, więc zmiany jednej transakcji mogą trafić do różnych paczek (decyzja 0019).
- Aplikacja, która wysyła kolejkę inaczej niż przez `SyncClient`, musi sama wywołać `enableSync()`.

## Historia

- 2026-10-06: kolejka jako odwołania do wierszy, odbudowywana z WAL.
- 2026-10-25: kolejka prowadzona dopiero od włączenia synchronizacji, zwarty zapis w snapshocie.
