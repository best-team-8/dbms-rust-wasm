# bridge – most JavaScript (WASM-Bridge)

Biblioteka, z której korzysta aplikacja. `FlatDB` otwiera bazę, ładuje moduł WebAssembly, odtwarza stan z magazynu i utrwala każdą zatwierdzoną transakcję, a `SyncClient` synchronizuje bazę z serwerem w tle. Silnik ([flatsql-wasm](../crates/flatsql-wasm/)) wykonuje zapytania synchronicznie w pamięci; most odpowiada za wszystko, co jest asynchroniczne: zapis do IndexedDB lub plików, checkpoint i sieć. Ten sam kod działa w przeglądarce i w Node.js.

## Struktura

| Plik | Rola |
|---|---|
| `index.js`, `index.d.ts` | wejście dla przeglądarki i typy TypeScript całego API |
| `node.js`, `node.d.ts` | wejście dla Node.js: ładuje `.wasm` z dysku i dodaje `FsStore` |
| `engine.js` | ładowanie i inicjalizacja modułu WebAssembly (`initEngine`) |
| `flatdb.js` | `FlatDB`: otwarcie i odtworzenie bazy, zapis WAL, checkpoint krokami, pamięć instrukcji przygotowanych, prośba o trwały magazyn |
| `sync.js` | `SyncClient`: rundy push i pull, kursor, ponawianie, kontrola zegara urządzenia |
| `stores/indexeddb.js`, `stores/memory.js`, `stores/fs.js` | magazyny `IndexedDbStore`, `MemoryStore`, `FsStore` |

## Użycie

```js
import { FlatDB, SyncClient } from './bridge/index.js';

const db = await FlatDB.open({ name: 'my-app' }); // w przeglądarce domyślnie IndexedDB

db.exec(`CREATE TABLE IF NOT EXISTS inventory (
  id INT PRIMARY KEY, player TEXT NOT NULL, item TEXT, qty INT)`);
db.exec("INSERT INTO inventory VALUES (1, 'ayla', 'sword', 1)");

const rows = db.query('SELECT item, qty FROM inventory WHERE player = ?', ['ayla']);
// [{ item: 'sword', qty: 1 }] – synchronicznie, bez sieci

await db.flush();                       // zmiany są trwałe w IndexedDB

const sync = new SyncClient(db, { url: 'https://example.org' });
await sync.start();                     // wysyła outbox i pobiera zmiany w tle
```

W Node.js importuje się `bridge/node.js` i podaje magazyn jawnie: `FlatDB.open({ store: new FsStore('dane') })` albo `new MemoryStore()` dla danych tylko w pamięci. Pełne API z opisami opcji: [index.d.ts](index.d.ts), [node.d.ts](node.d.ts).

## Magazyny

Magazyn wybiera aplikacja opcją `store` w `FlatDB.open`. Bez niej w przeglądarce baza trafia do `IndexedDbStore(name)` (opcja `name`, domyślnie `'default'`), więc dane przetrwają odświeżenie strony. Poza przeglądarką magazynu domyślnego nie ma i `FlatDB.open()` bez `store` kończy się błędem, żeby dane nie zostały po cichu tylko w pamięci. Zapis WAL i checkpoint w każdym magazynie opisuje [storage/README.md](../crates/flatsql-core/src/storage/README.md#magazyny).

Magazynem domyślnym jest IndexedDB, a OPFS tylko opcją, bo Flat-SQL czyta z pamięci, a na dysk zapisuje tylko małe ramki WAL. Wolne są obliczenia (kodowanie snapshotu, odtwarzanie bazy), których szybszy dysk nie przyspieszy, a szybki interfejs OPFS działa tylko w Web Workerze. IndexedDB pozwala też zapisać snapshot i wyczyścić WAL w jednej transakcji.

## Zatwierdzanie grupowe i checkpoint

`FlatDB.exec()` zwraca wynik synchronicznie. Jeśli transakcja wytworzyła bajty WAL, `FlatDB` planuje `flush()` w najbliższym mikrozadaniu, więc wszystkie zatwierdzenia z jednego obrotu pętli zdarzeń trafiają do IndexedDB jednym zapisem (ang. *group commit*; test: 101 zatwierdzeń, 1 zapis). `await db.flush()` gwarantuje, że wszystko zatwierdzone do tej chwili jest trwałe. Operacje na magazynie są wykonywane w jednej kolejce, a checkpoint jest bezpieczny wobec zapisów wykonywanych w trakcie, bo rozstrzyga LSN.

Checkpoint zaczyna się, gdy WAL przekroczy `checkpointBytes` (domyślnie 1 MiB). Nie blokuje strony na czas kodowania snapshotu: `FlatDB` koduje go krokami po `checkpointStepRows` wierszy (domyślnie 4096) z obrotem pętli zdarzeń między krokami, a najdłuższa pauza przy 100 000 wierszy wynosi 2,6 ms.

Semantyka trwałości odpowiada `synchronous=NORMAL` w SQLite: po `await flush()` dane są trwałe, zatwierdzenia z ostatnich milisekund mogą przepaść przy nagłym zamknięciu karty, ale baza zawsze wraca do spójnego stanu. Co znaczy „trwałe”, zależy od trybu trwałości transakcji IndexedDB (`FlatDB.open({ durability })`):

| Tryb | `flush()` kończy się, gdy dane | Przetrwa |
|---|---|---|
| `relaxed` | zostały przekazane systemowi operacyjnemu | zamknięcie karty, awarię przeglądarki |
| `strict` | zostały zapisane na dysk | także awarię systemu i utratę zasilania |
| `default` (domyślny) | według przeglądarki; w Chrome 121+ jak `relaxed` | jak wybrany tryb |

Tryb `strict` kosztuje około 0,6 ms na pojedynczym zatwierdzeniu. Opcja `durability` przy magazynie bez trybów (`MemoryStore`, `FsStore`, gdzie odpowiednikiem jest `fsync`) kończy się błędem.

## Usuwanie danych przez przeglądarkę

Magazyn strony jest domyślnie „najlepszej staranności” (ang. *best-effort*): przeglądarka może go usunąć przy braku miejsca, a Safari usuwa IndexedDB po 7 dniach bez interakcji z witryną. Razem z bazą zniknąłby niewysłany outbox. Dlatego `FlatDB.open` z IndexedDB prosi przeglądarkę o magazyn trwały (`navigator.storage.persist()`). Wynik jest w `db.persisted` (`true`, `false` albo `null`, gdy brak API) i w `stats().persisted`, a obietnica `db.persistence` rozstrzyga się po odpowiedzi. `open` nie czeka na nią, bo Firefox pyta użytkownika; aplikacja może wyłączyć prośbę (`persist: false`) i wywołać `db.requestPersistence()` później, np. po kliknięciu.

## Tryby działania w przeglądarce

Silnik nie wykonuje operacji wejścia/wyjścia i nie wie, w którym wątku działa, dlatego wątek i magazyn wybiera aplikacja przy otwarciu bazy. SQLite z trwałością w przeglądarce musi działać w Web Workerze, bo jego magazyn czyta strony z dysku synchronicznie z wnętrza silnika.

| | Wątek główny (domyślny) | Web Worker (opcja) |
|---|---|---|
| API | synchroniczne: `db.exec()` od razu zwraca wynik | asynchroniczne: wynik przychodzi wiadomością |
| Magazyn | IndexedDB | IndexedDB albo OPFS z dostępem synchronicznym |
| Koszt zapytania | odczyt po kluczu około 3 µs | dodatkowo około 50 µs na komunikat w obie strony |
| Długie operacje | muszą być krótkie (checkpoint krokami) | nie blokują strony |
| Integracja | jeden import, bez nagłówków COOP/COEP; działa w rozszerzeniach i na zwykłym hostingu | osobny plik workera, konfiguracja bundlera |

Wątek główny pasuje do małych i średnich danych, gier, rozszerzeń przeglądarki i dodania bazy do istniejącej strony; Worker do dużych baz i aplikacji, w których długie operacje nie mogą blokować interfejsu.

## Wiele kart

Dwie karty z własnymi kopiami bazy w pamięci, dopisujące do tego samego WAL, rozjechałyby się. Demo używa Web Locks API: pierwsza karta trzyma blokadę i zapisuje do IndexedDB, kolejne pracują tylko w pamięci z ostrzeżeniem. Docelowo jedna instancja silnika obsłuży wszystkie karty.

## Synchronizacja (`SyncClient`)

Kolejność kroków rundy zapewnia idempotencję: (1) wysłanie paczki z outboxu (`POST /sync/push`) i potwierdzenie (`ackOutbox`) dopiero po odpowiedzi 200, (2) pobranie zmian (`GET /sync/pull`) i ich zastosowanie, (3) `flush` WAL, (4) zapis kursora. Przerwanie w dowolnym miejscu kończy się powtórzeniem rundy, a powtórnie dostarczone zmiany są ignorowane. Przy braku sieci klient ponawia próby z wykładniczym odstępem (do 30 s) i od razu po zdarzeniu `online`.

Zegar serwera jest wzorcem. Klient mierzy odchylenie swojego zegara z nagłówka `X-Server-Time` (przed pierwszą wysyłką przez `/sync/time`). Gdy odchylenie przekracza 60 s albo serwer odrzucił zmiany za dryf, klient wstrzymuje synchronizację: nie potwierdza paczki, nie pobiera zmian, przechodzi w stan `clock` i zgłasza zdarzenie `clock`. Po poprawieniu zegara `FlatDB.rebaseClock()` nadaje nowe znaczniki niewysłanym zmianom „z przyszłości”, wykonuje checkpoint i synchronizacja rusza dalej. Konflikty schematu zgłasza zdarzenie `conflict`. Protokół HTTP: [server/README.md](../server/README.md#protokół-synchronizacji).

## Decyzje

- [silnik w głównym wątku strony, Web Worker jako opcja](../docs/adr/0007-watek-glowny-domyslnie.md)
- [IndexedDB jako magazyn domyślny](../docs/adr/0008-magazyn-domyslny.md)
- [zatwierdzanie grupowe](../docs/adr/0017-zatwierdzanie-grupowe.md)
- [granica dryfu zegara](../docs/adr/0020-granica-dryfu-zegara.md)

## Testy

Testy mostu działają w Node.js na zbudowanym `pkg/`, z IndexedDB symulowaną w pamięci: [test/README.md](../test/README.md).

## Ograniczenia

Zapisuje tylko jedna karta przeglądarki; tryb Web Worker jest dziś konfiguracją mierzoną w benchmarku, bez gotowego API. Dalszy rozwój: [na później](../docs/management/future.md).
