Przed: po-2.6 (core/bridge: encode checkpoints in steps); po: po-2.6-dirty (core/bridge: encode checkpoints in steps).
Obciążenie procesora przed pomiarem: 5,2%. Uruchomienia naprzemienne, 10 na wersję.
Moduł WebAssembly: 724002 → 725196 B, gzip -9: 274098 → 276427 B (+2329 B).

### node – memory – flatsql

| Metryka | Przed | Po | Zmiana mediany | 95% CI średnich rozłączne |
|---|---:|---:|---:|---|
| wasm_after_load (rozmiar) | 51,8 MiB | 51,8 MiB | +0,0% | nie (w rozrzucie) |
| js_heap_delta (rozmiar) | 83,0 KiB (83,0 KiB–83,9 KiB) | 83,0 KiB (83,0 KiB–84,1 KiB) | -0,1% | nie (w rozrzucie) |

### node – memory – flatsql-json

| Metryka | Przed | Po | Zmiana mediany | 95% CI średnich rozłączne |
|---|---:|---:|---:|---|
| wasm_after_load (rozmiar) | 51,8 MiB | 51,8 MiB | +0,0% | nie (w rozrzucie) |
| js_heap_delta (rozmiar) | 83,0 KiB (83,0 KiB–83,1 KiB) | 83,1 KiB (83,0 KiB–84,1 KiB) | +0,1% | nie (w rozrzucie) |

### node – workload – flatsql

| Metryka | Przed | Po | Zmiana mediany | 95% CI średnich rozłączne |
|---|---:|---:|---:|---|
| insert | 288 (286–289) ms | 288 (285–290) ms | +0,1% | nie (w rozrzucie) |
| point_lookup | 2,50 (2,46–2,53) µs | 2,47 (2,45–2,56) µs | -1,1% | nie (w rozrzucie) |
| index_lookup | 2,16 (1,73–2,24) µs | 1,81 (1,75–2,14) µs | -16,2% | nie (w rozrzucie) |
| range_scan | 40,9 (40,7–42,9) µs | 42,4 (41,0–43,0) µs | +3,7% | nie (w rozrzucie) |
| full_scan | 4,50 (4,49–4,52) ms | 4,52 (4,49–4,55) ms | +0,4% | nie (w rozrzucie) |
| group_by | 9,55 (9,52–9,59) ms | 10,0 (9,73–10,3) ms | +4,8% | tak, wolniej/więcej |
| top_k | 5,30 (5,26–5,32) ms | 5,25 (5,23–5,28) ms | -0,9% | nie (w rozrzucie) |
| materialize_10k | 1,74 (1,70–1,78) ms | 1,72 (1,67–1,78) ms | -1,2% | nie (w rozrzucie) |
| update_10pct | 15,4 (15,3–15,8) ms | 15,3 (15,2–15,5) ms | -0,4% | nie (w rozrzucie) |
| delete_10pct | 17,9 (17,7–18,1) ms | 17,6 (17,4–18,0) ms | -1,5% | nie (w rozrzucie) |
| snapshot | 22,1 (21,9–22,6) ms | 21,7 (21,6–22,1) ms | -2,0% | nie (w rozrzucie) |
| snapshot (rozmiar) | 3,71 MiB | 3,71 MiB | +0,0% | nie (w rozrzucie) |
| restore | 49,0 (48,6–49,9) ms | 46,7 (46,5–47,1) ms | -4,6% | tak, szybciej/mniej |
| insert_many | 151 (150–152) ms | 153 (151–153) ms | +1,3% | nie (w rozrzucie) |

Wyniki zgodne w obu wersjach (160 porównań sum kontrolnych i stanu tabeli).

### node – workload – flatsql-json

| Metryka | Przed | Po | Zmiana mediany | 95% CI średnich rozłączne |
|---|---:|---:|---:|---|
| insert | 288 (287–289) ms | 287 (285–288) ms | -0,3% | nie (w rozrzucie) |
| point_lookup | 4,10 (4,06–4,16) µs | 4,14 (4,05–4,21) µs | +0,9% | nie (w rozrzucie) |
| index_lookup | 3,42 (3,09–3,52) µs | 3,46 (3,18–3,56) µs | +1,0% | nie (w rozrzucie) |
| range_scan | 43,8 (43,2–44,7) µs | 44,3 (43,4–47,7) µs | +1,3% | nie (w rozrzucie) |
| full_scan | 4,52 (4,47–4,57) ms | 4,54 (4,50–4,55) ms | +0,4% | nie (w rozrzucie) |
| group_by | 9,97 (9,89–9,97) ms | 9,76 (9,69–9,88) ms | -2,1% | nie (w rozrzucie) |
| top_k | 5,30 (5,28–5,36) ms | 5,35 (5,28–5,53) ms | +0,8% | nie (w rozrzucie) |
| materialize_10k | 8,81 (8,78–8,87) ms | 8,83 (8,80–8,97) ms | +0,2% | nie (w rozrzucie) |
| update_10pct | 15,3 (15,1–15,5) ms | 15,5 (15,4–15,6) ms | +1,6% | nie (w rozrzucie) |
| delete_10pct | 17,6 (17,4–17,6) ms | 17,6 (17,5–18,1) ms | -0,0% | nie (w rozrzucie) |
| snapshot | 22,1 (21,9–22,5) ms | 22,1 (21,8–22,7) ms | -0,4% | nie (w rozrzucie) |
| snapshot (rozmiar) | 3,71 MiB | 3,71 MiB | +0,0% | nie (w rozrzucie) |
| restore | 48,0 (47,7–48,2) ms | 46,4 (46,0–46,8) ms | -3,4% | tak, szybciej/mniej |
| insert_many | 152 (150–153) ms | 153 (153–154) ms | +0,9% | nie (w rozrzucie) |

Wyniki zgodne w obu wersjach (160 porównań sum kontrolnych i stanu tabeli).

