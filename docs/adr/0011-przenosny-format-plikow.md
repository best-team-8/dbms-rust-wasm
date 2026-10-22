# ADR-0011: Przenośny format plików zamiast obrazu pamięci

| | |
|---|---|
| Status | przyjęta |
| Data | 2026-10-06 (formaty, [architecture.md §6](../architecture.md)); alternatywa odrzucona 2026-10-20 |
| Rodzaj | architektura |
| Powiązane | ADR-0005, ADR-0009; todo 1.8, 2.7 |

## Kontekst

Snapshot, WAL i paczki synchronizacji muszą być czytelne w każdym środowisku, w którym działa rdzeń (ADR-0005), i przetrwać aktualizacje silnika. W rewizji rozważano szybsze odtwarzanie przez zapis surowej pamięci modułu – pomysł z systemu Faasm, który przyspiesza start funkcji serverless, kopiując zrzut pamięci modułu WASM po inicjalizacji.

## Decyzja

Wszystkie dane trwałe mają własny, udokumentowany format binarny z nagłówkiem i sumą kontrolną: snapshot `FLATSQL1` (z polem wersji formatu), ramki WAL, paczki `FSB1`. Wspólny kodek: liczby jako LEB128 (ze znakiem w zig-zag), tekst jako długość + UTF-8, wartości ze znacznikiem typu. Ten sam katalog danych (`snapshot.fsql`, `wal.fsql`) otwiera serwer Node.js (`FsStore`) i natywne CLI (`FileStorage`); w przeglądarce te same bajty snapshotu i WAL trafiają do IndexedDB.

## Uzasadnienie

- Format jest niezależny od architektury i układu pamięci, więc bazę serwera można otworzyć w CLI, a w przyszłości w panelu WinUI.
- Pole wersji formatu pozwala czytać dane po aktualizacji silnika – dane żyją dłużej niż kod ([P1] rozdz. 4, ideał 5 z [L1]).
- Sumy kontrolne pozwalają wykryć urwany zapis i uszkodzony snapshot.

## Odrzucone alternatywy

- **Snapshot jako obraz pamięci modułu (dawne zadanie 4.8)** – pamięć zawiera wskaźniki do 32-bitowej przestrzeni WASM, nieczytelne dla natywnego Rusta z 64-bitowymi wskaźnikami; układ struktur zmienia się przy każdej kompilacji, więc obraz nie przetrwałby aktualizacji; zawiera też zwolnioną pamięć i nie da się go częściowo sprawdzić. Faasm rozwiązuje szybki start kodu, a nie trwałe przechowywanie danych.
- **Gotowy format serializacji (np. Protocol Buffers)** – dodatkowa zależność (ADR-0005); własny kodek jest odpowiednikiem varint/sint z Protocol Buffers.

## Konsekwencje

- Odtwarzanie wymaga dekodowania i wstawiania wierszy (200 ms dla 100 tys.) – przyspieszenie przez zwartą reprezentację wierszy (todo 2.7).
- Porównywanie replik po skrócie bajtów wymaga kanonizacji NaN (todo 1.8).

## Źródła

[P1] rozdz. 4, [L1], [P2] §2.5; Faasm (Shillaker i Pietzuch 2020) – poza rdzeniem literatury.
