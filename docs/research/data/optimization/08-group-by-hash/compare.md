Przed: po-2.12 (core: build secondary indexes in bulk from rows grouped by value); po: po-2.12-dirty (core: build secondary indexes in bulk from rows grouped by value).
Obciążenie procesora przed pomiarem: 16,3%. Uruchomienia naprzemienne, 10 na wersję.
Moduł WebAssembly: 684092 → 690130 B, gzip -9: 257430 → 259491 B (+2061 B).

### native – workload – flatsql-native

| Metryka | Przed | Po | Zmiana mediany | 95% CI średnich rozłączne |
|---|---:|---:|---:|---|
| insert | 169 (167–170) ms | 173 (171–175) ms | +2,5% | tak, wolniej/więcej |
| point_lookup | 1,37 (1,30–1,42) µs | 1,29 (1,27–1,35) µs | -6,0% | nie (w rozrzucie) |
| index_lookup | 0,73 (0,73–0,84) µs | 0,76 (0,76–0,77) µs | +4,2% | nie (w rozrzucie) |
| range_scan | 54,5 (52,7–56,3) µs | 53,3 (51,9–57,7) µs | -2,1% | nie (w rozrzucie) |
| full_scan | 5,88 (5,82–5,99) ms | 5,84 (5,76–5,91) ms | -0,6% | nie (w rozrzucie) |
| group_by | 16,7 (16,3–16,8) ms | 11,0 (10,8–11,2) ms | -34,3% | tak, szybciej/mniej |
| top_k | 6,47 (6,38–6,48) ms | 6,53 (6,38–6,55) ms | +0,9% | nie (w rozrzucie) |
| materialize_10k | 3,76 (3,72–3,91) ms | 4,00 (3,96–4,05) ms | +6,2% | tak, wolniej/więcej |
| update_10pct | 16,8 (16,2–17,1) ms | 16,2 (16,0–16,6) ms | -3,5% | nie (w rozrzucie) |
| delete_10pct | 17,1 (16,5–17,7) ms | 17,8 (17,5–18,1) ms | +4,3% | nie (w rozrzucie) |
| snapshot | 18,4 (18,3–19,4) ms | 18,7 (18,5–19,3) ms | +1,7% | nie (w rozrzucie) |
| snapshot (rozmiar) | 3,71 MiB | 3,71 MiB | +0,0% | nie (w rozrzucie) |
| restore | 49,8 (48,3–51,2) ms | 47,8 (46,2–48,6) ms | -4,0% | nie (w rozrzucie) |

Wyniki zgodne w obu wersjach (140 porównań sum kontrolnych i stanu tabeli).

### node – workload – flatsql

| Metryka | Przed | Po | Zmiana mediany | 95% CI średnich rozłączne |
|---|---:|---:|---:|---|
| insert | 313 (311–319) ms | 318 (314–326) ms | +1,8% | nie (w rozrzucie) |
| point_lookup | 9,11 (8,92–9,28) µs | 8,73 (8,64–9,07) µs | -4,1% | nie (w rozrzucie) |
| index_lookup | 12,4 (9,64–14,3) µs | 9,82 (9,44–13,1) µs | -20,6% | nie (w rozrzucie) |
| range_scan | 62,5 (60,5–72,8) µs | 67,2 (65,6–70,7) µs | +7,4% | nie (w rozrzucie) |
| full_scan | 5,64 (5,57–5,78) ms | 5,71 (5,54–5,96) ms | +1,2% | nie (w rozrzucie) |
| group_by | 19,2 (18,6–20,2) ms | 11,7 (11,6–12,3) ms | -38,9% | tak, szybciej/mniej |
| top_k | 5,99 (5,94–6,10) ms | 6,00 (5,93–6,09) ms | +0,2% | nie (w rozrzucie) |
| materialize_10k | 11,1 (11,0–11,8) ms | 11,2 (11,0–12,3) ms | +0,8% | nie (w rozrzucie) |
| update_10pct | 18,4 (18,1–19,4) ms | 18,8 (17,8–19,3) ms | +2,4% | nie (w rozrzucie) |
| delete_10pct | 24,8 (23,8–25,2) ms | 23,6 (22,4–24,7) ms | -4,7% | nie (w rozrzucie) |
| snapshot | 30,4 (29,5–32,2) ms | 31,9 (31,5–32,3) ms | +5,1% | nie (w rozrzucie) |
| snapshot (rozmiar) | 3,71 MiB | 3,71 MiB | +0,0% | nie (w rozrzucie) |
| restore | 62,8 (61,8–67,9) ms | 62,7 (61,4–64,4) ms | -0,1% | nie (w rozrzucie) |

Wyniki zgodne w obu wersjach (160 porównań sum kontrolnych i stanu tabeli).

### node – workload – flatsql-json

| Metryka | Przed | Po | Zmiana mediany | 95% CI średnich rozłączne |
|---|---:|---:|---:|---|
| insert | 317 (315–328) ms | 318 (317–327) ms | +0,3% | nie (w rozrzucie) |
| point_lookup | 5,27 (5,07–5,92) µs | 5,35 (4,98–5,59) µs | +1,5% | nie (w rozrzucie) |
| index_lookup | 3,29 (3,15–4,27) µs | 3,25 (3,12–3,48) µs | -1,2% | nie (w rozrzucie) |
| range_scan | 65,9 (61,0–67,1) µs | 63,4 (60,0–67,0) µs | -3,8% | nie (w rozrzucie) |
| full_scan | 5,58 (5,51–5,68) ms | 5,57 (5,49–6,11) ms | -0,2% | nie (w rozrzucie) |
| group_by | 18,9 (18,5–19,6) ms | 12,1 (11,9–12,4) ms | -35,9% | tak, szybciej/mniej |
| top_k | 5,97 (5,90–6,20) ms | 6,05 (5,97–6,21) ms | +1,3% | nie (w rozrzucie) |
| materialize_10k | 11,6 (11,4–12,2) ms | 11,6 (11,5–11,9) ms | -0,0% | nie (w rozrzucie) |
| update_10pct | 17,8 (17,6–18,3) ms | 19,0 (18,0–21,0) ms | +7,0% | nie (w rozrzucie) |
| delete_10pct | 23,0 (22,0–23,2) ms | 23,2 (22,7–24,2) ms | +0,9% | nie (w rozrzucie) |
| snapshot | 29,7 (28,6–32,2) ms | 31,3 (30,9–31,7) ms | +5,6% | nie (w rozrzucie) |
| snapshot (rozmiar) | 3,71 MiB | 3,71 MiB | +0,0% | nie (w rozrzucie) |
| restore | 61,9 (60,5–62,8) ms | 63,0 (62,1–64,8) ms | +1,7% | nie (w rozrzucie) |

Wyniki zgodne w obu wersjach (160 porównań sum kontrolnych i stanu tabeli).

