Przed: po-2.6 (core/bridge: encode checkpoints in steps); po: po-2.6-dirty (core/bridge: encode checkpoints in steps).
Obciążenie procesora przed pomiarem: 3,7%. Uruchomienia naprzemienne, 10 na wersję.
Moduł WebAssembly: 724002 → 727911 B, gzip -9: 274098 → 274159 B (+61 B).

### node – memory – flatsql

| Metryka | Przed | Po | Zmiana mediany | 95% CI średnich rozłączne |
|---|---:|---:|---:|---|
| wasm_after_load (rozmiar) | 51,8 MiB | 52,3 MiB | +1,0% | tak, wolniej/więcej |
| js_heap_delta (rozmiar) | 83,0 KiB (83,0 KiB–84,1 KiB) | 83,0 KiB (82,9 KiB–83,0 KiB) | -0,1% | nie (w rozrzucie) |

### node – memory – flatsql-json

| Metryka | Przed | Po | Zmiana mediany | 95% CI średnich rozłączne |
|---|---:|---:|---:|---|
| wasm_after_load (rozmiar) | 51,8 MiB | 52,3 MiB | +1,0% | tak, wolniej/więcej |
| js_heap_delta (rozmiar) | 84,1 KiB (83,0 KiB–84,1 KiB) | 83,0 KiB (83,0 KiB–83,0 KiB) | -1,3% | nie (w rozrzucie) |

### node – workload – flatsql

| Metryka | Przed | Po | Zmiana mediany | 95% CI średnich rozłączne |
|---|---:|---:|---:|---|
| insert | 288 (287–288) ms | 289 (287–289) ms | +0,2% | nie (w rozrzucie) |
| point_lookup | 2,47 (2,44–2,51) µs | 2,51 (2,48–2,58) µs | +1,4% | nie (w rozrzucie) |
| index_lookup | 2,37 (1,87–2,78) µs | 2,08 (1,79–2,58) µs | -12,2% | nie (w rozrzucie) |
| range_scan | 41,1 (40,9–41,8) µs | 42,2 (41,6–43,8) µs | +2,6% | nie (w rozrzucie) |
| full_scan | 4,54 (4,52–4,56) ms | 4,60 (4,58–4,62) ms | +1,3% | nie (w rozrzucie) |
| group_by | 9,65 (9,53–10,0) ms | 9,95 (9,80–10,1) ms | +3,1% | nie (w rozrzucie) |
| top_k | 5,36 (5,28–5,54) ms | 5,51 (5,48–5,53) ms | +2,8% | nie (w rozrzucie) |
| materialize_10k | 1,66 (1,63–1,72) ms | 1,82 (1,71–1,82) ms | +9,6% | nie (w rozrzucie) |
| update_10pct | 15,6 (15,3–15,9) ms | 15,9 (15,5–16,1) ms | +1,8% | nie (w rozrzucie) |
| delete_10pct | 18,0 (17,7–18,7) ms | 17,6 (17,3–17,7) ms | -2,5% | nie (w rozrzucie) |
| snapshot | 22,3 (22,1–22,8) ms | 22,1 (21,8–22,4) ms | -1,0% | nie (w rozrzucie) |
| snapshot (rozmiar) | 3,71 MiB | 3,71 MiB | +0,0% | nie (w rozrzucie) |
| restore | 48,1 (48,0–48,6) ms | 47,3 (47,2–47,8) ms | -1,6% | nie (w rozrzucie) |
| insert_many | 152 (151–153) ms | 153 (152–154) ms | +0,6% | nie (w rozrzucie) |

Wyniki zgodne w obu wersjach (160 porównań sum kontrolnych i stanu tabeli).

### node – workload – flatsql-json

| Metryka | Przed | Po | Zmiana mediany | 95% CI średnich rozłączne |
|---|---:|---:|---:|---|
| insert | 284 (284–286) ms | 286 (286–289) ms | +0,7% | nie (w rozrzucie) |
| point_lookup | 4,13 (4,06–4,50) µs | 4,20 (4,14–4,79) µs | +1,6% | nie (w rozrzucie) |
| index_lookup | 3,53 (3,06–3,79) µs | 3,08 (2,98–3,51) µs | -12,7% | nie (w rozrzucie) |
| range_scan | 44,0 (43,5–44,3) µs | 44,4 (43,8–44,7) µs | +0,9% | nie (w rozrzucie) |
| full_scan | 4,51 (4,49–4,55) ms | 4,62 (4,61–4,63) ms | +2,4% | tak, wolniej/więcej |
| group_by | 10,1 (9,80–10,3) ms | 9,89 (9,85–9,95) ms | -2,0% | nie (w rozrzucie) |
| top_k | 5,36 (5,28–5,86) ms | 5,55 (5,48–6,19) ms | +3,6% | nie (w rozrzucie) |
| materialize_10k | 8,92 (8,87–8,98) ms | 8,94 (8,78–9,03) ms | +0,3% | nie (w rozrzucie) |
| update_10pct | 15,4 (15,3–15,6) ms | 15,5 (15,5–15,6) ms | +1,0% | nie (w rozrzucie) |
| delete_10pct | 17,9 (17,7–18,0) ms | 17,9 (17,6–18,2) ms | -0,2% | nie (w rozrzucie) |
| snapshot | 21,6 (21,4–22,0) ms | 22,4 (22,1–22,9) ms | +3,9% | nie (w rozrzucie) |
| snapshot (rozmiar) | 3,71 MiB | 3,71 MiB | +0,0% | nie (w rozrzucie) |
| restore | 48,2 (47,8–48,7) ms | 47,1 (46,7–47,3) ms | -2,3% | tak, szybciej/mniej |
| insert_many | 151 (150–152) ms | 153 (152–154) ms | +0,8% | nie (w rozrzucie) |

Wyniki zgodne w obu wersjach (160 porównań sum kontrolnych i stanu tabeli).

