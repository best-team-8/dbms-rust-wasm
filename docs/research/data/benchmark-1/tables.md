# Benchmark 1 – tabele

Wygenerowano skryptem `make-report.mjs` z surowych wyników w `raw/`. Wartość: mediana z uruchomień procesu, w nawiasie pierwszy i trzeci kwartyl. Wersja Flat-SQL: `prototyp` (261d78fb44), obciążenie `benchmark-1`, N = 100 000 wierszy.

## Środowisko

| Element | Wartość |
|---|---|
| Procesor | AMD Ryzen 5 4600H with Radeon Graphics (12 wątków) |
| Pamięć | 15,4 GiB |
| System | win32 10.0.26200 x64 |
| Node.js | v24.21.0 |
| Przeglądarka | msedge 156.0.4314.42 (Chrome/156.0.0.0), crossOriginIsolated = true |
| Rust | rustc 1.99.0 (b940084d7 2026-09-28) |
| Uruchomienia | node: 10, native: 10, browser: 10 |
| sql.js | 1.14.2 |
| @sqlite.org/sqlite-wasm | 3.54.1-build1 |
| @journeyapps/wa-sqlite | 2.0.8 |
| @electric-sql/pglite | 0.5.11 |
| @duckdb/duckdb-wasm | 1.32.0 |
| alasql | 4.21.0 |
| dexie | 4.4.7 |
| pouchdb-browser | 9.0.0 |
| rxdb | 17.6.0 |
| playwright-core | 1.64.1 |
| esbuild | 0.28.2 |

## Obciążenie – Node.js, grupa „pamięć”: dane w pamięci, ten sam wątek

| Operacja | Flat-SQL | Flat-SQL (JSON) | sql.js | SQLite WASM (:memory:) | PGlite (pamięć) | DuckDB-Wasm | AlaSQL | czysty JS (Map) | Flat-SQL natywnie |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| Wstawienie N wierszy (1 transakcja) | 388 (383–399) ms | 391 (385–397) ms | 223 (222–224) ms | 910 (896–935) ms | 28,1 (28,0–28,6) s | 53,4 (52,0–54,3) s | 168 (163–189) ms | 49,6 (47,9–50,4) ms | 214 (213–216) ms |
| Odczyt po kluczu | 9,32 (9,14–9,41) µs | 5,65 (5,59–5,72) µs | 15,4 (15,3–15,6) µs | 9,59 (9,52–9,88) µs | 395 (392–404) µs | 1,41 (1,41–1,44) ms | 3,04 (2,96–3,50) ms [4388/10000] | 0,64 (0,62–0,70) µs | 2,38 (2,31–2,39) µs |
| COUNT po indeksie | 457 (450–473) µs | 442 (440–453) µs | 73,1 (71,2–73,4) µs | 59,6 (59,3–61,2) µs | 1,22 (1,20–1,26) ms | 1,15 (1,14–1,19) ms | 4,40 (4,25–4,85) ms | 0,69 (0,58–0,95) µs | 457 (457–461) µs |
| Zakres klucza (1000 wierszy) + SUM | 86,6 (80,3–89,1) µs | 88,1 (85,9–91,7) µs | 97,3 (96,2–102) µs | 83,7 (83,1–86,7) µs | 691 (673–704) µs | 1,44 (1,43–1,45) ms | 3,39 (3,24–3,76) ms | 42,3 (39,2–48,5) µs | 88,3 (83,2–88,3) µs |
| Pełny skan z filtrem | 7,97 (7,92–8,05) ms | 7,96 (7,91–8,08) ms | 7,96 (7,89–8,09) ms | 6,73 (6,68–6,76) ms | 11,4 (11,3–11,8) ms | 770 (744–791) µs | 4,18 (3,72–4,37) ms | 1,03 (0,90–1,83) ms | 8,14 (8,10–8,19) ms |
| GROUP BY (100 grup) | 16,9 (15,9–17,2) ms | 16,8 (16,3–16,9) ms | 116 (115–117) ms | 46,3 (45,7–47,3) ms | 31,6 (31,4–32,5) ms | 4,00 (3,96–4,02) ms | 6,76 (6,38–7,59) ms | 2,76 (2,68–2,86) ms | 14,9 (14,8–15,0) ms |
| ORDER BY … LIMIT 10 | 5,46 (5,36–5,64) ms | 5,44 (5,39–5,48) ms | 11,9 (11,9–12,1) ms | 10,9 (10,8–11,0) ms | 22,4 (22,2–23,1) ms | 1,38 (1,36–1,43) ms | 109 (106–112) ms | 1,58 (1,54–1,71) ms | 5,74 (5,72–5,78) ms |
| Pobranie 10 000 wierszy | 9,73 (9,62–9,86) ms | 9,69 (9,63–9,75) ms | 12,6 (12,4–13,0) ms | 51,1 (50,9–52,3) ms | 95,4 (95,0–95,7) ms | 11,1 (10,9–11,2) ms | 4,94 (4,89–5,86) ms | 1,34 (1,22–1,57) ms | 3,82 (3,80–3,85) ms |
| UPDATE 10% wierszy | 39,8 (32,7–43,1) ms | 35,3 (32,8–40,9) ms | 16,7 (15,5–26,2) ms | 9,43 (9,19–9,73) ms | 97,1 (96,4–98,8) ms | 11,1 (10,4–12,9) ms | 10,8 (9,65–11,4) ms | 2,88 (2,81–3,19) ms | 32,0 (31,7–32,2) ms |
| DELETE 10% wierszy | 28,2 (28,0–29,2) ms | 29,4 (28,2–31,2) ms | 20,2 (19,9–21,1) ms | 11,9 (11,6–12,4) ms | 48,9 (48,6–52,3) ms | 25,0 (22,7–26,1) ms | 7,55 (7,36–8,77) ms | 7,94 (7,00–8,20) ms | 25,6 (25,5–26,0) ms |
| Snapshot / eksport | 34,4 (34,1–35,1) ms | 34,2 (33,7–35,4) ms | 1,59 (1,43–1,96) ms | 3,41 (3,18–3,67) ms | 120 (118–122) ms | – | – | 23,1 (22,5–23,4) ms | 25,9 (25,6–26,1) ms |
| Odtworzenie / import | 116 (116–117) ms | 117 (116–119) ms | 1,24 (1,17–1,27) ms | 1,76 (1,71–1,87) ms | 547 (534–590) ms | – | – | 47,0 (44,6–49,7) ms | 114 (113–114) ms |

Zgodność wyników (sumy kontrolne wszystkich zapytań i stanu tabeli po każdej fazie, wszystkie uruchomienia): zgodne we wszystkich 860 porównaniach; przy operacjach przerwanych limitem czasu porównano pierwsze 3 wyniki.

## Obciążenie – przeglądarka, grupa „pamięć”: dane w pamięci, ten sam wątek

| Operacja | Flat-SQL | Flat-SQL (JSON) | sql.js | SQLite WASM (:memory:) | PGlite (pamięć) | DuckDB-Wasm | AlaSQL | czysty JS (Map) |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| Wstawienie N wierszy (1 transakcja) | 367 (364–369) ms | 343 (341–345) ms | 208 (208–213) ms | 846 (837–867) ms | 20,3 (20,2–20,7) s | 42,3 (42,0–42,6) s | 97,2 (95,4–102) ms | 53,9 (46,8–59,8) ms |
| Odczyt po kluczu | 9,15 (9,10–9,21) µs | 5,59 (5,51–5,65) µs | 14,1 (14,0–14,2) µs | 7,99 (7,92–8,16) µs | 299 (294–301) µs | 1,28 (1,28–1,28) ms | 1,64 (1,63–1,67) ms | 0,93 (0,86–0,98) µs |
| COUNT po indeksie | 438 (437–442) µs | 442 (437–448) µs | 68,9 (67,8–70,4) µs | 62,6 (61,6–65,7) µs | 1,12 (1,09–1,15) ms | 1,01 (1,00–1,04) ms | 2,86 (2,83–2,87) ms | 0,88 (0,78–0,99) µs |
| Zakres klucza (1000 wierszy) + SUM | 80,7 (79,5–81,5) µs | 78,9 (75,6–83,2) µs | 89,5 (88,9–89,7) µs | 78,2 (77,1–80,3) µs | 604 (594–613) µs | 1,31 (1,30–1,34) ms | 1,85 (1,83–1,88) ms | 41,1 (37,9–43,2) µs |
| Pełny skan z filtrem | 7,98 (7,92–8,09) ms | 7,97 (7,83–8,11) ms | 8,68 (8,31–8,74) ms | 7,19 (6,86–7,62) ms | 13,4 (13,3–13,5) ms | 589 (575–619) µs | 2,23 (2,19–2,29) ms | 632 (597–702) µs |
| GROUP BY (100 grup) | 15,7 (15,5–15,9) ms | 15,6 (15,4–15,8) ms | 110 (109–111) ms | 45,8 (45,0–46,3) ms | 33,2 (33,1–33,4) ms | 3,55 (3,52–3,58) ms | 4,10 (3,97–4,24) ms | 1,68 (1,63–1,83) ms |
| ORDER BY … LIMIT 10 | 5,52 (5,43–6,04) ms | 5,47 (5,45–5,50) ms | 11,6 (11,4–11,9) ms | 10,6 (10,5–10,7) ms | 23,2 (23,2–23,5) ms | 1,36 (1,34–1,39) ms | 87,4 (86,6–88,8) ms | 1,38 (1,25–1,55) ms |
| Pobranie 10 000 wierszy | 12,0 (11,9–12,1) ms | 7,75 (7,68–7,78) ms | 14,1 (14,0–14,2) ms | 37,9 (37,6–38,1) ms | 81,7 (81,3–82,8) ms | 12,2 (12,0–12,4) ms | 2,68 (2,60–2,76) ms | 598 (518–797) µs |
| UPDATE 10% wierszy | 31,0 (30,9–31,7) ms | 31,7 (31,2–32,0) ms | 13,7 (13,5–14,1) ms | 8,76 (8,68–8,93) ms | 101 (96,6–104) ms | 10,7 (10,0–12,1) ms | 8,19 (7,83–8,91) ms | 1,87 (1,77–2,26) ms |
| DELETE 10% wierszy | 27,3 (26,9–27,4) ms | 26,9 (26,7–27,2) ms | 18,9 (18,5–19,4) ms | 12,2 (11,9–12,3) ms | 49,7 (48,8–50,5) ms | 18,8 (18,4–19,3) ms | 6,69 (5,86–6,85) ms | 5,17 (5,08–5,68) ms |
| Snapshot / eksport | 32,1 (32,0–32,4) ms | 32,7 (32,5–33,1) ms | 1,54 (1,36–1,70) ms | 3,19 (2,78–3,41) ms | 131 (130–134) ms | – | – | 17,1 (16,9–17,4) ms |
| Odtworzenie / import | 116 (115–117) ms | 117 (116–117) ms | 1,19 (1,14–1,27) ms | 1,61 (1,57–1,95) ms | 470 (468–474) ms | – | – | 27,3 (27,0–28,0) ms |

Zgodność wyników (sumy kontrolne wszystkich zapytań i stanu tabeli po każdej fazie, wszystkie uruchomienia): zgodne we wszystkich 860 porównaniach.

## Obciążenie – przeglądarka, grupa „trwały zapis”: trwały zapis w IndexedDB, wątek główny

| Operacja | Flat-SQL + IndexedDB | Flat-SQL + IndexedDB (strict) | sql.js + eksport do IndexedDB | wa-sqlite + IDBBatchAtomicVFS | wa-sqlite + IDBBatchAtomicVFS (synchronous=normal) | PGlite + IndexedDB | Dexie.js | Dexie.js (strict) | PouchDB | RxDB (Dexie storage) |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| Wstawienie N wierszy (1 transakcja) | 471 (467–477) ms | 487 (477–491) ms | 235 (233–330) ms | 1,12 (1,10–1,17) s | 1,10 (1,09–1,13) s | 20,3 (20,2–20,7) s | 8,82 (8,74–9,00) s | 8,88 (8,76–9,07) s | 20,7 (20,4–21,1) s | 16,2 (16,0–16,4) s |
| Budowa indeksów (widoki PouchDB) | – | – | – | – | – | – | – | – | 96,2 (95,5–98,8) s | – |
| Odczyt po kluczu | 9,95 (9,85–10,0) µs | 9,93 (9,82–10,00) µs | 14,9 (14,7–15,1) µs | 1,31 (1,28–1,32) ms | 1,19 (1,17–1,23) ms | 36,1 (35,6–36,6) ms [538/10000] | 168 (162–172) µs | 171 (165–180) µs | 383 (381–389) µs | 10,8 (10,6–10,9) µs |
| COUNT po indeksie | 449 (439–460) µs | 450 (441–460) µs | 67,1 (66,8–68,9) µs | 1,36 (1,33–1,39) ms | 1,30 (1,24–1,31) ms | 36,8 (36,4–37,3) ms | 7,85 (7,77–7,91) ms | 7,86 (7,78–7,98) ms | 54,8 (54,7–56,0) ms [339/500] | 11,6 (11,6–11,8) ms |
| Zakres klucza (1000 wierszy) + SUM | 85,3 (82,9–87,3) µs | 84,8 (82,8–86,2) µs | 94,6 (93,5–95,6) µs | 2,15 (2,11–2,23) ms | 2,03 (1,98–2,14) ms | 36,3 (35,8–37,1) ms | 7,66 (7,55–7,96) ms | 7,57 (7,49–7,66) ms | 44,0 (43,4–44,6) ms | 30,3 (29,4–30,9) ms |
| Pełny skan z filtrem | 8,93 (8,86–9,06) ms | 9,04 (8,98–9,21) ms | 8,23 (8,22–8,30) ms | 163 (162–180) ms | 163 (161–170) ms | 49,0 (47,4–49,6) ms | 1,40 (1,37–1,43) s [14/20] | 1,42 (1,40–1,45) s [13/20] | 4,56 (4,53–4,61) s [5/20] | 1,92 (1,91–1,96) s [10/20] |
| GROUP BY (100 grup) | 16,2 (16,1–16,6) ms | 16,8 (16,3–17,3) ms | 110 (108–112) ms | 15,0 (14,8–15,2) s [4/20] | 14,9 (14,8–15,1) s [4/20] | 68,6 (67,2–69,4) ms | 573 (568–583) ms | 578 (573–581) ms | 4,61 (4,55–4,64) s [5/20] | 3,16 (3,11–3,24) s [7/20] |
| ORDER BY … LIMIT 10 | 5,95 (5,71–6,03) ms | 5,58 (5,51–5,86) ms | 11,3 (11,1–11,5) ms | 189 (180–193) ms | 177 (176–186) ms | 58,0 (56,8–58,7) ms | 700 (663–733) µs | 709 (664–747) µs | 7,47 (7,25–7,82) ms | 3,74 (3,69–3,75) s [6/20] |
| Pobranie 10 000 wierszy | 14,7 (14,6–14,8) ms | 14,7 (13,7–14,8) ms | 14,0 (13,8–14,2) ms | 29,3 (28,9–30,4) ms | 29,4 (28,8–43,0) ms | 115 (114–117) ms | 55,9 (55,6–56,4) ms | 55,5 (55,1–56,4) ms | 416 (414–427) ms | 267 (259–275) ms |
| UPDATE 10% wierszy | 38,9 (37,8–47,2) ms | 40,1 (39,0–41,1) ms | 36,8 (31,8–41,4) ms | 1,66 (1,59–1,72) s | 1,72 (1,70–2,02) s | 200 (198–202) ms | 2,88 (2,80–2,93) s | 2,90 (2,86–2,97) s | 7,17 (7,09–7,22) s | 5,82 (5,73–5,89) s |
| DELETE 10% wierszy | 33,4 (32,9–33,9) ms | 34,3 (34,3–34,7) ms | 37,7 (37,5–37,9) ms | 1,78 (1,75–1,80) s | 1,84 (1,83–1,87) s | 155 (148–158) ms | 2,49 (2,45–2,52) s | 2,55 (2,50–2,74) s | 7,15 (7,11–7,21) s | 4,87 (4,84–5,03) s |
| Trwałe zatwierdzenie 1 wiersza | 395 (384–403) µs | 1,00 (0,98–1,07) ms | 19,3 (19,2–19,3) ms | 7,52 (7,46–7,58) ms | 6,87 (6,79–7,13) ms | 54,5 (54,0–55,5) ms | 559 (550–587) µs | 1,28 (1,26–1,35) ms | 1,11 (1,07–1,15) ms | 1,33 (1,17–2,69) ms |
| 1000 zatwierdzeń, jedno oczekiwanie | 5,85 (5,73–5,95) ms | 6,66 (6,59–6,74) ms | 89,0 (85,9–93,6) ms | 4,79 (4,78–4,97) s | 3,70 (3,67–3,76) s | 57,4 (57,1–58,3) s | 358 (347–375) ms | 1,05 (0,99–1,09) s | 1,02 (0,99–1,05) s | 993 (917–1048) ms |
| Ponowne otwarcie + 1. zapytanie | 174 (172–175) ms | 173 (172–174) ms | 10,7 (10,5–10,8) ms | 7,69 (5,74–8,49) ms | 7,81 (6,96–8,05) ms | 847 (844–862) ms | 2,82 (2,77–2,96) ms | 3,12 (2,93–3,34) ms | 944 (937–957) ms | 13,6 (13,2–14,6) ms |

Zgodność wyników (sumy kontrolne wszystkich zapytań i stanu tabeli po każdej fazie, wszystkie uruchomienia): zgodne we wszystkich 1200 porównaniach; przy operacjach przerwanych limitem czasu porównano pierwsze 3 wyniki; stan tabeli zgodny w granicy błędu zaokrąglenia sumy (względnie 10^-9): RxDB (Dexie storage) (20 porównań).

## Obciążenie – przeglądarka, grupa „Worker”: silnik w Web Workerze, wywołanie = komunikat w obie strony

| Operacja | Flat-SQL w Workerze (pamięć) | Flat-SQL w Workerze + IndexedDB | SQLite WASM + opfs-sahpool | PGlite + OPFS (opfs-ahp) | DuckDB-Wasm w Workerze (pamięć) |
|---|---:|---:|---:|---:|---:|
| Wstawienie N wierszy (1 transakcja) | 367 (362–370) ms | 466 (464–468) ms | 930 (922–934) ms | 21,3 (21,2–21,3) s | 42,3 (41,9–42,7) s |
| Odczyt po kluczu | 65,4 (64,2–68,8) µs | 63,9 (62,8–65,2) µs | 93,0 (92,4–93,5) µs | 521 (511–543) µs | 1,58 (1,56–1,60) ms |
| COUNT po indeksie | 615 (587–628) µs | 601 (580–616) µs | 159 (157–160) µs | 1,45 (1,42–1,48) ms | 1,32 (1,31–1,35) ms |
| Zakres klucza (1000 wierszy) + SUM | 150 (148–153) µs | 149 (146–153) µs | 175 (174–178) µs | 920 (914–941) µs | 1,64 (1,62–1,67) ms |
| Pełny skan z filtrem | 8,42 (8,32–8,59) ms | 9,37 (9,32–9,45) ms | 8,18 (8,10–8,22) ms | 13,8 (13,7–14,1) ms | 914 (871–943) µs |
| GROUP BY (100 grup) | 18,2 (17,9–18,5) ms | 18,1 (16,9–19,7) ms | 46,5 (45,7–47,2) ms | 34,1 (33,8–34,7) ms | 4,10 (4,08–4,15) ms |
| ORDER BY … LIMIT 10 | 5,85 (5,78–5,94) ms | 5,93 (5,76–5,98) ms | 11,1 (11,0–11,1) ms | 23,4 (23,1–23,6) ms | 1,67 (1,61–1,71) ms |
| Pobranie 10 000 wierszy | 23,1 (23,0–23,4) ms | 25,9 (25,9–26,2) ms | 43,8 (43,4–44,2) ms | 95,1 (94,0–96,6) ms | 23,9 (23,7–24,1) ms |
| UPDATE 10% wierszy | 35,7 (35,2–36,7) ms | 44,4 (43,3–47,4) ms | 54,0 (53,4–57,9) ms | 117 (114–120) ms | 13,7 (13,1–14,5) ms |
| DELETE 10% wierszy | 27,2 (26,9–27,8) ms | 33,3 (32,9–33,7) ms | 54,2 (53,8–58,0) ms | 93,3 (90,8–97,7) ms | 18,9 (18,8–19,4) ms |
| Trwałe zatwierdzenie 1 wiersza | – | 457 (450–460) µs | 3,72 (3,66–3,95) ms | 1,26 (1,24–1,33) ms | – |
| 1000 zatwierdzeń, jedno oczekiwanie | – | 6,27 (6,00–6,43) ms | 3,48 (3,45–3,51) s | 1,04 (1,03–1,05) s | – |
| Ponowne otwarcie + 1. zapytanie | – | 181 (181–183) ms | 2,31 (2,16–2,46) ms | 7,14 (7,12–7,38) s | – |

Zgodność wyników (sumy kontrolne wszystkich zapytań i stanu tabeli po każdej fazie, wszystkie uruchomienia): zgodne we wszystkich 500 porównaniach.

## Narzut WebAssembly (H2): czas WASM / czas natywny, mediany

| Operacja | Node.js, obiekty JS | Node.js, JSON | przeglądarka, obiekty JS | przeglądarka, JSON |
|---|---:|---:|---:|---:|
| Wstawienie N wierszy (1 transakcja) | 1,81× | 1,83× | 1,71× | 1,60× |
| Odczyt po kluczu | 3,9× | 2,4× | 3,8× | 2,3× |
| COUNT po indeksie | 1,00× | 0,97× | 0,96× | 0,97× |
| Zakres klucza (1000 wierszy) + SUM | 0,98× | 1,00× | 0,91× | 0,89× |
| Pełny skan z filtrem | 0,98× | 0,98× | 0,98× | 0,98× |
| GROUP BY (100 grup) | 1,13× | 1,12× | 1,05× | 1,05× |
| ORDER BY … LIMIT 10 | 0,95× | 0,95× | 0,96× | 0,95× |
| Pobranie 10 000 wierszy | 2,5× | 2,5× | 3,1× | 2,0× |
| UPDATE 10% wierszy | 1,24× | 1,10× | 0,97× | 0,99× |
| DELETE 10% wierszy | 1,10× | 1,15× | 1,07× | 1,05× |
| Snapshot / eksport | 1,33× | 1,32× | 1,24× | 1,26× |
| Odtworzenie / import | 1,02× | 1,03× | 1,02× | 1,03× |

## Flat-SQL na tle SQLite (H4): czas silnika / czas Flat-SQL (JSON), mediany; > 1 – Flat-SQL szybszy

| Operacja | sql.js, Node.js | SQLite WASM, Node.js | sql.js, przeglądarka | SQLite WASM, przeglądarka |
|---|---:|---:|---:|---:|
| Wstawienie N wierszy (1 transakcja) | 0,57× | 2,3× | 0,61× | 2,5× |
| Odczyt po kluczu | 2,7× | 1,70× | 2,5× | 1,43× |
| COUNT po indeksie | 0,17× | 0,13× | 0,16× | 0,14× |
| Zakres klucza (1000 wierszy) + SUM | 1,10× | 0,95× | 1,13× | 0,99× |
| Pełny skan z filtrem | 1,00× | 0,85× | 1,09× | 0,90× |
| GROUP BY (100 grup) | 6,9× | 2,8× | 7,0× | 2,9× |
| ORDER BY … LIMIT 10 | 2,2× | 2,0× | 2,1× | 1,94× |
| Pobranie 10 000 wierszy | 1,30× | 5,3× | 1,82× | 4,9× |
| UPDATE 10% wierszy | 0,47× | 0,27× | 0,43× | 0,28× |
| DELETE 10% wierszy | 0,69× | 0,40× | 0,70× | 0,45× |
| Snapshot / eksport | 0,05× | 0,10× | 0,05× | 0,10× |
| Odtworzenie / import | 0,01× | 0,01× | 0,01× | 0,01× |

## Koszt Workera (4.12): czas w Workerze / czas w wątku głównym, przeglądarka

| Operacja | Flat-SQL, obiekty JS |
|---|---:|
| Wstawienie N wierszy (1 transakcja) | 1,00× |
| Odczyt po kluczu | 7,2× |
| COUNT po indeksie | 1,40× |
| Zakres klucza (1000 wierszy) + SUM | 1,85× |
| Pełny skan z filtrem | 1,06× |
| GROUP BY (100 grup) | 1,16× |
| ORDER BY … LIMIT 10 | 1,06× |
| Pobranie 10 000 wierszy | 1,93× |
| UPDATE 10% wierszy | 1,15× |
| DELETE 10% wierszy | 0,99× |

Różnica median (koszt komunikatu w obie strony): odczyt po kluczu 56,3 µs, pobranie 10 000 wierszy 11,1 ms.

## Rozmiar, zimny start, pamięć i wymagania (4.1, 4.3, 4.4)

Zimny start: nowy proces przeglądarki z pustym profilem (bez pamięci podręcznej HTTP i skompilowanego kodu), od rozpoczęcia importu silnika do wyniku pierwszego zapytania: import i kompilacja modułu, otwarcie bazy ze schematem, pierwszy zapis i odczyt. Rozmiar: wszystkie pliki pakietu silnika pobrane w zimnym starcie (bez kodu pomiaru), gzip -9; dla pakietu z kilkoma konfiguracjami największy zestaw, bo lista zasobów strony w wątku głównym nie zawierała pliku .wasm pakietu synchronicznego DuckDB-Wasm, choć silnik go wczytał. Pamięć: po wczytaniu 100 000 wierszy, pamięć liniowa WebAssembly (nie maleje) i przyrost sterty JS w Node.js po pełnym odśmiecaniu.

| Silnik | Grupa | Wymagania | Moduł (gzip) | Zimny start – przeglądarka | w tym import i kompilacja | Zimny start – Node.js | Pamięć WASM po wczytaniu | Przyrost sterty JS (Node.js) |
|---|---|---|---:|---:|---:|---:|---:|---:|
| Flat-SQL | pamięć | brak | 230 KiB | 47,9 (45,3–54,6) ms | 38,0 (34,7–41,8) ms | 56,3 (54,1–57,4) ms | 73,3 MiB | 0,0 MiB |
| Flat-SQL (JSON) | pamięć | brak | 230 KiB | 48,3 (45,8–51,6) ms | 36,3 (34,1–39,6) ms | 54,6 (53,3–57,3) ms | 73,3 MiB | 0,0 MiB |
| sql.js | pamięć | brak | 331 KiB | 63,2 (59,5–68,9) ms | 35,7 (34,7–38,3) ms | 49,4 (48,4–50,5) ms | 21,1 MiB | 0,1 MiB |
| SQLite WASM (:memory:) | pamięć | brak | 583 KiB | 97,2 (93,5–103) ms | 73,5 (69,4–75,8) ms | 59,8 (58,7–60,0) ms | 8,0 MiB | 0,2 MiB |
| PGlite (pamięć) | pamięć | brak | 5748 KiB | 1,57 (1,56–1,59) s | 50,3 (48,5–52,7) ms | 1,78 (1,73–1,84) s | 189 MiB | 0,6 MiB |
| DuckDB-Wasm | pamięć | brak | 7813 KiB | 844 (836–862) ms | 770 (759–786) ms | 1,12 (1,10–1,15) s | 23,1 MiB | 2,5 MiB |
| AlaSQL | pamięć | brak | 111 KiB | 72,2 (65,2–78,5) ms | 59,1 (54,9–67,7) ms | 82,7 (82,0–83,7) ms | nie dotyczy | 13,5 MiB |
| czysty JS (Map) | pamięć | brak | 0 KiB | 6,22 (5,72–6,66) ms | 6,02 (5,50–6,46) ms | 1,97 (1,79–2,04) ms | nie dotyczy | 20,7 MiB |
| Flat-SQL + IndexedDB | trwały zapis | IndexedDB | 230 KiB | 91,7 (83,2–102) ms | 40,2 (35,8–47,4) ms | – | – | – |
| Flat-SQL + IndexedDB (strict) | trwały zapis | IndexedDB | 230 KiB | 87,2 (84,1–95,2) ms | 35,4 (34,9–38,6) ms | – | – | – |
| sql.js + eksport do IndexedDB | trwały zapis | IndexedDB | 331 KiB | 97,7 (93,5–107) ms | 34,3 (32,5–38,4) ms | – | – | – |
| wa-sqlite + IDBBatchAtomicVFS | trwały zapis | IndexedDB | 821 KiB | 211 (204–216) ms | 95,7 (93,4–105) ms | – | – | – |
| wa-sqlite + IDBBatchAtomicVFS (synchronous=normal) | trwały zapis | IndexedDB | 821 KiB | 220 (211–379) ms | 111 (103–116) ms | – | – | – |
| PGlite + IndexedDB | trwały zapis | IndexedDB | 5748 KiB | 2,22 (2,19–2,53) s | 50,0 (47,2–54,6) ms | – | – | – |
| Dexie.js | trwały zapis | IndexedDB | 49 KiB | 68,1 (62,5–70,4) ms | 23,0 (21,6–25,7) ms | – | nie dotyczy | – |
| Dexie.js (strict) | trwały zapis | IndexedDB | 49 KiB | 64,4 (63,8–66,9) ms | 23,4 (21,6–23,9) ms | – | nie dotyczy | – |
| PouchDB | trwały zapis | IndexedDB | 42 KiB | 77,9 (76,3–83,7) ms | 28,7 (25,6–29,8) ms | – | nie dotyczy | – |
| RxDB (Dexie storage) | trwały zapis | IndexedDB | 80 KiB | 103 (95,7–109) ms | 31,1 (29,1–37,3) ms | – | nie dotyczy | – |
| Flat-SQL w Workerze (pamięć) | Worker | Worker | 230 KiB | 70,0 (65,7–74,2) ms | 56,8 (55,0–62,7) ms | – | – | – |
| Flat-SQL w Workerze + IndexedDB | Worker | Worker, IndexedDB | 230 KiB | 116 (112–122) ms | 61,2 (56,2–71,3) ms | – | – | – |
| SQLite WASM + opfs-sahpool | Worker | Worker, OPFS | 583 KiB | 490 (487–501) ms | 410 (404–418) ms | – | – | – |
| PGlite + OPFS (opfs-ahp) | Worker | Worker, OPFS | 5748 KiB | 9,67 (9,62–9,71) s | 75,2 (69,9–83,8) ms | – | – | – |
| DuckDB-Wasm w Workerze (pamięć) | Worker | Worker | 7813 KiB | 891 (887–903) ms | 805 (802–818) ms | – | – | – |

### Silniki niemierzone – dane deklarowane przez autorów

Wartości podane przez autorów, niepotwierdzone pomiarem w tym benchmarku; warunki ich pomiaru są inne niż w tabelach powyżej i nie można ich porównywać wprost.

| Silnik | Źródło | Odczyt po kluczu | Moduł (gzip) | Uwagi |
|---|---|---:|---:|---:|
| Zeta-Lite | [D5] | 3,20–3,70 µs | 2,87 MB | preprint, wrzesień 2026; warunki pomiaru autora, nie powtórzone w tym benchmarku |

## Trwały zapis (H5, 4.9)

| Silnik | Grupa | Wstawienie N wierszy do trwałości | Zatwierdzenie 1 wiersza | p95 zatwierdzenia | 1000 zatwierdzeń – na jedno | Ponowne otwarcie + 1. zapytanie |
|---|---|---:|---:|---:|---:|---:|
| Flat-SQL + IndexedDB | trwały zapis | 471 (467–477) ms | 395 (384–403) µs | 588 (551–615) µs | 5,85 (5,73–5,95) µs | 174 (172–175) ms |
| Flat-SQL + IndexedDB (strict) | trwały zapis | 487 (477–491) ms | 1,00 (0,98–1,07) ms | 1,24 (1,22–1,26) ms | 6,66 (6,59–6,74) µs | 173 (172–174) ms |
| sql.js + eksport do IndexedDB | trwały zapis | 235 (233–330) ms | 19,3 (19,2–19,3) ms | 21,1 (21,0–21,5) ms | 89,0 (85,9–93,6) µs | 10,7 (10,5–10,8) ms |
| wa-sqlite + IDBBatchAtomicVFS | trwały zapis | 1,12 (1,10–1,17) s | 7,52 (7,46–7,58) ms | 9,88 (9,60–10,2) ms | 4,79 (4,78–4,97) ms | 7,69 (5,74–8,49) ms |
| wa-sqlite + IDBBatchAtomicVFS (synchronous=normal) | trwały zapis | 1,10 (1,09–1,13) s | 6,87 (6,79–7,13) ms | 9,01 (8,78–9,11) ms | 3,70 (3,67–3,76) ms | 7,81 (6,96–8,05) ms |
| PGlite + IndexedDB | trwały zapis | 20,3 (20,2–20,7) s | 54,5 (54,0–55,5) ms | 57,9 (57,0–59,0) ms | 57,4 (57,1–58,3) ms | 847 (844–862) ms |
| Dexie.js | trwały zapis | 8,82 (8,74–9,00) s | 559 (550–587) µs | 777 (741–829) µs | 358 (347–375) µs | 2,82 (2,77–2,96) ms |
| Dexie.js (strict) | trwały zapis | 8,88 (8,76–9,07) s | 1,28 (1,26–1,35) ms | 1,56 (1,54–1,60) ms | 1,05 (0,99–1,09) ms | 3,12 (2,93–3,34) ms |
| PouchDB | trwały zapis | 20,7 (20,4–21,1) s | 1,11 (1,07–1,15) ms | 1,44 (1,42–1,49) ms | 1,02 (0,99–1,05) ms | 944 (937–957) ms |
| RxDB (Dexie storage) | trwały zapis | 16,2 (16,0–16,4) s | 1,33 (1,17–2,69) ms | 1,70 (1,50–13,5) ms | 993 (917–1048) µs | 13,6 (13,2–14,6) ms |
| Flat-SQL w Workerze + IndexedDB | Worker | 466 (464–468) ms | 457 (450–460) µs | 685 (642–720) µs | 6,27 (6,00–6,43) µs | 181 (181–183) ms |
| SQLite WASM + opfs-sahpool | Worker | 930 (922–934) ms | 3,72 (3,66–3,95) ms | 4,25 (4,22–4,64) ms | 3,48 (3,45–3,51) ms | 2,31 (2,16–2,46) ms |
| PGlite + OPFS (opfs-ahp) | Worker | 21,3 (21,2–21,3) s | 1,26 (1,24–1,33) ms | 1,58 (1,52–1,63) ms | 1,04 (1,03–1,05) ms | 7,14 (7,12–7,38) s |

## Pauza checkpointu w wątku głównym (4.10)

Najdłuższa przerwa między kolejnymi obrotami pętli zdarzeń (sonda `MessageChannel`) podczas `checkpoint()`; dla sql.js – eksport całej bazy i zapis do IndexedDB. Budżet jednej klatki przy 60 Hz: 16,7 ms.

| Silnik | 10 tys. – najdłuższa pauza | 10 tys. – cały checkpoint | 50 tys. – najdłuższa pauza | 50 tys. – cały checkpoint | 100 tys. – najdłuższa pauza | 100 tys. – cały checkpoint | Snapshot przy 100 tys. | Bezczynność – najdłuższa przerwa |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| Flat-SQL + IndexedDB | 6,66 (5,98–6,77) ms | 11,5 (11,0–11,7) ms | 20,9 (20,6–21,1) ms | 26,7 (26,2–27,1) ms | 40,3 (40,2–40,7) ms | 48,2 (47,6–48,6) ms | 5,7 MiB | 2,30 (1,17–2,49) ms |
| Flat-SQL + IndexedDB (strict) | 6,21 (6,19–6,34) ms | 12,9 (12,1–13,2) ms | 20,5 (20,3–20,7) ms | 28,7 (27,9–29,2) ms | 40,6 (40,4–40,9) ms | 53,3 (51,9–56,1) ms | 5,7 MiB | 2,25 (2,19–2,42) ms |
| sql.js + eksport do IndexedDB | 2,34 (2,05–2,53) ms | 6,45 (5,90–6,80) ms | 6,65 (6,54–7,06) ms | 11,9 (11,4–15,3) ms | 11,8 (11,5–12,1) ms | 19,2 (18,5–20,1) ms | 4,5 MiB | 2,34 (1,97–2,67) ms |

## Koszt utrwalania plików (4.13)

Linux: WSL 2 na tym samym laptopie, katalog roboczy /tmp na ext4 (wirtualny dysk WSL na tym samym SSD), CLI zbudowane na x86_64-unknown-linux-musl z rust-lld, bo w systemie nie ma kompilatora C; jądro 6.6.87.2-microsoft-standard-WSL2, Ubuntu 24.04.4 LTS, Node.js v24.21.0.

| Magazyn | Wstawienie N wierszy (1 transakcja) | Trwałe zatwierdzenie 1 wiersza | 1000 zatwierdzeń, jedno oczekiwanie | Checkpoint (snapshot + fsync katalogu) | Ponowne otwarcie + 1. zapytanie |
|---|---:|---:|---:|---:|---:|
| Windows: FileStorage (CLI), fsync | 282 (268–430) ms | 579 (525–596) µs | 3,61 (3,50–3,86) ms | 39,7 (38,2–40,9) ms | 120 (120–121) ms |
| Windows: FileStorage (CLI), bez fsync | 263 (258–267) ms | 10,1 (9,76–10,5) µs | 2,79 (2,75–2,92) ms | 40,1 (38,4–41,3) ms | 121 (120–121) ms |
| Windows: FsStore (Node.js), fsync | 610 (602–616) ms | 1,34 (1,30–1,36) ms | 7,05 (6,83–7,13) ms | 48,2 (47,4–48,6) ms | 112 (112–115) ms |
| Windows: FsStore (Node.js), bez fsync | 399 (397–402) ms | 602 (588–624) µs | 5,54 (5,18–5,61) ms | 47,8 (46,9–48,4) ms | 101 (99,0–102) ms |
| Linux: FileStorage (CLI), fsync | 358 (353–360) ms | 2,15 (2,10–2,18) ms | 7,68 (7,33–7,92) ms | 56,5 (55,8–56,6) ms | 166 (165–167) ms |
| Linux: FileStorage (CLI), bez fsync | 344 (342–348) ms | 9,01 (8,73–9,19) µs | 4,11 (3,96–4,30) ms | 54,5 (54,3–56,2) ms | 166 (165–167) ms |
| Linux: FsStore (Node.js), fsync | 628 (624–636) ms | 2,91 (2,84–2,95) ms | 10,5 (10,3–11,0) ms | 78,9 (76,1–80,9) ms | 133 (132–135) ms |
| Linux: FsStore (Node.js), bez fsync | 462 (461–466) ms | 400 (387–413) µs | 5,09 (4,94–5,31) ms | 74,8 (72,2–76,7) ms | 121 (119–121) ms |

Windows: runda synchronizacji z jedną zmianą (`/sync/push` + `/sync/pull`, localhost) wobec serwera z katalogiem danych (FsStore, fsync) 698 (676–730) µs, wobec serwera w pamięci 583 (571–607) µs.

Linux: runda synchronizacji z jedną zmianą (`/sync/push` + `/sync/pull`, localhost) wobec serwera z katalogiem danych (FsStore, fsync) 798 (761–830) µs, wobec serwera w pamięci 633 (620–671) µs.

## Local-First a Cloud-First (H1)

Odczyt po kluczu z tabeli 10 000 wierszy: lokalnie (`db.exec`, z parsowaniem SQL) i przez `POST /api/query` do serwera z tym samym silnikiem; opóźnienie sieci symulowane po stronie serwera.

| Tryb | Node.js | krotność lokalnego | Przeglądarka | krotność lokalnego |
|---|---:|---:|---:|---:|
| lokalnie (WASM w procesie) | 14,8 (14,6–15,3) µs | 1,00× | 21,9 (20,7–22,7) µs | 1,00× |
| serwer, localhost | 637 (622–665) µs | 43× | 2,28 (2,22–2,34) ms | 104× |
| serwer, RTT 20 ms | 20,8 (20,8–20,8) ms | 1403× | 22,8 (22,7–22,8) ms | 1040× |
| serwer, RTT 50 ms | 50,8 (50,8–50,8) ms | 3426× | 52,8 (52,7–52,9) ms | 2412× |
| serwer, RTT 100 ms | 101 (101–101) ms | 6796× | 103 (103–103) ms | 4701× |

## Rozrzut między uruchomieniami

Współczynnik zmienności (odchylenie standardowe / średnia) średnich z 386 par silnik–operacja: mediana 4,1%, 90. percentyl 16,5%, maksimum 98,8%. Największy rozrzut:

| Środowisko | Silnik | Operacja | CV |
|---|---|---|---:|
| przeglądarka | Flat-SQL + IndexedDB | UPDATE 10% wierszy | 98,8% |
| przeglądarka | sql.js + eksport do IndexedDB | 1000 zatwierdzeń, jedno oczekiwanie | 82,5% |
| przeglądarka | Flat-SQL + IndexedDB (strict) | UPDATE 10% wierszy | 74,5% |
| przeglądarka | SQLite WASM + opfs-sahpool | UPDATE 10% wierszy | 59,0% |
| przeglądarka | wa-sqlite + IDBBatchAtomicVFS | Ponowne otwarcie + 1. zapytanie | 51,7% |
| przeglądarka | PGlite + OPFS (opfs-ahp) | DELETE 10% wierszy | 47,8% |
| przeglądarka | Dexie.js (strict) | Ponowne otwarcie + 1. zapytanie | 43,7% |
| przeglądarka | RxDB (Dexie storage) | Trwałe zatwierdzenie 1 wiersza | 43,5% |

## Wykresy

- [figures/01-odczyt-po-kluczu.svg](figures/01-odczyt-po-kluczu.svg)
- [figures/02-flatsql-a-sqlite.svg](figures/02-flatsql-a-sqlite.svg)
- [figures/03-narzut-wasm.svg](figures/03-narzut-wasm.svg)
- [figures/04-zimny-start.svg](figures/04-zimny-start.svg)
- [figures/05-pauza-checkpointu.svg](figures/05-pauza-checkpointu.svg)
- [figures/06-trwale-zatwierdzenie.svg](figures/06-trwale-zatwierdzenie.svg)
- [figures/07-local-first-a-cloud-first.svg](figures/07-local-first-a-cloud-first.svg)
