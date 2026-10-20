# ADR-0018: Między replikami atomowy jest wiersz

| | |
|---|---|
| Status | przyjęta |
| Data | 2026-10-20 (rewizja literatury; zachowanie od 2026-10-06) |
| Rodzaj | architektura |
| Powiązane | ADR-0004, ADR-0008, ADR-0016; todo 0.1; future 12.7 |

## Kontekst

Lokalnie transakcja jest atomowa (ADR-0008). W synchronizacji outbox wysyła wiersze paczkami po 1000, a wiersz zmieniany ponownie przesuwa się na koniec kolejki. Gdy outbox jest większy niż jedna paczka, zmiany jednej transakcji mogą dojść w różnych paczkach.

Przykład: wymiana między graczami to dwa `UPDATE` w jednej transakcji (miecz z wiersza 1 do Borina, złoto z wiersza 7 do Ayli). Urządzenie, które pobierze dane między paczkami, przez chwilę widzi miecz u Borina i złoto też jeszcze u Borina; po następnej rundzie stan się zgadza.

## Decyzja

Gwarancja między replikami: **pojedynczy wiersz dochodzi zawsze w całości** (wszystkie kolumny ze znacznikiem); grupa wierszy z jednej transakcji może przez chwilę być widoczna częściowo. To, co musi zmieniać się atomowo także między replikami, modeluje się jako jeden wiersz (np. wymiana jako wiersz tabeli `trades` z obiema stronami).

## Uzasadnienie

- Encja pod kluczem jest jednostką atomowości w systemach rozproszonych [C7]; to wynika wprost z płaskiego modelu (ADR-0004).
- Simba, system synchronizacji tabel dla aplikacji mobilnych, świadomie daje atomowość tylko pojedynczego wiersza – atomowa synchronizacja wielu wierszy okazała się dla większości aplikacji zbędna i kosztowna [L27].
- Silniejsza gwarancja jest osiągalna bez koordynacji, gdyby okazała się potrzebna [C8].

## Odrzucone alternatywy

- **Monotonic Atomic View (wszystkie albo żadne zmiany transakcji) jako wymóg** – osiągalne przez eksport tylko pełnych transakcji [C8]; nie jest wymogiem, zostaje opcją (future 12.7), gdy demo pokaże potrzebę.

## Konsekwencje

- Dokumentacja i praca opisują gwarancję wprost: „transakcje są atomowe lokalnie; między replikami atomowy jest wiersz” (todo 0.1).
- Zapobieganie utraconym aktualizacjom nie jest możliwe bez koordynacji ([C8], [C9]) – niezależnie od tej decyzji.

## Źródła

[C7], [C8], [C9], [L27].
