Przed: po-2.9 (core: compile statements once and keep the plan in prepared statements); po: po-2.9-dirty (core: compile statements once and keep the plan in prepared statements).
Obciążenie procesora przed pomiarem: 7,4%. Uruchomienia naprzemienne, 20 na wersję.
Moduł WebAssembly: 608861 → 628161 B, gzip -9: 232563 → 238385 B (+5822 B).

### native – workload – flatsql-native

| Metryka | Przed | Po | Zmiana mediany | 95% CI średnich rozłączne |
|---|---:|---:|---:|---|
| insert | 205 (205–208) ms | 208 (206–209) ms | +1,0% | nie (w rozrzucie) |
| point_lookup | 1,20 (1,18–1,21) µs | 1,20 (1,17–1,22) µs | -0,2% | nie (w rozrzucie) |
| index_lookup | 0,73 (0,72–0,74) µs | 0,71 (0,70–0,74) µs | -2,8% | nie (w rozrzucie) |
| range_scan | 49,7 (48,2–51,7) µs | 49,3 (48,0–51,2) µs | -0,7% | nie (w rozrzucie) |
| full_scan | 7,62 (7,45–7,73) ms | 5,47 (5,38–5,54) ms | -28,2% | tak, szybciej/mniej |
| group_by | 15,4 (15,3–15,5) ms | 15,8 (15,6–15,9) ms | +2,5% | tak, wolniej/więcej |
| top_k | 5,83 (5,77–5,91) ms | 5,95 (5,92–6,02) ms | +2,1% | tak, wolniej/więcej |
| materialize_10k | 3,76 (3,69–3,81) ms | 3,65 (3,60–3,68) ms | -3,0% | nie (w rozrzucie) |
| update_10pct | 31,8 (31,4–32,3) ms | 30,4 (30,0–30,8) ms | -4,5% | tak, szybciej/mniej |
| delete_10pct | 25,8 (25,2–26,2) ms | 24,0 (23,6–24,4) ms | -7,0% | tak, szybciej/mniej |
| snapshot | 25,8 (25,4–26,1) ms | 25,6 (25,3–25,9) ms | -0,8% | nie (w rozrzucie) |
| snapshot (rozmiar) | 5,03 MiB | 5,03 MiB | +0,0% | nie (w rozrzucie) |
| restore | 112 (110–113) ms | 111 (110–114) ms | -0,4% | nie (w rozrzucie) |

Wyniki zgodne w obu wersjach (280 porównań sum kontrolnych i stanu tabeli).

