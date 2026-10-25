Przed: prototyp (bench: module size of Worker engines, plain values from DuckDB, table alignment in the report); po: prototyp-7-g8f92ab2-dirty (ci: benchmark smoke run with the options of the new runner, failed runs exit with an error, native benchmark formatted).
Obciążenie procesora przed pomiarem: 11,8%. Uruchomienia naprzemienne, 10 na wersję.
Moduł WebAssembly: 578926 → 586227 B, gzip -9: 219575 → 221642 B (+2067 B).

### native – workload – flatsql-native

| Metryka | Przed | Po | Zmiana mediany | 95% CI średnich rozłączne |
|---|---:|---:|---:|---|
| insert | 220 (218–223) ms | 219 (217–222) ms | -0,5% | nie (w rozrzucie) |
| point_lookup | 2,47 (2,39–2,60) µs | 2,40 (2,38–2,46) µs | -2,8% | nie (w rozrzucie) |
| index_lookup | 486 (478–488) µs | 1,66 (1,65–1,68) µs | -99,7% | tak, szybciej/mniej |
| range_scan | 88,7 (86,1–95,1) µs | 86,4 (83,6–91,9) µs | -2,6% | nie (w rozrzucie) |
| full_scan | 8,45 (8,38–8,53) ms | 8,27 (8,18–8,37) ms | -2,1% | nie (w rozrzucie) |
| group_by | 15,3 (15,1–15,4) ms | 15,3 (15,2–15,4) ms | -0,1% | nie (w rozrzucie) |
| top_k | 5,91 (5,84–5,99) ms | 5,79 (5,70–5,92) ms | -1,9% | nie (w rozrzucie) |
| materialize_10k | 3,96 (3,91–4,08) ms | 3,88 (3,83–3,96) ms | -2,1% | nie (w rozrzucie) |
| update_10pct | 33,1 (32,0–34,9) ms | 32,0 (31,8–33,4) ms | -3,5% | nie (w rozrzucie) |
| delete_10pct | 25,9 (25,7–26,3) ms | 25,8 (25,7–26,3) ms | -0,4% | nie (w rozrzucie) |
| snapshot | 26,7 (26,3–27,2) ms | 25,9 (25,8–26,6) ms | -3,0% | nie (w rozrzucie) |
| snapshot (rozmiar) | 5,03 MiB | 5,03 MiB | +0,0% | nie (w rozrzucie) |
| restore | 117 (115–120) ms | 115 (114–118) ms | -1,3% | nie (w rozrzucie) |

Wyniki zgodne w obu wersjach (140 porównań sum kontrolnych i stanu tabeli).

### node – workload – flatsql

| Metryka | Przed | Po | Zmiana mediany | 95% CI średnich rozłączne |
|---|---:|---:|---:|---|
| insert | 427 (412–439) ms | 402 (400–408) ms | -5,9% | tak, szybciej/mniej |
| point_lookup | 10,6 (10,3–10,8) µs | 9,76 (9,65–10,0) µs | -8,2% | nie (w rozrzucie) |
| index_lookup | 498 (482–537) µs | 9,47 (8,67–10,1) µs | -98,1% | tak, szybciej/mniej |
| range_scan | 96,5 (90,8–98,9) µs | 92,3 (89,5–93,6) µs | -4,3% | nie (w rozrzucie) |
| full_scan | 8,32 (8,19–8,89) ms | 8,36 (8,22–8,48) ms | +0,5% | nie (w rozrzucie) |
| group_by | 16,9 (16,7–17,5) ms | 17,1 (16,1–17,7) ms | +1,3% | nie (w rozrzucie) |
| top_k | 5,73 (5,64–6,01) ms | 5,71 (5,62–6,21) ms | -0,4% | nie (w rozrzucie) |
| materialize_10k | 9,94 (9,81–10,3) ms | 10,1 (9,96–10,4) ms | +1,2% | nie (w rozrzucie) |
| update_10pct | 44,0 (36,8–51,6) ms | 43,3 (38,3–49,3) ms | -1,6% | nie (w rozrzucie) |
| delete_10pct | 32,0 (31,1–35,1) ms | 31,4 (30,9–33,5) ms | -2,0% | nie (w rozrzucie) |
| snapshot | 37,6 (37,0–38,4) ms | 38,2 (37,9–38,6) ms | +1,7% | nie (w rozrzucie) |
| snapshot (rozmiar) | 5,03 MiB | 5,03 MiB | +0,0% | nie (w rozrzucie) |
| restore | 122 (121–126) ms | 125 (124–136) ms | +2,0% | nie (w rozrzucie) |

Wyniki zgodne w obu wersjach (160 porównań sum kontrolnych i stanu tabeli).

### node – workload – flatsql-json

| Metryka | Przed | Po | Zmiana mediany | 95% CI średnich rozłączne |
|---|---:|---:|---:|---|
| insert | 416 (412–462) ms | 413 (406–428) ms | -0,8% | nie (w rozrzucie) |
| point_lookup | 6,30 (6,19–6,72) µs | 6,40 (6,22–6,56) µs | +1,6% | nie (w rozrzucie) |
| index_lookup | 484 (461–520) µs | 5,58 (4,61–6,76) µs | -98,8% | tak, szybciej/mniej |
| range_scan | 93,4 (88,6–96,6) µs | 87,8 (81,8–90,2) µs | -6,0% | nie (w rozrzucie) |
| full_scan | 8,23 (8,14–8,38) ms | 8,16 (8,11–8,24) ms | -0,8% | nie (w rozrzucie) |
| group_by | 17,7 (17,2–18,4) ms | 16,6 (16,1–17,8) ms | -6,5% | nie (w rozrzucie) |
| top_k | 5,76 (5,58–6,67) ms | 5,65 (5,59–5,70) ms | -1,9% | nie (w rozrzucie) |
| materialize_10k | 10,2 (10,1–10,3) ms | 10,3 (9,98–10,8) ms | +0,4% | nie (w rozrzucie) |
| update_10pct | 44,9 (37,6–49,1) ms | 37,5 (36,8–39,2) ms | -16,5% | nie (w rozrzucie) |
| delete_10pct | 30,7 (30,1–31,9) ms | 31,7 (30,7–33,4) ms | +3,3% | nie (w rozrzucie) |
| snapshot | 37,5 (37,2–38,2) ms | 37,7 (37,3–38,6) ms | +0,4% | nie (w rozrzucie) |
| snapshot (rozmiar) | 5,03 MiB | 5,03 MiB | +0,0% | nie (w rozrzucie) |
| restore | 122 (120–124) ms | 122 (121–124) ms | -0,4% | nie (w rozrzucie) |

Wyniki zgodne w obu wersjach (160 porównań sum kontrolnych i stanu tabeli).

