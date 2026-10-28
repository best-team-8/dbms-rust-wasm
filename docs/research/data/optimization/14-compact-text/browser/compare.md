Przed: po-2.14 (core/wasm: execute a prepared write for many parameter sets in one call); po: po-2.14-dirty (core/wasm: execute a prepared write for many parameter sets in one call).
Obciążenie procesora przed pomiarem: 20,6%. Uruchomienia naprzemienne, 10 na wersję.
Moduł WebAssembly: 700803 → 711629 B, gzip -9: 264711 → 268883 B (+4172 B).

### browser – memory – flatsql

| Metryka | Przed | Po | Zmiana mediany | 95% CI średnich rozłączne |
|---|---:|---:|---:|---|
| wasm_after_load (rozmiar) | 54,8 MiB | 51,8 MiB | -5,5% | tak, szybciej/mniej |

### browser – memory – flatsql-json

| Metryka | Przed | Po | Zmiana mediany | 95% CI średnich rozłączne |
|---|---:|---:|---:|---|
| wasm_after_load (rozmiar) | 54,8 MiB | 51,8 MiB | -5,5% | tak, szybciej/mniej |

### browser – workload – flatsql

| Metryka | Przed | Po | Zmiana mediany | 95% CI średnich rozłączne |
|---|---:|---:|---:|---|
| insert | 272 (270–275) ms | 268 (266–271) ms | -1,6% | nie (w rozrzucie) |
| point_lookup | 3,17 (3,11–3,27) µs | 3,11 (3,02–3,18) µs | -1,9% | nie (w rozrzucie) |
| index_lookup | 1,96 (1,90–2,00) µs | 1,89 (1,85–1,91) µs | -3,3% | nie (w rozrzucie) |
| range_scan | 51,2 (50,7–52,5) µs | 45,4 (43,7–46,7) µs | -11,3% | tak, szybciej/mniej |
| full_scan | 4,87 (4,83–4,93) ms | 4,45 (4,40–4,47) ms | -8,7% | tak, szybciej/mniej |
| group_by | 10,9 (10,2–11,1) ms | 9,36 (9,33–9,42) ms | -14,4% | tak, szybciej/mniej |
| top_k | 5,43 (5,40–5,47) ms | 5,35 (5,34–5,40) ms | -1,5% | tak, szybciej/mniej |
| materialize_10k | 1,53 (1,46–1,60) ms | 1,55 (1,41–1,58) ms | +1,7% | nie (w rozrzucie) |
| update_10pct | 15,8 (15,6–16,3) ms | 14,8 (14,7–15,0) ms | -6,4% | tak, szybciej/mniej |
| delete_10pct | 19,7 (19,1–19,9) ms | 17,3 (17,1–17,4) ms | -12,3% | tak, szybciej/mniej |
| snapshot | 24,3 (24,2–24,5) ms | 21,9 (21,7–22,3) ms | -9,8% | tak, szybciej/mniej |
| snapshot (rozmiar) | 3,71 MiB | 3,71 MiB | +0,0% | nie (w rozrzucie) |
| restore | 49,1 (48,8–49,4) ms | 46,3 (46,0–46,5) ms | -5,9% | tak, szybciej/mniej |
| insert_many | 149 (148–152) ms | 147 (147–148) ms | -1,5% | nie (w rozrzucie) |

Wyniki zgodne w obu wersjach (160 porównań sum kontrolnych i stanu tabeli).

### browser – workload – flatsql-json

| Metryka | Przed | Po | Zmiana mediany | 95% CI średnich rozłączne |
|---|---:|---:|---:|---|
| insert | 270 (268–272) ms | 271 (266–274) ms | +0,2% | nie (w rozrzucie) |
| point_lookup | 4,31 (4,28–4,35) µs | 4,17 (4,14–4,21) µs | -3,3% | tak, szybciej/mniej |
| index_lookup | 2,85 (2,81–2,90) µs | 2,94 (2,88–2,97) µs | +3,3% | nie (w rozrzucie) |
| range_scan | 53,4 (52,2–54,4) µs | 45,8 (45,7–46,1) µs | -14,2% | tak, szybciej/mniej |
| full_scan | 4,86 (4,83–4,90) ms | 4,44 (4,42–4,49) ms | -8,5% | tak, szybciej/mniej |
| group_by | 11,4 (11,4–11,5) ms | 9,71 (9,67–9,76) ms | -15,1% | tak, szybciej/mniej |
| top_k | 5,89 (5,79–6,05) ms | 5,53 (5,49–5,57) ms | -6,1% | tak, szybciej/mniej |
| materialize_10k | 9,12 (8,98–9,66) ms | 7,31 (7,18–7,48) ms | -19,9% | tak, szybciej/mniej |
| update_10pct | 16,0 (15,9–16,7) ms | 15,2 (14,8–15,4) ms | -4,9% | tak, szybciej/mniej |
| delete_10pct | 20,6 (20,1–21,4) ms | 19,2 (18,9–19,8) ms | -6,7% | nie (w rozrzucie) |
| snapshot | 28,3 (25,5–28,7) ms | 25,0 (22,5–25,4) ms | -11,7% | tak, szybciej/mniej |
| snapshot (rozmiar) | 3,71 MiB | 3,71 MiB | +0,0% | nie (w rozrzucie) |
| restore | 55,6 (53,0–56,9) ms | 51,7 (49,9–52,3) ms | -7,1% | tak, szybciej/mniej |
| insert_many | 155 (151–158) ms | 156 (153–159) ms | +0,6% | nie (w rozrzucie) |

Wyniki zgodne w obu wersjach (160 porównań sum kontrolnych i stanu tabeli).

