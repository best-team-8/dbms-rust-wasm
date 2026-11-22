# 0025. Pamięć agentów przez gałęzie bazy ze scalaniem warunkowym, bez wielu wersji wierszy

- Status: proponowana
- Ostatnia zmiana: 2026-10-28

## Kontekst

Pamięć asystentów i agentów programowych w przeglądarce to rosnąca kategoria zastosowań. Zeta-Lite izoluje pracę agentów przez wiele wersji wierszy (MVCC) i rozgałęzianie bazy. Flat-SQL wykonuje transakcje po kolei, bez wersji wierszy (decyzja 0011). Pytanie brzmiało: co musiałoby się zmienić, żeby Flat-SQL nadawał się do pamięci agentów, bez straty dla innych zastosowań.

## Decyzja

1. Silnik domyślny zostaje bez wersji wierszy.
2. Izolację pracy agenta dają gałęzie bazy. Gałąź powstaje ze snapshotu jako osobna instancja z własnym identyfikatorem urządzenia, zbiera własne zmiany i jest scalana tą samą drogą co zmiany z serwera.
3. Scalanie warunkowe sprawdza znaczniki wierszy, które gałąź przeczytała i zmieniła. Jeśli któryś zmienił się w bazie głównej, całe scalenie jest odrzucane. W innym wypadku zmiany wchodzą jedną transakcją.
4. Funkcje dla agentów (wyszukiwanie pełnotekstowe, wektory, gałęzie) są dokładane poza gorącymi ścieżkami silnika i mogą być wyłączane przy kompilacji.

## Uzasadnienie

- Wersje wierszy objęłyby około 25–35% rdzenia. Każdy odczyt sprawdzałby widoczność wersji, a liczenie po indeksie bez czytania wierszy i `UPDATE` w miejscu przestałyby działać. Stare wersje zwiększyłyby pamięć, która jest głównym ograniczeniem modelu. Zyskałaby tylko pamięć agentów.
- W jednym wątku przeplatają się tylko transakcje, które czekają w środku (`await`). Agent, który najpierw rozumuje, a potem wykonuje krótką transakcję, ma pełną izolację.
- Gałęzie korzystają z tego, co już jest: snapshotu, odtworzenia i nakładania paczek. Reguła „ostatni zapis wygrywa” zbiega się przy dowolnej kolejności scaleń (Shapiro i in., 2011).
- Scalanie warunkowe zamyka luki samej reguły: ciche nadpisania, decyzje na nieaktualnych danych i częściowe scalenia.

## Odrzucone warianty

- Wiele wersji wierszy w silniku domyślnym. Koszt dla wszystkich, zysk dla jednej kategorii.
- Wiele wersji wierszy jako wariant kompilacji. Wykonalne, ale to osobny duży projekt.
- Gałęzie scalane bez sprawdzenia. Ciche nadpisania, groźne dla agentów działających na wspólnym stanie.

## Konsekwencje

- Utworzenie gałęzi kosztuje kopię bazy: przy 100 000 wierszy snapshot i odtworzenie trwają razem około 70 ms, przy kilku tysiącach wpisów kilka milisekund.
- Gwarancje scalania warunkowego dotyczą jednego urządzenia. Między urządzeniami atomowy pozostaje wiersz.

## Historia

- 2026-10-28: kierunek dla pamięci agentów.
