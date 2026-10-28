Przed: po-2.9 (core: compile statements once and keep the plan in prepared statements); po: po-2.9-dirty (core: compile statements once and keep the plan in prepared statements).
Obciążenie procesora przed pomiarem: 11,2%. Uruchomienia naprzemienne, 10 na wersję.
Moduł WebAssembly: 608861 → 628161 B, gzip -9: 232563 → 238385 B (+5822 B).

### native – workload – flatsql-native

| Metryka | Przed | Po | Zmiana mediany | 95% CI średnich rozłączne |
|---|---:|---:|---:|---|
| insert | 206 (205–207) ms | 210 (207–213) ms | +2,1% | nie (w rozrzucie) |
| point_lookup | 1,19 (1,17–1,23) µs | 1,19 (1,18–1,24) µs | +0,4% | nie (w rozrzucie) |
| index_lookup | 0,69 (0,69–0,70) µs | 0,75 (0,72–0,76) µs | +8,5% | nie (w rozrzucie) |
| range_scan | 48,4 (47,6–49,7) µs | 50,8 (48,7–51,4) µs | +4,9% | nie (w rozrzucie) |
| full_scan | 7,58 (7,56–7,66) ms | 5,52 (5,37–5,61) ms | -27,2% | tak, szybciej/mniej |
| group_by | 15,3 (15,3–15,4) ms | 15,7 (15,6–15,9) ms | +2,3% | nie (w rozrzucie) |
| top_k | 5,81 (5,75–5,83) ms | 5,95 (5,90–6,02) ms | +2,3% | nie (w rozrzucie) |
| materialize_10k | 3,78 (3,71–3,83) ms | 3,63 (3,62–3,75) ms | -4,0% | nie (w rozrzucie) |
| update_10pct | 32,2 (31,9–33,6) ms | 30,5 (30,2–31,4) ms | -5,4% | tak, szybciej/mniej |
| delete_10pct | 25,8 (25,3–26,2) ms | 23,8 (23,6–24,1) ms | -7,8% | tak, szybciej/mniej |
| snapshot | 26,2 (25,6–26,5) ms | 25,5 (25,2–25,9) ms | -2,7% | nie (w rozrzucie) |
| snapshot (rozmiar) | 5,03 MiB | 5,03 MiB | +0,0% | nie (w rozrzucie) |
| restore | 111 (110–112) ms | 112 (112–114) ms | +1,3% | nie (w rozrzucie) |

Wyniki zgodne w obu wersjach (140 porównań sum kontrolnych i stanu tabeli).

### node – workload – flatsql

| Metryka | Przed | Po | Zmiana mediany | 95% CI średnich rozłączne |
|---|---:|---:|---:|---|
| insert | 383 (380–391) ms | 374 (372–379) ms | -2,4% | nie (w rozrzucie) |
| point_lookup | 8,82 (8,66–9,06) µs | 8,14 (8,06–8,41) µs | -7,8% | nie (w rozrzucie) |
| index_lookup | 6,84 (6,18–7,45) µs | 6,25 (5,96–8,41) µs | -8,6% | nie (w rozrzucie) |
| range_scan | 57,2 (55,7–60,4) µs | 53,5 (52,4–58,2) µs | -6,5% | nie (w rozrzucie) |
| full_scan | 8,46 (8,37–8,56) ms | 5,38 (5,13–5,62) ms | -36,5% | tak, szybciej/mniej |
| group_by | 16,1 (16,0–17,1) ms | 17,0 (16,7–17,8) ms | +5,4% | nie (w rozrzucie) |
| top_k | 5,65 (5,56–6,13) ms | 5,78 (5,61–5,93) ms | +2,3% | nie (w rozrzucie) |
| materialize_10k | 9,93 (9,78–10,0) ms | 10,0 (9,83–10,7) ms | +1,0% | nie (w rozrzucie) |
| update_10pct | 38,2 (37,3–46,5) ms | 43,4 (35,8–47,3) ms | +13,7% | nie (w rozrzucie) |
| delete_10pct | 32,1 (31,1–33,7) ms | 32,3 (31,4–34,1) ms | +0,7% | nie (w rozrzucie) |
| snapshot | 37,8 (36,2–38,7) ms | 37,5 (37,1–38,1) ms | -0,8% | nie (w rozrzucie) |
| snapshot (rozmiar) | 5,03 MiB | 5,03 MiB | +0,0% | nie (w rozrzucie) |
| restore | 122 (120–132) ms | 123 (122–125) ms | +1,6% | nie (w rozrzucie) |

Wyniki zgodne w obu wersjach (160 porównań sum kontrolnych i stanu tabeli).

### node – workload – flatsql-json

| Metryka | Przed | Po | Zmiana mediany | 95% CI średnich rozłączne |
|---|---:|---:|---:|---|
| insert | 380 (374–384) ms | 375 (373–378) ms | -1,2% | nie (w rozrzucie) |
| point_lookup | 4,96 (4,78–5,02) µs | 4,83 (4,78–4,87) µs | -2,6% | nie (w rozrzucie) |
| index_lookup | 3,36 (3,09–3,78) µs | 3,44 (3,05–4,13) µs | +2,2% | nie (w rozrzucie) |
| range_scan | 56,7 (55,2–61,0) µs | 56,9 (55,9–58,1) µs | +0,4% | nie (w rozrzucie) |
| full_scan | 8,32 (8,21–8,33) ms | 5,23 (5,13–5,33) ms | -37,1% | tak, szybciej/mniej |
| group_by | 16,6 (16,3–17,4) ms | 17,1 (16,3–17,4) ms | +3,0% | nie (w rozrzucie) |
| top_k | 5,89 (5,72–5,98) ms | 5,92 (5,81–6,51) ms | +0,4% | nie (w rozrzucie) |
| materialize_10k | 9,84 (9,65–10,0) ms | 9,87 (9,77–9,99) ms | +0,3% | nie (w rozrzucie) |
| update_10pct | 46,8 (44,3–47,7) ms | 34,6 (32,3–41,4) ms | -26,0% | tak, szybciej/mniej |
| delete_10pct | 31,5 (31,0–32,5) ms | 30,7 (30,6–32,5) ms | -2,6% | nie (w rozrzucie) |
| snapshot | 38,0 (36,8–38,6) ms | 37,3 (36,9–38,8) ms | -1,7% | nie (w rozrzucie) |
| snapshot (rozmiar) | 5,03 MiB | 5,03 MiB | +0,0% | nie (w rozrzucie) |
| restore | 122 (120–123) ms | 123 (121–127) ms | +0,9% | nie (w rozrzucie) |

Wyniki zgodne w obu wersjach (160 porównań sum kontrolnych i stanu tabeli).

