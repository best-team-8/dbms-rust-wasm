Przed: po-2.6 (core/bridge: encode checkpoints in steps); po: po-2.6-dirty (core/bridge: encode checkpoints in steps).
Obciążenie procesora przed pomiarem: 18,3%. Uruchomienia naprzemienne, 10 na wersję.
Moduł WebAssembly: 724002 → 910423 B, gzip -9: 274098 → 291622 B (+17524 B).

### node – memory – flatsql

| Metryka | Przed | Po | Zmiana mediany | 95% CI średnich rozłączne |
|---|---:|---:|---:|---|
| wasm_after_load (rozmiar) | 51,8 MiB | 51,8 MiB | +0,0% | nie (w rozrzucie) |
| js_heap_delta (rozmiar) | 83,0 KiB (83,0 KiB–83,1 KiB) | 84,3 KiB (84,2 KiB–84,3 KiB) | +1,5% | tak, wolniej/więcej |

### node – memory – flatsql-json

| Metryka | Przed | Po | Zmiana mediany | 95% CI średnich rozłączne |
|---|---:|---:|---:|---|
| wasm_after_load (rozmiar) | 51,8 MiB | 51,8 MiB | +0,0% | nie (w rozrzucie) |
| js_heap_delta (rozmiar) | 82,9 KiB (82,9 KiB–83,0 KiB) | 84,3 KiB (84,3 KiB–84,3 KiB) | +1,6% | tak, wolniej/więcej |

### node – workload – flatsql

| Metryka | Przed | Po | Zmiana mediany | 95% CI średnich rozłączne |
|---|---:|---:|---:|---|
| insert | 288 (285–291) ms | 286 (285–287) ms | -0,7% | nie (w rozrzucie) |
| point_lookup | 2,47 (2,44–2,68) µs | 2,59 (2,57–3,25) µs | +5,1% | nie (w rozrzucie) |
| index_lookup | 1,88 (1,74–2,45) µs | 1,90 (1,83–2,13) µs | +1,2% | nie (w rozrzucie) |
| range_scan | 41,5 (41,0–43,7) µs | 42,6 (41,6–43,7) µs | +2,5% | nie (w rozrzucie) |
| full_scan | 4,54 (4,51–4,61) ms | 4,50 (4,49–4,51) ms | -0,9% | nie (w rozrzucie) |
| group_by | 9,62 (9,47–9,82) ms | 10,1 (9,90–10,1) ms | +4,8% | nie (w rozrzucie) |
| top_k | 5,26 (5,25–5,48) ms | 5,26 (5,23–5,35) ms | +0,1% | nie (w rozrzucie) |
| materialize_10k | 1,76 (1,68–1,83) ms | 1,71 (1,68–1,75) ms | -3,0% | nie (w rozrzucie) |
| update_10pct | 15,6 (15,4–16,1) ms | 15,5 (15,3–16,2) ms | -0,4% | nie (w rozrzucie) |
| delete_10pct | 17,8 (17,6–18,3) ms | 17,6 (17,5–18,1) ms | -0,7% | nie (w rozrzucie) |
| snapshot | 22,7 (22,5–23,1) ms | 22,0 (21,9–22,4) ms | -2,8% | nie (w rozrzucie) |
| snapshot (rozmiar) | 3,71 MiB | 3,71 MiB | +0,0% | nie (w rozrzucie) |
| restore | 48,2 (48,0–49,3) ms | 48,6 (48,2–48,9) ms | +0,8% | nie (w rozrzucie) |
| insert_many | 150 (149–151) ms | 146 (144–147) ms | -2,4% | tak, szybciej/mniej |

Wyniki zgodne w obu wersjach (160 porównań sum kontrolnych i stanu tabeli).

### node – workload – flatsql-json

| Metryka | Przed | Po | Zmiana mediany | 95% CI średnich rozłączne |
|---|---:|---:|---:|---|
| insert | 288 (287–290) ms | 285 (283–287) ms | -0,9% | nie (w rozrzucie) |
| point_lookup | 4,17 (4,11–4,51) µs | 4,51 (4,25–4,80) µs | +8,1% | nie (w rozrzucie) |
| index_lookup | 3,11 (3,03–3,60) µs | 3,22 (3,15–3,85) µs | +3,4% | nie (w rozrzucie) |
| range_scan | 43,9 (43,6–44,9) µs | 45,4 (44,4–46,4) µs | +3,3% | nie (w rozrzucie) |
| full_scan | 4,52 (4,50–4,54) ms | 4,51 (4,49–4,54) ms | -0,2% | nie (w rozrzucie) |
| group_by | 10,2 (10,0–10,2) ms | 10,1 (9,99–10,1) ms | -1,3% | nie (w rozrzucie) |
| top_k | 5,37 (5,33–5,80) ms | 5,32 (5,32–5,38) ms | -0,8% | nie (w rozrzucie) |
| materialize_10k | 8,87 (8,80–8,91) ms | 9,10 (9,06–9,19) ms | +2,6% | tak, wolniej/więcej |
| update_10pct | 15,3 (15,2–15,4) ms | 15,5 (15,3–15,8) ms | +1,3% | nie (w rozrzucie) |
| delete_10pct | 17,7 (17,6–18,1) ms | 17,3 (17,2–17,7) ms | -2,1% | nie (w rozrzucie) |
| snapshot | 21,7 (21,7–22,1) ms | 22,0 (21,8–22,6) ms | +1,2% | nie (w rozrzucie) |
| snapshot (rozmiar) | 3,71 MiB | 3,71 MiB | +0,0% | nie (w rozrzucie) |
| restore | 48,7 (48,2–48,9) ms | 48,3 (47,9–49,0) ms | -0,7% | nie (w rozrzucie) |
| insert_many | 153 (151–155) ms | 147 (145–149) ms | -3,7% | tak, szybciej/mniej |

Wyniki zgodne w obu wersjach (160 porównań sum kontrolnych i stanu tabeli).

