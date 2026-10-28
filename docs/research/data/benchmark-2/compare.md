Przed: prototyp (bench: module size of Worker engines, plain values from DuckDB, table alignment in the report); po: po-2.5 (docs: compare build options of the WebAssembly module).
Obciążenie procesora przed pomiarem: 6,2%. Uruchomienia naprzemienne, 10 na wersję.
Moduł WebAssembly: 578926 → 724002 B, gzip -9: 219575 → 274098 B (+54523 B).

### browser – checkpoint – flatsql-idb

| Metryka | Przed | Po | Zmiana mediany | 95% CI średnich rozłączne |
|---|---:|---:|---:|---|
| pause idle | 2,09 (1,92–2,31) ms | 2,11 (2,01–2,22) ms | +0,8% | nie (w rozrzucie) |
| pause 10000 | 6,27 (6,11–6,80) ms | 2,69 (2,46–2,92) ms | -57,2% | tak, szybciej/mniej |
| checkpoint 10000 | 10,8 (10,4–12,1) ms | 10,4 (9,77–11,1) ms | -3,8% | nie (w rozrzucie) |
| pause 50000 | 20,4 (20,1–20,5) ms | 1,22 (1,12–1,39) ms | -94,0% | tak, szybciej/mniej |
| checkpoint 50000 | 25,7 (25,5–26,3) ms | 19,2 (18,8–19,5) ms | -25,3% | tak, szybciej/mniej |
| pause 100000 | 40,3 (39,9–40,9) ms | 2,60 (1,50–3,29) ms | -93,6% | tak, szybciej/mniej |
| checkpoint 100000 | 47,9 (47,2–48,2) ms | 35,6 (35,0–36,9) ms | -25,5% | nie (w rozrzucie) |

### browser – checkpoint – flatsql-idb-strict

| Metryka | Przed | Po | Zmiana mediany | 95% CI średnich rozłączne |
|---|---:|---:|---:|---|
| pause idle | 2,14 (2,10–2,66) ms | 2,19 (2,10–2,57) ms | +2,7% | nie (w rozrzucie) |
| pause 10000 | 6,23 (6,09–6,58) ms | 2,56 (2,30–2,70) ms | -58,9% | tak, szybciej/mniej |
| checkpoint 10000 | 12,2 (12,2–13,1) ms | 11,8 (11,3–12,3) ms | -3,7% | nie (w rozrzucie) |
| pause 50000 | 20,6 (20,4–20,8) ms | 1,68 (1,46–1,87) ms | -91,8% | tak, szybciej/mniej |
| checkpoint 50000 | 28,4 (28,1–29,1) ms | 22,7 (22,3–23,4) ms | -20,2% | tak, szybciej/mniej |
| pause 100000 | 40,0 (39,6–40,3) ms | 1,56 (1,49–2,57) ms | -96,1% | tak, szybciej/mniej |
| checkpoint 100000 | 51,9 (51,5–53,3) ms | 40,1 (38,9–41,6) ms | -22,8% | tak, szybciej/mniej |

### browser – cold – flatsql

| Metryka | Przed | Po | Zmiana mediany | 95% CI średnich rozłączne |
|---|---:|---:|---:|---|
| init | 32,9 (29,4–36,2) ms | 38,0 (33,7–40,9) ms | +15,4% | nie (w rozrzucie) |
| open | 5,79 (5,71–6,64) ms | 4,44 (4,37–4,82) ms | -23,4% | tak, szybciej/mniej |
| firstQuery | 3,78 (3,56–3,92) ms | 5,55 (5,39–5,80) ms | +46,7% | tak, wolniej/więcej |
| total | 42,5 (39,6–45,9) ms | 49,0 (43,4–51,1) ms | +15,3% | nie (w rozrzucie) |

### browser – cold – flatsql-idb

| Metryka | Przed | Po | Zmiana mediany | 95% CI średnich rozłączne |
|---|---:|---:|---:|---|
| init | 35,4 (33,4–36,3) ms | 45,3 (38,0–50,4) ms | +28,1% | nie (w rozrzucie) |
| open | 40,1 (38,2–41,7) ms | 40,7 (37,4–41,7) ms | +1,7% | nie (w rozrzucie) |
| firstQuery | 4,01 (3,86–4,11) ms | 6,07 (5,86–6,40) ms | +51,2% | tak, wolniej/więcej |
| total | 80,8 (77,4–81,1) ms | 90,5 (84,7–96,2) ms | +12,0% | nie (w rozrzucie) |

### browser – cold – flatsql-idb-strict

| Metryka | Przed | Po | Zmiana mediany | 95% CI średnich rozłączne |
|---|---:|---:|---:|---|
| init | 35,6 (34,3–51,0) ms | 42,5 (37,5–52,9) ms | +19,3% | nie (w rozrzucie) |
| open | 42,8 (42,3–43,9) ms | 42,1 (40,1–45,1) ms | -1,6% | nie (w rozrzucie) |
| firstQuery | 4,66 (4,51–4,87) ms | 6,59 (6,26–7,01) ms | +41,4% | tak, wolniej/więcej |
| total | 84,9 (81,8–98,8) ms | 93,1 (86,0–101) ms | +9,7% | nie (w rozrzucie) |

### browser – cold – flatsql-json

| Metryka | Przed | Po | Zmiana mediany | 95% CI średnich rozłączne |
|---|---:|---:|---:|---|
| init | 37,5 (33,7–42,3) ms | 39,4 (37,0–39,7) ms | +4,8% | nie (w rozrzucie) |
| open | 5,58 (5,40–5,77) ms | 4,91 (4,65–5,65) ms | -12,1% | nie (w rozrzucie) |
| firstQuery | 4,20 (3,99–4,27) ms | 6,48 (6,31–7,04) ms | +54,3% | tak, wolniej/więcej |
| total | 47,6 (43,5–51,8) ms | 51,1 (48,5–51,9) ms | +7,4% | nie (w rozrzucie) |

### browser – cold – flatsql-worker

| Metryka | Przed | Po | Zmiana mediany | 95% CI średnich rozłączne |
|---|---:|---:|---:|---|
| init | 55,8 (49,2–63,7) ms | 63,5 (49,3–66,6) ms | +13,7% | nie (w rozrzucie) |
| open | 6,33 (5,66–6,88) ms | 4,99 (4,69–5,24) ms | -21,2% | tak, szybciej/mniej |
| firstQuery | 4,78 (4,34–5,29) ms | 6,86 (6,22–7,07) ms | +43,4% | tak, wolniej/więcej |
| total | 68,4 (59,6–75,5) ms | 76,2 (60,7–78,7) ms | +11,3% | nie (w rozrzucie) |

### browser – cold – flatsql-worker-idb

| Metryka | Przed | Po | Zmiana mediany | 95% CI średnich rozłączne |
|---|---:|---:|---:|---|
| init | 57,7 (54,2–62,5) ms | 58,8 (51,5–63,5) ms | +1,9% | nie (w rozrzucie) |
| open | 41,4 (40,7–43,8) ms | 40,1 (38,6–41,5) ms | -3,2% | nie (w rozrzucie) |
| firstQuery | 4,19 (3,95–4,32) ms | 6,41 (6,26–6,67) ms | +52,8% | tak, wolniej/więcej |
| total | 104 (101–111) ms | 106 (98,2–113) ms | +2,2% | nie (w rozrzucie) |

### browser – memory – flatsql

| Metryka | Przed | Po | Zmiana mediany | 95% CI średnich rozłączne |
|---|---:|---:|---:|---|
| wasm_after_load (rozmiar) | 73,3 MiB | 51,8 MiB | -29,3% | tak, szybciej/mniej |

### browser – memory – flatsql-json

| Metryka | Przed | Po | Zmiana mediany | 95% CI średnich rozłączne |
|---|---:|---:|---:|---|
| wasm_after_load (rozmiar) | 73,3 MiB | 51,8 MiB | -29,3% | tak, szybciej/mniej |

### browser – network – flatsql

| Metryka | Przed | Po | Zmiana mediany | 95% CI średnich rozłączne |
|---|---:|---:|---:|---|
| local (WASM, in-process) | 14,2 (12,7–14,8) µs | 14,2 (12,6–15,5) µs | -0,3% | nie (w rozrzucie) |
| cloud API (localhost HTTP) | 2,53 (2,41–2,58) ms | 2,55 (2,45–2,60) ms | +0,9% | nie (w rozrzucie) |
| cloud API (+20 ms RTT) | 22,9 (22,8–23,0) ms | 22,9 (22,8–22,9) ms | -0,3% | nie (w rozrzucie) |
| cloud API (+50 ms RTT) | 53,0 (52,9–53,0) ms | 52,9 (52,9–53,0) ms | -0,1% | nie (w rozrzucie) |
| cloud API (+100 ms RTT) | 103 (103–103) ms | 103 (103–103) ms | +0,1% | nie (w rozrzucie) |

### browser – workload – flatsql

| Metryka | Przed | Po | Zmiana mediany | 95% CI średnich rozłączne |
|---|---:|---:|---:|---|
| insert | 357 (355–361) ms | 267 (266–269) ms | -25,4% | tak, szybciej/mniej |
| point_lookup | 8,78 (8,72–8,90) µs | 3,28 (3,15–3,30) µs | -62,7% | tak, szybciej/mniej |
| index_lookup | 437 (428–449) µs | 1,90 (1,84–1,93) µs | -99,6% | tak, szybciej/mniej |
| range_scan | 77,6 (77,4–79,5) µs | 45,5 (43,7–47,1) µs | -41,4% | tak, szybciej/mniej |
| full_scan | 7,83 (7,78–7,90) ms | 4,40 (4,39–4,42) ms | -43,9% | tak, szybciej/mniej |
| group_by | 15,6 (15,4–15,7) ms | 9,38 (9,28–9,53) ms | -39,9% | tak, szybciej/mniej |
| top_k | 5,44 (5,38–5,56) ms | 5,33 (5,29–5,39) ms | -2,1% | nie (w rozrzucie) |
| materialize_10k | 11,4 (11,3–11,4) ms | 1,55 (1,49–1,60) ms | -86,3% | tak, szybciej/mniej |
| update_10pct | 30,9 (30,7–31,1) ms | 15,4 (15,1–15,8) ms | -50,1% | tak, szybciej/mniej |
| delete_10pct | 26,9 (26,6–27,5) ms | 17,5 (17,0–18,2) ms | -35,1% | tak, szybciej/mniej |
| snapshot | 32,2 (31,8–32,3) ms | 22,6 (22,2–23,0) ms | -29,8% | tak, szybciej/mniej |
| snapshot (rozmiar) | 5,03 MiB | 3,71 MiB | -26,2% | tak, szybciej/mniej |
| restore | 115 (114–116) ms | 46,4 (46,1–46,9) ms | -59,6% | tak, szybciej/mniej |
| insert_many | 405 (403–426) ms | 147 (146–148) ms | -63,7% | tak, szybciej/mniej |

Wyniki zgodne w obu wersjach (160 porównań sum kontrolnych i stanu tabeli).

### browser – workload – flatsql-idb

| Metryka | Przed | Po | Zmiana mediany | 95% CI średnich rozłączne |
|---|---:|---:|---:|---|
| insert | 442 (436–452) ms | 324 (323–366) ms | -26,6% | tak, szybciej/mniej |
| point_lookup | 9,47 (9,44–9,54) µs | 3,71 (3,62–3,88) µs | -60,8% | tak, szybciej/mniej |
| index_lookup | 437 (433–451) µs | 2,36 (2,31–2,38) µs | -99,5% | tak, szybciej/mniej |
| range_scan | 83,0 (82,0–85,5) µs | 44,3 (43,9–44,5) µs | -46,7% | tak, szybciej/mniej |
| full_scan | 8,92 (8,87–8,97) ms | 4,42 (4,40–4,45) ms | -50,4% | tak, szybciej/mniej |
| group_by | 16,3 (16,2–16,9) ms | 9,47 (9,40–9,49) ms | -42,0% | tak, szybciej/mniej |
| top_k | 5,47 (5,41–5,53) ms | 5,33 (5,32–5,34) ms | -2,6% | tak, szybciej/mniej |
| materialize_10k | 13,9 (13,8–14,1) ms | 1,50 (1,39–1,69) ms | -89,2% | tak, szybciej/mniej |
| update_10pct | 37,9 (37,0–39,4) ms | 26,8 (22,0–34,5) ms | -29,3% | nie (w rozrzucie) |
| delete_10pct | 32,9 (32,5–33,2) ms | 21,5 (21,3–22,3) ms | -34,8% | tak, szybciej/mniej |
| commit_single | 376 (364–399) µs | 364 (354–375) µs | -3,3% | nie (w rozrzucie) |
| commit_group | 5,86 (5,78–6,02) ms | 5,07 (5,02–5,13) ms | -13,5% | tak, szybciej/mniej |
| reopen | 173 (172–174) ms | 81,4 (81,3–81,6) ms | -53,0% | tak, szybciej/mniej |

Wyniki zgodne w obu wersjach (160 porównań sum kontrolnych i stanu tabeli).

### browser – workload – flatsql-idb-strict

| Metryka | Przed | Po | Zmiana mediany | 95% CI średnich rozłączne |
|---|---:|---:|---:|---|
| insert | 453 (445–454) ms | 334 (331–336) ms | -26,3% | tak, szybciej/mniej |
| point_lookup | 9,59 (9,45–9,70) µs | 3,63 (3,57–3,72) µs | -62,1% | tak, szybciej/mniej |
| index_lookup | 442 (437–453) µs | 2,31 (2,28–2,33) µs | -99,5% | tak, szybciej/mniej |
| range_scan | 82,5 (80,5–83,4) µs | 44,0 (43,6–46,0) µs | -46,6% | tak, szybciej/mniej |
| full_scan | 8,94 (8,89–9,05) ms | 4,43 (4,40–4,43) ms | -50,5% | tak, szybciej/mniej |
| group_by | 16,2 (16,0–16,4) ms | 9,48 (9,41–9,52) ms | -41,3% | tak, szybciej/mniej |
| top_k | 5,46 (5,43–5,48) ms | 5,29 (5,26–5,30) ms | -3,1% | tak, szybciej/mniej |
| materialize_10k | 13,9 (13,9–14,0) ms | 1,40 (1,38–1,56) ms | -90,0% | tak, szybciej/mniej |
| update_10pct | 40,1 (38,8–44,7) ms | 31,8 (27,1–36,7) ms | -20,7% | nie (w rozrzucie) |
| delete_10pct | 34,3 (34,0–34,7) ms | 23,4 (22,9–23,5) ms | -31,8% | tak, szybciej/mniej |
| commit_single | 975 (956–1048) µs | 979 (957–1032) µs | +0,5% | nie (w rozrzucie) |
| commit_group | 6,62 (6,35–6,81) ms | 5,89 (5,69–5,94) ms | -11,0% | tak, szybciej/mniej |
| reopen | 173 (172–175) ms | 81,4 (81,1–81,6) ms | -53,0% | tak, szybciej/mniej |

Wyniki zgodne w obu wersjach (160 porównań sum kontrolnych i stanu tabeli).

### browser – workload – flatsql-json

| Metryka | Przed | Po | Zmiana mediany | 95% CI średnich rozłączne |
|---|---:|---:|---:|---|
| insert | 334 (332–336) ms | 269 (266–271) ms | -19,5% | tak, szybciej/mniej |
| point_lookup | 5,50 (5,47–5,54) µs | 4,16 (4,13–4,25) µs | -24,4% | tak, szybciej/mniej |
| index_lookup | 427 (423–440) µs | 2,88 (2,87–2,90) µs | -99,3% | tak, szybciej/mniej |
| range_scan | 77,3 (76,6–77,6) µs | 47,4 (45,9–49,0) µs | -38,6% | tak, szybciej/mniej |
| full_scan | 7,79 (7,75–7,81) ms | 4,45 (4,43–4,54) ms | -42,9% | tak, szybciej/mniej |
| group_by | 15,4 (15,3–15,5) ms | 9,77 (9,71–9,80) ms | -36,6% | tak, szybciej/mniej |
| top_k | 5,42 (5,35–5,50) ms | 5,35 (5,33–5,36) ms | -1,3% | tak, szybciej/mniej |
| materialize_10k | 7,80 (7,76–7,84) ms | 7,06 (7,01–7,11) ms | -9,5% | tak, szybciej/mniej |
| update_10pct | 30,7 (30,5–30,9) ms | 15,0 (14,8–15,4) ms | -51,1% | tak, szybciej/mniej |
| delete_10pct | 26,8 (26,6–27,1) ms | 17,6 (17,0–17,6) ms | -34,5% | tak, szybciej/mniej |
| snapshot | 32,5 (32,3–33,1) ms | 21,6 (21,2–21,9) ms | -33,5% | tak, szybciej/mniej |
| snapshot (rozmiar) | 5,03 MiB | 3,71 MiB | -26,2% | tak, szybciej/mniej |
| restore | 115 (114–116) ms | 46,7 (46,4–47,1) ms | -59,5% | tak, szybciej/mniej |
| insert_many | 401 (397–404) ms | 146 (145–146) ms | -63,7% | tak, szybciej/mniej |

Wyniki zgodne w obu wersjach (160 porównań sum kontrolnych i stanu tabeli).

### browser – workload – flatsql-worker

| Metryka | Przed | Po | Zmiana mediany | 95% CI średnich rozłączne |
|---|---:|---:|---:|---|
| insert | 350 (348–353) ms | 271 (269–274) ms | -22,4% | tak, szybciej/mniej |
| point_lookup | 59,2 (57,3–60,1) µs | 48,1 (46,1–48,6) µs | -18,7% | tak, szybciej/mniej |
| index_lookup | 566 (558–569) µs | 45,8 (43,2–48,5) µs | -91,9% | tak, szybciej/mniej |
| range_scan | 142 (135–147) µs | 96,7 (93,0–100) µs | -31,7% | tak, szybciej/mniej |
| full_scan | 8,52 (8,45–8,57) ms | 5,21 (5,09–5,30) ms | -38,8% | tak, szybciej/mniej |
| group_by | 18,1 (18,0–18,4) ms | 10,6 (10,5–10,8) ms | -41,0% | tak, szybciej/mniej |
| top_k | 6,05 (5,95–6,11) ms | 6,13 (5,99–6,21) ms | +1,3% | nie (w rozrzucie) |
| materialize_10k | 23,2 (23,1–23,4) ms | 13,5 (13,4–13,6) ms | -41,6% | tak, szybciej/mniej |
| update_10pct | 34,0 (33,8–34,8) ms | 15,8 (15,6–15,9) ms | -53,7% | tak, szybciej/mniej |
| delete_10pct | 27,1 (26,6–27,9) ms | 17,7 (17,4–18,1) ms | -34,8% | tak, szybciej/mniej |

Wyniki zgodne w obu wersjach (160 porównań sum kontrolnych i stanu tabeli).

### browser – workload – flatsql-worker-idb

| Metryka | Przed | Po | Zmiana mediany | 95% CI średnich rozłączne |
|---|---:|---:|---:|---|
| insert | 449 (448–454) ms | 332 (327–337) ms | -26,1% | tak, szybciej/mniej |
| point_lookup | 59,4 (58,2–61,2) µs | 49,2 (47,6–50,5) µs | -17,1% | tak, szybciej/mniej |
| index_lookup | 561 (545–571) µs | 46,8 (43,6–48,4) µs | -91,7% | tak, szybciej/mniej |
| range_scan | 139 (136–155) µs | 95,9 (89,8–97,6) µs | -31,2% | tak, szybciej/mniej |
| full_scan | 9,50 (9,46–9,54) ms | 5,47 (5,31–5,52) ms | -42,5% | tak, szybciej/mniej |
| group_by | 17,3 (16,8–18,5) ms | 10,7 (10,6–10,7) ms | -38,2% | tak, szybciej/mniej |
| top_k | 6,02 (5,94–6,13) ms | 6,39 (6,20–6,56) ms | +6,2% | nie (w rozrzucie) |
| materialize_10k | 25,7 (25,6–26,1) ms | 13,8 (13,5–13,9) ms | -46,1% | tak, szybciej/mniej |
| update_10pct | 43,9 (41,9–53,6) ms | 25,7 (20,9–32,4) ms | -41,5% | nie (w rozrzucie) |
| delete_10pct | 32,6 (32,4–33,4) ms | 21,6 (21,1–22,1) ms | -33,9% | tak, szybciej/mniej |
| commit_single | 419 (397–427) µs | 448 (424–465) µs | +7,0% | nie (w rozrzucie) |
| commit_group | 6,15 (6,03–6,27) ms | 5,21 (5,15–5,27) ms | -15,3% | tak, szybciej/mniej |
| reopen | 179 (177–180) ms | 88,7 (86,9–89,5) ms | -50,3% | tak, szybciej/mniej |

Wyniki zgodne w obu wersjach (160 porównań sum kontrolnych i stanu tabeli).

### native – fsync – flatsql-native

| Metryka | Przed | Po | Zmiana mediany | 95% CI średnich rozłączne |
|---|---:|---:|---:|---|
| FileStorage fsync insert | 255 (254–257) ms | 202 (201–202) ms | -20,9% | tak, szybciej/mniej |
| FileStorage fsync commit_single | 485 (481–491) µs | 491 (488–496) µs | +1,2% | nie (w rozrzucie) |
| FileStorage fsync commit_group | 3,43 (3,37–3,45) ms | 2,78 (2,76–2,92) ms | -18,8% | tak, szybciej/mniej |
| FileStorage fsync checkpoint | 38,7 (38,2–39,2) ms | 29,8 (29,4–33,6) ms | -22,9% | tak, szybciej/mniej |
| FileStorage fsync reopen | 118 (118–119) ms | 45,9 (45,7–46,2) ms | -61,1% | tak, szybciej/mniej |
| FileStorage nofsync insert | 254 (251–256) ms | 193 (192–194) ms | -23,9% | tak, szybciej/mniej |
| FileStorage nofsync commit_single | 9,98 (9,52–10,1) µs | 9,23 (9,08–10,3) µs | -7,6% | nie (w rozrzucie) |
| FileStorage nofsync commit_group | 2,82 (2,72–2,86) ms | 2,20 (2,09–2,21) ms | -22,1% | tak, szybciej/mniej |
| FileStorage nofsync checkpoint | 38,0 (37,5–46,6) ms | 29,0 (28,3–32,2) ms | -23,6% | tak, szybciej/mniej |
| FileStorage nofsync reopen | 118 (117–119) ms | 45,7 (45,2–45,8) ms | -61,3% | tak, szybciej/mniej |

### native – workload – flatsql-native

| Metryka | Przed | Po | Zmiana mediany | 95% CI średnich rozłączne |
|---|---:|---:|---:|---|
| insert | 208 (207–208) ms | 162 (161–163) ms | -22,2% | tak, szybciej/mniej |
| point_lookup | 2,28 (2,27–2,31) µs | 0,91 (0,90–0,93) µs | -59,9% | tak, szybciej/mniej |
| index_lookup | 435 (431–459) µs | 0,71 (0,70–0,71) µs | -99,8% | tak, szybciej/mniej |
| range_scan | 83,9 (81,7–86,5) µs | 48,7 (48,0–49,5) µs | -42,0% | tak, szybciej/mniej |
| full_scan | 8,06 (8,03–8,12) ms | 5,35 (5,32–5,40) ms | -33,6% | tak, szybciej/mniej |
| group_by | 14,6 (14,5–14,8) ms | 10,8 (10,8–10,9) ms | -26,1% | tak, szybciej/mniej |
| top_k | 5,63 (5,61–5,72) ms | 6,82 (6,79–6,85) ms | +21,0% | tak, wolniej/więcej |
| materialize_10k | 3,72 (3,70–3,76) ms | 2,86 (2,84–2,88) ms | -23,2% | tak, szybciej/mniej |
| update_10pct | 30,9 (30,8–31,1) ms | 14,6 (14,5–14,8) ms | -52,7% | tak, szybciej/mniej |
| delete_10pct | 24,8 (24,8–25,4) ms | 14,3 (14,2–14,5) ms | -42,6% | tak, szybciej/mniej |
| snapshot | 25,3 (25,2–25,4) ms | 17,3 (17,1–17,4) ms | -31,6% | tak, szybciej/mniej |
| snapshot (rozmiar) | 5,03 MiB | 3,71 MiB | -26,2% | tak, szybciej/mniej |
| restore | 111 (111–112) ms | 40,4 (40,0–40,9) ms | -63,7% | tak, szybciej/mniej |
| insert_many | – | 144 (143–145) ms | – | – |

Wyniki zgodne w obu wersjach (140 porównań sum kontrolnych i stanu tabeli).

### node – cold – flatsql

| Metryka | Przed | Po | Zmiana mediany | 95% CI średnich rozłączne |
|---|---:|---:|---:|---|
| init | 43,4 (42,9–44,0) ms | 46,5 (45,5–47,4) ms | +7,3% | tak, wolniej/więcej |
| open | 6,32 (6,24–6,59) ms | 5,60 (5,52–5,80) ms | -11,4% | tak, szybciej/mniej |
| firstQuery | 3,93 (3,88–4,10) ms | 6,25 (6,22–6,30) ms | +59,0% | tak, wolniej/więcej |
| total | 53,9 (53,1–54,8) ms | 58,3 (57,4–59,3) ms | +8,3% | tak, wolniej/więcej |

### node – cold – flatsql-json

| Metryka | Przed | Po | Zmiana mediany | 95% CI średnich rozłączne |
|---|---:|---:|---:|---|
| init | 43,1 (42,4–43,7) ms | 45,8 (44,9–46,2) ms | +6,3% | tak, wolniej/więcej |
| open | 6,34 (6,26–6,46) ms | 5,51 (5,48–5,60) ms | -13,1% | tak, szybciej/mniej |
| firstQuery | 4,30 (4,24–4,42) ms | 6,67 (6,61–6,81) ms | +55,0% | tak, wolniej/więcej |
| total | 53,9 (53,7–54,2) ms | 57,9 (57,1–58,5) ms | +7,4% | tak, wolniej/więcej |

### node – fsync – flatsql

| Metryka | Przed | Po | Zmiana mediany | 95% CI średnich rozłączne |
|---|---:|---:|---:|---|
| FsStore fsync insert | 579 (578–586) ms | 388 (386–394) ms | -33,0% | tak, szybciej/mniej |
| FsStore fsync commit_single | 1,36 (1,29–1,39) ms | 677 (668–710) µs | -50,0% | tak, szybciej/mniej |
| FsStore fsync commit_group | 6,86 (6,69–6,95) ms | 5,79 (5,62–5,87) ms | -15,6% | tak, szybciej/mniej |
| FsStore fsync checkpoint | 47,1 (46,4–47,8) ms | 37,3 (36,6–44,1) ms | -20,8% | tak, szybciej/mniej |
| FsStore fsync reopen | 111 (110–111) ms | 52,2 (51,9–52,5) ms | -52,9% | tak, szybciej/mniej |
| FsStore nofsync insert | 388 (384–390) ms | 337 (335–337) ms | -13,2% | tak, szybciej/mniej |
| FsStore nofsync commit_single | 585 (562–600) µs | 77,7 (75,5–84,1) µs | -86,7% | tak, szybciej/mniej |
| FsStore nofsync commit_group | 5,28 (5,16–5,52) ms | 3,84 (3,81–3,91) ms | -27,3% | tak, szybciej/mniej |
| FsStore nofsync checkpoint | 47,1 (46,0–47,6) ms | 34,0 (33,1–34,7) ms | -28,0% | tak, szybciej/mniej |
| FsStore nofsync reopen | 98,8 (98,5–100) ms | 41,4 (40,9–41,8) ms | -58,1% | tak, szybciej/mniej |
| sync round durableServer | 632 (621–668) µs | 640 (621–652) µs | +1,4% | nie (w rozrzucie) |
| sync round memoryServer | 528 (522–530) µs | 527 (515–547) µs | -0,2% | nie (w rozrzucie) |

### node – memory – flatsql

| Metryka | Przed | Po | Zmiana mediany | 95% CI średnich rozłączne |
|---|---:|---:|---:|---|
| wasm_after_load (rozmiar) | 73,3 MiB | 51,8 MiB | -29,3% | tak, szybciej/mniej |
| js_heap_delta (rozmiar) | -1297080 B (-1298232 B–-1297044 B) | 83,1 KiB (83,0 KiB–83,6 KiB) | +-106,6% | tak, wolniej/więcej |

### node – memory – flatsql-json

| Metryka | Przed | Po | Zmiana mediany | 95% CI średnich rozłączne |
|---|---:|---:|---:|---|
| wasm_after_load (rozmiar) | 73,3 MiB | 51,8 MiB | -29,3% | tak, szybciej/mniej |
| js_heap_delta (rozmiar) | -1297032 B (-1297068 B–-1297032 B) | 83,0 KiB (82,9 KiB–83,0 KiB) | +-106,6% | tak, wolniej/więcej |

### node – network – flatsql

| Metryka | Przed | Po | Zmiana mediany | 95% CI średnich rozłączne |
|---|---:|---:|---:|---|
| local (WASM, in-process) | 14,5 (14,4–14,9) µs | 10,1 (9,60–10,4) µs | -30,7% | tak, szybciej/mniej |
| cloud API (localhost HTTP) | 599 (580–608) µs | 666 (641–716) µs | +11,2% | tak, wolniej/więcej |
| cloud API (+20 ms RTT) | 20,8 (20,8–20,8) ms | 20,8 (20,8–20,8) ms | +0,1% | nie (w rozrzucie) |
| cloud API (+50 ms RTT) | 50,8 (50,7–50,8) ms | 50,8 (50,7–50,8) ms | -0,0% | nie (w rozrzucie) |
| cloud API (+100 ms RTT) | 101 (101–101) ms | 101 (101–101) ms | +0,0% | nie (w rozrzucie) |

### node – workload – flatsql

| Metryka | Przed | Po | Zmiana mediany | 95% CI średnich rozłączne |
|---|---:|---:|---:|---|
| insert | 371 (369–376) ms | 286 (285–287) ms | -23,1% | tak, szybciej/mniej |
| point_lookup | 8,94 (8,83–9,04) µs | 2,45 (2,44–2,47) µs | -72,6% | tak, szybciej/mniej |
| index_lookup | 427 (422–437) µs | 2,09 (1,79–2,46) µs | -99,5% | tak, szybciej/mniej |
| range_scan | 80,8 (79,8–82,6) µs | 41,0 (40,7–41,5) µs | -49,2% | tak, szybciej/mniej |
| full_scan | 7,74 (7,71–7,77) ms | 4,49 (4,47–4,52) ms | -41,9% | tak, szybciej/mniej |
| group_by | 15,5 (15,4–16,1) ms | 9,58 (9,52–9,87) ms | -38,3% | tak, szybciej/mniej |
| top_k | 5,24 (5,22–5,35) ms | 5,27 (5,23–5,52) ms | +0,6% | nie (w rozrzucie) |
| materialize_10k | 9,54 (9,49–9,73) ms | 1,65 (1,64–1,75) ms | -82,7% | tak, szybciej/mniej |
| update_10pct | 41,4 (40,6–41,8) ms | 15,4 (15,3–15,5) ms | -62,9% | tak, szybciej/mniej |
| delete_10pct | 27,7 (27,4–28,1) ms | 17,6 (17,5–18,0) ms | -36,3% | tak, szybciej/mniej |
| snapshot | 33,8 (33,6–34,3) ms | 22,1 (22,0–22,7) ms | -34,8% | tak, szybciej/mniej |
| snapshot (rozmiar) | 5,03 MiB | 3,71 MiB | -26,2% | tak, szybciej/mniej |
| restore | 115 (114–116) ms | 48,2 (48,0–48,5) ms | -58,1% | tak, szybciej/mniej |
| insert_many | 1,16 (1,09–1,19) s | 151 (150–152) ms | -87,0% | tak, szybciej/mniej |

Wyniki zgodne w obu wersjach (160 porównań sum kontrolnych i stanu tabeli).

### node – workload – flatsql-json

| Metryka | Przed | Po | Zmiana mediany | 95% CI średnich rozłączne |
|---|---:|---:|---:|---|
| insert | 369 (368–374) ms | 285 (284–287) ms | -22,7% | tak, szybciej/mniej |
| point_lookup | 5,36 (5,33–5,41) µs | 4,11 (4,07–4,13) µs | -23,2% | tak, szybciej/mniej |
| index_lookup | 416 (413–424) µs | 3,40 (3,06–4,15) µs | -99,2% | tak, szybciej/mniej |
| range_scan | 82,2 (81,2–83,2) µs | 44,1 (43,5–44,8) µs | -46,4% | tak, szybciej/mniej |
| full_scan | 7,79 (7,74–7,83) ms | 4,51 (4,48–4,54) ms | -42,2% | tak, szybciej/mniej |
| group_by | 16,4 (16,3–16,8) ms | 10,0 (9,88–10,1) ms | -39,1% | tak, szybciej/mniej |
| top_k | 5,27 (5,23–5,37) ms | 5,34 (5,29–5,66) ms | +1,4% | nie (w rozrzucie) |
| materialize_10k | 9,48 (9,45–9,55) ms | 8,76 (8,71–8,86) ms | -7,6% | tak, szybciej/mniej |
| update_10pct | 31,9 (31,8–32,2) ms | 15,2 (15,2–15,5) ms | -52,3% | tak, szybciej/mniej |
| delete_10pct | 28,9 (27,6–29,1) ms | 17,7 (17,5–17,7) ms | -38,8% | tak, szybciej/mniej |
| snapshot | 33,6 (33,2–33,9) ms | 21,5 (21,5–22,4) ms | -35,9% | tak, szybciej/mniej |
| snapshot (rozmiar) | 5,03 MiB | 3,71 MiB | -26,2% | tak, szybciej/mniej |
| restore | 114 (113–114) ms | 48,2 (47,9–49,0) ms | -57,7% | tak, szybciej/mniej |
| insert_many | 990 (945–1007) ms | 151 (150–152) ms | -84,7% | tak, szybciej/mniej |

Wyniki zgodne w obu wersjach (160 porównań sum kontrolnych i stanu tabeli).

