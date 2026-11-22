# 0008. IndexedDB jako magazyn domyślny w przeglądarce, jawny wybór poza nią

- Status: obowiązuje
- Ostatnia zmiana: 2026-10-23

## Kontekst

Przeglądarka ma dwa trwałe magazyny nadające się do bazy. IndexedDB to wbudowana baza klucz–wartość. OPFS to prywatny system plików strony. Szybki, synchroniczny dostęp do OPFS istnieje tylko w Web Workerze. Poza przeglądarką nie ma oczywistego miejsca na dane: magazyn plikowy potrzebuje katalogu.

## Decyzja

1. W przeglądarce, także w Web Workerze, `FlatDB.open()` bez opcji `store` używa `IndexedDbStore`. WAL jest dopisywany jako kolejne wpisy, a checkpoint zapisuje snapshot i czyści WAL w jednej transakcji IndexedDB.
2. Poza przeglądarką `FlatDB.open()` bez `store` kończy się błędem z podpowiedzią: `FsStore` dla plików albo `MemoryStore` dla danych tylko w pamięci.
3. OPFS może być dodatkowym magazynem dla trybu Worker.

## Uzasadnienie

- Wybór magazynu nie jest wyborem silnika. Rdzeń nie zapisuje na dysk (decyzja 0005), więc nowy magazyn nie zmienia silnika ani formatu danych.
- To, co jest wolne, nie zależy od dysku. Trwałe zatwierdzenie jednego wiersza w IndexedDB trwa około 0,36 ms. Wolne są kodowanie snapshotu i odtwarzanie bazy, czyli praca procesora.
- IndexedDB zapisuje snapshot i czyści WAL w jednej transakcji. OPFS nie ma transakcji obejmujących kilka plików, więc potrzebny byłby protokół ze zmianą nazwy pliku i jego pułapkami (Pillai i in., 2014).
- IndexedDB działa wszędzie: w wątku głównym, w Workerach i w rozszerzeniach.
- Błąd zamiast cichego magazynu w pamięci. Ostrzeżenie w konsoli łatwo przeoczyć, a przeoczenie oznacza utratę danych po zamknięciu procesu.

## Odrzucone warianty

- OPFS jako magazyn domyślny. Szybki dostęp wymaga Workera, a domyślny jest wątek główny (decyzja 0007).
- Magazyn w pamięci z ostrzeżeniem w Node.js. Zgodne wstecz, ale grozi cichą utratą danych.
- Pliki w bieżącym katalogu jako domyślny magazyn w Node.js. Zapis w miejscu, którego aplikacja nie wybrała.

## Konsekwencje

- Chrome domyślnie potwierdza zapis IndexedDB, gdy dane trafią do systemu operacyjnego, a nie na dysk (tryb `relaxed`). Taki zapis przetrwa awarię karty i przeglądarki, ale nie systemu. Opcja `durability: 'strict'` czeka na dysk (dokumentacja Chrome o trwałości IndexedDB).
- Przeglądarka może usunąć dane strony przy braku miejsca albo po okresie bez wizyt. Dlatego `FlatDB.open` prosi o trwały magazyn (`navigator.storage.persist()`).

## Historia

- 2026-10-06: IndexedDB jako magazyn mostu JavaScript.
- 2026-10-23: `FlatDB.open` bez magazynu przestało tworzyć magazyn w pamięci; w Node.js błąd zamiast ostrzeżenia.
