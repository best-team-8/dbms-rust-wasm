Przed: po-2.2 (core: skip the conjuncts of WHERE that the access path guarantees); po: po-2.2-dirty (core: skip the conjuncts of WHERE that the access path guarantees).
Obciążenie procesora przed pomiarem: 12,4%. Uruchomienia naprzemienne, 10 na wersję.
Moduł WebAssembly: 590964 → 608941 B, gzip -9: 223497 → 232549 B (+9052 B).

### native – workload – flatsql-native

| Metryka | Przed | Po | Zmiana mediany | 95% CI średnich rozłączne |
|---|---:|---:|---:|---|
| insert | 246 (233–253) ms | 229 (226–235) ms | -7,0% | nie (w rozrzucie) |
| point_lookup | 2,71 (2,59–2,87) µs | 1,27 (1,22–1,31) µs | -53,3% | tak, szybciej/mniej |
| index_lookup | 1,73 (1,68–2,25) µs | 0,70 (0,68–0,72) µs | -59,6% | tak, szybciej/mniej |
| range_scan | 58,3 (52,8–62,2) µs | 51,7 (49,9–57,1) µs | -11,2% | nie (w rozrzucie) |
| full_scan | 8,57 (8,36–8,78) ms | 8,56 (8,50–8,62) ms | -0,1% | nie (w rozrzucie) |
| group_by | 17,8 (17,1–20,4) ms | 17,1 (16,7–17,4) ms | -3,6% | nie (w rozrzucie) |
| top_k | 6,17 (5,96–6,78) ms | 6,23 (6,10–6,41) ms | +0,9% | nie (w rozrzucie) |
| materialize_10k | 3,99 (3,84–4,24) ms | 4,15 (4,05–4,39) ms | +4,0% | nie (w rozrzucie) |
| update_10pct | 34,6 (32,7–36,1) ms | 37,4 (35,0–39,2) ms | +8,2% | nie (w rozrzucie) |
| delete_10pct | 27,4 (26,8–29,3) ms | 27,6 (26,7–28,7) ms | +0,7% | nie (w rozrzucie) |
| snapshot | 29,4 (28,4–31,7) ms | 27,2 (26,8–28,2) ms | -7,4% | tak, szybciej/mniej |
| snapshot (rozmiar) | 5,03 MiB | 5,03 MiB | +0,0% | nie (w rozrzucie) |
| restore | 127 (126–130) ms | 122 (116–131) ms | -3,8% | nie (w rozrzucie) |

Wyniki zgodne w obu wersjach (140 porównań sum kontrolnych i stanu tabeli).

### node – workload – flatsql

| Metryka | Przed | Po | Zmiana mediany | 95% CI średnich rozłączne |
|---|---:|---:|---:|---|
| insert | 459 (443–501) ms | 458 (447–469) ms | -0,3% | nie (w rozrzucie) |
| point_lookup | 10,1 (9,96–10,6) µs | 9,13 (8,87–9,40) µs | -9,7% | nie (w rozrzucie) |
| index_lookup | 9,79 (9,72–9,99) µs | 6,58 (6,03–8,66) µs | -32,7% | nie (w rozrzucie) |
| range_scan | 57,5 (55,4–63,3) µs | 58,7 (56,6–63,7) µs | +2,1% | nie (w rozrzucie) |
| full_scan | 9,45 (8,80–10,5) ms | 8,89 (8,74–9,46) ms | -6,0% | nie (w rozrzucie) |
| group_by | 19,1 (18,2–25,0) ms | 19,8 (17,5–21,6) ms | +3,9% | nie (w rozrzucie) |
| top_k | 6,31 (5,96–6,52) ms | 6,33 (5,98–8,81) ms | +0,3% | nie (w rozrzucie) |
| materialize_10k | 11,8 (11,0–13,6) ms | 12,5 (11,6–14,1) ms | +6,2% | nie (w rozrzucie) |
| update_10pct | 41,9 (38,1–52,9) ms | 40,2 (38,5–49,4) ms | -4,0% | nie (w rozrzucie) |
| delete_10pct | 33,8 (32,2–36,2) ms | 34,9 (32,8–36,8) ms | +3,5% | nie (w rozrzucie) |
| snapshot | 39,5 (38,4–41,1) ms | 38,3 (37,9–40,0) ms | -3,0% | nie (w rozrzucie) |
| snapshot (rozmiar) | 5,03 MiB | 5,03 MiB | +0,0% | nie (w rozrzucie) |
| restore | 132 (127–135) ms | 133 (127–141) ms | +0,4% | nie (w rozrzucie) |

Wyniki zgodne w obu wersjach (160 porównań sum kontrolnych i stanu tabeli).

### node – workload – flatsql-json

| Metryka | Przed | Po | Zmiana mediany | 95% CI średnich rozłączne |
|---|---:|---:|---:|---|
| insert | 475 (464–491) ms | 478 (443–483) ms | +0,6% | nie (w rozrzucie) |
| point_lookup | 6,62 (6,46–6,86) µs | 5,23 (5,01–6,09) µs | -20,9% | tak, szybciej/mniej |
| index_lookup | 5,38 (4,70–6,25) µs | 3,35 (3,10–4,42) µs | -37,8% | nie (w rozrzucie) |
| range_scan | 54,2 (52,4–56,1) µs | 56,3 (56,2–63,3) µs | +3,9% | nie (w rozrzucie) |
| full_scan | 8,49 (8,32–8,54) ms | 8,76 (8,41–9,11) ms | +3,2% | nie (w rozrzucie) |
| group_by | 17,7 (17,4–18,8) ms | 17,9 (17,4–19,5) ms | +1,2% | nie (w rozrzucie) |
| top_k | 6,32 (5,84–6,77) ms | 6,25 (6,14–6,58) ms | -1,2% | nie (w rozrzucie) |
| materialize_10k | 11,4 (11,0–12,4) ms | 11,5 (10,9–11,8) ms | +0,6% | nie (w rozrzucie) |
| update_10pct | 44,3 (42,3–46,2) ms | 48,4 (40,0–49,7) ms | +9,3% | nie (w rozrzucie) |
| delete_10pct | 33,2 (31,3–36,3) ms | 33,0 (32,0–35,6) ms | -0,7% | nie (w rozrzucie) |
| snapshot | 39,0 (38,6–39,7) ms | 38,6 (37,6–39,8) ms | -1,2% | nie (w rozrzucie) |
| snapshot (rozmiar) | 5,03 MiB | 5,03 MiB | +0,0% | nie (w rozrzucie) |
| restore | 137 (134–139) ms | 130 (127–137) ms | -5,0% | nie (w rozrzucie) |

Wyniki zgodne w obu wersjach (160 porównań sum kontrolnych i stanu tabeli).

