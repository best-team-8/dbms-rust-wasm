Przed: po-2.8 (core: update rows in place, touch only changed indexes, read the clock once per statement); po: po-2.8-dirty (core: update rows in place, touch only changed indexes, read the clock once per statement).
Obciążenie procesora przed pomiarem: 20,0%. Uruchomienia naprzemienne, 10 na wersję.
Moduł WebAssembly: 655868 → 684172 B, gzip -9: 247915 → 257469 B (+9554 B).

### native – workload – flatsql-native

| Metryka | Przed | Po | Zmiana mediany | 95% CI średnich rozłączne |
|---|---:|---:|---:|---|
| insert | 163 (162–164) ms | 163 (161–167) ms | +0,2% | nie (w rozrzucie) |
| point_lookup | 1,22 (1,20–1,32) µs | 1,22 (1,20–1,27) µs | +0,2% | nie (w rozrzucie) |
| index_lookup | 0,74 (0,73–0,75) µs | 0,74 (0,73–0,75) µs | -0,1% | nie (w rozrzucie) |
| range_scan | 50,3 (49,7–52,2) µs | 48,8 (48,5–50,4) µs | -3,1% | nie (w rozrzucie) |
| full_scan | 5,64 (5,56–5,72) ms | 5,43 (5,43–5,46) ms | -3,6% | tak, szybciej/mniej |
| group_by | 15,8 (15,7–15,9) ms | 15,7 (15,6–15,8) ms | -0,6% | nie (w rozrzucie) |
| top_k | 6,01 (5,96–6,15) ms | 6,03 (5,99–6,10) ms | +0,4% | nie (w rozrzucie) |
| materialize_10k | 3,62 (3,54–3,66) ms | 3,53 (3,48–3,54) ms | -2,6% | nie (w rozrzucie) |
| update_10pct | 15,5 (15,3–15,8) ms | 15,6 (15,4–15,8) ms | +0,6% | nie (w rozrzucie) |
| delete_10pct | 16,4 (15,9–16,7) ms | 16,1 (15,9–16,6) ms | -2,2% | nie (w rozrzucie) |
| snapshot | 18,0 (17,7–18,7) ms | 17,8 (17,6–18,0) ms | -1,3% | nie (w rozrzucie) |
| snapshot (rozmiar) | 3,71 MiB | 3,71 MiB | +0,0% | nie (w rozrzucie) |
| restore | 53,9 (53,7–54,9) ms | 46,2 (45,9–47,7) ms | -14,3% | tak, szybciej/mniej |

Wyniki zgodne w obu wersjach (140 porównań sum kontrolnych i stanu tabeli).

### node – workload – flatsql

| Metryka | Przed | Po | Zmiana mediany | 95% CI średnich rozłączne |
|---|---:|---:|---:|---|
| insert | 295 (294–306) ms | 300 (296–302) ms | +1,9% | nie (w rozrzucie) |
| point_lookup | 8,34 (8,23–8,73) µs | 8,27 (8,12–8,39) µs | -0,9% | nie (w rozrzucie) |
| index_lookup | 8,02 (7,30–9,35) µs | 9,83 (8,34–11,3) µs | +22,6% | nie (w rozrzucie) |
| range_scan | 59,7 (56,6–61,4) µs | 57,4 (56,4–60,7) µs | -3,9% | nie (w rozrzucie) |
| full_scan | 5,27 (5,26–5,32) ms | 5,19 (5,14–5,33) ms | -1,4% | nie (w rozrzucie) |
| group_by | 18,8 (18,3–19,2) ms | 17,9 (17,3–19,0) ms | -4,7% | nie (w rozrzucie) |
| top_k | 5,93 (5,64–6,53) ms | 5,63 (5,61–5,82) ms | -5,0% | nie (w rozrzucie) |
| materialize_10k | 10,8 (10,6–11,2) ms | 10,7 (10,5–10,8) ms | -1,1% | nie (w rozrzucie) |
| update_10pct | 17,4 (17,0–17,8) ms | 17,7 (17,1–18,7) ms | +1,5% | nie (w rozrzucie) |
| delete_10pct | 22,1 (21,2–22,8) ms | 22,0 (21,4–23,8) ms | -0,4% | nie (w rozrzucie) |
| snapshot | 30,8 (27,4–31,8) ms | 30,8 (27,0–31,2) ms | -0,1% | nie (w rozrzucie) |
| snapshot (rozmiar) | 3,71 MiB | 3,71 MiB | +0,0% | nie (w rozrzucie) |
| restore | 63,7 (62,5–66,3) ms | 61,2 (59,3–63,2) ms | -4,0% | nie (w rozrzucie) |

Wyniki zgodne w obu wersjach (160 porównań sum kontrolnych i stanu tabeli).

### node – workload – flatsql-json

| Metryka | Przed | Po | Zmiana mediany | 95% CI średnich rozłączne |
|---|---:|---:|---:|---|
| insert | 295 (294–298) ms | 298 (294–303) ms | +0,9% | nie (w rozrzucie) |
| point_lookup | 4,60 (4,51–4,97) µs | 4,87 (4,50–4,94) µs | +5,8% | nie (w rozrzucie) |
| index_lookup | 3,21 (2,99–3,68) µs | 3,89 (3,17–4,17) µs | +21,2% | nie (w rozrzucie) |
| range_scan | 54,2 (53,1–55,1) µs | 54,0 (53,1–56,6) µs | -0,3% | nie (w rozrzucie) |
| full_scan | 5,46 (5,20–5,57) ms | 5,17 (5,15–5,25) ms | -5,2% | nie (w rozrzucie) |
| group_by | 17,3 (17,2–17,8) ms | 18,1 (17,8–18,3) ms | +4,3% | nie (w rozrzucie) |
| top_k | 5,78 (5,64–5,99) ms | 5,60 (5,56–5,76) ms | -3,0% | nie (w rozrzucie) |
| materialize_10k | 10,9 (10,7–11,7) ms | 10,9 (10,8–10,9) ms | -0,3% | nie (w rozrzucie) |
| update_10pct | 17,8 (17,0–18,6) ms | 17,0 (16,9–17,3) ms | -4,5% | nie (w rozrzucie) |
| delete_10pct | 22,4 (22,0–22,8) ms | 22,0 (21,5–22,2) ms | -2,0% | nie (w rozrzucie) |
| snapshot | 27,6 (27,2–29,9) ms | 27,7 (26,4–30,1) ms | +0,1% | nie (w rozrzucie) |
| snapshot (rozmiar) | 3,71 MiB | 3,71 MiB | +0,0% | nie (w rozrzucie) |
| restore | 66,8 (63,3–73,8) ms | 59,1 (58,4–59,8) ms | -11,5% | tak, szybciej/mniej |

Wyniki zgodne w obu wersjach (160 porównań sum kontrolnych i stanu tabeli).

