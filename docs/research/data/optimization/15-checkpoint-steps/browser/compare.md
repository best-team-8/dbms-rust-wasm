Przed: po-2.7 (core: keep short texts inside the value); po: po-2.7-dirty (core: keep short texts inside the value).
Obciążenie procesora przed pomiarem: 4,8%. Uruchomienia naprzemienne, 10 na wersję.
Moduł WebAssembly: 711549 → 724082 B, gzip -9: 268907 → 274096 B (+5189 B).

### browser – checkpoint – flatsql-idb

| Metryka | Przed | Po | Zmiana mediany | 95% CI średnich rozłączne |
|---|---:|---:|---:|---|
| pause idle | 2,08 (1,97–2,43) ms | 2,45 (2,03–10,4) ms | +17,6% | nie (w rozrzucie) |
| pause 10000 | 6,06 (5,96–6,27) ms | 2,56 (2,35–2,80) ms | -57,7% | tak, szybciej/mniej |
| checkpoint 10000 | 10,9 (10,2–231) ms | 10,4 (9,66–10,8) ms | -4,9% | nie (w rozrzucie) |
| pause 50000 | 14,7 (14,5–15,0) ms | 1,38 (1,33–1,63) ms | -90,6% | tak, szybciej/mniej |
| checkpoint 50000 | 20,2 (19,9–20,3) ms | 18,6 (18,2–19,3) ms | -7,7% | nie (w rozrzucie) |
| pause 100000 | 27,8 (27,6–28,1) ms | 1,70 (1,55–1,99) ms | -93,9% | tak, szybciej/mniej |
| checkpoint 100000 | 34,0 (33,6–34,3) ms | 33,7 (33,5–34,6) ms | -1,0% | nie (w rozrzucie) |

### browser – checkpoint – flatsql-idb-strict

| Metryka | Przed | Po | Zmiana mediany | 95% CI średnich rozłączne |
|---|---:|---:|---:|---|
| pause idle | 2,33 (2,04–2,70) ms | 2,05 (1,92–2,30) ms | -11,9% | nie (w rozrzucie) |
| pause 10000 | 5,91 (5,71–6,34) ms | 2,62 (2,20–2,84) ms | -55,6% | tak, szybciej/mniej |
| checkpoint 10000 | 12,2 (11,9–13,0) ms | 11,9 (11,6–12,3) ms | -2,1% | nie (w rozrzucie) |
| pause 50000 | 14,6 (14,4–14,9) ms | 1,61 (1,46–2,04) ms | -89,0% | tak, szybciej/mniej |
| checkpoint 50000 | 22,6 (22,3–23,1) ms | 22,9 (22,1–23,7) ms | +1,3% | nie (w rozrzucie) |
| pause 100000 | 27,3 (27,1–27,8) ms | 1,91 (1,70–2,81) ms | -93,0% | tak, szybciej/mniej |
| checkpoint 100000 | 37,2 (36,4–37,8) ms | 40,0 (39,2–42,2) ms | +7,4% | tak, wolniej/więcej |

### browser – workload – flatsql

| Metryka | Przed | Po | Zmiana mediany | 95% CI średnich rozłączne |
|---|---:|---:|---:|---|
| insert | 268 (265–271) ms | 268 (267–269) ms | -0,0% | nie (w rozrzucie) |
| point_lookup | 3,06 (3,03–3,08) µs | 3,11 (3,05–3,17) µs | +1,6% | nie (w rozrzucie) |
| index_lookup | 1,89 (1,86–1,91) µs | 1,87 (1,83–1,87) µs | -1,3% | nie (w rozrzucie) |
| range_scan | 45,8 (43,3–47,9) µs | 45,4 (43,8–46,0) µs | -0,9% | nie (w rozrzucie) |
| full_scan | 4,41 (4,37–4,48) ms | 4,42 (4,39–4,46) ms | +0,2% | nie (w rozrzucie) |
| group_by | 9,40 (9,39–9,42) ms | 9,32 (9,25–9,46) ms | -0,9% | nie (w rozrzucie) |
| top_k | 5,36 (5,35–5,37) ms | 5,33 (5,30–5,38) ms | -0,5% | nie (w rozrzucie) |
| materialize_10k | 1,49 (1,44–1,57) ms | 1,48 (1,44–1,54) ms | -1,0% | nie (w rozrzucie) |
| update_10pct | 14,5 (14,3–14,9) ms | 15,6 (15,3–15,8) ms | +7,5% | tak, wolniej/więcej |
| delete_10pct | 17,6 (17,1–17,8) ms | 17,3 (17,0–17,5) ms | -1,7% | nie (w rozrzucie) |
| snapshot | 22,1 (21,5–22,7) ms | 21,8 (21,3–22,4) ms | -1,7% | nie (w rozrzucie) |
| snapshot (rozmiar) | 3,71 MiB | 3,71 MiB | +0,0% | nie (w rozrzucie) |
| restore | 46,3 (46,0–47,3) ms | 46,4 (46,3–47,1) ms | +0,3% | nie (w rozrzucie) |
| insert_many | 147 (146–149) ms | 146 (146–147) ms | -0,5% | nie (w rozrzucie) |

Wyniki zgodne w obu wersjach (160 porównań sum kontrolnych i stanu tabeli).

### browser – workload – flatsql-idb

| Metryka | Przed | Po | Zmiana mediany | 95% CI średnich rozłączne |
|---|---:|---:|---:|---|
| insert | 335 (331–403) ms | 331 (324–373) ms | -1,1% | nie (w rozrzucie) |
| point_lookup | 3,64 (3,62–3,75) µs | 3,76 (3,54–3,84) µs | +3,1% | nie (w rozrzucie) |
| index_lookup | 2,32 (2,29–2,43) µs | 2,33 (2,29–2,36) µs | +0,6% | nie (w rozrzucie) |
| range_scan | 44,8 (43,8–46,9) µs | 44,7 (44,2–46,8) µs | -0,0% | nie (w rozrzucie) |
| full_scan | 4,43 (4,41–4,47) ms | 4,40 (4,40–4,46) ms | -0,5% | nie (w rozrzucie) |
| group_by | 9,29 (9,25–9,32) ms | 9,47 (9,42–9,50) ms | +1,9% | tak, wolniej/więcej |
| top_k | 5,32 (5,30–5,35) ms | 5,29 (5,28–5,34) ms | -0,5% | nie (w rozrzucie) |
| materialize_10k | 1,59 (1,33–1,69) ms | 1,47 (1,34–1,64) ms | -7,1% | nie (w rozrzucie) |
| update_10pct | 25,2 (24,0–28,3) ms | 25,9 (23,6–33,9) ms | +2,8% | nie (w rozrzucie) |
| delete_10pct | 22,1 (21,8–23,4) ms | 22,2 (21,9–22,5) ms | +0,5% | nie (w rozrzucie) |
| commit_single | 359 (353–373) µs | 376 (367–423) µs | +4,9% | nie (w rozrzucie) |
| commit_group | 5,11 (4,93–5,23) ms | 4,98 (4,88–5,15) ms | -2,4% | nie (w rozrzucie) |
| reopen | 85,5 (85,0–86,0) ms | 81,6 (81,1–82,0) ms | -4,5% | tak, szybciej/mniej |

Wyniki zgodne w obu wersjach (160 porównań sum kontrolnych i stanu tabeli).

### browser – workload – flatsql-idb-strict

| Metryka | Przed | Po | Zmiana mediany | 95% CI średnich rozłączne |
|---|---:|---:|---:|---|
| insert | 349 (335–353) ms | 339 (336–343) ms | -2,8% | nie (w rozrzucie) |
| point_lookup | 3,75 (3,68–3,78) µs | 3,61 (3,58–3,64) µs | -3,9% | nie (w rozrzucie) |
| index_lookup | 2,32 (2,29–2,35) µs | 2,29 (2,28–2,38) µs | -1,5% | nie (w rozrzucie) |
| range_scan | 44,2 (43,9–44,7) µs | 44,1 (43,9–45,1) µs | -0,1% | nie (w rozrzucie) |
| full_scan | 4,41 (4,40–4,43) ms | 4,43 (4,41–4,45) ms | +0,3% | nie (w rozrzucie) |
| group_by | 9,29 (9,25–9,32) ms | 9,43 (9,38–9,46) ms | +1,5% | tak, wolniej/więcej |
| top_k | 5,32 (5,29–5,34) ms | 5,32 (5,30–5,34) ms | -0,1% | nie (w rozrzucie) |
| materialize_10k | 1,46 (1,34–1,57) ms | 1,38 (1,37–1,41) ms | -4,8% | nie (w rozrzucie) |
| update_10pct | 22,5 (21,6–30,4) ms | 25,9 (22,6–32,0) ms | +15,1% | nie (w rozrzucie) |
| delete_10pct | 23,2 (23,0–23,7) ms | 23,4 (23,1–24,3) ms | +1,1% | nie (w rozrzucie) |
| commit_single | 992 (972–1021) µs | 1,03 (0,98–1,07) ms | +4,1% | nie (w rozrzucie) |
| commit_group | 5,88 (5,78–6,09) ms | 5,94 (5,72–6,05) ms | +0,9% | nie (w rozrzucie) |
| reopen | 86,4 (84,7–87,1) ms | 81,4 (80,7–82,1) ms | -5,7% | tak, szybciej/mniej |

Wyniki zgodne w obu wersjach (160 porównań sum kontrolnych i stanu tabeli).

### browser – workload – flatsql-json

| Metryka | Przed | Po | Zmiana mediany | 95% CI średnich rozłączne |
|---|---:|---:|---:|---|
| insert | 267 (265–270) ms | 269 (268–270) ms | +0,8% | nie (w rozrzucie) |
| point_lookup | 4,21 (4,17–4,24) µs | 4,10 (4,09–4,20) µs | -2,5% | nie (w rozrzucie) |
| index_lookup | 2,90 (2,87–2,93) µs | 2,88 (2,86–2,93) µs | -1,0% | nie (w rozrzucie) |
| range_scan | 46,9 (46,0–47,6) µs | 49,5 (46,4–50,9) µs | +5,7% | nie (w rozrzucie) |
| full_scan | 4,45 (4,43–4,48) ms | 4,43 (4,42–4,46) ms | -0,4% | nie (w rozrzucie) |
| group_by | 9,70 (9,64–9,73) ms | 9,77 (9,74–9,79) ms | +0,8% | nie (w rozrzucie) |
| top_k | 5,54 (5,51–5,57) ms | 5,34 (5,33–5,38) ms | -3,5% | tak, szybciej/mniej |
| materialize_10k | 7,04 (7,00–7,10) ms | 7,02 (7,00–7,17) ms | -0,3% | nie (w rozrzucie) |
| update_10pct | 15,2 (14,9–15,5) ms | 15,2 (15,0–15,6) ms | -0,3% | nie (w rozrzucie) |
| delete_10pct | 17,6 (17,4–17,8) ms | 17,5 (17,3–17,8) ms | -0,8% | nie (w rozrzucie) |
| snapshot | 21,6 (21,2–21,9) ms | 21,6 (20,9–22,0) ms | -0,1% | nie (w rozrzucie) |
| snapshot (rozmiar) | 3,71 MiB | 3,71 MiB | +0,0% | nie (w rozrzucie) |
| restore | 46,2 (45,9–47,1) ms | 46,1 (46,0–47,6) ms | -0,4% | nie (w rozrzucie) |
| insert_many | 146 (145–147) ms | 146 (145–148) ms | -0,0% | nie (w rozrzucie) |

Wyniki zgodne w obu wersjach (160 porównań sum kontrolnych i stanu tabeli).

