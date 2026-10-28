Przed: po-2.4 (core/wasm: encode results straight from the table in a binary format decoded by JavaScript); po: po-2.4-dirty (core/wasm: encode results straight from the table in a binary format decoded by JavaScript).
Obciążenie procesora przed pomiarem: 10,4%. Uruchomienia naprzemienne, 10 na wersję.
Moduł WebAssembly: 695364 → 700875 B, gzip -9: 262201 → 264753 B (+2552 B).

### browser – workload – flatsql

| Metryka | Przed | Po | Zmiana mediany | 95% CI średnich rozłączne |
|---|---:|---:|---:|---|
| insert | 283 (281–284) ms | 281 (279–286) ms | -0,6% | nie (w rozrzucie) |
| point_lookup | 3,36 (3,28–3,51) µs | 3,32 (3,27–3,36) µs | -1,3% | nie (w rozrzucie) |
| index_lookup | 1,92 (1,89–1,98) µs | 1,90 (1,86–1,91) µs | -1,3% | nie (w rozrzucie) |
| range_scan | 52,6 (51,4–54,8) µs | 54,2 (52,4–55,2) µs | +3,1% | nie (w rozrzucie) |
| full_scan | 5,08 (5,00–5,13) ms | 5,03 (5,02–5,06) ms | -0,9% | nie (w rozrzucie) |
| group_by | 11,5 (11,2–11,5) ms | 11,4 (11,3–11,5) ms | -0,4% | nie (w rozrzucie) |
| top_k | 5,56 (5,52–5,60) ms | 5,54 (5,51–5,57) ms | -0,5% | nie (w rozrzucie) |
| materialize_10k | 1,61 (1,55–1,62) ms | 1,60 (1,53–1,64) ms | -0,7% | nie (w rozrzucie) |
| update_10pct | 16,7 (16,2–17,1) ms | 16,1 (15,7–16,4) ms | -3,6% | nie (w rozrzucie) |
| delete_10pct | 20,0 (19,1–20,7) ms | 19,2 (19,0–19,6) ms | -3,9% | nie (w rozrzucie) |
| snapshot | 25,0 (24,6–25,5) ms | 25,0 (24,8–26,1) ms | -0,1% | nie (w rozrzucie) |
| snapshot (rozmiar) | 3,71 MiB | 3,71 MiB | +0,0% | nie (w rozrzucie) |
| restore | 51,5 (50,6–51,8) ms | 51,8 (50,8–52,1) ms | +0,6% | nie (w rozrzucie) |
| insert_many | 273 (270–278) ms | 151 (150–153) ms | -44,9% | tak, szybciej/mniej |

Wyniki zgodne w obu wersjach (160 porównań sum kontrolnych i stanu tabeli).

### browser – workload – flatsql-json

| Metryka | Przed | Po | Zmiana mediany | 95% CI średnich rozłączne |
|---|---:|---:|---:|---|
| insert | 276 (273–279) ms | 279 (275–284) ms | +1,1% | nie (w rozrzucie) |
| point_lookup | 4,48 (4,43–4,59) µs | 4,51 (4,48–4,52) µs | +0,5% | nie (w rozrzucie) |
| index_lookup | 2,94 (2,88–3,04) µs | 2,90 (2,87–2,96) µs | -1,2% | nie (w rozrzucie) |
| range_scan | 54,8 (54,4–57,4) µs | 56,1 (54,0–57,0) µs | +2,4% | nie (w rozrzucie) |
| full_scan | 5,02 (4,97–5,11) ms | 5,05 (4,99–5,11) ms | +0,6% | nie (w rozrzucie) |
| group_by | 10,8 (10,6–10,9) ms | 12,0 (11,9–12,2) ms | +11,6% | tak, wolniej/więcej |
| top_k | 5,82 (5,62–6,09) ms | 6,10 (5,94–6,17) ms | +4,7% | nie (w rozrzucie) |
| materialize_10k | 9,20 (9,04–9,46) ms | 9,27 (9,12–10,00) ms | +0,8% | nie (w rozrzucie) |
| update_10pct | 17,5 (17,1–18,1) ms | 17,2 (16,3–18,0) ms | -2,2% | nie (w rozrzucie) |
| delete_10pct | 21,1 (20,4–21,5) ms | 21,7 (20,4–22,9) ms | +2,8% | nie (w rozrzucie) |
| snapshot | 28,8 (28,0–30,0) ms | 28,5 (27,7–29,3) ms | -1,0% | nie (w rozrzucie) |
| snapshot (rozmiar) | 3,71 MiB | 3,71 MiB | +0,0% | nie (w rozrzucie) |
| restore | 57,9 (56,9–59,0) ms | 57,0 (56,4–57,7) ms | -1,6% | nie (w rozrzucie) |
| insert_many | 276 (273–281) ms | 161 (158–166) ms | -41,7% | tak, szybciej/mniej |

Wyniki zgodne w obu wersjach (160 porównań sum kontrolnych i stanu tabeli).

