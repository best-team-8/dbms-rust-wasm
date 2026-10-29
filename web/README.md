# web – aplikacja demonstracyjna

Strona, która pokazuje Flat-SQL w działaniu: konsola SQL nad lokalną bazą w IndexedDB, panel magazynu i synchronizacji, praca offline z Service Workerem. Nie jest częścią biblioteki; korzysta z niej tak, jak zrobiłaby to aplikacja ([bridge/](../bridge/)). Serwuje ją [serwer](../server/).

## Uruchomienie

```bash
npm run build
```

```bash
npm start
```

Strona jest pod adresem <http://127.0.0.1:8080>. Parametr `?durability=strict|relaxed|default` wybiera tryb trwałości IndexedDB.

## Scenariusz demonstracji

1. Kliknij „Dane: ekwipunek gry” (2000 przedmiotów w jednej transakcji) i wybierz zapytanie z listy „Przykłady”; czas wykonania jest pokazywany w mikrosekundach.
2. Zaznacz „Symuluj brak sieci” i wykonaj kilka `INSERT` lub `UPDATE`: zmiany czekają w outboksie, a po odznaczeniu zostają wysłane.
3. Prawdziwa praca offline: zatrzymaj serwer (`Ctrl+C`) i odśwież stronę. Aplikacja startuje z pamięci podręcznej Service Workera, dane wczytuje z IndexedDB, a zapisy działają. Po ponownym `npm start` oczekujące zmiany są wysyłane bez udziału użytkownika.
4. Otwórz stronę w drugiej przeglądarce albo w oknie prywatnym, żeby zobaczyć replikację między klientami.

Wynik tego scenariusza opisuje [tests-results.md §2.5](../docs/research/tests-results.md#25-praca-offline-w-aplikacji-demo).

## Struktura

| Plik | Rola |
|---|---|
| `index.html`, `styles.css`, `icon.svg` | strona: konsola SQL, przykłady zapytań, panele magazynu, synchronizacji i dziennika |
| `app.js` | logika: otwarcie bazy (`FlatDB`), synchronizacja (`SyncClient`), symulacja braku sieci, odchylenie zegara, konflikty schematu, blokada Web Locks dla wielu kart |
| `sw.js` | Service Worker: pamięć podręczna plików aplikacji, mostu i modułu WebAssembly razem z plikami JavaScript wiązań (`pkg/snippets/`, odczytywanymi z importów `pkg/flatsql.js`), dzięki której strona startuje bez serwera już po pierwszej wizycie |

W wielu kartach zapisuje tylko pierwsza: trzyma blokadę Web Locks, a kolejne pracują na kopii w pamięci z ostrzeżeniem ([bridge/README.md](../bridge/README.md#wiele-kart)).
