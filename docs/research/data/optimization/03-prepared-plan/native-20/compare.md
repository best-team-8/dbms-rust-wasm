Przed: po-2.2 (core: skip the conjuncts of WHERE that the access path guarantees); po: po-2.2-dirty (core: skip the conjuncts of WHERE that the access path guarantees).
Obciążenie procesora przed pomiarem: 10,6%. Uruchomienia naprzemienne, 20 na wersję.
Moduł WebAssembly: 590964 → 608941 B, gzip -9: 223497 → 232549 B (+9052 B).

### native – workload – flatsql-native

| Metryka | Przed | Po | Zmiana mediany | 95% CI średnich rozłączne |
|---|---:|---:|---:|---|
| insert | 239 (231–245) ms | 233 (221–240) ms | -2,4% | nie (w rozrzucie) |
| point_lookup | 2,69 (2,53–2,90) µs | 1,35 (1,27–1,42) µs | -49,9% | tak, szybciej/mniej |
| index_lookup | 1,78 (1,67–2,12) µs | 0,72 (0,71–0,73) µs | -59,4% | tak, szybciej/mniej |
| range_scan | 53,1 (50,5–59,0) µs | 52,8 (51,9–58,1) µs | -0,5% | nie (w rozrzucie) |
| full_scan | 8,64 (8,18–9,06) ms | 8,19 (8,01–8,30) ms | -5,2% | nie (w rozrzucie) |
| group_by | 17,2 (16,7–17,9) ms | 17,3 (16,8–18,7) ms | +0,6% | nie (w rozrzucie) |
| top_k | 6,37 (6,13–6,96) ms | 6,36 (6,17–6,52) ms | -0,2% | nie (w rozrzucie) |
| materialize_10k | 4,14 (3,98–4,63) ms | 4,17 (3,79–4,65) ms | +0,7% | nie (w rozrzucie) |
| update_10pct | 35,5 (34,2–36,7) ms | 34,7 (34,1–37,4) ms | -2,1% | nie (w rozrzucie) |
| delete_10pct | 27,8 (27,0–29,9) ms | 28,0 (27,1–30,1) ms | +0,6% | nie (w rozrzucie) |
| snapshot | 28,0 (26,7–30,6) ms | 27,8 (26,7–29,2) ms | -0,8% | nie (w rozrzucie) |
| snapshot (rozmiar) | 5,03 MiB | 5,03 MiB | +0,0% | nie (w rozrzucie) |
| restore | 126 (121–135) ms | 125 (118–132) ms | -0,6% | nie (w rozrzucie) |

Wyniki zgodne w obu wersjach (280 porównań sum kontrolnych i stanu tabeli).

