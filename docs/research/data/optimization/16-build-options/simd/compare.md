Przed: po-2.6 (core/bridge: encode checkpoints in steps); po: po-2.6 (core/bridge: encode checkpoints in steps).
Obciążenie procesora przed pomiarem: 2,0%. Uruchomienia naprzemienne, 10 na wersję.
Moduł WebAssembly: 724002 → 708494 B, gzip -9: 274098 → 270683 B (-3415 B).

### node – memory – flatsql

| Metryka | Przed | Po | Zmiana mediany | 95% CI średnich rozłączne |
|---|---:|---:|---:|---|
| wasm_after_load (rozmiar) | 51,8 MiB | 51,8 MiB | +0,0% | nie (w rozrzucie) |
| js_heap_delta (rozmiar) | 83,0 KiB (82,9 KiB–83,8 KiB) | 83,0 KiB (82,9 KiB–83,0 KiB) | +0,0% | nie (w rozrzucie) |

### node – memory – flatsql-json

| Metryka | Przed | Po | Zmiana mediany | 95% CI średnich rozłączne |
|---|---:|---:|---:|---|
| wasm_after_load (rozmiar) | 51,8 MiB | 51,8 MiB | +0,0% | nie (w rozrzucie) |
| js_heap_delta (rozmiar) | 83,0 KiB (83,0 KiB–84,1 KiB) | 83,4 KiB (83,0 KiB–84,2 KiB) | +0,6% | nie (w rozrzucie) |

### node – workload – flatsql

| Metryka | Przed | Po | Zmiana mediany | 95% CI średnich rozłączne |
|---|---:|---:|---:|---|
| insert | 287 (285–288) ms | 287 (284–289) ms | +0,0% | nie (w rozrzucie) |
| point_lookup | 2,50 (2,45–2,78) µs | 2,47 (2,45–2,49) µs | -1,0% | nie (w rozrzucie) |
| index_lookup | 1,80 (1,75–2,19) µs | 2,12 (1,75–2,28) µs | +17,7% | nie (w rozrzucie) |
| range_scan | 41,2 (41,0–42,2) µs | 41,5 (41,3–41,9) µs | +0,6% | nie (w rozrzucie) |
| full_scan | 4,52 (4,51–4,54) ms | 4,51 (4,49–4,53) ms | -0,3% | nie (w rozrzucie) |
| group_by | 9,56 (9,51–9,87) ms | 10,0 (9,87–10,2) ms | +4,9% | nie (w rozrzucie) |
| top_k | 5,29 (5,25–5,48) ms | 5,26 (5,24–5,29) ms | -0,4% | nie (w rozrzucie) |
| materialize_10k | 1,70 (1,67–1,75) ms | 1,73 (1,72–1,81) ms | +1,7% | nie (w rozrzucie) |
| update_10pct | 15,8 (15,4–15,8) ms | 15,6 (15,5–15,9) ms | -1,0% | nie (w rozrzucie) |
| delete_10pct | 17,9 (17,7–18,3) ms | 17,7 (17,5–17,8) ms | -1,2% | nie (w rozrzucie) |
| snapshot | 22,1 (21,9–22,3) ms | 21,8 (21,5–22,3) ms | -1,1% | nie (w rozrzucie) |
| snapshot (rozmiar) | 3,71 MiB | 3,71 MiB | +0,0% | nie (w rozrzucie) |
| restore | 48,6 (48,4–48,9) ms | 46,1 (45,9–46,6) ms | -5,1% | tak, szybciej/mniej |
| insert_many | 150 (150–151) ms | 151 (151–152) ms | +0,8% | nie (w rozrzucie) |

Wyniki zgodne w obu wersjach (160 porównań sum kontrolnych i stanu tabeli).

### node – workload – flatsql-json

| Metryka | Przed | Po | Zmiana mediany | 95% CI średnich rozłączne |
|---|---:|---:|---:|---|
| insert | 286 (284–287) ms | 287 (286–289) ms | +0,3% | nie (w rozrzucie) |
| point_lookup | 4,08 (4,07–4,11) µs | 4,09 (4,06–4,16) µs | +0,1% | nie (w rozrzucie) |
| index_lookup | 3,04 (2,98–3,59) µs | 3,36 (3,06–3,68) µs | +10,6% | nie (w rozrzucie) |
| range_scan | 43,8 (43,4–44,2) µs | 45,2 (43,4–46,5) µs | +3,3% | nie (w rozrzucie) |
| full_scan | 4,51 (4,50–4,54) ms | 4,56 (4,56–4,59) ms | +1,1% | tak, wolniej/więcej |
| group_by | 10,0 (9,83–10,1) ms | 10,1 (10,0–10,1) ms | +0,7% | nie (w rozrzucie) |
| top_k | 5,36 (5,30–5,70) ms | 5,26 (5,25–5,26) ms | -1,8% | tak, szybciej/mniej |
| materialize_10k | 8,79 (8,67–8,87) ms | 8,92 (8,83–9,01) ms | +1,4% | nie (w rozrzucie) |
| update_10pct | 15,3 (15,2–15,6) ms | 15,5 (15,3–15,6) ms | +1,3% | nie (w rozrzucie) |
| delete_10pct | 17,8 (17,5–18,2) ms | 17,7 (17,5–17,8) ms | -0,6% | nie (w rozrzucie) |
| snapshot | 21,6 (21,5–21,7) ms | 21,8 (21,7–21,9) ms | +0,9% | nie (w rozrzucie) |
| snapshot (rozmiar) | 3,71 MiB | 3,71 MiB | +0,0% | nie (w rozrzucie) |
| restore | 48,4 (48,2–49,4) ms | 45,8 (45,6–45,9) ms | -5,4% | tak, szybciej/mniej |
| insert_many | 152 (151–153) ms | 153 (151–156) ms | +1,1% | nie (w rozrzucie) |

Wyniki zgodne w obu wersjach (160 porównań sum kontrolnych i stanu tabeli).

