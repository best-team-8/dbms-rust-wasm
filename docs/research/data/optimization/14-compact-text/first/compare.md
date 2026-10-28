Przed: po-2.14 (core/wasm: execute a prepared write for many parameter sets in one call); po: po-2.14-dirty (core/wasm: execute a prepared write for many parameter sets in one call).
Obciążenie procesora przed pomiarem: 9,4%. Uruchomienia naprzemienne, 10 na wersję.
Moduł WebAssembly: 700803 → 665164 B, gzip -9: 264711 → 259731 B (-4980 B).

### native – workload – flatsql-native

| Metryka | Przed | Po | Zmiana mediany | 95% CI średnich rozłączne |
|---|---:|---:|---:|---|
| insert | 160 (159–163) ms | 178 (177–180) ms | +11,2% | tak, wolniej/więcej |
| point_lookup | 1,18 (1,17–1,21) µs | 0,97 (0,94–1,02) µs | -17,7% | tak, szybciej/mniej |
| index_lookup | 0,77 (0,76–0,78) µs | 0,70 (0,70–0,70) µs | -8,9% | tak, szybciej/mniej |
| range_scan | 49,6 (47,8–51,2) µs | 51,4 (49,3–52,4) µs | +3,7% | nie (w rozrzucie) |
| full_scan | 5,41 (5,36–5,53) ms | 5,44 (5,34–5,46) ms | +0,6% | nie (w rozrzucie) |
| group_by | 10,1 (10,0–10,2) ms | 11,0 (10,9–11,1) ms | +8,2% | tak, wolniej/więcej |
| top_k | 5,82 (5,78–5,85) ms | 6,92 (6,85–7,01) ms | +19,0% | tak, wolniej/więcej |
| materialize_10k | 3,42 (3,39–3,50) ms | 3,07 (3,05–3,10) ms | -10,0% | tak, szybciej/mniej |
| update_10pct | 15,2 (15,0–15,4) ms | 15,0 (14,9–15,3) ms | -1,3% | nie (w rozrzucie) |
| delete_10pct | 16,2 (15,7–16,4) ms | 15,0 (14,8–15,1) ms | -7,5% | tak, szybciej/mniej |
| snapshot | 17,6 (17,4–17,7) ms | 17,2 (17,2–17,4) ms | -1,9% | tak, szybciej/mniej |
| snapshot (rozmiar) | 3,71 MiB | 3,71 MiB | +0,0% | nie (w rozrzucie) |
| restore | 44,5 (44,3–45,2) ms | 42,4 (42,2–42,9) ms | -4,7% | tak, szybciej/mniej |
| insert_many | 144 (142–145) ms | 163 (162–164) ms | +13,7% | tak, wolniej/więcej |

Wyniki zgodne w obu wersjach (140 porównań sum kontrolnych i stanu tabeli).

### node – memory – flatsql

| Metryka | Przed | Po | Zmiana mediany | 95% CI średnich rozłączne |
|---|---:|---:|---:|---|
| wasm_after_load (rozmiar) | 54,8 MiB | 51,8 MiB | -5,5% | tak, szybciej/mniej |
| js_heap_delta (rozmiar) | 82,5 KiB (82,5 KiB–83,2 KiB) | 82,8 KiB (82,7 KiB–83,9 KiB) | +0,4% | nie (w rozrzucie) |

### node – memory – flatsql-json

| Metryka | Przed | Po | Zmiana mediany | 95% CI średnich rozłączne |
|---|---:|---:|---:|---|
| wasm_after_load (rozmiar) | 54,8 MiB | 51,8 MiB | -5,5% | tak, szybciej/mniej |
| js_heap_delta (rozmiar) | 82,5 KiB (82,4 KiB–83,6 KiB) | 83,8 KiB (83,0 KiB–83,9 KiB) | +1,5% | nie (w rozrzucie) |

### node – workload – flatsql

| Metryka | Przed | Po | Zmiana mediany | 95% CI średnich rozłączne |
|---|---:|---:|---:|---|
| insert | 298 (292–301) ms | 316 (312–318) ms | +6,1% | nie (w rozrzucie) |
| point_lookup | 2,60 (2,55–3,00) µs | 2,54 (2,52–2,55) µs | -2,5% | nie (w rozrzucie) |
| index_lookup | 2,41 (1,75–2,75) µs | 1,74 (1,71–1,84) µs | -27,9% | nie (w rozrzucie) |
| range_scan | 50,6 (49,8–52,3) µs | 42,9 (42,5–43,6) µs | -15,2% | tak, szybciej/mniej |
| full_scan | 5,08 (4,95–5,19) ms | 4,59 (4,54–4,71) ms | -9,7% | tak, szybciej/mniej |
| group_by | 10,5 (10,4–10,6) ms | 9,67 (9,55–9,82) ms | -8,0% | tak, szybciej/mniej |
| top_k | 5,45 (5,39–5,61) ms | 5,60 (5,48–5,80) ms | +2,7% | nie (w rozrzucie) |
| materialize_10k | 1,72 (1,65–1,74) ms | 1,76 (1,72–1,91) ms | +2,8% | nie (w rozrzucie) |
| update_10pct | 16,2 (16,0–16,6) ms | 15,9 (15,6–16,3) ms | -2,0% | nie (w rozrzucie) |
| delete_10pct | 19,4 (19,3–19,7) ms | 19,9 (19,1–20,6) ms | +2,4% | nie (w rozrzucie) |
| snapshot | 26,6 (26,1–27,3) ms | 23,7 (22,7–24,6) ms | -11,0% | tak, szybciej/mniej |
| snapshot (rozmiar) | 3,71 MiB | 3,71 MiB | +0,0% | nie (w rozrzucie) |
| restore | 52,1 (51,3–53,4) ms | 46,9 (46,8–49,0) ms | -10,0% | tak, szybciej/mniej |
| insert_many | 155 (153–157) ms | 178 (177–184) ms | +15,3% | tak, wolniej/więcej |

Wyniki zgodne w obu wersjach (160 porównań sum kontrolnych i stanu tabeli).

### node – workload – flatsql-json

| Metryka | Przed | Po | Zmiana mediany | 95% CI średnich rozłączne |
|---|---:|---:|---:|---|
| insert | 292 (289–302) ms | 311 (310–315) ms | +6,4% | nie (w rozrzucie) |
| point_lookup | 4,55 (4,34–4,78) µs | 4,27 (4,21–4,72) µs | -6,3% | nie (w rozrzucie) |
| index_lookup | 3,18 (3,12–3,51) µs | 2,98 (2,96–3,13) µs | -6,4% | nie (w rozrzucie) |
| range_scan | 52,8 (51,4–57,2) µs | 45,7 (45,1–47,7) µs | -13,4% | tak, szybciej/mniej |
| full_scan | 5,05 (5,01–5,12) ms | 4,54 (4,53–4,55) ms | -10,0% | tak, szybciej/mniej |
| group_by | 10,8 (10,6–11,0) ms | 9,82 (9,65–10,00) ms | -9,2% | tak, szybciej/mniej |
| top_k | 5,54 (5,43–5,71) ms | 5,58 (5,50–5,66) ms | +0,7% | nie (w rozrzucie) |
| materialize_10k | 10,9 (10,8–11,2) ms | 8,98 (8,86–9,22) ms | -17,5% | tak, szybciej/mniej |
| update_10pct | 16,3 (15,7–17,4) ms | 15,6 (15,4–15,7) ms | -4,3% | nie (w rozrzucie) |
| delete_10pct | 19,4 (19,0–20,4) ms | 19,1 (18,8–19,6) ms | -1,5% | nie (w rozrzucie) |
| snapshot | 26,2 (25,9–27,8) ms | 22,4 (22,1–22,7) ms | -14,8% | tak, szybciej/mniej |
| snapshot (rozmiar) | 3,71 MiB | 3,71 MiB | +0,0% | nie (w rozrzucie) |
| restore | 54,1 (52,0–59,6) ms | 47,2 (46,8–47,5) ms | -12,7% | nie (w rozrzucie) |
| insert_many | 159 (155–169) ms | 177 (176–178) ms | +11,6% | nie (w rozrzucie) |

Wyniki zgodne w obu wersjach (160 porównań sum kontrolnych i stanu tabeli).

