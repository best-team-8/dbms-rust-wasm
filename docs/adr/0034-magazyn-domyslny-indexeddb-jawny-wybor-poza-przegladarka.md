# ADR-0034: Magazyn domyślny – IndexedDB w przeglądarce, jawny wybór poza nią

| | |
|---|---|
| Status | przyjęta |
| Data | 2026-10-23 |
| Rodzaj | architektura |
| Powiązane | ADR-0003, ADR-0012 (realizuje decyzję o magazynie domyślnym), ADR-0013; todo 1.10, 1.5 |

## Kontekst

ADR-0012 przyjmuje IndexedDB jako magazyn domyślny przeglądarki, ale `FlatDB.open()` bez opcji `store` tworzył `MemoryStore`: dane znikały po odświeżeniu strony, a aplikacja nie dostawała żadnego sygnału. Poza przeglądarką (Node.js) nie ma oczywistego miejsca na trwałe dane – `FsStore` potrzebuje katalogu. Zadanie 1.10 dopuszczało w Node.js jawny wybór magazynu albo ostrzeżenie; wybór należał do autora projektu.

## Decyzja

1. W przeglądarce (gdy istnieje `indexedDB`, także w Web Workerze) `FlatDB.open()` bez `store` tworzy `IndexedDbStore(name)`; nazwę podaje opcja `name` (domyślnie `'default'`), a opcja `durability` (ADR-0010, todo 1.4) dotyczy tego magazynu.
2. Poza przeglądarką `FlatDB.open()` bez `store` kończy się błędem z podpowiedzią: `FsStore` dla danych w plikach albo `MemoryStore` dla danych tylko w pamięci.
3. Opcja `name` razem z `store` jest błędem, bo nie miałaby żadnego skutku.

## Uzasadnienie

- Dane domyślnie lokalne i trwałe to sens założeń ADR-0002 i ADR-0003; magazyn tylko w pamięci jako domyślny przeczy im po cichu.
- Błąd zamiast ostrzeżenia: ostrzeżenie w konsoli łatwo przeoczyć (zwłaszcza w procesie serwera), a jego przeoczenie oznacza utratę danych po zakończeniu procesu. Wymaganie jawnego `MemoryStore` kosztuje jedną linię tam, gdzie pamięć jest zamierzona (testy, benchmarki).
- Test (`test/browser.test.js`, [tests-results.md §1.6](../research/tests-results.md#16-magazyn-domyślny)) pokazał na kodzie sprzed zmiany oba problemy: w przeglądarce magazyn w pamięci, w Node.js brak błędu.

## Odrzucone alternatywy

- **`MemoryStore` z ostrzeżeniem w Node.js** – zgodne wstecz, ale przeoczone ostrzeżenie to cicha utrata danych.
- **Domyślny `FsStore` w katalogu bieżącym** – zapis plików w miejscu, którego aplikacja nie wybrała; serwer i tak podaje katalog jawnie.
- **Magazyn wybierany wyłącznie jawnie także w przeglądarce** – sprzeczne z ADR-0012 i z prostym startem w aplikacji (`FlatDB.open({ name })`).

## Konsekwencje

- Kod Node.js, który polegał na domyślnym `MemoryStore`, musi go podać jawnie (zmienione testy end-to-end).
- Domyślny magazyn przeglądarki może zostać usunięty przez przeglądarkę przy braku miejsca albo po okresie bez interakcji – stąd prośba o trwały magazyn (todo 1.5).
- Moduł `bridge/flatdb.js` importuje `IndexedDbStore` także w Node.js; moduł nie ma efektów ubocznych.

## Źródła

ADR-0012; [Q1]–[Q4]; [tests-results.md §1.6](../research/tests-results.md#16-magazyn-domyślny).
