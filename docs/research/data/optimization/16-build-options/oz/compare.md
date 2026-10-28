Przed: po-2.6 (core/bridge: encode checkpoints in steps); po: po-2.6-dirty (core/bridge: encode checkpoints in steps).
Obciążenie procesora przed pomiarem: 3,3%. Uruchomienia naprzemienne, 10 na wersję.
Moduł WebAssembly: 724002 → 493980 B, gzip -9: 274098 → 205859 B (-68239 B).

### node – memory – flatsql

| Metryka | Przed | Po | Zmiana mediany | 95% CI średnich rozłączne |
|---|---:|---:|---:|---|
| wasm_after_load (rozmiar) | 51,8 MiB | 51,8 MiB | +0,0% | nie (w rozrzucie) |
| js_heap_delta (rozmiar) | 83,0 KiB (83,0 KiB–83,9 KiB) | 85,0 KiB (84,9 KiB–85,0 KiB) | +2,3% | tak, wolniej/więcej |

### node – memory – flatsql-json

| Metryka | Przed | Po | Zmiana mediany | 95% CI średnich rozłączne |
|---|---:|---:|---:|---|
| wasm_after_load (rozmiar) | 51,8 MiB | 51,8 MiB | +0,0% | nie (w rozrzucie) |
| js_heap_delta (rozmiar) | 82,9 KiB (82,9 KiB–83,0 KiB) | 85,0 KiB (85,0 KiB–85,0 KiB) | +2,5% | tak, wolniej/więcej |

### node – workload – flatsql

| Metryka | Przed | Po | Zmiana mediany | 95% CI średnich rozłączne |
|---|---:|---:|---:|---|
| insert | 286 (286–289) ms | 356 (353–360) ms | +24,4% | tak, wolniej/więcej |
| point_lookup | 2,51 (2,46–2,59) µs | 3,04 (3,00–3,07) µs | +21,1% | tak, wolniej/więcej |
| index_lookup | 2,33 (1,99–2,81) µs | 2,64 (2,51–3,09) µs | +13,5% | nie (w rozrzucie) |
| range_scan | 42,0 (41,3–43,1) µs | 74,6 (74,3–75,8) µs | +77,8% | tak, wolniej/więcej |
| full_scan | 4,53 (4,51–4,54) ms | 6,72 (6,61–6,79) ms | +48,4% | tak, wolniej/więcej |
| group_by | 9,87 (9,66–9,91) ms | 14,4 (14,4–14,5) ms | +46,1% | tak, wolniej/więcej |
| top_k | 5,42 (5,25–5,61) ms | 8,45 (8,38–8,58) ms | +56,0% | tak, wolniej/więcej |
| materialize_10k | 1,71 (1,68–1,84) ms | 2,61 (2,53–2,79) ms | +52,6% | tak, wolniej/więcej |
| update_10pct | 15,7 (15,3–15,9) ms | 21,0 (20,5–21,2) ms | +33,5% | tak, wolniej/więcej |
| delete_10pct | 18,2 (17,7–18,6) ms | 24,0 (23,7–24,6) ms | +31,6% | tak, wolniej/więcej |
| snapshot | 22,3 (22,1–22,7) ms | 26,8 (26,3–27,2) ms | +20,1% | tak, wolniej/więcej |
| snapshot (rozmiar) | 3,71 MiB | 3,71 MiB | +0,0% | nie (w rozrzucie) |
| restore | 48,3 (48,0–48,8) ms | 70,4 (69,7–73,2) ms | +45,5% | tak, wolniej/więcej |
| insert_many | 150 (150–153) ms | 204 (202–205) ms | +35,6% | tak, wolniej/więcej |

Wyniki zgodne w obu wersjach (160 porównań sum kontrolnych i stanu tabeli).

### node – workload – flatsql-json

| Metryka | Przed | Po | Zmiana mediany | 95% CI średnich rozłączne |
|---|---:|---:|---:|---|
| insert | 286 (285–287) ms | 354 (353–356) ms | +23,9% | tak, wolniej/więcej |
| point_lookup | 4,26 (4,15–4,36) µs | 5,35 (5,26–5,51) µs | +25,8% | tak, wolniej/więcej |
| index_lookup | 3,15 (3,03–3,94) µs | 4,15 (4,01–4,61) µs | +31,4% | tak, wolniej/więcej |
| range_scan | 44,6 (43,9–45,7) µs | 73,7 (72,8–75,4) µs | +65,2% | tak, wolniej/więcej |
| full_scan | 4,51 (4,49–4,56) ms | 6,64 (6,60–6,74) ms | +47,2% | tak, wolniej/więcej |
| group_by | 10,1 (9,96–10,1) ms | 14,0 (14,0–14,1) ms | +39,2% | tak, wolniej/więcej |
| top_k | 5,30 (5,28–5,40) ms | 9,69 (8,81–9,83) ms | +82,7% | tak, wolniej/więcej |
| materialize_10k | 8,98 (8,95–9,04) ms | 11,4 (11,3–11,5) ms | +26,8% | tak, wolniej/więcej |
| update_10pct | 15,5 (15,3–15,6) ms | 20,9 (20,6–21,0) ms | +35,2% | tak, wolniej/więcej |
| delete_10pct | 17,7 (17,6–18,0) ms | 23,5 (22,9–24,1) ms | +32,6% | tak, wolniej/więcej |
| snapshot | 22,4 (21,8–22,9) ms | 26,7 (26,3–27,0) ms | +19,2% | tak, wolniej/więcej |
| snapshot (rozmiar) | 3,71 MiB | 3,71 MiB | +0,0% | nie (w rozrzucie) |
| restore | 48,3 (48,2–49,5) ms | 69,9 (69,6–70,4) ms | +44,8% | tak, wolniej/więcej |
| insert_many | 151 (151–154) ms | 203 (202–205) ms | +34,4% | tak, wolniej/więcej |

Wyniki zgodne w obu wersjach (160 porównań sum kontrolnych i stanu tabeli).

