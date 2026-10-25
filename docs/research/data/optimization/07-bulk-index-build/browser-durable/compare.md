Przed: po-2.8 (core: update rows in place, touch only changed indexes, read the clock once per statement); po: po-2.8-dirty (core: update rows in place, touch only changed indexes, read the clock once per statement).
Obciążenie procesora przed pomiarem: 8,1%. Uruchomienia naprzemienne, 10 na wersję.
Moduł WebAssembly: 655868 → 684172 B, gzip -9: 247915 → 257469 B (+9554 B).

### browser – workload – flatsql-idb

| Metryka | Przed | Po | Zmiana mediany | 95% CI średnich rozłączne |
|---|---:|---:|---:|---|
| insert | 386 (378–395) ms | 384 (381–392) ms | -0,5% | nie (w rozrzucie) |
| point_lookup | 10,0 (9,81–10,2) µs | 9,75 (9,59–9,87) µs | -2,6% | nie (w rozrzucie) |
| index_lookup | 6,34 (6,15–6,49) µs | 6,11 (6,06–6,27) µs | -3,7% | nie (w rozrzucie) |
| range_scan | 58,2 (56,0–59,6) µs | 57,5 (54,5–58,8) µs | -1,2% | nie (w rozrzucie) |
| full_scan | 5,30 (5,23–5,54) ms | 5,21 (5,20–5,26) ms | -1,7% | nie (w rozrzucie) |
| group_by | 17,0 (16,7–18,2) ms | 18,0 (17,4–18,7) ms | +5,6% | nie (w rozrzucie) |
| top_k | 5,71 (5,64–5,89) ms | 5,82 (5,65–6,24) ms | +2,1% | nie (w rozrzucie) |
| materialize_10k | 14,2 (13,5–14,7) ms | 13,4 (13,3–13,6) ms | -5,3% | nie (w rozrzucie) |
| update_10pct | 29,5 (27,4–36,0) ms | 31,6 (28,5–34,4) ms | +7,1% | nie (w rozrzucie) |
| delete_10pct | 26,3 (26,0–26,9) ms | 28,2 (27,6–28,6) ms | +6,9% | nie (w rozrzucie) |
| commit_single | 497 (480–503) µs | 490 (473–495) µs | -1,4% | nie (w rozrzucie) |
| commit_group | 5,66 (5,51–5,74) ms | 5,74 (5,49–5,88) ms | +1,5% | nie (w rozrzucie) |
| reopen | 102 (100–105) ms | 98,4 (96,9–101) ms | -3,7% | nie (w rozrzucie) |

Wyniki zgodne w obu wersjach (160 porównań sum kontrolnych i stanu tabeli).

