# ADR-0035: Granica dryfu zegara – odrzucenie, wstrzymanie synchronizacji i nowe znaczniki

| | |
|---|---|
| Status | przyjęta |
| Data | 2026-10-23 |
| Rodzaj | architektura |
| Powiązane | ADR-0002, ADR-0015 (realizuje konsekwencję o granicy dryfu), ADR-0016, ADR-0017, ADR-0028; todo 1.1, 1.9; future 2.3 |

## Kontekst

Znacznik zmiany pochodzi z hybrydowego zegara logicznego (HLC), który wchłania znaczniki innych replik (ADR-0015). Urządzenie z zegarem ustawionym w przyszłość wygrywało więc każdy konflikt LWW, a jego czas rozlewał się na serwer i wszystkie repliki: każda, która odebrała jego zmianę, zaczynała nadawać znaczniki z przyszłości. Kulkarni i in. [C2] (§4) ograniczają to granicą ε: wiadomość ze znacznikiem dalej niż ε przed zegarem fizycznym odbiorcy nie jest wchłaniana, a sytuacja jest zgłaszana. Samo odrzucenie zostawia pytanie, co robi urządzenie, którego zmiany odrzucono: porzucone zmiany oznaczają utratę pracy offline albo trwały rozjazd repliki, a ponawiane – wysyłanie bez końca, bo znacznik się nie zmienia. Wybór między wariantami o różnych gwarancjach należał do autora projektu (23.10.2026).

## Decyzja

1. Stała `MAX_DRIFT_MS` = 60 s. `Hlc::observe` nie wchłania znacznika dalszego niż 60 s przed zegarem ściennym; `apply_changes` liczy taką zmianę jako odrzuconą (`clock_drift`), nie stosuje jej i nie przesuwa zegara. Własna historia (snapshot, WAL) jest wchłaniana bez granicy.
2. Zegar serwera jest wzorcem: każda odpowiedź `/sync/*` ma nagłówek `x-server-time`, `/sync/push` zwraca `clockDrift`, a `GET /sync/time` służy do samego sprawdzenia zegara.
3. Klient przed pierwszą wysyłką i w każdej odpowiedzi mierzy odchylenie swojego zegara od zegara serwera. Gdy przekracza 60 s albo serwer odrzucił zmiany za dryf, klient **wstrzymuje synchronizację**: nie potwierdza paczki (zostaje w outboksie), nie pobiera zmian, ustawia stan `clock` i zgłasza zdarzenie; kolejne rundy sprawdzają tylko zegar.
4. Po poprawieniu zegara `Database::rebase_clock` nadaje **nowe znaczniki** zmianom z outboksu, których znacznik wykracza poza granicę, i cofa HLC do najnowszego znacznika, który zostaje (wszystko spoza tych zmian mogło trafić do innych replik). Nowe znaczniki nie są logowane w WAL, więc host od razu wykonuje checkpoint (`FlatDB.rebaseClock`, `Persistent::rebase_clock`).
5. Zmiany pobrane z serwera, które są dalej niż 60 s przed zegarem klienta, nie przesuwają kursora i są pobierane ponownie.

## Uzasadnienie

- Granica dryfu i odrzucenie zamiast wchłonięcia to rozwiązanie z pracy o HLC [C2]. 60 s to wartość przyjęta w zadaniu; jej wpływu nie mierzono.
- Wstrzymanie zachowuje pracę offline (ADR-0002): zmiany czekają w outboksie tak jak przy braku sieci, a po poprawie zegara trafiają do wszystkich replik. Wysyłka nie krąży bez końca, bo wstrzymany klient niczego nie wysyła.
- Cofnięcie zegara jest bezpieczne tylko dla znaczników, których nikt poza urządzeniem nie zna. Znaczniki spoza granicy zostały odrzucone przez serwer, więc nie istnieją nigdzie indziej; pozostałe znaczniki ograniczają cofnięcie od dołu, więc porządek LWW między replikami się nie zmienia.
- Testy (`crates/flatsql-core/tests/sync.rs`, `test/clock.test.js`, [tests-results.md §2.1](../research/tests-results.md#21-granica-dryfu-zegara)) pokazały na kodzie sprzed zmiany przyjęcie zmiany z zegara przesuniętego o godzinę, a na kliencie bez wstrzymania – potwierdzenie i utratę odrzuconej paczki.

## Odrzucone alternatywy

- **Odrzucenie bez cofania** – klient usuwa odrzucone zmiany z outboksu; jego wiersze ze znacznikiem z przyszłości wygrywają lokalnie z każdą późniejszą zmianą innych replik, więc rozjazd jest trwały, a HLC urządzenia zostaje w przyszłości, aż dogoni go czas rzeczywisty.
- **Odrzucenie i cofnięcie lokalne** (usunięcie lokalnych wersji i pobranie stanu serwera) – zbieżne, ale praca wykonana z błędnym zegarem przepada; to ogólna kompensacja odrzuconych zmian, planowana w future 2.3.
- **Korekta zegara czasem serwera** (przesunięcie zegara HLC o zmierzone odchylenie) – urządzenie nie musiałoby poprawiać zegara, ale znaczniki zależałyby od pomiaru przez sieć, a urządzenie, które nigdy nie połączyło się z serwerem, i tak nie miałoby korekty; poza zakresem.
- **Ponawianie odrzuconych zmian** – wysyłanie bez końca, bez szansy na przyjęcie, dopóki zegar ścienny nie dogoni znacznika.

## Konsekwencje

- Urządzenie z błędnym zegarem nie synchronizuje się, dopóki zegar nie zostanie poprawiony; aplikacja dostaje stan `clock` i zdarzenie `clock` z odchyleniem.
- Dwa urządzenia mogą mieć zegary różniące się prawie o 2 × 60 s, każde w granicy wobec serwera. Zmiana z jednego bywa wtedy odrzucana przez drugie, które pobiera ją ponownie, aż jego zegar ją dogoni (najwyżej 60 s).
- Protokół ma nowy endpoint i nagłówek; starszy klient nadal działa, ale potwierdza odrzucone paczki.
- Wartość 60 s jest stałą; konfiguracja per przestrzeń – w razie potrzeby razem z future §2.
- Testy synchronizacji w trudnych warunkach (todo 1.9) obejmują urządzenie z zegarem w przyszłości.

## Źródła

[C2] §4, [P1] rozdz. 8; [tests-results.md §2.1](../research/tests-results.md#21-granica-dryfu-zegara).
