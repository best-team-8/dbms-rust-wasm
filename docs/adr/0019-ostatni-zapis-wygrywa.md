# 0019. Konflikty rozstrzyga „ostatni zapis wygrywa” dla całego wiersza

- Status: obowiązuje
- Ostatnia zmiana: 2026-10-20

## Kontekst

Przy pracy offline każde urządzenie przyjmuje zapisy, więc konflikty są nieuniknione (Kleppmann, 2017, rozdz. 5). Potrzebna jest reguła, która na każdym urządzeniu da ten sam wynik bez porozumiewania się, niezależnie od kolejności, w jakiej dotrą zmiany.

## Decyzja

1. Każda zmiana ma znacznik: czas z hybrydowego zegara logicznego i losowy identyfikator urządzenia. Zegar hybrydowy (HLC) łączy czas fizyczny z licznikiem w jednej 64-bitowej liczbie, nigdy się nie cofa i przejmuje znaczniki otrzymane od innych urządzeń.
2. Zmiana z innego urządzenia jest stosowana tylko wtedy, gdy jej znacznik jest większy od znacznika wiersza. Reguła działa dla całego wiersza (ang. *last-writer-wins*).
3. Usunięty wiersz zostawia znacznik usunięcia (ang. *tombstone*), żeby spóźniona zmiana go nie przywróciła.
4. Między urządzeniami atomowy jest pojedynczy wiersz: dochodzi zawsze w całości. Zmiany wielu wierszy z jednej transakcji mogą przez chwilę być widoczne częściowo. Co musi zmieniać się atomowo także między urządzeniami, aplikacja modeluje jako jeden wiersz.

## Uzasadnienie

- Wybór większego znacznika jest łączny, przemienny i idempotentny, więc urządzenia zbiegają się do tego samego stanu. To rejestr „ostatni zapis wygrywa” z teorii bezkonfliktowych typów replikowanych, CRDT (Shapiro i in., 2011).
- Zegar hybrydowy zachowuje kolejność przyczynową i pozostaje blisko czasu fizycznego (Kulkarni i in., 2014). W płaskim modelu konflikt zawsze dotyczy jednego klucza.
- Encja pod kluczem to jednostka atomowości w systemach rozproszonych (Helland, 2007). Simba, system synchronizacji tabel dla aplikacji mobilnych, świadomie daje atomowość tylko pojedynczego wiersza (Go i in., 2015).

## Odrzucone warianty

- Wektory wersji i scalanie przez aplikację. Większe metadane i obowiązek pisania procedur scalania.
- Pełna historia operacji. Log rośnie bez końca. Flat-SQL przesyła bieżący stan zmienionych wierszy.
- Atomowość całych transakcji między urządzeniami. Osiągalna bez koordynacji (Bailis i in., 2013), ale dla większości aplikacji zbędna.
- Scalanie tekstu edytowanego na żywo. Robią to biblioteki CRDT tekstu, poza silnikiem.

## Konsekwencje

- Reguła zbiega się kosztem utraconych zmian: współbieżna zmiana z mniejszym znacznikiem znika bez śladu (Gray i in., 1996). Dotyczy to zmian różnych kolumn jednego wiersza i liczników.
- Urządzenie z zegarem ustawionym w przyszłość wygrałoby każdy konflikt. Chroni przed tym granica dryfu zegara (decyzja 0020).
- Unikalność klucza głównego jest tylko lokalna. Dwa wstawienia tego samego klucza offline nadpisują się po cichu, więc aplikacja zapisująca offline potrzebuje kluczy bez kolizji.
- Znaczników usunięcia przybywa. Silnik ich nie kasuje.

## Historia

- 2026-10-06: reguła z pierwszej wersji silnika.
- 2026-10-20: dopisano gwarancję atomowości pojedynczego wiersza między urządzeniami.
