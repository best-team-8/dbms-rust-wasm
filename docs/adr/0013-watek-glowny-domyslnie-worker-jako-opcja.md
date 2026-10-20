# ADR-0013: Wątek główny jako tryb domyślny, Worker jako opcja

| | |
|---|---|
| Status | przyjęta |
| Data | 2026-10-06 (bridge, [architecture.md §1](../architecture.md)); uzasadnienie doprecyzowane 2026-10-20 |
| Rodzaj | architektura |
| Powiązane | ADR-0001, ADR-0005, ADR-0006, ADR-0012; todo 2.6, 4.12; future 1.1, 1.5 |

## Kontekst

Przeglądarka wykonuje JavaScript strony w jednym wątku, który także rysuje interfejs. Bazę można uruchomić w nim albo w Web Workerze (osobny wątek w tle, komunikacja wiadomościami). Osobno istnieją nagłówki COOP/COEP, wymagane do pamięci współdzielonej (`SharedArrayBuffer`) i wielowątkowego WASM – sam Worker ich nie wymaga. Konkurencyjne silniki z trwałością (SQLite na OPFS, DuckDB-Wasm) muszą działać w Workerze, bo potrzebują synchronicznego dostępu do plików.

## Decyzja

Flat-SQL domyślnie działa w wątku, z którego jest wywoływany, z synchronicznym API (`db.exec()` od razu zwraca wynik), bez wymogu Workera i bez nagłówków COOP/COEP. Uruchomienie w Workerze jest dozwolone i będzie wspierane jako opcja (future 1.1 – `SharedWorker` dla wielu kart, future 1.5 – OPFS). Wielowątkowość wymagająca COOP/COEP jest wykluczona.

## Uzasadnienie

Założenie „bez serwera” (ADR-0001) **nie** przesądza tej decyzji – Worker działa na tym samym urządzeniu. Przesądza o niej to, co tryb domyślny daje:

- **wynik od razu:** zapytanie jest wywołaniem funkcji, bez `await` i bez przesyłania wiadomości; w grze można odczytać dane w środku klatki, a interfejs nie pokazuje stanu „w trakcie ładowania” – z tego samego powodu Riffle wykonuje zapytania synchronicznie w wątku interfejsu [D4];
- **osadzanie bez przebudowy aplikacji:** jeden import, bez osobnego pliku workera, konfiguracji bundlera i nagłówków serwera; działa na hostingu statycznym, w rozszerzeniach i widżetach na cudzych stronach;
- **mniej kopiowania:** wynik przechodzi z pamięci WASM do JS raz, a nie dodatkowo między wątkami;
- **jest to możliwe**, bo dane są w pamięci (ADR-0006), a zapis na dysk jest asynchroniczny (ADR-0010) – silnik nie potrzebuje synchronicznego dostępu do plików.

## Odrzucone alternatywy

- **Worker jako wymóg** – interfejs byłby zawsze płynny i dostępny byłby szybki OPFS, ale API stałoby się asynchroniczne, każde zapytanie płaciłoby za przesłanie między wątkami, a osadzanie wymagałoby przebudowy aplikacji. Flat-SQL straciłby wyróżnik wobec SQLite WASM.
- **Wątki WASM (`SharedArrayBuffer`)** – wymagają COOP/COEP, co blokuje zasoby z innych domen i wymaga kontroli nad serwerem; odrzucone także jako droga do równoległego odtwarzania (ADR-0009).

## Konsekwencje

- Każda długa operacja blokuje interfejs – trzeba je skracać zamiast przenosić do innego wątku: checkpoint porcjami między zadaniami pętli zdarzeń (todo 2.6), szybsze otwarcie bazy (todo 2.7).
- Obsługę wielu kart trzeba dobudować; naturalną drogą jest `SharedWorker`, czyli tryb Worker jako opcja (future 1.1).
- Otwarte pytanie ilościowe: koszt przesłania zapytania i wyniku do Workera nie był mierzony (todo 4.12); wynik może przesunąć granicę, od której tryb Worker się opłaca.

## Źródła

[D4], [Q5], [Q7], [Q12]; [wasm-engines-comparison.md](../research/wasm-engines-comparison.md).
