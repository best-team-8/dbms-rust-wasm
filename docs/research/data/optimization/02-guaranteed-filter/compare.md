Przed: po-2.1 (core: count rows by the access path when it guarantees the whole WHERE clause); po: po-2.1-dirty (core: count rows by the access path when it guarantees the whole WHERE clause).
Obciążenie procesora przed pomiarem: 10,5%. Uruchomienia naprzemienne, 10 na wersję.
Moduł WebAssembly: 586155 → 591036 B, gzip -9: 221633 → 223514 B (+1881 B).

### native – workload – flatsql-native

| Metryka | Przed | Po | Zmiana mediany | 95% CI średnich rozłączne |
|---|---:|---:|---:|---|
| insert | 235 (226–280) ms | 232 (227–238) ms | -1,1% | nie (w rozrzucie) |
| point_lookup | 2,78 (2,63–2,81) µs | 2,77 (2,68–3,18) µs | -0,4% | nie (w rozrzucie) |
| index_lookup | 1,68 (1,67–1,72) µs | 1,77 (1,75–1,94) µs | +5,3% | nie (w rozrzucie) |
| range_scan | 96,8 (85,8–106) µs | 56,1 (54,4–57,8) µs | -42,1% | tak, szybciej/mniej |
| full_scan | 9,08 (8,73–9,23) ms | 8,36 (8,12–8,73) ms | -7,9% | nie (w rozrzucie) |
| group_by | 17,4 (17,2–17,7) ms | 16,7 (16,5–17,1) ms | -3,7% | nie (w rozrzucie) |
| top_k | 6,07 (5,86–6,33) ms | 6,39 (6,03–6,81) ms | +5,4% | nie (w rozrzucie) |
| materialize_10k | 4,33 (4,13–4,69) ms | 4,14 (4,06–4,26) ms | -4,5% | nie (w rozrzucie) |
| update_10pct | 35,1 (33,4–36,4) ms | 33,7 (32,6–34,7) ms | -3,9% | nie (w rozrzucie) |
| delete_10pct | 28,0 (26,6–31,4) ms | 29,1 (27,7–31,2) ms | +4,2% | nie (w rozrzucie) |
| snapshot | 27,6 (27,4–27,9) ms | 27,2 (26,4–28,2) ms | -1,4% | nie (w rozrzucie) |
| snapshot (rozmiar) | 5,03 MiB | 5,03 MiB | +0,0% | nie (w rozrzucie) |
| restore | 128 (127–132) ms | 126 (120–140) ms | -2,0% | nie (w rozrzucie) |

Wyniki zgodne w obu wersjach (140 porównań sum kontrolnych i stanu tabeli).

### node – workload – flatsql

| Metryka | Przed | Po | Zmiana mediany | 95% CI średnich rozłączne |
|---|---:|---:|---:|---|
| insert | 487 (468–527) ms | 472 (464–507) ms | -3,0% | nie (w rozrzucie) |
| point_lookup | 10,3 (10,1–12,2) µs | 10,3 (10,0–10,6) µs | -0,2% | nie (w rozrzucie) |
| index_lookup | 9,39 (8,89–9,88) µs | 9,38 (8,79–9,82) µs | -0,1% | nie (w rozrzucie) |
| range_scan | 91,0 (83,5–100) µs | 58,7 (54,4–62,0) µs | -35,5% | nie (w rozrzucie) |
| full_scan | 8,98 (8,82–9,61) ms | 9,05 (8,66–9,19) ms | +0,8% | nie (w rozrzucie) |
| group_by | 18,4 (16,8–25,0) ms | 17,7 (17,4–17,9) ms | -3,9% | nie (w rozrzucie) |
| top_k | 5,96 (5,66–7,49) ms | 6,08 (5,85–6,21) ms | +1,9% | nie (w rozrzucie) |
| materialize_10k | 11,8 (11,2–13,5) ms | 10,9 (10,7–11,7) ms | -7,3% | nie (w rozrzucie) |
| update_10pct | 48,3 (47,4–51,8) ms | 42,1 (38,7–54,8) ms | -12,9% | nie (w rozrzucie) |
| delete_10pct | 33,6 (31,9–34,7) ms | 33,9 (31,9–37,9) ms | +0,8% | nie (w rozrzucie) |
| snapshot | 39,7 (38,3–40,7) ms | 37,5 (36,9–38,8) ms | -5,4% | nie (w rozrzucie) |
| snapshot (rozmiar) | 5,03 MiB | 5,03 MiB | +0,0% | nie (w rozrzucie) |
| restore | 141 (130–146) ms | 130 (128–163) ms | -7,8% | nie (w rozrzucie) |

Wyniki zgodne w obu wersjach (160 porównań sum kontrolnych i stanu tabeli).

### node – workload – flatsql-json

| Metryka | Przed | Po | Zmiana mediany | 95% CI średnich rozłączne |
|---|---:|---:|---:|---|
| insert | 473 (455–514) ms | 504 (469–565) ms | +6,6% | nie (w rozrzucie) |
| point_lookup | 6,80 (6,44–6,97) µs | 6,61 (6,41–7,18) µs | -2,8% | nie (w rozrzucie) |
| index_lookup | 5,79 (5,09–6,07) µs | 5,60 (4,89–6,01) µs | -3,3% | nie (w rozrzucie) |
| range_scan | 96,2 (91,6–99,4) µs | 56,5 (54,5–63,6) µs | -41,3% | tak, szybciej/mniej |
| full_scan | 8,80 (8,56–9,43) ms | 8,53 (8,42–9,03) ms | -3,0% | nie (w rozrzucie) |
| group_by | 17,9 (17,3–18,5) ms | 18,3 (17,6–20,0) ms | +2,2% | nie (w rozrzucie) |
| top_k | 6,05 (5,84–6,27) ms | 6,37 (6,27–6,68) ms | +5,3% | nie (w rozrzucie) |
| materialize_10k | 10,8 (10,5–12,0) ms | 11,6 (10,8–12,2) ms | +6,8% | nie (w rozrzucie) |
| update_10pct | 42,8 (38,7–47,3) ms | 42,3 (37,2–46,8) ms | -1,2% | nie (w rozrzucie) |
| delete_10pct | 32,4 (31,5–33,8) ms | 33,9 (32,2–34,7) ms | +4,4% | nie (w rozrzucie) |
| snapshot | 39,0 (37,7–42,1) ms | 38,7 (37,5–39,4) ms | -0,8% | nie (w rozrzucie) |
| snapshot (rozmiar) | 5,03 MiB | 5,03 MiB | +0,0% | nie (w rozrzucie) |
| restore | 145 (133–156) ms | 131 (124–136) ms | -9,4% | nie (w rozrzucie) |

Wyniki zgodne w obu wersjach (160 porównań sum kontrolnych i stanu tabeli).

