Przed: po-2.13 (wasm: build the result object in JavaScript and keep column names per plan); po: po-2.13-dirty (wasm: build the result object in JavaScript and keep column names per plan).
Obciążenie procesora przed pomiarem: 6,5%. Uruchomienia naprzemienne, 10 na wersję.
Moduł WebAssembly: 690178 → 695444 B, gzip -9: 259727 → 262194 B (+2467 B).

### native – workload – flatsql-native

| Metryka | Przed | Po | Zmiana mediany | 95% CI średnich rozłączne |
|---|---:|---:|---:|---|
| insert | 165 (163–166) ms | 166 (163–168) ms | +0,7% | nie (w rozrzucie) |
| point_lookup | 1,21 (1,20–1,32) µs | 1,26 (1,24–1,31) µs | +3,5% | nie (w rozrzucie) |
| index_lookup | 0,74 (0,73–0,75) µs | 0,78 (0,75–0,81) µs | +4,9% | nie (w rozrzucie) |
| range_scan | 50,2 (48,9–53,4) µs | 49,9 (47,9–51,4) µs | -0,4% | nie (w rozrzucie) |
| full_scan | 5,51 (5,48–5,54) ms | 5,55 (5,49–5,70) ms | +0,6% | nie (w rozrzucie) |
| group_by | 10,4 (10,3–10,6) ms | 10,4 (10,4–10,5) ms | +0,1% | nie (w rozrzucie) |
| top_k | 6,08 (6,05–6,12) ms | 6,06 (6,01–6,09) ms | -0,2% | nie (w rozrzucie) |
| materialize_10k | 3,82 (3,79–3,92) ms | 3,86 (3,83–3,91) ms | +1,1% | nie (w rozrzucie) |
| update_10pct | 15,5 (15,4–16,1) ms | 15,8 (15,4–16,0) ms | +2,1% | nie (w rozrzucie) |
| delete_10pct | 16,9 (16,2–17,6) ms | 16,7 (16,0–16,8) ms | -1,4% | nie (w rozrzucie) |
| snapshot | 17,8 (17,8–17,9) ms | 17,7 (17,6–18,0) ms | -0,5% | nie (w rozrzucie) |
| snapshot (rozmiar) | 3,71 MiB | 3,71 MiB | +0,0% | nie (w rozrzucie) |
| restore | 45,3 (44,5–46,1) ms | 44,7 (44,1–45,4) ms | -1,4% | nie (w rozrzucie) |

Wyniki zgodne w obu wersjach (140 porównań sum kontrolnych i stanu tabeli).

### node – workload – flatsql

| Metryka | Przed | Po | Zmiana mediany | 95% CI średnich rozłączne |
|---|---:|---:|---:|---|
| insert | 299 (295–306) ms | 298 (295–299) ms | -0,5% | nie (w rozrzucie) |
| point_lookup | 3,12 (3,02–3,24) µs | 2,60 (2,56–2,65) µs | -16,7% | tak, szybciej/mniej |
| index_lookup | 2,03 (1,95–2,09) µs | 1,97 (1,76–2,19) µs | -2,8% | nie (w rozrzucie) |
| range_scan | 54,3 (53,8–59,0) µs | 52,0 (49,7–53,8) µs | -4,2% | nie (w rozrzucie) |
| full_scan | 5,22 (5,17–5,40) ms | 5,08 (5,05–5,09) ms | -2,7% | nie (w rozrzucie) |
| group_by | 11,3 (11,2–11,6) ms | 10,7 (10,6–10,9) ms | -5,2% | tak, szybciej/mniej |
| top_k | 5,55 (5,52–5,80) ms | 5,62 (5,52–5,69) ms | +1,2% | nie (w rozrzucie) |
| materialize_10k | 10,8 (10,3–11,4) ms | 1,93 (1,83–1,94) ms | -82,1% | tak, szybciej/mniej |
| update_10pct | 17,3 (17,1–17,8) ms | 16,6 (16,4–17,2) ms | -4,0% | nie (w rozrzucie) |
| delete_10pct | 22,2 (21,8–22,7) ms | 20,1 (19,9–20,5) ms | -9,7% | tak, szybciej/mniej |
| snapshot | 31,1 (30,7–31,5) ms | 26,2 (25,7–26,6) ms | -15,8% | tak, szybciej/mniej |
| snapshot (rozmiar) | 3,71 MiB | 3,71 MiB | +0,0% | nie (w rozrzucie) |
| restore | 58,3 (57,1–61,1) ms | 51,9 (51,9–52,5) ms | -11,0% | tak, szybciej/mniej |

Wyniki zgodne w obu wersjach (160 porównań sum kontrolnych i stanu tabeli).

### node – workload – flatsql-json

| Metryka | Przed | Po | Zmiana mediany | 95% CI średnich rozłączne |
|---|---:|---:|---:|---|
| insert | 300 (294–304) ms | 296 (295–301) ms | -1,3% | nie (w rozrzucie) |
| point_lookup | 4,86 (4,55–5,32) µs | 4,44 (4,39–4,52) µs | -8,6% | nie (w rozrzucie) |
| index_lookup | 3,14 (3,02–3,59) µs | 3,35 (3,08–3,98) µs | +6,8% | nie (w rozrzucie) |
| range_scan | 52,9 (52,0–55,8) µs | 68,0 (66,4–72,6) µs | +28,5% | tak, wolniej/więcej |
| full_scan | 5,21 (5,12–5,28) ms | 5,14 (5,09–5,32) ms | -1,3% | nie (w rozrzucie) |
| group_by | 12,6 (12,5–13,1) ms | 11,0 (10,9–11,1) ms | -12,7% | tak, szybciej/mniej |
| top_k | 5,64 (5,61–5,77) ms | 5,75 (5,62–5,83) ms | +2,1% | nie (w rozrzucie) |
| materialize_10k | 11,0 (10,8–11,1) ms | 10,8 (10,7–11,0) ms | -1,7% | nie (w rozrzucie) |
| update_10pct | 17,4 (17,1–17,9) ms | 17,5 (17,2–18,3) ms | +0,7% | nie (w rozrzucie) |
| delete_10pct | 21,2 (20,8–21,7) ms | 22,6 (21,9–23,1) ms | +6,5% | nie (w rozrzucie) |
| snapshot | 31,0 (28,6–31,4) ms | 29,0 (26,9–30,5) ms | -6,4% | nie (w rozrzucie) |
| snapshot (rozmiar) | 3,71 MiB | 3,71 MiB | +0,0% | nie (w rozrzucie) |
| restore | 59,4 (57,4–61,0) ms | 58,2 (56,8–59,7) ms | -1,9% | nie (w rozrzucie) |

Wyniki zgodne w obu wersjach (160 porównań sum kontrolnych i stanu tabeli).

