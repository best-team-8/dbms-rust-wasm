Przed: po-2.13 (wasm: build the result object in JavaScript and keep column names per plan); po: po-2.13-dirty (wasm: build the result object in JavaScript and keep column names per plan).
Obciążenie procesora przed pomiarem: 7,8%. Uruchomienia naprzemienne, 10 na wersję.
Moduł WebAssembly: 690178 → 695444 B, gzip -9: 259727 → 262194 B (+2467 B).

### node – workload – flatsql-json

| Metryka | Przed | Po | Zmiana mediany | 95% CI średnich rozłączne |
|---|---:|---:|---:|---|
| insert | 298 (296–301) ms | 296 (295–305) ms | -0,5% | nie (w rozrzucie) |
| point_lookup | 4,41 (4,37–4,51) µs | 4,62 (4,45–4,75) µs | +4,7% | nie (w rozrzucie) |
| index_lookup | 3,22 (3,09–3,67) µs | 3,13 (3,03–3,42) µs | -2,9% | nie (w rozrzucie) |
| range_scan | 53,2 (52,4–55,2) µs | 66,7 (63,3–68,4) µs | +25,3% | tak, wolniej/więcej |
| full_scan | 5,17 (5,10–5,19) ms | 5,14 (5,10–5,18) ms | -0,5% | nie (w rozrzucie) |
| group_by | 12,5 (12,5–12,7) ms | 11,0 (10,9–11,4) ms | -12,1% | tak, szybciej/mniej |
| top_k | 5,57 (5,51–5,90) ms | 5,60 (5,56–5,66) ms | +0,5% | nie (w rozrzucie) |
| materialize_10k | 10,9 (10,7–11,0) ms | 11,0 (10,8–11,2) ms | +0,4% | nie (w rozrzucie) |
| update_10pct | 17,3 (16,9–18,1) ms | 17,1 (16,6–17,5) ms | -1,0% | nie (w rozrzucie) |
| delete_10pct | 22,6 (22,1–22,9) ms | 21,8 (21,6–22,3) ms | -3,2% | nie (w rozrzucie) |
| snapshot | 30,7 (30,1–30,9) ms | 26,9 (26,8–28,7) ms | -12,4% | nie (w rozrzucie) |
| snapshot (rozmiar) | 3,71 MiB | 3,71 MiB | +0,0% | nie (w rozrzucie) |
| restore | 58,1 (57,4–60,0) ms | 58,6 (57,7–60,2) ms | +0,8% | nie (w rozrzucie) |

Wyniki zgodne w obu wersjach (160 porównań sum kontrolnych i stanu tabeli).

