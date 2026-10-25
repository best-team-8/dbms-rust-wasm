Przed: po-2.3 (docs: profile of a primary key lookup at the JavaScript and WebAssembly boundary); po: po-2.3-dirty (docs: profile of a primary key lookup at the JavaScript and WebAssembly boundary).
Obciążenie procesora przed pomiarem: 6,5%. Uruchomienia naprzemienne, 10 na wersję.
Moduł WebAssembly: 690050 → 690258 B, gzip -9: 259478 → 259766 B (+288 B).

### node – workload – flatsql-json

| Metryka | Przed | Po | Zmiana mediany | 95% CI średnich rozłączne |
|---|---:|---:|---:|---|
| insert | 300 (297–302) ms | 299 (297–300) ms | -0,4% | nie (w rozrzucie) |
| point_lookup | 5,02 (4,58–5,52) µs | 4,52 (4,43–4,80) µs | -9,8% | nie (w rozrzucie) |
| index_lookup | 3,04 (3,01–4,10) µs | 3,13 (3,09–3,29) µs | +2,8% | nie (w rozrzucie) |
| range_scan | 54,7 (54,0–61,2) µs | 54,7 (52,5–58,2) µs | -0,0% | nie (w rozrzucie) |
| full_scan | 5,26 (5,11–5,34) ms | 5,12 (5,10–5,27) ms | -2,7% | nie (w rozrzucie) |
| group_by | 12,1 (11,3–12,9) ms | 12,6 (12,5–13,0) ms | +4,4% | nie (w rozrzucie) |
| top_k | 5,58 (5,53–5,87) ms | 5,56 (5,53–5,67) ms | -0,3% | nie (w rozrzucie) |
| materialize_10k | 11,1 (10,7–11,8) ms | 11,0 (10,9–11,7) ms | -1,4% | nie (w rozrzucie) |
| update_10pct | 17,5 (17,1–18,8) ms | 17,5 (16,9–18,2) ms | +0,0% | nie (w rozrzucie) |
| delete_10pct | 22,0 (21,5–22,6) ms | 21,9 (21,3–22,6) ms | -0,8% | nie (w rozrzucie) |
| snapshot | 27,0 (26,5–30,3) ms | 30,4 (29,0–30,8) ms | +12,4% | nie (w rozrzucie) |
| snapshot (rozmiar) | 3,71 MiB | 3,71 MiB | +0,0% | nie (w rozrzucie) |
| restore | 61,0 (58,8–63,7) ms | 59,1 (58,5–59,5) ms | -3,0% | nie (w rozrzucie) |

Wyniki zgodne w obu wersjach (160 porównań sum kontrolnych i stanu tabeli).

