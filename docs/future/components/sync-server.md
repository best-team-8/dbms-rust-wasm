# Serwer synchronizacji – projekt docelowy

Opis docelowego serwera synchronizacji Flat-SQL: z czego się składa, jak działa i z jakich decyzji wynika. Zbiera ustalenia z rejestru decyzji ([ADR](../../adr/README.md)) i z list zadań ([todo.md](../../todo.md), [todo_future.md](../../todo_future.md)). Stan na 21 października 2026.

**Zakres.** To kierunek rozwoju po pracy inżynierskiej. Karta pracy wymaga tylko walidacji pracy offline, na którą wystarcza obecny prototyp ([server/server.js](../../../server/server.js), test [test/offline-sync.test.js](../../../test/offline-sync.test.js)). Różnice między prototypem a projektem – §13.

## 1. Rola

Serwer synchronizacji to osobna usługa uruchamiana obok backendu aplikacji. Przechowuje zmiany z lokalnych baz Flat-SQL na urządzeniach i oddaje je pozostałym urządzeniom, które mają dostęp do tych samych danych.

**Czym nie jest:**

- nie zna użytkowników – nie ma kont, haseł ani logowania (ADR-0025);
- nie zawiera logiki biznesowej – reguły pisze programista w backendzie (ADR-0026);
- nie jest główną bazą aplikacji – dane porównywane między użytkownikami (np. ranking) zwykle trzyma baza backendu (ADR-0002 pkt 3).

Synchronizacja jest dla programisty opcją: `FlatDB` działa bez serwera, a `SyncClient` dokłada się tylko wtedy, gdy dane mają trafiać na inne urządzenia.

## 2. Miejsce w architekturze

```text
frontend (FlatDB + SyncClient)
   │  POST /sync/push, GET /sync/pull + sesja użytkownika (np. ciasteczko)
   ▼
backend aplikacji = bramka (dowolny język)
   1. uwierzytelnianie: kim jest użytkownik           → z sesji backendu
   2. autoryzacja: do której przestrzeni ma dostęp    → np. user-42, team-7
   3. przekazanie bajtów paczki bez czytania
   │  sieć wewnętrzna; nagłówki x-flatsql-space i x-flatsql-key
   ▼
serwer synchronizacji Flat-SQL (Node.js + silnik WASM), bez publicznego portu
   4. sprawdzenie klucza bramki → zapis do przestrzeni / odczyt z niej
   │
   ▼
adapter magazynu: dysk │ magazyn obiektów (S3) │ tabela w bazie SQL
```

Integracja z backendem odbywa się wyłącznie przez HTTP – backend w dowolnym języku przekazuje żądania standardowymi narzędziami (reverse proxy), bez wiązań Flat-SQL (ADR-0024, ADR-0025).

## 3. Bramka, uwierzytelnianie i autoryzacja (ADR-0025)

- **Uwierzytelnianie i autoryzacja odbywają się w backendzie.** Backend ustala z sesji, kim jest użytkownik, i decyduje, do której przestrzeni może się synchronizować. Serwer synchronizacji wie tylko, do której przestrzeni bramka kazała zapisać paczkę.
- **Wybór przestrzeni** to kod backendu oparty na danych, które backend już ma – nie wymaga osobnej tabeli z nazwami przestrzeni:

  ```js
  app.all('/sync/*', express.raw({ type: '*/*' }), requireLogin, async (req, res) => {
    const space = `user-${req.user.id}`;            // albo team-7 po sprawdzeniu członkostwa
    const r = await fetch(`http://sync:8081${req.url}`, {
      method: req.method,
      headers: { 'x-flatsql-space': space, 'x-flatsql-key': process.env.SYNC_KEY },
      body: req.method === 'POST' ? req.body : undefined,
    });
    for (const h of ['x-sync-cursor', 'x-sync-epoch']) {
      if (r.headers.has(h)) res.set(h, r.headers.get(h));
    }
    res.status(r.status).send(Buffer.from(await r.arrayBuffer()));
  });
  ```

- **Członkostwo w przestrzeniach wspólnych** (czy Ania należy do `team-7`) sprawdza backend we własnej bazie przed przekazaniem żądania. Serwer nie przechowuje list członków.
- **Klucz bramki** to tajny, losowy ciąg (np. 32 losowe bajty) wygenerowany przez programistę i wpisany w dwa miejsca: do backendu (zmienna środowiskowa) i do konfiguracji serwera. Backend dołącza go do każdego przekazywanego żądania, serwer odrzuca żądania bez niego. To druga warstwa ochrony na wypadek przypadkowego wystawienia serwera albo przejętego programu w sieci wewnętrznej. Klucz jest jeden dla bramki, nigdy nie trafia do przeglądarki, po wycieku zmienia się go w obu miejscach.

## 4. Przestrzenie (ADR-0023)

Przestrzeń to **jeden folder na serwerze, czyli jedna osobna baza** z dowolną liczbą tabel. Nazwę wybiera backend, np.:

| Przestrzeń | Zawartość |
|---|---|
| `user-42` | prywatne dane jednej osoby: notatki, zapis gry, ustawienia |
| `team-7` | dane zespołu |
| `game-1234` | dane jednej rozgrywki |
| `shop` | dane wspólne dla wszystkich, np. cennik |

- Przestrzeń powstaje przy pierwszej synchronizacji – backend nie musi jej zakładać. Wyjątek do przewidzenia: usunięcie przestrzeni przy usuwaniu konta (żądanie administracyjne z kluczem bramki).
- Wszystkie urządzenia jednego użytkownika używają tej samej przestrzeni; każde ma własną lokalną kopię bazy, własny outbox i własny kursor.
- **Zapytania SQL działają w obrębie jednej przestrzeni.** Dane porównywane między użytkownikami trzeba od początku trzymać w jednym miejscu: w bazie backendu albo we wspólnej przestrzeni, do której pisze backend.

## 5. Tryby pracy (ADR-0023)

Tryb wybiera się dla każdej przestrzeni osobno.

| | Magazyn paczek (domyślny) | Folder z silnikiem (opcja) |
|---|---|---|
| Co robi serwer | przechowuje paczki w logu i oddaje je od kursora, nie czytając treści | stosuje paczki we własnej bazie przestrzeni |
| Na dysku | log paczek + punkt startowy | log paczek + baza (`snapshot.fsql`, `wal.fsql`) |
| W pamięci | prawie nic | otwarte bazy aktywnych przestrzeni |
| Mechanizmy dla backendu (§8) | brak | tabele tylko do odczytu, webhook, `/api/query` |
| Szyfrowanie end-to-end | możliwe – serwer jest ślepym przekaźnikiem | niemożliwe – serwer musi czytać dane |
| Przykłady | zapis gry jednoosobowej, notatki, aplikacje osobiste | cennik sklepu, gra wieloosobowa, kasy, dane zespołu |

**Dlaczego magazyn paczek nie scala stanu:** paczka to kilka zmian, a nie cała baza. Złożenie „stary stan + paczka” wymaga rozpoznania wierszy i rozstrzygnięcia LWW, czyli pracy silnika. Wysyłanie przez urządzenia całej bazy zamiast zmian gubiłoby dane – stan wysłany później nadpisałby zmiany z innego urządzenia.

**Silnik w trybie z silnikiem:** jeden proces serwera wczytuje moduł WASM raz i trzyma wiele obiektów bazy, po jednym na aktywną przestrzeń. Baza jest wczytywana z plików przy pierwszym żądaniu i odkładana na dysk po okresie bezczynności (future 10.1).

## 6. Dane na serwerze

```text
data/spaces/
  user-42/
    space.json          {"mode": "packets", "epoch": "a1b2"}
    log/
      000001-000500.fsb paczki w kolejności przyjęcia
      000501-000732.fsb
    base.fsb            punkt startowy (pełny stan) po kompaktowaniu
  shop/
    space.json          {"mode": "engine", "epoch": "c3d4"}
    snapshot.fsql       baza przestrzeni – format wspólny z CLI i przeglądarką
    wal.fsql
    log/ …              log jest w obu trybach – z niego urządzenia pobierają zmiany
```

- **Paczka** – plik binarny `FSB1` z listą zmian (wstawienie, zmiana, usunięcie wiersza), każda ze znacznikiem HLC, z sumą kontrolną. Powstaje z outboksu urządzenia, z zapisu backendu przez `/api/query` albo jako punkt startowy.
- **Punkt startowy** – pełny stan bazy zapisany w formacie paczki; urządzenie stosuje go jak każdą paczkę.
- **Adapter magazynu** decyduje, gdzie fizycznie leży ta zawartość – odpowiednik `IndexedDbStore` / `FsStore` / `MemoryStore` w bridge. Dostarczane adaptery, wybierane w konfiguracji:
  - **dysk** (domyślny) – foldery jak wyżej; do bazy przestrzeni w trybie z silnikiem wykorzystuje mechanizm dzisiejszego `FsStore`;
  - **magazyn obiektów (S3, R2, MinIO)** – pliki jako obiekty pod nazwami `spaces/user-42/log/…`; dla serwerów w kontenerach bez trwałego dysku i dla kilku instancji;
  - **tabela w bazie SQL** – wiersze `(przestrzeń, numer, paczka)` w bazie, którą programista już ma (np. PostgreSQL); paczki są dla niej nieprzezroczystymi bajtami, a kopia zapasowa obejmuje wszystko naraz.
  Programista może dopisać własny adapter (operacje: dopisz paczkę, odczytaj od numeru, zapisz punkt startowy, usuń starsze paczki; w trybie z silnikiem także snapshot i WAL).

## 7. Protokół i przepływ zmian (ADR-0016, ADR-0017, ADR-0028)

**Outbox** – kolejka lokalnych zmian, których serwer jeszcze nie potwierdził. Jest częścią WAL, więc przeżywa zamknięcie przeglądarki; kolejne zmiany jednego wiersza dają jeden wpis.

**Runda synchronizacji** (`SyncClient`, co kilka sekund, po zapisie lokalnym i po odzyskaniu sieci):

1. `POST /sync/push` – wysłanie outboksu jako paczki; serwer odpowiada, ile zmian przyjął, zignorował i odrzucił; urządzenie usuwa potwierdzone wpisy z outboksu.
2. `GET /sync/pull?since=N&epoch=E` – pobranie paczek po kursorze `N`, bez własnych paczek urządzenia.
3. Zastosowanie paczek w lokalnej bazie, utrwalenie WAL, zapis nowego kursora.

**Kursor** to numer przyjęcia nadany przez serwer (`seq` w logu przestrzeni), a nie znacznik czasu. Znacznik HLC mówi, kiedy zmianę zrobiono na urządzeniu – zmiana zrobiona offline i wysłana później ma stary znacznik i przy kursorze czasowym zginęłaby (ADR-0028).

**Epoka** identyfikuje „wydanie” logu. Gdy log zostanie utracony albo przebudowany, epoka się zmienia, a urządzenie ze starą epoką dostaje pełny stan.

**Kompaktowanie** zastępuje stare paczki punktem startowym. Urządzenie z kursorem sprzed kompaktowania dostaje punkt startowy i resztę logu, więc niczego nie gubi. W magazynie paczek punkt startowy przysyła urządzenie (serwer nie umie go zbudować); w przestrzeniach wspólnych bezpieczniej kompaktuje tryb z silnikiem.

**Powtarzalność.** Runda przerwana w dowolnym miejscu jest po prostu powtarzana – ponowne zastosowanie tej samej zmiany nic nie zmienia, bo ma ten sam znacznik.

**Konflikty.** Każda zmiana wiersza ma znacznik HLC – zegara, który nie cofa się i jest zgrywany między urządzeniami. Gdy dwa urządzenia zmienią ten sam wiersz, wygrywa późniejszy znacznik (LWW per wiersz, ADR-0015). Zasadę stosuje każde urządzenie i serwer w trybie z silnikiem, więc wszyscy dochodzą do tego samego stanu. Znaczniki zbyt daleko w przyszłości są odrzucane (todo 1.1).

**Push z serwera** (WebSocket lub SSE, future 6.2) przyspieszy dostarczanie zmian połączonym urządzeniom; kursor nadal jest potrzebny, żeby urządzenie po przerwie dostało wszystko, co je ominęło.

## 8. Mechanizmy dla backendu (ADR-0026)

Reguły biznesowe pisze programista w backendzie. Serwer w trybie z silnikiem daje mu trzy opcjonalne mechanizmy, konfigurowane dla przestrzeni:

1. **Tabele tylko do odczytu dla klientów** (`clientReadOnly`) – serwer odrzuca paczki urządzeń zmieniające te tabele; zapisuje je tylko backend. Zamyka obejście, w którym użytkownik zmienia lokalną kopię w narzędziach przeglądarki, a automatyczna synchronizacja wysyła zmianę obok backendu. Bez kodu po stronie programisty.
2. **Webhook** (`validate`) – przed przyjęciem paczki serwer wysyła backendowi zmiany jako JSON (stan przed i po), jedno żądanie na paczkę; backend odpowiada, które zmiany odrzucić; serwer ich nie zapisuje i odsyła urządzeniu poprawny stan. Dla danych zmienianych offline, którym nie wolno ufać.
3. **`/api/query`** – backend czyta i zapisuje dane przestrzeni przez SQL w sieci wewnętrznej, z kluczem bramki. Zapisy trafiają do urządzeń jak każda inna zmiana.

| Zastosowanie | Webhook |
|---|---|
| aplikacje osobiste, zapis gry jednoosobowej | zbędny |
| zbieranie danych w terenie | czasem – poprawność formularzy |
| kasy offline | tak – stan magazynu i ceny zamówień |
| pamięć podręczna danych SaaS | tak – uprawnienia w zespole |
| gry wieloosobowe | tak – oszustwa |

## 9. Konfiguracja (propozycja)

```json
{
  "gatewayKey": "env:SYNC_KEY",
  "storage": { "adapter": "disk", "path": "/data" },
  "default": { "mode": "packets" },
  "spaces": {
    "user-*":   { "mode": "packets" },
    "shop":     { "mode": "engine", "clientReadOnly": ["prices"] },
    "player-*": { "mode": "engine", "validate": "http://backend:3000/flatsql/validate" }
  }
}
```

Serwer dopasowuje nazwę przestrzeni od bramki do wzorca, a bez dopasowania używa `default`. Tryb jest zapisywany w `space.json` przy tworzeniu przestrzeni; zmiana trybu istniejącej przestrzeni wymaga przekształcenia danych (z logu do bazy albo z bazy do punktu startowego).

**Webhook – żądanie serwera** (z kluczem bramki w nagłówku `x-flatsql-key`, żeby backend wiedział, że pyta prawdziwy serwer):

```json
{
  "space": "player-42",
  "batch": "7f3c9a1e",
  "changes": [
    { "index": 0, "op": "update", "table": "player", "key": 42,
      "before": { "id": 42, "gold": 120, "level": 7 },
      "after":  { "id": 42, "gold": 999999, "level": 7 } },
    { "index": 1, "op": "insert", "table": "inventory", "key": 1001,
      "before": null,
      "after":  { "id": 1001, "player": 42, "item": "sword" } },
    { "index": 2, "op": "delete", "table": "inventory", "key": 990,
      "before": { "id": 990, "player": 42, "item": "potion" },
      "after":  null }
  ]
}
```

**Odpowiedź backendu:** `{ "reject": [ { "index": 0, "reason": "za duży przyrost złota" } ] }` – pusta lista oznacza przyjęcie wszystkiego.

- Odrzucenie dotyczy pojedynczych wierszy, reszta paczki przechodzi (ADR-0018).
- Brak odpowiedzi w limicie czasu albo błąd backendu – paczka nie jest przyjęta i zostaje w outboksie urządzenia do następnej rundy; żadna zmiana nie przechodzi bez sprawdzenia.
- `batch` identyfikuje paczkę, więc przy ponowieniu backend może zwrócić tę samą decyzję.

## 10. Struktura tabel (ADR-0027)

- Struktura żyje w kodzie aplikacji, z numerami wersji; lokalna baza pamięta wersję i przy otwarciu wykonuje tylko brakujące kroki (propozycja API: `FlatDB.open({ versions: [...] })`).
- Polecenia zmiany struktury nie są rozsyłane do innych replik; serwer odrzuca w paczkach `CREATE/DROP` od klientów (future 2.6).
- Między wersjami wolno tylko dokładać: tabele, kolumny dopuszczające `NULL`, indeksy. Starsza wersja przechowuje wartości kolumn, których nie zna; nowsza widzi `NULL` w kolumnach, których brakuje w wierszach starszej. Dzięki temu urządzenia z różnymi wersjami aplikacji synchronizują się w jednej przestrzeni.
- Otwarte: skąd serwer w trybie z silnikiem zna strukturę przestrzeni – deklaracje struktury scalane jako suma tabel i kolumn albo szablon w konfiguracji (future 6.1).

## 11. Gdzie trzymać jakie dane – przykład gry

| Dane | Gdzie | Jak |
|---|---|---|
| zapis gry jednoosobowej | lokalnie, przestrzeń `user-*` | magazyn paczek, bez sprawdzania – oszust oszukuje sam siebie |
| ekwipunek w grze wieloosobowej | lokalnie, przestrzeń `player-*` | tryb z silnikiem + webhook |
| cennik sklepu | lokalnie tylko do odczytu, przestrzeń `shop` | backend zapisuje przez `/api/query`, urządzenia czytają lokalnie, także offline |
| zakupy za prawdziwe pieniądze | backend | zawsze online, bez synchronizacji |
| ranking | baza backendu (endpoint `GET /ranking`) albo wspólna przestrzeń, do której pisze backend | zapytanie przez wiele przestrzeni nie istnieje (§4) |

**Skala na przykładzie 1000 graczy, 100 naraz:** magazyn paczek – kilka MB pamięci niezależnie od liczby graczy; tryb z silnikiem – rzędu 100 KB na otwartą przestrzeń, ok. 10 MB w szczycie. Webhook – jedno żądanie na paczkę; przy paczce co ok. 5 s na gracza to ok. 20 żądań na sekundę do backendu.

## 12. Ograniczenia i skala

- **Limit 4 GiB pamięci WASM** (`wasm32`, ADR-0007) dotyczy jednego procesu w trybie z silnikiem – wszystkie otwarte bazy procesu dzielą tę pamięć. Magazynu paczek nie dotyczy. Obejścia: zwalnianie nieaktywnych baz (future 10.1), kilka procesów z przestrzeniami rozdzielonymi między nie (future 10.2), w razie potrzeby natywny silnik w Node.js zamiast WASM.
- Pojedyncza przestrzeń musi mieścić się w pamięci – ten sam limit dotyczy urządzeń (dane w pamięci, ADR-0006); dla dużych przestrzeni wspólnych – replikacja częściowa (future 3.2).
- Przy bramce cały ruch synchronizacji przechodzi przez backend.
- LWW per wiersz gubi współbieżne zmiany różnych kolumn jednego wiersza (todo 3.1) i liczników (future 2.5); unikalność klucza głównego jest tylko lokalna (todo 0.3, 3.3).

## 13. Prototyp a projekt docelowy

| | Prototyp ([server/server.js](../../../server/server.js)) | Projekt docelowy |
|---|---|---|
| Podział danych | jedna baza dla wszystkich klientów | przestrzenie |
| Tryb | z silnikiem, jedna baza | magazyn paczek lub silnik, per przestrzeń |
| Log | w pamięci; restart = pełny stan dla wszystkich | trwały, per przestrzeń (todo 3.2) |
| Dostęp | publiczny, bez uwierzytelniania | tylko przez bramkę z kluczem |
| Reguły | brak | tabele tylko do odczytu, webhook |
| Struktura tabel | rozsyłana jak dane | w kodzie aplikacji, tylko dokładanie |
| Magazyn | pliki na dysku (`FsStore`) | adaptery: dysk, S3, tabela SQL |
| Push | brak (odpytywanie co kilka sekund) | WebSocket/SSE (future 6.2) |

## 14. Rozszerzenia bez terminu

- **Weryfikacja przepustek JWT przez sam serwer** – dla aplikacji bez backendu; drugie źródło identyfikatora przestrzeni, wymaga publicznego wystawienia serwera ([todo_future.md – Kierunki na później](../../todo_future.md#kierunki-na-później-bez-zadań)).
- **Serwer jako biblioteka w procesie backendu** – przez interfejs C i nakładki dla języków (future 9.7); dla backendów w Node.js wystarczyłoby wydzielenie serwera jako modułu JS.
- **Węzły edge z bazą główną** (future §7), **replikacja częściowa w przestrzeniach wspólnych** (future 3.2), **szyfrowanie end-to-end** w magazynie paczek (future 4.3).
- **Tryb z silnikiem bez logu** – stan z numerem przyjęcia przy każdym wierszu zamiast osobnego logu (ADR-0028, odrzucone alternatywy).

## 15. Oparcie w badaniach

| Element | Źródło |
|---|---|
| log z numerami przyjęcia, kursor, pełny stan przy nieznanym kursorze | Bayou [C3], offsety i CDC ze snapshotem [P1] rozdz. 11 |
| trwały outbox, idempotentne rundy | Coda [C10], [C7], [P1] rozdz. 12 |
| tryb z silnikiem dla danych wspólnych, zatwierdzanie zmian przez backend | replikacja dwuwarstwowa [C9], kontrole zależności dostarczane przez aplikację w Bayou [C3] |
| magazyn paczek i szyfrowanie end-to-end | [L11], [L1] ideał 6 |
| przestrzenie jako zgrubna replikacja częściowa | Cimbiosys [C11]; migracja Productlane ([real-world-usage.md](../../research/real-world-usage.md)) |
| struktura w kodzie, tylko dokładanie | zgodność wstecz i w przód [P1] rozdz. 4; problem otwarty w [L1]; Cambria [L26] jako odrzucona alternatywa |
| bramka i reguły w backendzie | praktyka: ElectricSQL (proxy auth), PowerSync (zapisy przez backend) |

## Decyzje i zadania

ADR: [0002](../../adr/0002-local-first-z-serwerem-koordynujacym.md), [0015](../../adr/0015-hlc-i-lww-per-wiersz.md), [0016](../../adr/0016-outbox-w-wal.md), [0017](../../adr/0017-protokol-push-pull-z-kursorem.md), [0018](../../adr/0018-atomowosc-wiersza-miedzy-replikami.md), [0023](../../adr/0023-konfigurowalne-tryby-serwera-synchronizacji.md), [0024](../../adr/0024-bez-wiazan-dla-pythona.md), [0025](../../adr/0025-serwer-synchronizacji-za-bramka-backendu.md), [0026](../../adr/0026-reguly-w-backendzie-mechanizmy-w-serwerze.md), [0027](../../adr/0027-struktura-w-kodzie-aplikacji-tylko-dokladanie.md), [0028](../../adr/0028-kursor-jako-numer-przyjecia.md).

Zadania: todo 1.1, 3.1–3.4; future 2.1–2.6, 3.2, 4.1–4.3, 6.1–6.2, §7, §8, 9.7, §10.
