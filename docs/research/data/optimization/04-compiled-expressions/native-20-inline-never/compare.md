Przed: po-2.9 (core: compile statements once and keep the plan in prepared statements); po: po-2.9-dirty (core: compile statements once and keep the plan in prepared statements).
Obciążenie procesora przed pomiarem: 9,1%. Uruchomienia naprzemienne, 20 na wersję.
Moduł WebAssembly: 608861 → 628161 B, gzip -9: 232563 → 238385 B (+5822 B).

### native – workload – flatsql-native

| Metryka | Przed | Po | Zmiana mediany | 95% CI średnich rozłączne |
|---|---:|---:|---:|---|
| insert | 208 (206–210) ms | 207 (206–208) ms | -0,5% | nie (w rozrzucie) |
| point_lookup | 1,20 (1,17–1,23) µs | 1,23 (1,18–1,27) µs | +2,8% | nie (w rozrzucie) |
| index_lookup | 0,73 (0,72–0,74) µs | 0,72 (0,70–0,74) µs | -1,6% | nie (w rozrzucie) |
| range_scan | 48,8 (48,1–49,9) µs | 50,3 (48,1–51,8) µs | +3,1% | nie (w rozrzucie) |
| full_scan | 7,51 (7,46–7,63) ms | 6,29 (6,23–6,41) ms | -16,2% | tak, szybciej/mniej |
| group_by | 15,4 (15,3–15,5) ms | 15,7 (15,6–15,8) ms | +2,2% | tak, wolniej/więcej |
| top_k | 5,82 (5,76–5,85) ms | 5,92 (5,89–5,98) ms | +1,8% | tak, wolniej/więcej |
| materialize_10k | 3,73 (3,70–3,77) ms | 3,90 (3,86–3,95) ms | +4,8% | tak, wolniej/więcej |
| update_10pct | 32,5 (31,8–33,1) ms | 30,7 (30,3–31,2) ms | -5,4% | tak, szybciej/mniej |
| delete_10pct | 25,4 (25,1–25,8) ms | 24,2 (23,8–24,5) ms | -4,9% | tak, szybciej/mniej |
| snapshot | 25,9 (25,6–26,9) ms | 25,7 (25,3–26,8) ms | -1,0% | nie (w rozrzucie) |
| snapshot (rozmiar) | 5,03 MiB | 5,03 MiB | +0,0% | nie (w rozrzucie) |
| restore | 111 (110–113) ms | 113 (111–115) ms | +1,5% | nie (w rozrzucie) |

Wyniki zgodne w obu wersjach (280 porównań sum kontrolnych i stanu tabeli).

