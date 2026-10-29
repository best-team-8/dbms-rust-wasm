# server – serwer synchronizacji i węzeł edge

Serwer Node.js z tym samym modułem WebAssembly co przeglądarka. Pełni trzy role: węzeł synchronizacji, do którego klienci wysyłają i z którego pobierają paczki zmian, zdalne API zapytań używane w badaniach jako model Cloud-First oraz serwer plików aplikacji demonstracyjnej. To prototyp: wystarcza do walidacji pracy offline, ale nie ma uwierzytelniania ani podziału danych między użytkowników, więc nasłuchuje tylko na `127.0.0.1`. Docelowy serwer synchronizacji opisuje [docs/future/README.md §5](../docs/future/README.md#5-serwer-synchronizacji-projekt-docelowy).

## Uruchomienie

```bash
npm run build
```

```bash
npm start
```

`npm start` uruchamia serwer pod adresem <http://127.0.0.1:8080> z bazą w pamięci, a `npm run start:persistent` zapisuje dane w `server/data/` (ten sam format co CLI). Opcje `node server/server.js`:

| Opcja | Domyślnie | Znaczenie |
|---|---|---|
| `--port` | 8080 | port HTTP |
| `--host` | `127.0.0.1` | adres nasłuchu |
| `--data KATALOG` | brak (pamięć) | katalog bazy serwera (`FsStore`) |
| `--latency MS` | 0 | symulowane opóźnienie każdej odpowiedzi |

## Protokół synchronizacji

| Żądanie | Treść | Odpowiedź |
|---|---|---|
| `POST /sync/push?client=ID` | binarna paczka zmian z outboxu klienta | JSON `{applied, ignored, rejected, clockDrift, errors, conflicts, seq, serverTime}` |
| `GET /sync/pull?since=N&epoch=E&client=ID` | – | binarna paczka zmian; nagłówki `X-Sync-Cursor`, `X-Sync-Epoch` |
| `GET /sync/time` | – | JSON `{serverTime}`; klient sprawdza nim swój zegar |

Każda odpowiedź `/sync/*` ma nagłówek `X-Server-Time` z zegarem serwera w milisekundach. Serwer utrzymuje własną replikę i log przyjętych paczek z numerami przyjęcia `seq` ([ADR-0017](../docs/adr/0017-protokol-push-pull-z-kursorem.md), [ADR-0028](../docs/adr/0028-kursor-jako-numer-przyjecia.md)). Pull zwraca konkatenację paczek po kursorze, bez własnych zmian klienta. Gdy kursor jest nieznany (nowy klient, restart serwera z nową epoką, log skompaktowany po 2000 wpisach), serwer odsyła pełny stan bazy, który dzięki regule „ostatni zapis wygrywa” można bezpiecznie zastosować ponownie. Format paczki: [storage/README.md](../crates/flatsql-core/src/storage/README.md#formaty-binarne); zachowanie klienta: [bridge/README.md](../bridge/README.md#synchronizacja-syncclient).

## API zapytań i tryb edge

| Żądanie | Działanie |
|---|---|
| `POST /api/query` | wykonuje SQL z treści żądania na bazie serwera i zwraca wynik jako JSON; parametr `latency` symuluje opóźnienie sieci, `publish=0` zostawia zapis poza logiem synchronizacji (dane benchmarku) |
| `GET /api/status` | epoka, numer ostatniej paczki, rozmiar logu, statystyki serwera i bazy |

Zapisy przez `/api/query` trafiają do logu synchronizacji, więc docierają do klientów. Serwer z tym samym silnikiem odpowiada wdrożeniu na węźle brzegowym; węzłów synchronizowanych z bazą główną jeszcze nie ma. `/api/query` wykonuje dowolny SQL bez logowania i nie wolno go wystawiać publicznie.

Pozostałe ścieżki serwer obsługuje jako pliki: aplikację z [web/](../web/), most z [bridge/](../bridge/) i moduł z `pkg/`.

## Testy

Testy end-to-end uruchamiają prawdziwy proces serwera i klientów `FlatDB` + `SyncClient`: [test/offline-sync.test.js](../test/offline-sync.test.js) i [test/clock.test.js](../test/clock.test.js) ([test/README.md](../test/README.md)).

## Ograniczenia

Log synchronizacji jest tylko w pamięci, więc każdy restart serwera wysyła wszystkim klientom pełny stan. Restart bez katalogu danych usuwa też bazę serwera: zmiany wierszy tabel, których serwer już nie zna, są odrzucane, a klient potwierdza je bez zgłoszenia konfliktu, więc zostają tylko na urządzeniu ([tests-results.md §2.5](../docs/research/tests-results.md#25-praca-offline-w-aplikacji-demo)). Pozostałe ograniczenia: jedna baza dla wszystkich klientów, brak uwierzytelniania i reguł, nagłówek CORS `*`. Kierunki: [docs/future/README.md §1.5](../docs/future/README.md#15-serwer-synchronizacji).
