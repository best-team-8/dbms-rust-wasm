Przed: po-2.10 (core: compile WHERE conjuncts and SET expressions specialised to column types); po: po-2.10-dirty (core: compile WHERE conjuncts and SET expressions specialised to column types).
Obciążenie procesora przed pomiarem: 9,5%. Uruchomienia naprzemienne, 10 na wersję.
Moduł WebAssembly: 628081 → 643544 B, gzip -9: 238401 → 242791 B (+4390 B).

### node – workload – flatsql

| Metryka | Przed | Po | Zmiana mediany | 95% CI średnich rozłączne |
|---|---:|---:|---:|---|
| insert | 379 (377–387) ms | 295 (292–308) ms | -22,0% | tak, szybciej/mniej |
| point_lookup | 8,15 (7,99–8,49) µs | 8,31 (8,21–8,86) µs | +2,0% | nie (w rozrzucie) |
| index_lookup | 5,90 (5,89–5,96) µs | 7,82 (7,57–8,74) µs | +32,5% | tak, wolniej/więcej |
| range_scan | 54,3 (53,2–56,9) µs | 59,9 (55,4–63,0) µs | +10,4% | tak, wolniej/więcej |
| full_scan | 5,16 (5,11–5,19) ms | 5,18 (5,12–5,28) ms | +0,3% | nie (w rozrzucie) |
| group_by | 17,3 (16,4–18,1) ms | 17,4 (17,1–18,3) ms | +0,4% | nie (w rozrzucie) |
| top_k | 5,83 (5,58–6,06) ms | 5,76 (5,58–5,80) ms | -1,2% | nie (w rozrzucie) |
| materialize_10k | 10,00 (9,88–10,1) ms | 10,7 (10,5–11,5) ms | +6,7% | tak, wolniej/więcej |
| update_10pct | 39,4 (33,4–46,3) ms | 35,7 (26,8–40,8) ms | -9,3% | nie (w rozrzucie) |
| delete_10pct | 31,8 (30,6–33,6) ms | 25,2 (23,6–26,5) ms | -20,6% | tak, szybciej/mniej |
| snapshot | 38,1 (37,9–39,4) ms | 31,2 (30,2–31,5) ms | -18,1% | tak, szybciej/mniej |
| snapshot (rozmiar) | 5,03 MiB | 3,71 MiB | -26,2% | tak, szybciej/mniej |
| restore | 123 (122–125) ms | 65,3 (64,2–67,3) ms | -47,0% | tak, szybciej/mniej |

Wyniki zgodne w obu wersjach (160 porównań sum kontrolnych i stanu tabeli).

### node – workload – flatsql-json

| Metryka | Przed | Po | Zmiana mediany | 95% CI średnich rozłączne |
|---|---:|---:|---:|---|
| insert | 400 (386–413) ms | 297 (293–302) ms | -25,8% | tak, szybciej/mniej |
| point_lookup | 4,92 (4,78–5,02) µs | 4,46 (4,34–4,75) µs | -9,2% | nie (w rozrzucie) |
| index_lookup | 3,17 (3,04–4,01) µs | 3,25 (3,05–3,57) µs | +2,3% | nie (w rozrzucie) |
| range_scan | 54,0 (53,5–58,8) µs | 57,5 (54,8–65,7) µs | +6,6% | nie (w rozrzucie) |
| full_scan | 5,12 (5,09–5,16) ms | 5,18 (5,15–5,24) ms | +1,1% | nie (w rozrzucie) |
| group_by | 16,3 (16,1–17,0) ms | 17,1 (16,9–17,7) ms | +5,1% | nie (w rozrzucie) |
| top_k | 5,88 (5,78–6,24) ms | 5,56 (5,49–5,72) ms | -5,5% | nie (w rozrzucie) |
| materialize_10k | 9,84 (9,73–9,96) ms | 10,8 (10,7–11,1) ms | +9,3% | nie (w rozrzucie) |
| update_10pct | 34,5 (33,1–40,8) ms | 30,3 (26,3–35,9) ms | -12,1% | nie (w rozrzucie) |
| delete_10pct | 31,9 (30,6–33,6) ms | 25,2 (22,9–25,9) ms | -20,9% | tak, szybciej/mniej |
| snapshot | 37,5 (35,2–38,8) ms | 31,0 (29,6–31,5) ms | -17,4% | tak, szybciej/mniej |
| snapshot (rozmiar) | 5,03 MiB | 3,71 MiB | -26,2% | tak, szybciej/mniej |
| restore | 124 (122–127) ms | 64,2 (62,2–65,2) ms | -48,1% | tak, szybciej/mniej |

Wyniki zgodne w obu wersjach (160 porównań sum kontrolnych i stanu tabeli).

