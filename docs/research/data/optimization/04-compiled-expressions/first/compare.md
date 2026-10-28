Przed: po-2.9 (core: compile statements once and keep the plan in prepared statements); po: po-2.9-dirty (core: compile statements once and keep the plan in prepared statements).
Obciążenie procesora przed pomiarem: 7,4%. Uruchomienia naprzemienne, 10 na wersję.
Moduł WebAssembly: 608861 → 628161 B, gzip -9: 232563 → 238385 B (+5822 B).

### native – workload – flatsql-native

| Metryka | Przed | Po | Zmiana mediany | 95% CI średnich rozłączne |
|---|---:|---:|---:|---|
| insert | 207 (206–207) ms | 209 (207–212) ms | +1,1% | nie (w rozrzucie) |
| point_lookup | 1,19 (1,18–1,20) µs | 1,22 (1,18–1,31) µs | +2,3% | nie (w rozrzucie) |
| index_lookup | 0,70 (0,70–0,72) µs | 0,70 (0,69–0,72) µs | +0,4% | nie (w rozrzucie) |
| range_scan | 51,5 (50,1–53,9) µs | 48,6 (48,2–49,6) µs | -5,6% | nie (w rozrzucie) |
| full_scan | 7,41 (7,35–7,49) ms | 5,38 (5,34–5,43) ms | -27,4% | tak, szybciej/mniej |
| group_by | 15,3 (15,2–15,3) ms | 15,6 (15,4–15,8) ms | +1,9% | tak, wolniej/więcej |
| top_k | 5,77 (5,76–5,78) ms | 6,00 (5,98–6,24) ms | +4,1% | tak, wolniej/więcej |
| materialize_10k | 3,68 (3,68–3,71) ms | 3,62 (3,61–3,70) ms | -1,6% | nie (w rozrzucie) |
| update_10pct | 32,2 (31,5–32,6) ms | 30,7 (30,1–31,1) ms | -4,6% | nie (w rozrzucie) |
| delete_10pct | 25,5 (25,0–26,4) ms | 24,3 (23,7–24,9) ms | -4,8% | tak, szybciej/mniej |
| snapshot | 26,0 (25,7–26,2) ms | 25,3 (25,3–25,8) ms | -2,6% | nie (w rozrzucie) |
| snapshot (rozmiar) | 5,03 MiB | 5,03 MiB | +0,0% | nie (w rozrzucie) |
| restore | 111 (111–112) ms | 111 (110–113) ms | -0,2% | nie (w rozrzucie) |

Wyniki zgodne w obu wersjach (140 porównań sum kontrolnych i stanu tabeli).

### node – workload – flatsql

| Metryka | Przed | Po | Zmiana mediany | 95% CI średnich rozłączne |
|---|---:|---:|---:|---|
| insert | 380 (377–392) ms | 381 (379–383) ms | +0,3% | nie (w rozrzucie) |
| point_lookup | 8,52 (8,42–8,64) µs | 8,44 (8,31–8,67) µs | -1,0% | nie (w rozrzucie) |
| index_lookup | 5,92 (5,90–6,06) µs | 6,72 (6,29–7,44) µs | +13,6% | nie (w rozrzucie) |
| range_scan | 57,3 (55,6–58,4) µs | 57,0 (53,7–66,1) µs | -0,5% | nie (w rozrzucie) |
| full_scan | 8,30 (8,27–8,42) ms | 5,20 (5,18–5,25) ms | -37,3% | tak, szybciej/mniej |
| group_by | 16,8 (16,5–17,0) ms | 17,6 (16,4–18,1) ms | +4,8% | nie (w rozrzucie) |
| top_k | 5,77 (5,63–5,96) ms | 5,63 (5,52–5,91) ms | -2,5% | nie (w rozrzucie) |
| materialize_10k | 10,0 (9,93–10,3) ms | 10,1 (9,93–10,3) ms | +0,8% | nie (w rozrzucie) |
| update_10pct | 42,6 (37,2–49,9) ms | 36,4 (32,9–42,0) ms | -14,6% | nie (w rozrzucie) |
| delete_10pct | 33,6 (31,0–35,6) ms | 33,1 (31,0–35,9) ms | -1,4% | nie (w rozrzucie) |
| snapshot | 37,4 (36,8–37,9) ms | 37,8 (37,2–38,9) ms | +1,0% | nie (w rozrzucie) |
| snapshot (rozmiar) | 5,03 MiB | 5,03 MiB | +0,0% | nie (w rozrzucie) |
| restore | 122 (122–123) ms | 122 (120–123) ms | -0,6% | nie (w rozrzucie) |

Wyniki zgodne w obu wersjach (160 porównań sum kontrolnych i stanu tabeli).

### node – workload – flatsql-json

| Metryka | Przed | Po | Zmiana mediany | 95% CI średnich rozłączne |
|---|---:|---:|---:|---|
| insert | 385 (376–396) ms | 385 (376–392) ms | -0,1% | nie (w rozrzucie) |
| point_lookup | 4,96 (4,94–4,97) µs | 4,84 (4,71–4,92) µs | -2,4% | nie (w rozrzucie) |
| index_lookup | 4,04 (3,53–4,10) µs | 3,10 (3,01–3,39) µs | -23,3% | nie (w rozrzucie) |
| range_scan | 55,4 (54,7–56,3) µs | 56,7 (52,1–58,5) µs | +2,3% | nie (w rozrzucie) |
| full_scan | 8,35 (8,24–8,49) ms | 5,18 (5,11–5,20) ms | -38,0% | tak, szybciej/mniej |
| group_by | 16,4 (16,2–16,8) ms | 16,3 (16,1–16,8) ms | -0,8% | nie (w rozrzucie) |
| top_k | 5,93 (5,76–5,99) ms | 5,90 (5,66–6,30) ms | -0,4% | nie (w rozrzucie) |
| materialize_10k | 10,2 (9,87–10,8) ms | 9,97 (9,84–10,0) ms | -1,9% | nie (w rozrzucie) |
| update_10pct | 37,7 (37,4–45,6) ms | 40,4 (34,6–43,4) ms | +7,1% | nie (w rozrzucie) |
| delete_10pct | 31,6 (30,8–34,3) ms | 32,8 (30,9–34,3) ms | +3,7% | nie (w rozrzucie) |
| snapshot | 37,9 (37,1–38,2) ms | 38,6 (37,1–39,3) ms | +2,0% | nie (w rozrzucie) |
| snapshot (rozmiar) | 5,03 MiB | 5,03 MiB | +0,0% | nie (w rozrzucie) |
| restore | 125 (123–127) ms | 123 (121–124) ms | -2,1% | nie (w rozrzucie) |

Wyniki zgodne w obu wersjach (160 porównań sum kontrolnych i stanu tabeli).

