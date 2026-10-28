Przed: po-2.6 (core/bridge: encode checkpoints in steps); po: po-2.6-dirty (core/bridge: encode checkpoints in steps).
Obciążenie procesora przed pomiarem: 8,4%. Uruchomienia naprzemienne, 10 na wersję.
Moduł WebAssembly: 724002 → 662271 B, gzip -9: 274098 → 260281 B (-13817 B).

### node – memory – flatsql

| Metryka | Przed | Po | Zmiana mediany | 95% CI średnich rozłączne |
|---|---:|---:|---:|---|
| wasm_after_load (rozmiar) | 51,8 MiB | 51,8 MiB | +0,0% | nie (w rozrzucie) |
| js_heap_delta (rozmiar) | 83,0 KiB (82,9 KiB–83,0 KiB) | 83,9 KiB (82,8 KiB–83,9 KiB) | +1,1% | tak, wolniej/więcej |

### node – memory – flatsql-json

| Metryka | Przed | Po | Zmiana mediany | 95% CI średnich rozłączne |
|---|---:|---:|---:|---|
| wasm_after_load (rozmiar) | 51,8 MiB | 51,8 MiB | +0,0% | nie (w rozrzucie) |
| js_heap_delta (rozmiar) | 83,0 KiB (83,0 KiB–83,0 KiB) | 83,9 KiB (83,0 KiB–83,9 KiB) | +1,1% | nie (w rozrzucie) |

### node – workload – flatsql

| Metryka | Przed | Po | Zmiana mediany | 95% CI średnich rozłączne |
|---|---:|---:|---:|---|
| insert | 289 (287–291) ms | 316 (313–320) ms | +9,5% | tak, wolniej/więcej |
| point_lookup | 2,52 (2,51–2,61) µs | 2,51 (2,49–2,54) µs | -0,6% | nie (w rozrzucie) |
| index_lookup | 1,95 (1,79–2,52) µs | 1,77 (1,70–2,55) µs | -9,1% | nie (w rozrzucie) |
| range_scan | 42,5 (41,1–43,4) µs | 47,0 (46,4–47,8) µs | +10,6% | tak, wolniej/więcej |
| full_scan | 4,52 (4,50–4,53) ms | 4,58 (4,56–4,62) ms | +1,4% | nie (w rozrzucie) |
| group_by | 9,55 (9,50–9,83) ms | 9,84 (9,72–10,00) ms | +3,1% | nie (w rozrzucie) |
| top_k | 5,34 (5,25–5,41) ms | 5,48 (5,44–5,57) ms | +2,6% | nie (w rozrzucie) |
| materialize_10k | 1,77 (1,73–1,86) ms | 1,80 (1,77–1,89) ms | +1,9% | nie (w rozrzucie) |
| update_10pct | 15,4 (15,2–15,6) ms | 15,8 (15,7–15,9) ms | +2,8% | nie (w rozrzucie) |
| delete_10pct | 18,0 (17,8–18,3) ms | 19,1 (19,0–19,9) ms | +6,3% | nie (w rozrzucie) |
| snapshot | 23,2 (22,2–24,3) ms | 19,8 (19,4–20,5) ms | -14,6% | tak, szybciej/mniej |
| snapshot (rozmiar) | 3,71 MiB | 3,71 MiB | +0,0% | nie (w rozrzucie) |
| restore | 49,1 (48,4–50,4) ms | 47,5 (47,1–48,6) ms | -3,3% | nie (w rozrzucie) |
| insert_many | 152 (151–154) ms | 177 (176–180) ms | +16,5% | tak, wolniej/więcej |

Wyniki zgodne w obu wersjach (160 porównań sum kontrolnych i stanu tabeli).

### node – workload – flatsql-json

| Metryka | Przed | Po | Zmiana mediany | 95% CI średnich rozłączne |
|---|---:|---:|---:|---|
| insert | 288 (287–290) ms | 317 (315–321) ms | +10,1% | tak, wolniej/więcej |
| point_lookup | 4,36 (4,24–4,46) µs | 4,26 (4,18–4,65) µs | -2,3% | nie (w rozrzucie) |
| index_lookup | 3,10 (3,01–3,43) µs | 3,18 (3,04–4,17) µs | +2,5% | nie (w rozrzucie) |
| range_scan | 44,3 (44,1–53,2) µs | 50,7 (49,1–52,3) µs | +14,6% | nie (w rozrzucie) |
| full_scan | 4,55 (4,52–4,71) ms | 4,62 (4,58–4,64) ms | +1,5% | nie (w rozrzucie) |
| group_by | 10,1 (9,90–10,2) ms | 9,94 (9,82–10,1) ms | -1,2% | nie (w rozrzucie) |
| top_k | 5,33 (5,31–5,58) ms | 5,42 (5,37–5,44) ms | +1,6% | nie (w rozrzucie) |
| materialize_10k | 9,05 (8,85–9,17) ms | 9,31 (9,14–9,50) ms | +3,0% | nie (w rozrzucie) |
| update_10pct | 16,0 (15,5–16,4) ms | 16,3 (16,0–16,4) ms | +2,0% | nie (w rozrzucie) |
| delete_10pct | 18,1 (17,9–18,3) ms | 19,1 (18,9–20,1) ms | +5,8% | tak, wolniej/więcej |
| snapshot | 22,2 (21,9–22,7) ms | 20,4 (20,0–20,9) ms | -8,1% | tak, szybciej/mniej |
| snapshot (rozmiar) | 3,71 MiB | 3,71 MiB | +0,0% | nie (w rozrzucie) |
| restore | 48,6 (48,3–48,9) ms | 47,8 (47,5–49,1) ms | -1,6% | nie (w rozrzucie) |
| insert_many | 151 (150–152) ms | 180 (177–180) ms | +19,5% | tak, wolniej/więcej |

Wyniki zgodne w obu wersjach (160 porównań sum kontrolnych i stanu tabeli).

