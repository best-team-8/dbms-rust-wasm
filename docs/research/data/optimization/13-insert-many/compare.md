Przed: po-2.4 (core/wasm: encode results straight from the table in a binary format decoded by JavaScript); po: po-2.4-dirty (core/wasm: encode results straight from the table in a binary format decoded by JavaScript).
Obciążenie procesora przed pomiarem: 8,8%. Uruchomienia naprzemienne, 10 na wersję.
Moduł WebAssembly: 695364 → 700875 B, gzip -9: 262201 → 264753 B (+2552 B).

### native – workload – flatsql-native

| Metryka | Przed | Po | Zmiana mediany | 95% CI średnich rozłączne |
|---|---:|---:|---:|---|
| insert | 163 (162–165) ms | 163 (163–166) ms | -0,0% | nie (w rozrzucie) |
| point_lookup | 1,22 (1,20–1,23) µs | 1,18 (1,17–1,21) µs | -2,9% | nie (w rozrzucie) |
| index_lookup | 0,77 (0,77–0,80) µs | 0,77 (0,76–0,78) µs | -0,3% | nie (w rozrzucie) |
| range_scan | 49,2 (48,3–52,8) µs | 50,0 (48,7–50,4) µs | +1,5% | nie (w rozrzucie) |
| full_scan | 5,50 (5,46–5,54) ms | 5,53 (5,45–5,64) ms | +0,6% | nie (w rozrzucie) |
| group_by | 10,5 (10,3–10,7) ms | 10,3 (10,2–10,4) ms | -1,8% | nie (w rozrzucie) |
| top_k | 5,89 (5,83–6,02) ms | 5,97 (5,91–6,04) ms | +1,3% | nie (w rozrzucie) |
| materialize_10k | 3,73 (3,70–3,79) ms | 3,50 (3,47–3,60) ms | -6,1% | tak, szybciej/mniej |
| update_10pct | 15,5 (15,4–15,7) ms | 15,6 (15,3–16,0) ms | +0,2% | nie (w rozrzucie) |
| delete_10pct | 15,9 (15,8–16,8) ms | 16,1 (16,0–16,6) ms | +1,5% | nie (w rozrzucie) |
| snapshot | 18,2 (17,8–19,1) ms | 17,9 (17,6–18,2) ms | -1,8% | nie (w rozrzucie) |
| snapshot (rozmiar) | 3,71 MiB | 3,71 MiB | +0,0% | nie (w rozrzucie) |
| restore | 45,5 (44,4–46,7) ms | 46,0 (44,9–46,7) ms | +1,1% | nie (w rozrzucie) |
| insert_many | – | 145 (144–149) ms | – | – |

Wyniki zgodne w obu wersjach (140 porównań sum kontrolnych i stanu tabeli).

### node – workload – flatsql

| Metryka | Przed | Po | Zmiana mediany | 95% CI średnich rozłączne |
|---|---:|---:|---:|---|
| insert | 298 (297–302) ms | 302 (298–303) ms | +1,1% | nie (w rozrzucie) |
| point_lookup | 2,65 (2,57–2,72) µs | 2,61 (2,55–2,71) µs | -1,7% | nie (w rozrzucie) |
| index_lookup | 1,87 (1,74–2,01) µs | 1,82 (1,75–2,05) µs | -2,8% | nie (w rozrzucie) |
| range_scan | 50,3 (49,8–52,6) µs | 51,9 (50,9–54,2) µs | +3,1% | nie (w rozrzucie) |
| full_scan | 5,12 (5,09–5,15) ms | 5,12 (5,04–5,17) ms | -0,1% | nie (w rozrzucie) |
| group_by | 10,8 (10,6–10,9) ms | 10,8 (10,6–10,8) ms | -0,0% | nie (w rozrzucie) |
| top_k | 5,58 (5,55–5,69) ms | 5,51 (5,47–5,56) ms | -1,3% | nie (w rozrzucie) |
| materialize_10k | 1,97 (1,87–2,07) ms | 1,85 (1,72–1,96) ms | -6,1% | nie (w rozrzucie) |
| update_10pct | 16,9 (16,8–17,3) ms | 17,1 (16,3–18,8) ms | +1,2% | nie (w rozrzucie) |
| delete_10pct | 20,4 (19,5–21,6) ms | 19,5 (19,3–19,9) ms | -4,5% | nie (w rozrzucie) |
| snapshot | 26,1 (26,1–27,2) ms | 26,2 (25,8–27,3) ms | +0,2% | nie (w rozrzucie) |
| snapshot (rozmiar) | 3,71 MiB | 3,71 MiB | +0,0% | nie (w rozrzucie) |
| restore | 54,2 (52,9–54,7) ms | 54,1 (52,3–54,8) ms | -0,1% | nie (w rozrzucie) |
| insert_many | 283 (280–285) ms | 160 (157–162) ms | -43,3% | tak, szybciej/mniej |

Wyniki zgodne w obu wersjach (160 porównań sum kontrolnych i stanu tabeli).

### node – workload – flatsql-json

| Metryka | Przed | Po | Zmiana mediany | 95% CI średnich rozłączne |
|---|---:|---:|---:|---|
| insert | 299 (294–304) ms | 301 (298–304) ms | +0,6% | nie (w rozrzucie) |
| point_lookup | 4,57 (4,48–4,80) µs | 4,45 (4,35–4,50) µs | -2,6% | nie (w rozrzucie) |
| index_lookup | 3,14 (3,08–3,26) µs | 3,08 (3,04–3,13) µs | -2,0% | nie (w rozrzucie) |
| range_scan | 67,1 (66,4–68,6) µs | 53,1 (52,7–55,5) µs | -20,9% | tak, szybciej/mniej |
| full_scan | 5,12 (5,11–5,20) ms | 5,18 (5,15–5,21) ms | +1,1% | nie (w rozrzucie) |
| group_by | 11,2 (11,1–11,7) ms | 10,9 (10,9–11,9) ms | -2,6% | nie (w rozrzucie) |
| top_k | 5,56 (5,51–5,63) ms | 5,56 (5,53–5,62) ms | +0,1% | nie (w rozrzucie) |
| materialize_10k | 11,3 (10,8–11,5) ms | 11,1 (11,0–11,4) ms | -1,7% | nie (w rozrzucie) |
| update_10pct | 17,5 (17,3–17,7) ms | 17,2 (16,7–17,9) ms | -1,9% | nie (w rozrzucie) |
| delete_10pct | 22,0 (21,4–22,4) ms | 21,7 (21,3–22,2) ms | -1,3% | nie (w rozrzucie) |
| snapshot | 29,1 (27,5–30,5) ms | 30,2 (29,1–30,8) ms | +3,8% | nie (w rozrzucie) |
| snapshot (rozmiar) | 3,71 MiB | 3,71 MiB | +0,0% | nie (w rozrzucie) |
| restore | 59,4 (58,5–60,9) ms | 58,5 (57,5–60,0) ms | -1,5% | nie (w rozrzucie) |
| insert_many | 290 (286–291) ms | 164 (163–170) ms | -43,5% | tak, szybciej/mniej |

Wyniki zgodne w obu wersjach (160 porównań sum kontrolnych i stanu tabeli).

