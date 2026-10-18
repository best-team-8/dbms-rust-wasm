# Kleppmann (2017) – „Designing Data-Intensive Applications”

Notatka do książki poleconej przez promotora. Książka jest komercyjna – w bazie wiedzy nie ma jej tekstu, są natomiast bibliografie wszystkich rozdziałów udostępnione przez autora ([../ddia-references/](../ddia-references/), licencja CC BY-NC 4.0) i jego bezpłatne notatki do wykładu z systemów rozproszonych ([L23](../papers/kleppmann-distributed-systems-lecture-notes.pdf)). Mapa poniżej odnosi się do **wydania 1 (2017)** i jego polskiego tłumaczenia; tytuły podrozdziałów podano w oryginale, żeby łatwo było je odnaleźć.

## 1. Dane bibliograficzne

| Wydanie | Opis |
|---|---|
| 1 (oryginał) | Kleppmann M. *Designing Data-Intensive Applications: The Big Ideas Behind Reliable, Scalable, and Maintainable Systems.* O'Reilly Media, Sebastopol 2017. ISBN 978-1-449-37332-0 |
| polskie | Kleppmann M. *Przetwarzanie danych w dużej skali. Niezawodność, skalowalność i łatwość konserwacji systemów.* Helion, Gliwice 2018 (ISBN wg Goodreads 978-83-283-4065-7, wg katalogu TUM 978-83-283-4066-4 – sprawdzić na egzemplarzu) |
| 2 | Kleppmann M., Riccomini C. *Designing Data-Intensive Applications*, 2nd Edition. O'Reilly, luty–marzec 2026 (daty różnią się między wydawcą a księgarniami) |

Przy cytowaniu należy podać wydanie i numer rozdziału – w wydaniu 2 układ rozdziałów zmieniono, a jego spisu treści nie udało się zweryfikować (strona wydawcy odrzucała automatyczne pobranie). Jeśli w pracy korzysta się z tłumaczenia Helionu, cytuje się wydanie z 2018 r. (odpowiada wydaniu 1).

```bibtex
@book{kleppmann2017ddia,
  author    = {Kleppmann, Martin},
  title     = {Designing Data-Intensive Applications: The Big Ideas Behind Reliable,
               Scalable, and Maintainable Systems},
  publisher = {O'Reilly Media},
  address   = {Sebastopol, CA},
  year      = {2017},
  isbn      = {978-1-449-37332-0}
}
```

## 2. Mapa rozdziałów na projekt

| Rozdz. (wyd. 1) | Fragmenty istotne dla projektu | Element Flat-SQL | Wniosek |
|---|---|---|---|
| 1. Reliable, Scalable, and Maintainable Applications | *Describing Performance*: percentyle czasu odpowiedzi, ogony rozkładu, rozróżnienie latencji i czasu odpowiedzi | raporty p50/p95/p99 w benchmarku | zgodne; w pracy warto doprecyzować, że „latencja lokalnego zapytania” to czas odpowiedzi |
| 2. Data Models and Query Languages | *Relational Model Versus Document Model*, relacje wiele-do-jednego i wiele-do-wielu, *schema-on-read* vs *schema-on-write*, lokalność danych | płaskie tabele bez JOIN, ścisłe typy przy zapisie | Flat-SQL łączy model dokumentowy (encja pod kluczem, brak złączeń) ze schematem przy zapisie; relacje wiele-do-wielu muszą być składane w aplikacji – trzeba to nazwać kosztem modelu |
| 3. Storage and Retrieval | *Hash Indexes* (log tylko do dopisywania, kompaktowanie, odtwarzanie, sumy kontrolne częściowych zapisów), *SSTables and LSM-Trees*, *B-Trees* (WAL), *Storing values within the index* (indeksy klastrowane i pokrywające), *Keeping everything in memory*, OLTP vs OLAP | WAL z CRC, snapshot, odtwarzanie, `BTreeMap` w pamięci; brak indeksów pokrywających | rdzeń projektu realizuje wzorzec bazy w pamięci opisany w książce: trwałość przez log i okresowe snapshoty, zysk wydajności głównie z braku kodowania struktur na dysk. Przegrana z SQLite w `COUNT` po indeksie (H4) to dokładnie przypadek indeksu pokrywającego |
| 4. Encoding and Evolution | binarne kodowanie zmiennej długości (Protocol Buffers), znaczniki pól i ewolucja schematu, zgodność w przód i wstecz, *Dataflow Through Databases* („dane żyją dłużej niż kod”) | LEB128 i zig-zag (jak varint/sint w Protocol Buffers), nagłówki `FLATSQL1`, `FSB1`, pole `format` | kodowanie zgodne z praktyką; brakuje polityki zgodności: klient offline może wrócić po miesiącach ze starszą wersją formatu zmian i schematu tabel |
| 5. Replication | *Clients with offline operation* (każde urządzenie to lider w replikacji multi-leader), *Handling Write Conflicts* (konwergencja przez LWW, własna logika scalania, CRDT), topologie, *Detecting Concurrent Writes*: *Last write wins (discarding concurrent writes)*, *happens-before*, wektory wersji, tombstone'y | architektura Local-First, LWW per wiersz na HLC, tombstone'y, topologia gwiazdy przez serwer | model Flat-SQL to wprost przypadek „klient offline = lider”. Kleppmann ostrzega, że LWW osiąga zbieżność kosztem trwałości: współbieżne zapisy są po cichu odrzucane, a bezpiecznie używa się go głównie dla kluczy zapisywanych raz (np. UUID). W Flat-SQL dotyczy to dwóch wstawień tego samego klucza offline i edycji różnych kolumn tego samego wiersza |
| 6. Partitioning | indeksy wtórne lokalne i globalne | – | nie dotyczy (jedna replika na urządzenie) |
| 7. Transactions | znaczenie ACID (atomowość jako możliwość przerwania), operacje jedno- i wieloobiektowe, *Preventing Lost Updates* (LWW a utracone aktualizacje), *Actual Serial Execution* (VoltDB/H-Store, Redis), uwagi o niedoskonałej trwałości (fsync, dyski) | undo log, atomowość instrukcji, wykonanie jednowątkowe, `flush()` | wykonanie szeregowe w jednym wątku jest uznaną techniką dla danych w pamięci – izolacja serializowalna lokalnie jest „za darmo”. Po replikacji gwarancje są słabsze (utracone aktualizacje) |
| 8. The Trouble with Distributed Systems | *Unreliable Clocks*: zegary monotoniczne vs czasu rzeczywistego, dokładność NTP, **zegarom urządzeń, których nie kontrolujemy, nie można ufać** (użytkownicy celowo zmieniają czas), *Timestamps for ordering events* (LWW przy rozjechanych zegarach gubi zapisy), przedziały niepewności | HLC z `Date.now()` klienta, LWW | HLC łagodzi problem (monotoniczność, wchłanianie znaczników), ale nie chroni przed urządzeniem z zegarem ustawionym w przyszłość – brakuje reguły odrzucania znaczników spoza granicy dryfu (zob. [C2] i porównanie §4.1) |
| 9. Consistency and Consensus | linearizowalność vs spójność ostateczna, *Ordering and Causality*, *Lamport timestamps*, *Timestamp ordering is not sufficient* (ograniczenia unikalności wymagają konsensusu), rozgłaszanie z porządkiem całkowitym | `Stamp {time, node}`, unikalność `PRIMARY KEY` | porządek znaczników to znaczniki Lamporta z identyfikatorem węzła. Unikalność klucza głównego jest gwarantowana tylko lokalnie – przy wstawieniach offline potrzebne są klucze generowane bez kolizji (UUID/ULID, prefiks węzła) |
| 10. Batch Processing | – | – | nie dotyczy |
| 11. Stream Processing | *Partitioned Logs* (offsety konsumentów), *Change Data Capture* (początkowy snapshot + log zmian), *Log compaction* (ostatnia wartość klucza, tombstone przy usunięciu), *Event Sourcing* | outbox, log serwera z `seq`, kursor klienta, `exportState` przy nieznanym kursorze, kompaktowanie logu | protokół synchronizacji to CDC na poziomie wierszy z kompaktowaniem logu; kursor to offset konsumenta. Zgodne z opisanymi wzorcami; Flat-SQL nie stosuje event sourcingu (przesyła stan, nie intencje) |
| 12. The Future of Data Systems | *Stateful, offline-capable clients*, *Pushing state changes to clients*, *The End-to-End Argument for Databases* (idempotencja, identyfikatory operacji, tłumienie duplikatów), *Uniqueness constraints require consensus*, *Doing the Right Thing* (prywatność, nadzór) | `SyncClient` (pull z wykładniczym odstępem), idempotentne rundy, ignorowanie zmian o równych znacznikach | idempotencja end-to-end zrealizowana zgodnie z zaleceniami; dostarczanie zmian jest ciągnięte (pull) – książka wskazuje model push (np. WebSocket/SSE) dla aplikacji reagujących na zmiany |

## 3. Najważniejsze tezy do wykorzystania w pracy

1. **Lokalna baza jako lider.** Aplikacja działająca offline to szczególny przypadek replikacji multi-leader: każde urządzenie przyjmuje zapisy, a konflikty są nieuniknione (rozdz. 5). To uzasadnia, dlaczego Flat-SQL potrzebuje deterministycznej reguły rozstrzygania konfliktów, a nie tylko kolejki żądań.
2. **Cena LWW.** LWW daje zbieżność bez koordynacji, ale traci współbieżne zapisy bez ostrzeżenia (rozdz. 5, 7, 8). Uczciwy opis Flat-SQL powinien to wprost przyznać i wskazać, które dane aplikacji to tolerują (ekwipunek gry, ustawienia), a które nie (np. liczniki, tekst edytowany wspólnie).
3. **Zegary klientów nie są wiarygodne.** Kleppmann zwraca uwagę, że czasu na urządzeniach użytkowników nie można zakładać (rozdz. 8). HLC rozwiązuje problem monotoniczności i przyczynowości, ale wymaga ograniczenia dopuszczalnego dryfu.
4. **Baza w pamięci + log + snapshot** to sprawdzony wzorzec, a nie uproszczenie prototypu (rozdz. 3); koszt przeniesiono na czas odtwarzania i rozmiar danych ograniczony pamięcią.
5. **Idempotencja end-to-end** (rozdz. 12): powtórzona runda synchronizacji nie może zmienić stanu – Flat-SQL spełnia to dzięki znacznikom i ignorowaniu zmian o równym znaczniku.

## 4. Gdzie szukać dalej

- Bibliografie rozdziałów: [ch. 3](../ddia-references/chapter-03-refs.md), [ch. 5](../ddia-references/chapter-05-refs.md), [ch. 7](../ddia-references/chapter-07-refs.md), [ch. 8](../ddia-references/chapter-08-refs.md), [ch. 9](../ddia-references/chapter-09-refs.md), [ch. 11](../ddia-references/chapter-11-refs.md), [ch. 12](../ddia-references/chapter-12-refs.md).
- Rozwinięcie wątków z rozdziałów 5 i 12 w późniejszych pracach autora: [L1] (local-first), [L4] (weryfikacja zbieżności), [L5] (JSON CRDT), [L12] (spójność przy uczestnikach bizantyjskich), [L16] (Eg-walker, 2025).
