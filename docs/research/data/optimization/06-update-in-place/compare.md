Przed: po-2.11 (core: keep the outbox only once synchronisation is enabled, compact outbox in snapshots); po: po-2.11-dirty (core: keep the outbox only once synchronisation is enabled, compact outbox in snapshots).
Obciążenie procesora przed pomiarem: 12,1%. Uruchomienia naprzemienne, 10 na wersję.
Moduł WebAssembly: 643472 → 655948 B, gzip -9: 242780 → 247941 B (+5161 B).

### native – workload – flatsql-native

| Metryka | Przed | Po | Zmiana mediany | 95% CI średnich rozłączne |
|---|---:|---:|---:|---|
| insert | 160 (158–161) ms | 162 (161–164) ms | +1,4% | nie (w rozrzucie) |
| point_lookup | 1,20 (1,19–1,23) µs | 1,25 (1,22–1,27) µs | +4,0% | nie (w rozrzucie) |
| index_lookup | 0,74 (0,73–0,76) µs | 0,75 (0,74–0,75) µs | +0,8% | nie (w rozrzucie) |
| range_scan | 50,0 (48,7–52,0) µs | 48,7 (47,9–49,5) µs | -2,7% | nie (w rozrzucie) |
| full_scan | 5,44 (5,43–5,50) ms | 5,60 (5,51–5,64) ms | +3,0% | tak, wolniej/więcej |
| group_by | 15,9 (15,8–16,0) ms | 15,8 (15,6–16,0) ms | -0,5% | nie (w rozrzucie) |
| top_k | 6,00 (5,96–6,04) ms | 6,00 (5,95–6,09) ms | -0,1% | nie (w rozrzucie) |
| materialize_10k | 3,80 (3,77–3,83) ms | 3,61 (3,56–3,71) ms | -5,0% | tak, szybciej/mniej |
| update_10pct | 23,1 (22,8–23,4) ms | 15,7 (15,2–16,1) ms | -32,1% | tak, szybciej/mniej |
| delete_10pct | 16,6 (16,2–17,1) ms | 15,9 (15,8–16,2) ms | -4,0% | nie (w rozrzucie) |
| snapshot | 18,3 (18,1–18,8) ms | 18,1 (17,6–19,1) ms | -0,9% | nie (w rozrzucie) |
| snapshot (rozmiar) | 3,71 MiB | 3,71 MiB | +0,0% | nie (w rozrzucie) |
| restore | 54,9 (54,6–55,6) ms | 54,5 (54,3–54,6) ms | -0,9% | nie (w rozrzucie) |

Wyniki zgodne w obu wersjach (140 porównań sum kontrolnych i stanu tabeli).

### node – workload – flatsql

| Metryka | Przed | Po | Zmiana mediany | 95% CI średnich rozłączne |
|---|---:|---:|---:|---|
| insert | 299 (297–305) ms | 298 (296–300) ms | -0,2% | nie (w rozrzucie) |
| point_lookup | 8,33 (8,20–8,58) µs | 8,15 (8,10–8,31) µs | -2,1% | nie (w rozrzucie) |
| index_lookup | 7,48 (7,01–9,14) µs | 7,69 (7,52–10,8) µs | +2,7% | nie (w rozrzucie) |
| range_scan | 56,6 (55,3–58,6) µs | 57,3 (56,0–58,1) µs | +1,2% | nie (w rozrzucie) |
| full_scan | 5,24 (5,20–5,26) ms | 5,21 (5,17–5,30) ms | -0,5% | nie (w rozrzucie) |
| group_by | 17,0 (16,8–17,6) ms | 17,4 (17,4–18,2) ms | +2,6% | nie (w rozrzucie) |
| top_k | 5,75 (5,68–6,93) ms | 5,66 (5,60–5,81) ms | -1,5% | nie (w rozrzucie) |
| materialize_10k | 10,6 (10,5–10,9) ms | 10,5 (10,4–11,0) ms | -0,5% | nie (w rozrzucie) |
| update_10pct | 33,8 (28,1–38,8) ms | 17,5 (17,5–17,7) ms | -48,2% | tak, szybciej/mniej |
| delete_10pct | 25,2 (23,5–26,9) ms | 22,4 (21,7–22,8) ms | -11,0% | tak, szybciej/mniej |
| snapshot | 30,8 (30,2–31,8) ms | 27,7 (27,0–29,3) ms | -9,8% | nie (w rozrzucie) |
| snapshot (rozmiar) | 3,71 MiB | 3,71 MiB | +0,0% | nie (w rozrzucie) |
| restore | 65,7 (64,8–66,6) ms | 64,0 (63,0–66,1) ms | -2,6% | nie (w rozrzucie) |

Wyniki zgodne w obu wersjach (160 porównań sum kontrolnych i stanu tabeli).

### node – workload – flatsql-json

| Metryka | Przed | Po | Zmiana mediany | 95% CI średnich rozłączne |
|---|---:|---:|---:|---|
| insert | 296 (293–301) ms | 297 (296–299) ms | +0,6% | nie (w rozrzucie) |
| point_lookup | 4,63 (4,46–5,28) µs | 4,54 (4,47–4,87) µs | -2,0% | nie (w rozrzucie) |
| index_lookup | 3,13 (3,05–3,44) µs | 3,63 (3,08–4,09) µs | +16,1% | nie (w rozrzucie) |
| range_scan | 62,1 (53,5–69,4) µs | 56,0 (54,4–61,0) µs | -9,8% | nie (w rozrzucie) |
| full_scan | 5,14 (5,11–5,22) ms | 5,39 (5,20–5,51) ms | +4,8% | nie (w rozrzucie) |
| group_by | 17,0 (16,8–17,5) ms | 17,6 (17,1–18,5) ms | +4,1% | nie (w rozrzucie) |
| top_k | 5,57 (5,52–5,63) ms | 5,53 (5,52–5,76) ms | -0,7% | nie (w rozrzucie) |
| materialize_10k | 10,9 (10,7–11,0) ms | 10,8 (10,7–11,1) ms | -0,4% | nie (w rozrzucie) |
| update_10pct | 28,1 (26,4–35,5) ms | 17,4 (17,2–17,7) ms | -38,0% | tak, szybciej/mniej |
| delete_10pct | 23,8 (22,8–25,8) ms | 21,7 (21,2–22,7) ms | -8,8% | nie (w rozrzucie) |
| snapshot | 31,4 (30,4–31,7) ms | 30,2 (27,2–31,0) ms | -3,8% | nie (w rozrzucie) |
| snapshot (rozmiar) | 3,71 MiB | 3,71 MiB | +0,0% | nie (w rozrzucie) |
| restore | 63,4 (62,8–65,0) ms | 63,4 (62,5–65,2) ms | -0,0% | nie (w rozrzucie) |

Wyniki zgodne w obu wersjach (160 porównań sum kontrolnych i stanu tabeli).

