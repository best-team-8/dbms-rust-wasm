# Projekt docelowego serwera synchronizacji

Opis serwera, do którego prowadzą zadania z części „Serwer synchronizacji” w [liście na później](future.md). Decyzje i ich uzasadnienie: [decyzja 0024](../adr/0024-serwer-synchronizacji.md). Obecny prototyp ([server/](../../server/README.md)) wystarcza do pracy offline i jej testów, ale nie ma przestrzeni, trwałego logu ani uwierzytelniania.

## Prototyp a projekt docelowy

| | Prototyp | Projekt docelowy |
|---|---|---|
| Podział danych | jedna baza dla wszystkich klientów | przestrzenie |
| Tryb | silnik z jedną bazą | magazyn paczek albo silnik, osobno dla każdej przestrzeni |
| Log | w pamięci; restart oznacza pełny stan dla wszystkich | trwały, osobny dla każdej przestrzeni |
| Dostęp | publiczny, bez uwierzytelniania | tylko przez bramkę backendu z kluczem |
| Reguły | brak | tabele tylko do odczytu, webhook |
| Struktura tabel | rozsyłana jak dane | w kodzie aplikacji, tylko dokładanie |
| Magazyn | pliki na dysku | dysk, magazyn obiektów (S3) albo tabela w bazie SQL |
| Powiadomienia | odpytywanie co kilka sekund | WebSocket albo SSE |

## Rola

Serwer synchronizacji to osobna usługa obok backendu aplikacji. Przechowuje zmiany z lokalnych baz na urządzeniach i oddaje je innym urządzeniom z dostępem do tych samych danych. Nie zna użytkowników, nie zawiera logiki biznesowej i nie jest główną bazą aplikacji: dane porównywane między użytkownikami, na przykład ranking, trzyma zwykle baza backendu. Synchronizacja jest opcją: `FlatDB` działa bez serwera.

## Miejsce w architekturze

```text
przeglądarka (FlatDB + SyncClient)
   │  POST /sync/push, GET /sync/pull + sesja użytkownika (np. ciasteczko)
   ▼
backend aplikacji = bramka (dowolny język)
   1. kim jest użytkownik                  → z sesji backendu
   2. do której przestrzeni ma dostęp      → np. user-42, team-7
   3. przekazanie paczki bez czytania
   │  sieć wewnętrzna; nagłówki x-flatsql-space i x-flatsql-key
   ▼
serwer synchronizacji (Node.js + silnik WebAssembly), bez publicznego portu
   │
   ▼
magazyn: dysk │ magazyn obiektów (S3) │ tabela w bazie SQL
```

Przestrzeń wybiera kod backendu na podstawie danych, które backend już ma:

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

Klucz bramki to losowy ciąg wpisany do backendu i do konfiguracji serwera. Nigdy nie trafia do przeglądarki. Serwer odrzuca żądania bez niego, co chroni przed przypadkowym wystawieniem serwera.

## Przestrzenie

Przestrzeń to jedna osobna baza z dowolną liczbą tabel, na przykład `user-42` (dane jednej osoby), `team-7` (dane zespołu), `game-1234` (jedna rozgrywka), `shop` (cennik wspólny dla wszystkich). Powstaje przy pierwszej synchronizacji. Wszystkie urządzenia jednego użytkownika używają tej samej przestrzeni, ale każde ma własną kopię bazy, kolejkę zmian i kursor. Zapytania SQL działają w obrębie jednej przestrzeni.

## Tryby pracy

| | Magazyn paczek (domyślny) | Tryb z silnikiem (opcja) |
|---|---|---|
| Co robi serwer | dopisuje paczki do logu i oddaje je od kursora, bez czytania treści | stosuje paczki we własnej bazie przestrzeni |
| Na dysku | log paczek i punkt startowy | log paczek i baza (`snapshot.fsql`, `wal.fsql`) |
| W pamięci | prawie nic | otwarte bazy aktywnych przestrzeni |
| Mechanizmy dla backendu | brak | tabele tylko do odczytu, webhook, `/api/query` |
| Szyfrowanie end-to-end | możliwe | niemożliwe, bo serwer czyta dane |
| Przykłady | zapis gry jednoosobowej, notatki | cennik, gra wieloosobowa, kasy, dane zespołu |

Magazyn paczek nie składa stanu, bo złożenie „stary stan + paczka” wymaga rozstrzygania konfliktów, czyli silnika. Gdyby urządzenia wysyłały całą bazę zamiast zmian, stan wysłany później nadpisałby zmiany innego urządzenia.

## Dane na serwerze

```text
data/spaces/
  user-42/
    space.json          {"mode": "packets", "epoch": "a1b2"}
    log/
      000001-000500.fsb paczki w kolejności przyjęcia
    base.fsb            punkt startowy (pełny stan) po skróceniu logu
  shop/
    space.json          {"mode": "engine", "epoch": "c3d4"}
    snapshot.fsql       baza przestrzeni w formacie wspólnym z przeglądarką
    wal.fsql
    log/ …
```

Punkt startowy to pełny stan bazy zapisany w formacie paczki. Skrócenie logu zastępuje stare paczki punktem startowym, a urządzenie z kursorem sprzed skrócenia dostaje punkt startowy i resztę logu. Magazyn decyduje, gdzie leżą pliki: na dysku, w magazynie obiektów albo jako wiersze `(przestrzeń, numer, paczka)` w bazie SQL backendu.

## Mechanizmy dla backendu

Reguły biznesowe pisze programista w backendzie. Serwer w trybie z silnikiem daje mu trzy opcjonalne mechanizmy:

1. **Tabele tylko do odczytu dla klientów.** Serwer odrzuca paczki urządzeń, które je zmieniają. Zamyka obejście, w którym użytkownik zmienia lokalną kopię w narzędziach przeglądarki.
2. **Webhook.** Przed przyjęciem paczki serwer pyta backend, które zmiany odrzucić. Odrzucone zmiany nie są zapisywane, a urządzenie dostaje poprawny stan.
3. **`/api/query`.** Backend czyta i zapisuje dane przestrzeni przez SQL w sieci wewnętrznej. Zapisy trafiają do urządzeń jak każda zmiana.

Webhook przydaje się w kasach (stan magazynu, ceny), w danych zespołu (uprawnienia) i w grach wieloosobowych (oszustwa). W aplikacjach osobistych jest zbędny.

### Konfiguracja (propozycja)

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

Tryb zapisuje się przy tworzeniu przestrzeni. Zmiana trybu istniejącej przestrzeni wymaga przekształcenia danych.

### Żądanie webhooka

```json
{
  "space": "player-42",
  "batch": "7f3c9a1e",
  "changes": [
    { "index": 0, "op": "update", "table": "player", "key": 42,
      "before": { "id": 42, "gold": 120 }, "after": { "id": 42, "gold": 999999 } },
    { "index": 1, "op": "insert", "table": "inventory", "key": 1001,
      "before": null, "after": { "id": 1001, "player": 42, "item": "sword" } }
  ]
}
```

Odpowiedź: `{ "reject": [ { "index": 0, "reason": "za duży przyrost złota" } ] }`; pusta lista oznacza przyjęcie wszystkiego. Odrzucenie dotyczy pojedynczych wierszy. Gdy backend nie odpowie w limicie czasu, paczka nie jest przyjęta i zostaje w kolejce urządzenia. `batch` pozwala backendowi zwrócić tę samą decyzję przy ponowieniu.

## Przykład: gra

| Dane | Gdzie | Jak |
|---|---|---|
| zapis gry jednoosobowej | przestrzeń `user-*` | magazyn paczek bez sprawdzania; oszust oszukuje sam siebie |
| ekwipunek w grze wieloosobowej | przestrzeń `player-*` | tryb z silnikiem i webhook |
| cennik sklepu | przestrzeń `shop`, tylko do odczytu | backend zapisuje, urządzenia czytają lokalnie, także offline |
| zakupy za prawdziwe pieniądze | backend | zawsze online, bez synchronizacji |
| ranking | baza backendu albo wspólna przestrzeń, do której pisze backend | zapytanie przez wiele przestrzeni nie istnieje |

Przy 1000 graczy, z których 100 gra naraz, magazyn paczek zajmuje kilka MB pamięci niezależnie od liczby graczy. Tryb z silnikiem zajmuje rzędu 100 KB na otwartą przestrzeń. Webhook przy paczce co około 5 s na gracza to około 20 żądań na sekundę do backendu.

## Ograniczenia

- Wszystkie otwarte bazy jednego procesu w trybie z silnikiem dzielą 4 GiB pamięci modułu WebAssembly. Obejścia: zwalnianie nieaktywnych baz albo kilka procesów.
- Pojedyncza przestrzeń musi zmieścić się w pamięci. Dla dużych przestrzeni wspólnych potrzebna jest replikacja częściowa.
- Cały ruch synchronizacji przechodzi przez backend.
