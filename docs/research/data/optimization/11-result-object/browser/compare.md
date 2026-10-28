Przed: po-2.3 (docs: profile of a primary key lookup at the JavaScript and WebAssembly boundary); po: po-2.3-dirty (docs: profile of a primary key lookup at the JavaScript and WebAssembly boundary).
Obciążenie procesora przed pomiarem: 10,3%. Uruchomienia naprzemienne, 10 na wersję.
Moduł WebAssembly: 690050 → 690258 B, gzip -9: 259478 → 259766 B (+288 B).

### browser – workload – flatsql

| Metryka | Przed | Po | Zmiana mediany | 95% CI średnich rozłączne |
|---|---:|---:|---:|---|
| insert | 289 (286–293) ms | 286 (284–294) ms | -1,0% | nie (w rozrzucie) |
| point_lookup | 8,99 (8,84–9,05) µs | 3,95 (3,86–4,02) µs | -56,1% | tak, szybciej/mniej |
| index_lookup | 5,65 (5,46–6,34) µs | 2,00 (1,93–2,14) µs | -64,6% | tak, szybciej/mniej |
| range_scan | 55,2 (54,6–56,8) µs | 53,5 (52,2–54,2) µs | -3,1% | nie (w rozrzucie) |
| full_scan | 5,37 (5,24–5,47) ms | 5,35 (5,29–5,51) ms | -0,5% | nie (w rozrzucie) |
| group_by | 11,0 (10,7–11,7) ms | 11,7 (10,7–12,6) ms | +6,8% | nie (w rozrzucie) |
| top_k | 5,58 (5,55–5,68) ms | 5,64 (5,60–5,74) ms | +1,1% | nie (w rozrzucie) |
| materialize_10k | 13,9 (13,9–14,4) ms | 13,9 (13,9–14,6) ms | +0,2% | nie (w rozrzucie) |
| update_10pct | 16,6 (16,2–17,8) ms | 17,3 (16,8–17,6) ms | +4,5% | nie (w rozrzucie) |
| delete_10pct | 20,6 (20,3–20,8) ms | 22,3 (20,8–23,7) ms | +8,4% | nie (w rozrzucie) |
| snapshot | 28,4 (28,2–29,4) ms | 28,0 (27,0–28,8) ms | -1,5% | nie (w rozrzucie) |
| snapshot (rozmiar) | 3,71 MiB | 3,71 MiB | +0,0% | nie (w rozrzucie) |
| restore | 55,5 (54,6–59,5) ms | 57,5 (55,4–58,4) ms | +3,5% | nie (w rozrzucie) |

Wyniki zgodne w obu wersjach (160 porównań sum kontrolnych i stanu tabeli).

### browser – workload – flatsql-json

| Metryka | Przed | Po | Zmiana mediany | 95% CI średnich rozłączne |
|---|---:|---:|---:|---|
| insert | 276 (275–278) ms | 281 (274–284) ms | +1,8% | nie (w rozrzucie) |
| point_lookup | 4,66 (4,51–4,71) µs | 4,61 (4,51–4,84) µs | -1,0% | nie (w rozrzucie) |
| index_lookup | 2,94 (2,88–3,00) µs | 2,99 (2,90–3,11) µs | +1,7% | nie (w rozrzucie) |
| range_scan | 52,2 (52,0–54,3) µs | 53,6 (52,8–54,0) µs | +2,5% | nie (w rozrzucie) |
| full_scan | 5,21 (5,17–5,27) ms | 5,26 (5,23–5,47) ms | +0,9% | nie (w rozrzucie) |
| group_by | 12,0 (11,0–12,6) ms | 12,4 (11,6–12,7) ms | +3,2% | nie (w rozrzucie) |
| top_k | 6,19 (5,81–6,62) ms | 5,76 (5,73–5,90) ms | -7,0% | nie (w rozrzucie) |
| materialize_10k | 8,98 (8,83–9,30) ms | 9,18 (9,08–9,20) ms | +2,2% | nie (w rozrzucie) |
| update_10pct | 17,3 (17,1–18,2) ms | 16,9 (16,6–17,2) ms | -2,3% | nie (w rozrzucie) |
| delete_10pct | 22,0 (21,6–22,7) ms | 20,5 (20,2–21,5) ms | -7,1% | nie (w rozrzucie) |
| snapshot | 28,5 (27,4–29,5) ms | 28,2 (27,4–29,0) ms | -1,1% | nie (w rozrzucie) |
| snapshot (rozmiar) | 3,71 MiB | 3,71 MiB | +0,0% | nie (w rozrzucie) |
| restore | 56,2 (55,5–57,1) ms | 56,1 (55,1–56,3) ms | -0,2% | nie (w rozrzucie) |

Wyniki zgodne w obu wersjach (160 porównań sum kontrolnych i stanu tabeli).

