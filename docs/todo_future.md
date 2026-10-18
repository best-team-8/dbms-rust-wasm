# Zadania na przyszłość – Flat-SQL jako baza dla aplikacji z backendem

Zadania wykraczające poza obecny etap pracy. Opisują, czego brakuje, żeby silnik mógł działać obok dowolnego backendu (np. FastAPI w Pythonie) i synchronizować dane z aplikacjami klienckimi wielu użytkowników. Bieżące zadania: [todo.md](todo.md). Wnioski z analizy projektu: [references/conclusions.md](references/conclusions.md). Identyfikatory w nawiasach (np. [L20]) odsyłają do [indeksu literatury](references/README.md).

Nakład: **mały** – pojedyncze funkcje i testy; **średni** – kilka modułów, nowe testy; **duży** – nowy podsystem.

## Scenariusz docelowy

```text
przeglądarka (FlatDB + SyncClient)
      │ synchronizacja + token
      ▼
backend, np. FastAPI (logowanie, uprawnienia) ──przekazuje──► serwer Flat-SQL (kontener)
      │                                                             ▲
      └── logika biznesowa: odczyty i zapisy przez API ─────────────┘
```

Już działa: nowy klient bez kursora dostaje od serwera pełny stan bazy (`/sync/pull`), a potem `SyncClient` synchronizuje zmiany w obie strony; zapisy wykonane przez `/api/query` trafiają do logu serwera, więc dostają je klienci.

Możliwe są dwa warianty:

| Wariant | Opis | Stan |
|---|---|---|
| A. Kontener obok backendu | obecny `server/server.js` w osobnym kontenerze, backend jako bramka i klient `/api/query` | do testów działa już dziś (sekcja 3) |
| B. Silnik w procesie backendu | rdzeń w Ruście jako moduł Pythona, synchronizacja obsługiwana przez FastAPI | wymaga wiązań (sekcja 4) |

> **Uwaga bezpieczeństwa:** `/api/query` wykonuje dowolny SQL bez logowania. Do czasu zadań z sekcji 2 nie wolno go wystawiać publicznie – tylko w sieci wewnętrznej, dla backendu.

## 1. Rozdzielenie danych użytkowników (najważniejsze)

Dziś serwer ma jedną bazę dla wszystkich: każdy klient wysyła do niej swoje zmiany i pobiera zmiany wszystkich pozostałych.

- [ ] **1.1 Osobna baza na użytkownika** (średni) – mapa `użytkownik → baza + log + epoka`, ładowanie z dysku przy pierwszym żądaniu, zwalnianie nieużywanych baz z pamięci, osobny katalog danych dla każdego użytkownika.
- [ ] **1.2 Replikacja częściowa** (duży) – alternatywa dla 1.1 przy danych współdzielonych: reguły, które wiersze trafiają do którego klienta (np. kolumna właściciela), filtrowanie w `/sync/pull` i odrzucanie w `/sync/push`. [L20].
- [ ] **1.3 Przestrzenie współdzielone** (średni) – baza zespołu, pokoju gry lub dokumentu z listą członków i ich uprawnieniami. [L18].
- [ ] **1.4 Testy izolacji** (mały) – klient użytkownika A nigdy nie otrzymuje wierszy użytkownika B, także po kompaktowaniu logu i przy pełnym stanie.
- [ ] **1.5 Pomiar przy wielu użytkownikach** (mały) – pamięć serwera, czas ładowania bazy użytkownika, czas pierwszej synchronizacji.

## 2. Uwierzytelnianie i reguły po stronie serwera

- [ ] **2.1 Tokeny** (średni) – backend wydaje i sprawdza token; serwer synchronizacji przyjmuje tylko żądania z poprawnym tokenem albo jest dostępny wyłącznie przez bramkę backendu. Po stronie przeglądarki `SyncClient` przyjmuje własną funkcję `fetch`, więc nagłówek z tokenem można dodać bez zmian w bibliotece. Rozszerza [todo.md 4.4](todo.md). [L18].
- [ ] **2.2 Walidacja zmian w `/sync/push`** (średni) – tabele i kolumny tylko do odczytu dla klientów (np. ceny), sprawdzanie właściciela wiersza, zakresy wartości; odrzucone zmiany raportowane klientowi tak jak dziś `rejected`. Serwer jako autorytet porządkujący zmiany: [C3]; więzy przy słabej spójności: [L8].
- [ ] **2.3 Kompensacja odrzuconych zmian** (średni) – serwer odsyła poprawną wartość z nowszym znacznikiem, żeby replika klienta wróciła do dozwolonego stanu (wzór: cofanie zmian naruszających więzy przy scalaniu w CRR [L24]).
- [ ] **2.4 Gry i dane wrażliwe na oszustwa** (średni) – dane wpływające na rozgrywkę zmieniane tylko przez serwer; klient dostaje je jako tylko do odczytu.
- [ ] **2.5 `/api/query` tylko dla backendu** (mały) – osobny port lub klucz API, wyłączanie tego endpointu w konfiguracji.
- [ ] **2.6 Schemat zarządzany przez backend** (mały) – klienci bez prawa do `CREATE/DROP TABLE` i `INDEX` w synchronizacji. Powiązane z [todo.md 1.7 i 3.5](todo.md).

## 3. Docker i instrukcja wdrożenia

- [ ] **3.1 Dockerfile** (mały) – obraz Node.js 22; moduł `pkg/` budowany w osobnym etapie (Rust + `wasm-pack`) albo kopiowany gotowy; uruchomienie `node server/server.js --host 0.0.0.0 --port 8080 --data /data` (serwer domyślnie słucha tylko na `127.0.0.1`), wolumen na `/data`, healthcheck przez `/api/status`.
- [ ] **3.2 Tryb bez dema** (mały) – opcja wyłączająca hosting aplikacji `web/`, żeby kontener udostępniał tylko synchronizację i API.
- [ ] **3.3 Konfigurowalny CORS** (mały) – dziś `access-control-allow-origin: *`.
- [ ] **3.4 Przykład `docker-compose` z FastAPI** (średni) – backend jako bramka: logowanie, przekazywanie `/sync/*` do kontenera Flat-SQL, logika biznesowa przez `/api/query` w sieci wewnętrznej.
- [ ] **3.5 Instrukcja wdrożenia** (mały) – po polsku: uruchomienie, parametry, kopie zapasowe katalogu danych (`snapshot.fsql`, `wal.fsql`), aktualizacja wersji obrazu.

## 4. Wiązania dla Pythona przez interfejs C

- [ ] **4.1 Interfejs C nad `flatsql-core`** (średni) – biblioteka `cdylib`: otwarcie bazy z katalogu, `execute`, przygotowane instrukcje, `export_outbox` / `ack_outbox` / `apply_changes` / `export_state`, zwalnianie pamięci. Ten sam interfejs posłuży panelowi WinUI (README, prezentacja projektu).
- [ ] **4.2 Pakiet Pythona** (średni) – przez `ctypes`/`cffi` na interfejsie C albo PyO3 + maturin; paczki dla Windows, Linuksa i macOS.
- [ ] **4.3 Synchronizacja w FastAPI** (średni) – `/sync/push` i `/sync/pull` zaimplementowane w Pythonie na tych wiązaniach (przeniesienie logiki z `server/server.js`: log, epoka, kursory, kompaktowanie); wtedy kontener Node.js nie jest potrzebny.
- [ ] **4.4 Testy zgodności** (mały) – ten sam klient przeglądarkowy synchronizuje się identycznie z serwerem Node.js i z FastAPI (te same paczki `FSB1`).
- [ ] **4.5 Inne języki** (średni) – ten sam interfejs C dla Go lub C#.

## 5. Skala i trwałość serwera

- [ ] **5.1 Trwały log synchronizacji** (średni) – dziś log jest tylko w pamięci, więc po restarcie zaczyna się nowa epoka i klienci pobierają pełny stan. Wspólne z [todo.md 4.10](todo.md).
- [ ] **5.2 Zarządzanie pamięcią** (średni) – limit baz w pamięci, zrzucanie nieużywanych (powiązane z 1.1).
- [ ] **5.3 Wiele instancji serwera** (duży) – przypisanie użytkowników do instancji (np. po identyfikatorze) albo węzły edge synchronizowane z bazą główną ([todo.md 4.10–4.11](todo.md)).
- [ ] **5.4 Monitoring** (mały) – liczba klientów, rozmiar logu, czas `push`/`pull`, liczba odrzuconych zmian (rozszerzenie `/api/status`).

## 6. Integracja z istniejącą bazą główną (np. PostgreSQL)

Jeśli backend ma już własną główną bazę, Flat-SQL staje się warstwą synchronizacji obok niej.

- [ ] **6.1 Łącznik PostgreSQL ↔ Flat-SQL** (duży) – przekazywanie zmian z PostgreSQL do Flat-SQL (przechwytywanie zmian, np. przez replikację logiczną) i z Flat-SQL do PostgreSQL z walidacją. Wzorzec: Change Data Capture ([P1] rozdz. 11); rozwiązania tej klasy: ElectricSQL, PowerSync ([Q11]).
- [ ] **6.2 Mapowanie typów i kluczy** (średni) – typy PostgreSQL na `INT/FLOAT/TEXT/BOOL`, klucze złożone na jednokolumnowe.
- [ ] **6.3 Konflikty między źródłami** (średni) – znaczniki HLC dla zapisów pochodzących z PostgreSQL, żeby LWW rozstrzygało je spójnie z zapisami klientów.
