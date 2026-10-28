Przed: po-2.1 (core: count rows by the access path when it guarantees the whole WHERE clause); po: po-2.1-dirty (core: count rows by the access path when it guarantees the whole WHERE clause).
Obciążenie procesora przed pomiarem: 3,6%. Uruchomienia naprzemienne, 10 na wersję.
Moduł WebAssembly: 586155 → 590884 B, gzip -9: 221633 → 223196 B (+1563 B).

### native – workload – flatsql-native

| Metryka | Przed | Po | Zmiana mediany | 95% CI średnich rozłączne |
|---|---:|---:|---:|---|
| insert | 208 (207–209) ms | 208 (205–210) ms | -0,0% | nie (w rozrzucie) |
| point_lookup | 2,32 (2,28–2,36) µs | 2,25 (2,22–2,26) µs | -3,1% | nie (w rozrzucie) |
| index_lookup | 1,68 (1,66–1,68) µs | 1,65 (1,64–1,73) µs | -1,3% | nie (w rozrzucie) |
| range_scan | 82,2 (81,4–84,6) µs | 51,5 (50,9–52,1) µs | -37,3% | tak, szybciej/mniej |
| full_scan | 8,01 (7,98–8,11) ms | 7,57 (7,45–7,66) ms | -5,5% | tak, szybciej/mniej |
| group_by | 14,8 (14,7–15,0) ms | 15,2 (15,2–15,4) ms | +3,1% | tak, wolniej/więcej |
| top_k | 5,52 (5,50–5,59) ms | 5,70 (5,66–5,74) ms | +3,4% | nie (w rozrzucie) |
| materialize_10k | 3,72 (3,69–3,88) ms | 3,47 (3,44–3,50) ms | -6,6% | tak, szybciej/mniej |
| update_10pct | 31,0 (30,7–31,9) ms | 31,2 (31,0–31,6) ms | +0,5% | nie (w rozrzucie) |
| delete_10pct | 25,4 (25,0–26,1) ms | 25,2 (25,1–25,8) ms | -0,9% | nie (w rozrzucie) |
| snapshot | 25,4 (25,2–26,9) ms | 25,4 (25,3–25,7) ms | -0,2% | nie (w rozrzucie) |
| snapshot (rozmiar) | 5,03 MiB | 5,03 MiB | +0,0% | nie (w rozrzucie) |
| restore | 112 (111–114) ms | 110 (109–110) ms | -2,1% | nie (w rozrzucie) |

Wyniki zgodne w obu wersjach (140 porównań sum kontrolnych i stanu tabeli).

### node – workload – flatsql

| Metryka | Przed | Po | Zmiana mediany | 95% CI średnich rozłączne |
|---|---:|---:|---:|---|
| insert | 378 (369–386) ms | 368 (367–372) ms | -2,7% | nie (w rozrzucie) |
| point_lookup | 9,67 (9,23–9,91) µs | 9,13 (9,03–9,17) µs | -5,5% | nie (w rozrzucie) |
| index_lookup | 9,32 (8,77–9,61) µs | 9,15 (8,61–9,30) µs | -1,9% | nie (w rozrzucie) |
| range_scan | 87,8 (81,1–90,9) µs | 57,6 (57,1–62,4) µs | -34,5% | tak, szybciej/mniej |
| full_scan | 8,20 (8,05–8,62) ms | 7,99 (7,86–8,06) ms | -2,6% | tak, szybciej/mniej |
| group_by | 15,7 (15,4–16,7) ms | 16,0 (15,9–16,2) ms | +2,2% | nie (w rozrzucie) |
| top_k | 5,37 (5,33–5,50) ms | 5,96 (5,79–6,01) ms | +11,0% | nie (w rozrzucie) |
| materialize_10k | 10,0 (9,85–10,2) ms | 9,81 (9,67–9,92) ms | -2,3% | nie (w rozrzucie) |
| update_10pct | 37,5 (35,8–43,1) ms | 43,1 (36,2–47,2) ms | +14,9% | nie (w rozrzucie) |
| delete_10pct | 30,8 (29,6–31,5) ms | 31,2 (29,9–32,0) ms | +1,3% | nie (w rozrzucie) |
| snapshot | 37,1 (36,3–38,2) ms | 37,3 (37,0–38,1) ms | +0,5% | nie (w rozrzucie) |
| snapshot (rozmiar) | 5,03 MiB | 5,03 MiB | +0,0% | nie (w rozrzucie) |
| restore | 119 (118–120) ms | 122 (119–124) ms | +2,4% | nie (w rozrzucie) |

Wyniki zgodne w obu wersjach (160 porównań sum kontrolnych i stanu tabeli).

### node – workload – flatsql-json

| Metryka | Przed | Po | Zmiana mediany | 95% CI średnich rozłączne |
|---|---:|---:|---:|---|
| insert | 375 (368–388) ms | 367 (365–371) ms | -2,1% | nie (w rozrzucie) |
| point_lookup | 6,20 (5,93–6,53) µs | 6,06 (5,92–6,20) µs | -2,3% | nie (w rozrzucie) |
| index_lookup | 5,52 (4,86–5,66) µs | 4,89 (4,87–5,52) µs | -11,5% | nie (w rozrzucie) |
| range_scan | 83,8 (79,5–87,9) µs | 55,1 (54,3–56,9) µs | -34,3% | tak, szybciej/mniej |
| full_scan | 7,90 (7,87–7,94) ms | 7,94 (7,91–8,08) ms | +0,6% | nie (w rozrzucie) |
| group_by | 15,6 (15,4–15,8) ms | 16,1 (15,8–16,6) ms | +3,5% | nie (w rozrzucie) |
| top_k | 5,36 (5,34–5,41) ms | 5,79 (5,71–5,90) ms | +7,9% | tak, wolniej/więcej |
| materialize_10k | 9,66 (9,46–10,2) ms | 9,59 (9,54–9,74) ms | -0,7% | nie (w rozrzucie) |
| update_10pct | 40,6 (36,4–43,8) ms | 36,3 (36,0–37,5) ms | -10,6% | nie (w rozrzucie) |
| delete_10pct | 29,9 (29,2–31,8) ms | 30,4 (29,7–31,6) ms | +1,7% | nie (w rozrzucie) |
| snapshot | 37,6 (36,7–38,2) ms | 37,8 (37,2–38,3) ms | +0,5% | nie (w rozrzucie) |
| snapshot (rozmiar) | 5,03 MiB | 5,03 MiB | +0,0% | nie (w rozrzucie) |
| restore | 120 (118–124) ms | 121 (119–125) ms | +0,6% | nie (w rozrzucie) |

Wyniki zgodne w obu wersjach (160 porównań sum kontrolnych i stanu tabeli).

