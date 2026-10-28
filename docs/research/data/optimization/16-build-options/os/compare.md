Przed: po-2.6 (core/bridge: encode checkpoints in steps); po: po-2.6-dirty (core/bridge: encode checkpoints in steps).
Obciążenie procesora przed pomiarem: 9,2%. Uruchomienia naprzemienne, 10 na wersję.
Moduł WebAssembly: 724002 → 561671 B, gzip -9: 274098 → 228306 B (-45792 B).

### node – memory – flatsql

| Metryka | Przed | Po | Zmiana mediany | 95% CI średnich rozłączne |
|---|---:|---:|---:|---|
| wasm_after_load (rozmiar) | 51,8 MiB | 51,8 MiB | +0,0% | nie (w rozrzucie) |
| js_heap_delta (rozmiar) | 83,0 KiB (83,0 KiB–83,1 KiB) | 83,4 KiB | +0,6% | nie (w rozrzucie) |

### node – memory – flatsql-json

| Metryka | Przed | Po | Zmiana mediany | 95% CI średnich rozłączne |
|---|---:|---:|---:|---|
| wasm_after_load (rozmiar) | 51,8 MiB | 51,8 MiB | +0,0% | nie (w rozrzucie) |
| js_heap_delta (rozmiar) | 83,0 KiB (82,9 KiB–83,8 KiB) | 83,4 KiB (83,4 KiB–83,5 KiB) | +0,6% | nie (w rozrzucie) |

### node – workload – flatsql

| Metryka | Przed | Po | Zmiana mediany | 95% CI średnich rozłączne |
|---|---:|---:|---:|---|
| insert | 286 (285–288) ms | 358 (353–361) ms | +25,4% | tak, wolniej/więcej |
| point_lookup | 2,47 (2,46–2,49) µs | 2,87 (2,82–2,90) µs | +16,1% | tak, wolniej/więcej |
| index_lookup | 1,78 (1,76–2,27) µs | 2,36 (2,23–2,58) µs | +32,9% | nie (w rozrzucie) |
| range_scan | 41,1 (41,1–42,2) µs | 57,4 (55,6–59,1) µs | +39,6% | tak, wolniej/więcej |
| full_scan | 4,51 (4,48–4,56) ms | 5,16 (5,00–5,22) ms | +14,5% | tak, wolniej/więcej |
| group_by | 9,76 (9,50–9,94) ms | 10,6 (10,4–10,8) ms | +8,5% | tak, wolniej/więcej |
| top_k | 5,32 (5,27–5,50) ms | 5,98 (5,56–6,03) ms | +12,4% | tak, wolniej/więcej |
| materialize_10k | 1,76 (1,67–1,85) ms | 2,36 (2,30–2,44) ms | +34,4% | tak, wolniej/więcej |
| update_10pct | 15,7 (15,3–16,2) ms | 16,5 (16,3–16,6) ms | +5,3% | tak, wolniej/więcej |
| delete_10pct | 17,9 (17,7–18,0) ms | 20,7 (20,5–21,0) ms | +15,6% | tak, wolniej/więcej |
| snapshot | 22,6 (22,3–22,9) ms | 22,1 (22,0–22,5) ms | -2,1% | nie (w rozrzucie) |
| snapshot (rozmiar) | 3,71 MiB | 3,71 MiB | +0,0% | nie (w rozrzucie) |
| restore | 48,7 (48,3–49,4) ms | 66,0 (65,2–67,4) ms | +35,6% | tak, wolniej/więcej |
| insert_many | 151 (149–154) ms | 200 (197–201) ms | +32,4% | tak, wolniej/więcej |

Wyniki zgodne w obu wersjach (160 porównań sum kontrolnych i stanu tabeli).

### node – workload – flatsql-json

| Metryka | Przed | Po | Zmiana mediany | 95% CI średnich rozłączne |
|---|---:|---:|---:|---|
| insert | 286 (283–287) ms | 355 (352–358) ms | +24,0% | tak, wolniej/więcej |
| point_lookup | 4,13 (4,09–4,57) µs | 4,64 (4,51–4,75) µs | +12,3% | nie (w rozrzucie) |
| index_lookup | 3,06 (3,02–3,32) µs | 3,65 (3,53–4,15) µs | +19,4% | nie (w rozrzucie) |
| range_scan | 43,7 (43,3–44,3) µs | 57,4 (53,8–58,2) µs | +31,4% | tak, wolniej/więcej |
| full_scan | 4,52 (4,51–4,54) ms | 5,12 (4,98–5,30) ms | +13,3% | tak, wolniej/więcej |
| group_by | 10,1 (10,1–10,4) ms | 10,9 (10,7–11,1) ms | +7,7% | tak, wolniej/więcej |
| top_k | 5,68 (5,31–5,89) ms | 5,48 (5,45–5,51) ms | -3,4% | nie (w rozrzucie) |
| materialize_10k | 8,81 (8,60–8,98) ms | 9,93 (9,76–10,1) ms | +12,6% | tak, wolniej/więcej |
| update_10pct | 15,4 (15,1–15,9) ms | 16,8 (16,6–16,9) ms | +9,1% | tak, wolniej/więcej |
| delete_10pct | 17,8 (17,6–17,9) ms | 20,5 (20,3–21,1) ms | +15,5% | tak, wolniej/więcej |
| snapshot | 21,9 (21,7–22,3) ms | 22,0 (21,8–22,4) ms | +0,4% | nie (w rozrzucie) |
| snapshot (rozmiar) | 3,71 MiB | 3,71 MiB | +0,0% | nie (w rozrzucie) |
| restore | 48,4 (48,0–49,4) ms | 66,6 (65,8–67,0) ms | +37,7% | tak, wolniej/więcej |
| insert_many | 151 (150–155) ms | 199 (199–199) ms | +31,3% | tak, wolniej/więcej |

Wyniki zgodne w obu wersjach (160 porównań sum kontrolnych i stanu tabeli).

