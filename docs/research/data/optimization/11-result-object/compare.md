Przed: po-2.3 (docs: profile of a primary key lookup at the JavaScript and WebAssembly boundary); po: po-2.3-dirty (docs: profile of a primary key lookup at the JavaScript and WebAssembly boundary).
Obciążenie procesora przed pomiarem: 20,7%. Uruchomienia naprzemienne, 10 na wersję.
Moduł WebAssembly: 690050 → 690258 B, gzip -9: 259478 → 259766 B (+288 B).

### native – workload – flatsql-native

| Metryka | Przed | Po | Zmiana mediany | 95% CI średnich rozłączne |
|---|---:|---:|---:|---|
| insert | 163 (163–164) ms | 163 (162–165) ms | +0,2% | nie (w rozrzucie) |
| point_lookup | 1,22 (1,19–1,26) µs | 1,21 (1,19–1,29) µs | -0,7% | nie (w rozrzucie) |
| index_lookup | 0,75 (0,75–0,75) µs | 0,76 (0,75–0,77) µs | +1,1% | nie (w rozrzucie) |
| range_scan | 50,0 (48,7–52,7) µs | 48,3 (47,9–49,2) µs | -3,2% | nie (w rozrzucie) |
| full_scan | 5,48 (5,43–5,66) ms | 5,56 (5,53–5,60) ms | +1,5% | nie (w rozrzucie) |
| group_by | 10,4 (10,3–10,4) ms | 10,3 (10,3–10,5) ms | -0,4% | nie (w rozrzucie) |
| top_k | 6,04 (6,02–6,08) ms | 6,01 (5,94–6,12) ms | -0,6% | nie (w rozrzucie) |
| materialize_10k | 3,84 (3,80–3,91) ms | 3,78 (3,74–3,83) ms | -1,4% | nie (w rozrzucie) |
| update_10pct | 15,6 (15,3–15,8) ms | 16,1 (15,7–16,6) ms | +3,3% | nie (w rozrzucie) |
| delete_10pct | 17,3 (16,6–17,9) ms | 17,0 (16,3–17,6) ms | -1,5% | nie (w rozrzucie) |
| snapshot | 18,1 (18,0–18,3) ms | 17,9 (17,8–18,4) ms | -1,2% | nie (w rozrzucie) |
| snapshot (rozmiar) | 3,71 MiB | 3,71 MiB | +0,0% | nie (w rozrzucie) |
| restore | 45,0 (44,4–45,7) ms | 45,1 (44,8–46,0) ms | +0,4% | nie (w rozrzucie) |

Wyniki zgodne w obu wersjach (140 porównań sum kontrolnych i stanu tabeli).

### node – workload – flatsql

| Metryka | Przed | Po | Zmiana mediany | 95% CI średnich rozłączne |
|---|---:|---:|---:|---|
| insert | 299 (295–305) ms | 298 (294–302) ms | -0,5% | nie (w rozrzucie) |
| point_lookup | 8,20 (8,09–8,42) µs | 3,04 (2,96–3,07) µs | -62,9% | tak, szybciej/mniej |
| index_lookup | 7,41 (7,18–7,51) µs | 1,95 (1,90–2,11) µs | -73,7% | tak, szybciej/mniej |
| range_scan | 57,6 (55,9–59,3) µs | 52,9 (52,5–54,2) µs | -8,3% | nie (w rozrzucie) |
| full_scan | 5,19 (5,16–5,26) ms | 5,23 (5,11–5,30) ms | +0,9% | nie (w rozrzucie) |
| group_by | 11,0 (10,9–11,4) ms | 11,5 (11,4–11,5) ms | +4,2% | nie (w rozrzucie) |
| top_k | 5,67 (5,55–5,73) ms | 5,57 (5,52–5,68) ms | -1,7% | nie (w rozrzucie) |
| materialize_10k | 10,5 (10,4–10,5) ms | 10,5 (10,3–11,0) ms | +0,3% | nie (w rozrzucie) |
| update_10pct | 17,4 (17,3–17,5) ms | 17,4 (17,2–17,8) ms | -0,1% | nie (w rozrzucie) |
| delete_10pct | 22,1 (21,3–23,0) ms | 22,0 (21,8–22,3) ms | -0,5% | nie (w rozrzucie) |
| snapshot | 30,0 (28,9–30,8) ms | 27,8 (26,9–30,1) ms | -7,4% | nie (w rozrzucie) |
| snapshot (rozmiar) | 3,71 MiB | 3,71 MiB | +0,0% | nie (w rozrzucie) |
| restore | 59,0 (58,0–60,5) ms | 59,3 (56,6–59,6) ms | +0,4% | nie (w rozrzucie) |

Wyniki zgodne w obu wersjach (160 porównań sum kontrolnych i stanu tabeli).

### node – workload – flatsql-json

| Metryka | Przed | Po | Zmiana mediany | 95% CI średnich rozłączne |
|---|---:|---:|---:|---|
| insert | 297 (296–297) ms | 295 (293–301) ms | -0,5% | nie (w rozrzucie) |
| point_lookup | 4,76 (4,40–5,42) µs | 4,46 (4,42–4,60) µs | -6,3% | nie (w rozrzucie) |
| index_lookup | 3,15 (3,00–3,71) µs | 3,21 (3,08–3,42) µs | +1,8% | nie (w rozrzucie) |
| range_scan | 54,7 (52,8–56,9) µs | 53,0 (52,2–55,0) µs | -3,1% | nie (w rozrzucie) |
| full_scan | 5,20 (5,15–5,31) ms | 5,18 (5,08–5,35) ms | -0,5% | nie (w rozrzucie) |
| group_by | 11,8 (11,5–12,2) ms | 12,6 (12,4–12,8) ms | +6,3% | tak, wolniej/więcej |
| top_k | 5,62 (5,50–6,04) ms | 5,52 (5,48–5,62) ms | -1,9% | nie (w rozrzucie) |
| materialize_10k | 11,0 (10,8–11,1) ms | 10,9 (10,8–11,1) ms | -0,3% | nie (w rozrzucie) |
| update_10pct | 17,0 (16,9–17,3) ms | 17,0 (16,8–17,3) ms | +0,1% | nie (w rozrzucie) |
| delete_10pct | 22,3 (21,8–22,5) ms | 21,5 (21,0–22,6) ms | -3,5% | nie (w rozrzucie) |
| snapshot | 27,3 (26,7–29,8) ms | 30,6 (30,2–31,3) ms | +12,2% | tak, wolniej/więcej |
| snapshot (rozmiar) | 3,71 MiB | 3,71 MiB | +0,0% | nie (w rozrzucie) |
| restore | 60,0 (58,6–60,4) ms | 59,6 (59,4–60,1) ms | -0,7% | nie (w rozrzucie) |

Wyniki zgodne w obu wersjach (160 porównań sum kontrolnych i stanu tabeli).

