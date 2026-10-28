Przed: po-2.14 (core/wasm: execute a prepared write for many parameter sets in one call); po: po-2.14-dirty (core/wasm: execute a prepared write for many parameter sets in one call).
Obciążenie procesora przed pomiarem: 5,8%. Uruchomienia naprzemienne, 10 na wersję.
Moduł WebAssembly: 700803 → 711629 B, gzip -9: 264711 → 268883 B (+4172 B).

### native – workload – flatsql-native

| Metryka | Przed | Po | Zmiana mediany | 95% CI średnich rozłączne |
|---|---:|---:|---:|---|
| insert | 158 (158–159) ms | 158 (157–159) ms | +0,2% | nie (w rozrzucie) |
| point_lookup | 1,15 (1,14–1,16) µs | 0,96 (0,93–1,11) µs | -15,9% | nie (w rozrzucie) |
| index_lookup | 0,76 (0,76–0,76) µs | 0,69 (0,69–0,71) µs | -8,6% | tak, szybciej/mniej |
| range_scan | 47,3 (46,4–48,5) µs | 49,2 (48,7–50,9) µs | +4,1% | nie (w rozrzucie) |
| full_scan | 5,31 (5,27–5,40) ms | 5,43 (5,38–5,49) ms | +2,4% | nie (w rozrzucie) |
| group_by | 10,0 (9,97–10,1) ms | 10,8 (10,7–10,9) ms | +7,7% | tak, wolniej/więcej |
| top_k | 5,82 (5,81–5,86) ms | 6,87 (6,80–6,88) ms | +17,9% | tak, wolniej/więcej |
| materialize_10k | 3,38 (3,36–3,42) ms | 2,87 (2,85–2,88) ms | -15,1% | tak, szybciej/mniej |
| update_10pct | 15,0 (15,0–15,1) ms | 14,7 (14,4–14,7) ms | -2,3% | tak, szybciej/mniej |
| delete_10pct | 15,8 (15,7–16,0) ms | 14,5 (14,3–14,6) ms | -8,0% | tak, szybciej/mniej |
| snapshot | 17,5 (17,5–17,6) ms | 16,7 (16,6–16,8) ms | -4,7% | tak, szybciej/mniej |
| snapshot (rozmiar) | 3,71 MiB | 3,71 MiB | +0,0% | nie (w rozrzucie) |
| restore | 44,4 (44,1–45,0) ms | 40,7 (39,7–41,0) ms | -8,4% | tak, szybciej/mniej |
| insert_many | 142 (141–142) ms | 141 (140–142) ms | -0,3% | nie (w rozrzucie) |

Wyniki zgodne w obu wersjach (140 porównań sum kontrolnych i stanu tabeli).

### node – memory – flatsql

| Metryka | Przed | Po | Zmiana mediany | 95% CI średnich rozłączne |
|---|---:|---:|---:|---|
| wasm_after_load (rozmiar) | 54,8 MiB | 51,8 MiB | -5,5% | tak, szybciej/mniej |
| js_heap_delta (rozmiar) | 83,1 KiB (82,5 KiB–83,7 KiB) | 83,5 KiB (82,9 KiB–84,1 KiB) | +0,5% | nie (w rozrzucie) |

### node – memory – flatsql-json

| Metryka | Przed | Po | Zmiana mediany | 95% CI średnich rozłączne |
|---|---:|---:|---:|---|
| wasm_after_load (rozmiar) | 54,8 MiB | 51,8 MiB | -5,5% | tak, szybciej/mniej |
| js_heap_delta (rozmiar) | 83,0 KiB (82,5 KiB–83,6 KiB) | 82,9 KiB (82,9 KiB–83,0 KiB) | -0,1% | nie (w rozrzucie) |

### node – workload – flatsql

| Metryka | Przed | Po | Zmiana mediany | 95% CI średnich rozłączne |
|---|---:|---:|---:|---|
| insert | 287 (286–289) ms | 288 (287–290) ms | +0,6% | nie (w rozrzucie) |
| point_lookup | 2,78 (2,49–3,09) µs | 2,48 (2,44–2,53) µs | -10,8% | tak, szybciej/mniej |
| index_lookup | 1,77 (1,74–2,16) µs | 1,83 (1,75–2,21) µs | +3,2% | nie (w rozrzucie) |
| range_scan | 48,7 (48,2–49,5) µs | 40,7 (40,3–41,6) µs | -16,3% | tak, szybciej/mniej |
| full_scan | 4,91 (4,87–5,01) ms | 4,53 (4,51–4,54) ms | -7,6% | tak, szybciej/mniej |
| group_by | 10,3 (10,3–10,4) ms | 9,60 (9,50–9,77) ms | -7,0% | tak, szybciej/mniej |
| top_k | 5,33 (5,31–5,34) ms | 5,30 (5,27–5,34) ms | -0,5% | nie (w rozrzucie) |
| materialize_10k | 1,66 (1,61–1,78) ms | 1,88 (1,77–2,04) ms | +13,2% | nie (w rozrzucie) |
| update_10pct | 15,9 (15,9–16,2) ms | 15,2 (14,8–16,0) ms | -4,7% | nie (w rozrzucie) |
| delete_10pct | 19,1 (18,9–19,4) ms | 18,2 (18,0–18,8) ms | -4,5% | nie (w rozrzucie) |
| snapshot | 25,6 (25,4–26,2) ms | 22,5 (22,4–22,8) ms | -11,9% | tak, szybciej/mniej |
| snapshot (rozmiar) | 3,71 MiB | 3,71 MiB | +0,0% | nie (w rozrzucie) |
| restore | 51,0 (50,7–51,5) ms | 48,4 (48,2–49,0) ms | -5,1% | tak, szybciej/mniej |
| insert_many | 154 (153–154) ms | 150 (150–151) ms | -2,3% | tak, szybciej/mniej |

Wyniki zgodne w obu wersjach (160 porównań sum kontrolnych i stanu tabeli).

### node – workload – flatsql-json

| Metryka | Przed | Po | Zmiana mediany | 95% CI średnich rozłączne |
|---|---:|---:|---:|---|
| insert | 289 (288–291) ms | 289 (287–291) ms | -0,1% | nie (w rozrzucie) |
| point_lookup | 4,26 (4,22–4,84) µs | 4,20 (4,14–4,42) µs | -1,3% | nie (w rozrzucie) |
| index_lookup | 3,21 (3,04–3,47) µs | 3,30 (3,05–3,68) µs | +2,8% | nie (w rozrzucie) |
| range_scan | 51,1 (50,3–51,6) µs | 44,0 (43,7–45,4) µs | -13,9% | tak, szybciej/mniej |
| full_scan | 4,92 (4,91–4,94) ms | 4,53 (4,52–4,54) ms | -7,9% | tak, szybciej/mniej |
| group_by | 10,9 (10,5–11,1) ms | 9,79 (9,72–10,0) ms | -10,5% | tak, szybciej/mniej |
| top_k | 5,49 (5,42–5,81) ms | 5,66 (5,62–5,70) ms | +3,1% | nie (w rozrzucie) |
| materialize_10k | 10,7 (10,7–11,1) ms | 9,23 (9,00–9,49) ms | -14,1% | tak, szybciej/mniej |
| update_10pct | 17,2 (16,7–17,3) ms | 15,5 (15,4–15,8) ms | -9,9% | nie (w rozrzucie) |
| delete_10pct | 21,8 (21,0–22,0) ms | 19,9 (19,8–20,9) ms | -8,4% | nie (w rozrzucie) |
| snapshot | 27,4 (26,3–30,3) ms | 23,6 (23,1–27,9) ms | -13,7% | nie (w rozrzucie) |
| snapshot (rozmiar) | 3,71 MiB | 3,71 MiB | +0,0% | nie (w rozrzucie) |
| restore | 59,0 (58,4–60,2) ms | 55,2 (55,1–57,0) ms | -6,4% | nie (w rozrzucie) |
| insert_many | 168 (164–175) ms | 161 (159–162) ms | -4,3% | nie (w rozrzucie) |

Wyniki zgodne w obu wersjach (160 porównań sum kontrolnych i stanu tabeli).

