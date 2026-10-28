Przed: po-2.14 (core/wasm: execute a prepared write for many parameter sets in one call); po: po-2.14-dirty (core/wasm: execute a prepared write for many parameter sets in one call).
Obciążenie procesora przed pomiarem: 4,1%. Uruchomienia naprzemienne, 10 na wersję.
Moduł WebAssembly: 700803 → 665164 B, gzip -9: 264711 → 259731 B (-4980 B).

### browser – memory – flatsql

| Metryka | Przed | Po | Zmiana mediany | 95% CI średnich rozłączne |
|---|---:|---:|---:|---|
| wasm_after_load (rozmiar) | 54,8 MiB | 51,8 MiB | -5,5% | tak, szybciej/mniej |

### browser – memory – flatsql-json

| Metryka | Przed | Po | Zmiana mediany | 95% CI średnich rozłączne |
|---|---:|---:|---:|---|
| wasm_after_load (rozmiar) | 54,8 MiB | 51,8 MiB | -5,5% | tak, szybciej/mniej |

### browser – workload – flatsql

| Metryka | Przed | Po | Zmiana mediany | 95% CI średnich rozłączne |
|---|---:|---:|---:|---|
| insert | 275 (272–277) ms | 293 (288–296) ms | +6,4% | tak, wolniej/więcej |
| point_lookup | 3,09 (3,02–3,12) µs | 3,08 (3,03–3,19) µs | -0,2% | nie (w rozrzucie) |
| index_lookup | 1,89 (1,86–1,92) µs | 1,82 (1,81–1,91) µs | -3,4% | nie (w rozrzucie) |
| range_scan | 51,6 (50,8–53,5) µs | 42,9 (42,4–44,6) µs | -17,0% | tak, szybciej/mniej |
| full_scan | 4,91 (4,83–4,95) ms | 4,53 (4,51–4,61) ms | -7,6% | tak, szybciej/mniej |
| group_by | 10,8 (10,3–11,1) ms | 9,26 (9,17–9,29) ms | -14,4% | tak, szybciej/mniej |
| top_k | 5,41 (5,39–5,45) ms | 5,40 (5,38–5,42) ms | -0,2% | nie (w rozrzucie) |
| materialize_10k | 1,59 (1,56–1,64) ms | 1,56 (1,52–1,57) ms | -1,7% | nie (w rozrzucie) |
| update_10pct | 15,9 (15,6–16,7) ms | 15,6 (15,1–16,1) ms | -2,0% | nie (w rozrzucie) |
| delete_10pct | 19,1 (18,9–19,5) ms | 18,3 (18,1–18,8) ms | -4,6% | nie (w rozrzucie) |
| snapshot | 24,6 (23,9–25,2) ms | 22,1 (21,7–22,4) ms | -10,1% | tak, szybciej/mniej |
| snapshot (rozmiar) | 3,71 MiB | 3,71 MiB | +0,0% | nie (w rozrzucie) |
| restore | 50,0 (49,4–51,6) ms | 44,9 (44,5–45,8) ms | -10,3% | tak, szybciej/mniej |
| insert_many | 151 (150–154) ms | 176 (170–180) ms | +16,7% | tak, wolniej/więcej |

Wyniki zgodne w obu wersjach (160 porównań sum kontrolnych i stanu tabeli).

### browser – workload – flatsql-json

| Metryka | Przed | Po | Zmiana mediany | 95% CI średnich rozłączne |
|---|---:|---:|---:|---|
| insert | 272 (271–274) ms | 290 (287–295) ms | +6,5% | tak, wolniej/więcej |
| point_lookup | 4,32 (4,29–4,35) µs | 4,24 (4,21–4,30) µs | -2,0% | nie (w rozrzucie) |
| index_lookup | 2,85 (2,81–2,90) µs | 2,86 (2,79–2,91) µs | +0,4% | nie (w rozrzucie) |
| range_scan | 54,1 (52,4–54,6) µs | 44,1 (43,7–44,9) µs | -18,5% | tak, szybciej/mniej |
| full_scan | 4,89 (4,86–4,93) ms | 4,63 (4,60–4,64) ms | -5,5% | tak, szybciej/mniej |
| group_by | 11,4 (11,3–11,5) ms | 10,5 (10,4–10,6) ms | -8,1% | tak, szybciej/mniej |
| top_k | 5,74 (5,70–5,77) ms | 5,34 (5,33–5,38) ms | -7,0% | tak, szybciej/mniej |
| materialize_10k | 8,92 (8,78–8,99) ms | 7,19 (7,13–7,22) ms | -19,4% | tak, szybciej/mniej |
| update_10pct | 15,9 (15,8–16,2) ms | 15,0 (14,7–15,4) ms | -5,6% | tak, szybciej/mniej |
| delete_10pct | 19,1 (18,9–19,7) ms | 18,2 (18,1–18,6) ms | -4,6% | tak, szybciej/mniej |
| snapshot | 25,4 (25,3–25,7) ms | 21,6 (21,3–21,8) ms | -14,9% | tak, szybciej/mniej |
| snapshot (rozmiar) | 3,71 MiB | 3,71 MiB | +0,0% | nie (w rozrzucie) |
| restore | 50,0 (49,8–50,8) ms | 45,0 (44,8–45,1) ms | -10,1% | tak, szybciej/mniej |
| insert_many | 150 (148–152) ms | 172 (171–173) ms | +14,6% | tak, wolniej/więcej |

Wyniki zgodne w obu wersjach (160 porównań sum kontrolnych i stanu tabeli).

