Przed: po-2.13 (wasm: build the result object in JavaScript and keep column names per plan); po: po-2.13-dirty (wasm: build the result object in JavaScript and keep column names per plan).
Obciążenie procesora przed pomiarem: 8,8%. Uruchomienia naprzemienne, 10 na wersję.
Moduł WebAssembly: 690178 → 695444 B, gzip -9: 259727 → 262194 B (+2467 B).

### browser – workload – flatsql

| Metryka | Przed | Po | Zmiana mediany | 95% CI średnich rozłączne |
|---|---:|---:|---:|---|
| insert | 292 (290–295) ms | 281 (276–283) ms | -3,8% | tak, szybciej/mniej |
| point_lookup | 3,99 (3,83–4,05) µs | 3,29 (3,26–3,39) µs | -17,4% | tak, szybciej/mniej |
| index_lookup | 1,99 (1,97–2,02) µs | 1,87 (1,85–1,94) µs | -6,0% | nie (w rozrzucie) |
| range_scan | 51,9 (51,1–54,8) µs | 51,7 (51,1–53,7) µs | -0,4% | nie (w rozrzucie) |
| full_scan | 5,30 (5,25–5,34) ms | 5,11 (5,01–5,16) ms | -3,6% | nie (w rozrzucie) |
| group_by | 12,5 (11,4–12,8) ms | 11,5 (11,0–11,7) ms | -8,0% | nie (w rozrzucie) |
| top_k | 5,69 (5,60–6,73) ms | 5,62 (5,56–5,63) ms | -1,1% | nie (w rozrzucie) |
| materialize_10k | 14,0 (13,6–14,4) ms | 1,59 (1,49–1,68) ms | -88,6% | tak, szybciej/mniej |
| update_10pct | 18,5 (17,5–20,0) ms | 17,0 (16,7–17,2) ms | -8,2% | tak, szybciej/mniej |
| delete_10pct | 23,2 (22,3–24,5) ms | 19,9 (19,2–20,8) ms | -13,9% | tak, szybciej/mniej |
| snapshot | 28,4 (27,5–28,8) ms | 25,2 (25,0–26,0) ms | -11,4% | tak, szybciej/mniej |
| snapshot (rozmiar) | 3,71 MiB | 3,71 MiB | +0,0% | nie (w rozrzucie) |
| restore | 56,1 (55,7–57,6) ms | 50,8 (50,7–51,2) ms | -9,4% | tak, szybciej/mniej |

Wyniki zgodne w obu wersjach (160 porównań sum kontrolnych i stanu tabeli).

### browser – workload – flatsql-json

| Metryka | Przed | Po | Zmiana mediany | 95% CI średnich rozłączne |
|---|---:|---:|---:|---|
| insert | 277 (276–281) ms | 275 (274–277) ms | -0,8% | nie (w rozrzucie) |
| point_lookup | 4,70 (4,57–4,84) µs | 4,59 (4,41–4,66) µs | -2,2% | nie (w rozrzucie) |
| index_lookup | 2,88 (2,83–2,91) µs | 2,91 (2,84–2,94) µs | +0,9% | nie (w rozrzucie) |
| range_scan | 53,3 (52,0–54,6) µs | 55,3 (54,1–56,7) µs | +3,7% | nie (w rozrzucie) |
| full_scan | 5,39 (5,23–5,53) ms | 5,07 (4,99–5,16) ms | -6,0% | nie (w rozrzucie) |
| group_by | 12,0 (11,3–12,4) ms | 10,6 (10,5–10,7) ms | -12,3% | tak, szybciej/mniej |
| top_k | 5,84 (5,72–6,16) ms | 5,70 (5,65–5,79) ms | -2,3% | nie (w rozrzucie) |
| materialize_10k | 9,08 (8,98–9,14) ms | 9,01 (8,95–9,07) ms | -0,8% | nie (w rozrzucie) |
| update_10pct | 16,7 (16,2–17,0) ms | 16,7 (16,6–17,5) ms | +0,0% | nie (w rozrzucie) |
| delete_10pct | 20,6 (20,3–21,0) ms | 20,5 (20,3–20,8) ms | -0,2% | nie (w rozrzucie) |
| snapshot | 27,9 (27,0–28,7) ms | 28,5 (28,4–29,1) ms | +2,1% | nie (w rozrzucie) |
| snapshot (rozmiar) | 3,71 MiB | 3,71 MiB | +0,0% | nie (w rozrzucie) |
| restore | 55,2 (53,5–56,2) ms | 55,7 (55,3–56,3) ms | +0,9% | nie (w rozrzucie) |

Wyniki zgodne w obu wersjach (160 porównań sum kontrolnych i stanu tabeli).

