Przed: po-2.10 (core: compile WHERE conjuncts and SET expressions specialised to column types); po: po-2.10-dirty (core: compile WHERE conjuncts and SET expressions specialised to column types).
Obciążenie procesora przed pomiarem: 10,3%. Uruchomienia naprzemienne, 10 na wersję.
Moduł WebAssembly: 628081 → 643544 B, gzip -9: 238401 → 242791 B (+4390 B).

### native – workload – flatsql-native

| Metryka | Przed | Po | Zmiana mediany | 95% CI średnich rozłączne |
|---|---:|---:|---:|---|
| insert | 206 (205–208) ms | 160 (157–160) ms | -22,2% | tak, szybciej/mniej |
| point_lookup | 1,21 (1,17–1,25) µs | 1,24 (1,20–1,24) µs | +2,6% | nie (w rozrzucie) |
| index_lookup | 0,70 (0,70–0,73) µs | 0,74 (0,73–0,75) µs | +4,5% | nie (w rozrzucie) |
| range_scan | 49,5 (48,6–51,1) µs | 48,2 (47,5–51,7) µs | -2,7% | nie (w rozrzucie) |
| full_scan | 5,45 (5,41–5,59) ms | 5,52 (5,46–5,58) ms | +1,3% | nie (w rozrzucie) |
| group_by | 15,5 (15,4–15,6) ms | 15,7 (15,6–15,9) ms | +1,7% | nie (w rozrzucie) |
| top_k | 5,97 (5,94–6,04) ms | 5,95 (5,92–6,02) ms | -0,2% | nie (w rozrzucie) |
| materialize_10k | 3,70 (3,65–3,80) ms | 3,80 (3,78–3,86) ms | +2,8% | nie (w rozrzucie) |
| update_10pct | 29,8 (29,7–30,3) ms | 22,9 (22,8–23,2) ms | -23,1% | tak, szybciej/mniej |
| delete_10pct | 24,0 (23,8–24,3) ms | 16,4 (16,2–16,6) ms | -31,9% | tak, szybciej/mniej |
| snapshot | 25,7 (25,5–26,1) ms | 18,2 (18,1–18,3) ms | -29,4% | tak, szybciej/mniej |
| snapshot (rozmiar) | 5,03 MiB | 3,71 MiB | -26,2% | tak, szybciej/mniej |
| restore | 111 (110–113) ms | 54,7 (54,3–55,1) ms | -50,8% | tak, szybciej/mniej |

Wyniki zgodne w obu wersjach (140 porównań sum kontrolnych i stanu tabeli).

### node – memory – flatsql

| Metryka | Przed | Po | Zmiana mediany | 95% CI średnich rozłączne |
|---|---:|---:|---:|---|
| wasm_after_load (rozmiar) | 73,3 MiB | 54,8 MiB | -25,2% | tak, szybciej/mniej |
| js_heap_delta (rozmiar) | -1292496 B | 88,5 KiB (88,5 KiB–89,4 KiB) | +-107,0% | tak, wolniej/więcej |

### node – memory – flatsql-json

| Metryka | Przed | Po | Zmiana mediany | 95% CI średnich rozłączne |
|---|---:|---:|---:|---|
| wasm_after_load (rozmiar) | 73,3 MiB | 54,8 MiB | -25,2% | tak, szybciej/mniej |
| js_heap_delta (rozmiar) | -1292496 B (-1292544 B–-1292496 B) | 88,5 KiB | +-107,0% | tak, wolniej/więcej |

### node – workload – flatsql

| Metryka | Przed | Po | Zmiana mediany | 95% CI średnich rozłączne |
|---|---:|---:|---:|---|
| insert | 377 (372–384) ms | 294 (292–296) ms | -22,0% | tak, szybciej/mniej |
| point_lookup | 8,33 (8,19–8,46) µs | 8,17 (8,03–8,33) µs | -1,9% | nie (w rozrzucie) |
| index_lookup | 5,98 (5,88–6,27) µs | 7,47 (7,32–10,6) µs | +25,0% | tak, wolniej/więcej |
| range_scan | 58,2 (54,5–64,1) µs | 57,4 (54,9–60,6) µs | -1,5% | nie (w rozrzucie) |
| full_scan | 5,17 (5,15–5,21) ms | 5,11 (5,07–5,18) ms | -1,1% | nie (w rozrzucie) |
| group_by | 17,7 (16,7–18,0) ms | 18,0 (17,0–18,7) ms | +1,7% | nie (w rozrzucie) |
| top_k | 5,71 (5,60–5,99) ms | 5,51 (5,46–5,80) ms | -3,7% | nie (w rozrzucie) |
| materialize_10k | 9,92 (9,83–10,1) ms | 10,6 (10,4–10,7) ms | +6,7% | tak, wolniej/więcej |
| update_10pct | 38,9 (33,1–45,3) ms | 36,9 (29,1–38,1) ms | -5,1% | nie (w rozrzucie) |
| delete_10pct | 31,2 (30,6–34,4) ms | 25,1 (23,4–27,8) ms | -19,6% | tak, szybciej/mniej |
| snapshot | 37,3 (36,4–38,5) ms | 31,2 (30,1–32,4) ms | -16,4% | tak, szybciej/mniej |
| snapshot (rozmiar) | 5,03 MiB | 3,71 MiB | -26,2% | tak, szybciej/mniej |
| restore | 124 (121–125) ms | 68,5 (66,3–74,6) ms | -44,6% | tak, szybciej/mniej |

Wyniki zgodne w obu wersjach (160 porównań sum kontrolnych i stanu tabeli).

### node – workload – flatsql-json

| Metryka | Przed | Po | Zmiana mediany | 95% CI średnich rozłączne |
|---|---:|---:|---:|---|
| insert | 379 (375–386) ms | 296 (294–298) ms | -21,8% | nie (w rozrzucie) |
| point_lookup | 4,91 (4,78–5,03) µs | 4,47 (4,37–4,58) µs | -8,9% | nie (w rozrzucie) |
| index_lookup | 3,03 (2,99–3,83) µs | 3,25 (3,13–3,85) µs | +7,1% | nie (w rozrzucie) |
| range_scan | 53,7 (52,9–54,7) µs | 62,2 (53,4–68,5) µs | +16,0% | tak, wolniej/więcej |
| full_scan | 5,08 (5,04–5,14) ms | 5,14 (5,09–5,21) ms | +1,0% | nie (w rozrzucie) |
| group_by | 16,1 (16,0–16,3) ms | 17,1 (16,9–18,6) ms | +6,3% | nie (w rozrzucie) |
| top_k | 5,80 (5,67–6,12) ms | 5,52 (5,51–5,55) ms | -4,8% | nie (w rozrzucie) |
| materialize_10k | 10,2 (9,93–10,3) ms | 10,8 (10,6–11,3) ms | +6,8% | nie (w rozrzucie) |
| update_10pct | 40,5 (35,2–43,0) ms | 26,9 (26,3–27,7) ms | -33,6% | tak, szybciej/mniej |
| delete_10pct | 30,4 (30,1–31,4) ms | 25,9 (25,6–26,4) ms | -14,6% | tak, szybciej/mniej |
| snapshot | 38,6 (37,5–40,0) ms | 31,1 (30,7–31,5) ms | -19,6% | tak, szybciej/mniej |
| snapshot (rozmiar) | 5,03 MiB | 3,71 MiB | -26,2% | tak, szybciej/mniej |
| restore | 125 (123–139) ms | 64,6 (62,9–66,8) ms | -48,2% | tak, szybciej/mniej |

Wyniki zgodne w obu wersjach (160 porównań sum kontrolnych i stanu tabeli).

