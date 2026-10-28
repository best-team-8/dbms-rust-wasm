Przed: po-2.1 (core: count rows by the access path when it guarantees the whole WHERE clause); po: po-2.1-dirty (core: count rows by the access path when it guarantees the whole WHERE clause).
Obciążenie procesora przed pomiarem: 21,7%. Uruchomienia naprzemienne, 20 na wersję.
Moduł WebAssembly: 586155 → 591036 B, gzip -9: 221633 → 223514 B (+1881 B).

### native – workload – flatsql-native

| Metryka | Przed | Po | Zmiana mediany | 95% CI średnich rozłączne |
|---|---:|---:|---:|---|
| insert | 235 (224–252) ms | 238 (231–263) ms | +1,4% | nie (w rozrzucie) |
| point_lookup | 2,65 (2,54–2,75) µs | 2,63 (2,54–2,73) µs | -0,8% | nie (w rozrzucie) |
| index_lookup | 1,82 (1,69–2,45) µs | 1,76 (1,70–2,03) µs | -3,4% | nie (w rozrzucie) |
| range_scan | 92,3 (86,3–97,5) µs | 51,5 (50,4–55,2) µs | -44,2% | tak, szybciej/mniej |
| full_scan | 8,99 (8,61–9,43) ms | 8,52 (8,24–9,70) ms | -5,2% | nie (w rozrzucie) |
| group_by | 16,7 (16,2–18,0) ms | 16,8 (16,3–17,8) ms | +0,6% | nie (w rozrzucie) |
| top_k | 6,00 (5,83–6,46) ms | 6,13 (6,10–6,29) ms | +2,2% | nie (w rozrzucie) |
| materialize_10k | 4,18 (4,10–4,39) ms | 3,86 (3,78–4,08) ms | -7,7% | nie (w rozrzucie) |
| update_10pct | 34,6 (33,2–37,5) ms | 36,7 (34,8–37,2) ms | +6,0% | nie (w rozrzucie) |
| delete_10pct | 27,5 (26,7–29,4) ms | 27,3 (26,5–30,4) ms | -0,6% | nie (w rozrzucie) |
| snapshot | 27,8 (26,4–29,1) ms | 28,5 (27,0–30,1) ms | +2,3% | nie (w rozrzucie) |
| snapshot (rozmiar) | 5,03 MiB | 5,03 MiB | +0,0% | nie (w rozrzucie) |
| restore | 128 (124–134) ms | 139 (125–148) ms | +8,7% | nie (w rozrzucie) |

Wyniki zgodne w obu wersjach (280 porównań sum kontrolnych i stanu tabeli).

