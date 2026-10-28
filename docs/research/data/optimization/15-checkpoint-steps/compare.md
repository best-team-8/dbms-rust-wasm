Przed: po-2.7 (core: keep short texts inside the value); po: po-2.7-dirty (core: keep short texts inside the value).
Obciążenie procesora przed pomiarem: 8,0%. Uruchomienia naprzemienne, 10 na wersję.
Moduł WebAssembly: 711549 → 724082 B, gzip -9: 268907 → 274096 B (+5189 B).

### native – fsync – flatsql-native

| Metryka | Przed | Po | Zmiana mediany | 95% CI średnich rozłączne |
|---|---:|---:|---:|---|
| FileStorage fsync insert | 197 (195–201) ms | 203 (197–204) ms | +3,1% | nie (w rozrzucie) |
| FileStorage fsync commit_single | 517 (500–528) µs | 504 (487–518) µs | -2,7% | nie (w rozrzucie) |
| FileStorage fsync commit_group | 2,69 (2,67–2,73) ms | 2,80 (2,69–2,86) ms | +4,1% | nie (w rozrzucie) |
| FileStorage fsync checkpoint | 30,3 (29,7–31,3) ms | 30,9 (30,0–33,8) ms | +2,0% | nie (w rozrzucie) |
| FileStorage fsync reopen | 45,7 (45,4–46,4) ms | 45,9 (45,5–46,1) ms | +0,6% | nie (w rozrzucie) |
| FileStorage nofsync insert | 191 (190–195) ms | 192 (191–195) ms | +0,0% | nie (w rozrzucie) |
| FileStorage nofsync commit_single | 9,30 (8,88–9,62) µs | 9,50 (9,07–10,8) µs | +2,1% | nie (w rozrzucie) |
| FileStorage nofsync commit_group | 2,09 (2,09–2,10) ms | 2,25 (2,13–2,39) ms | +7,5% | nie (w rozrzucie) |
| FileStorage nofsync checkpoint | 29,1 (28,9–29,5) ms | 30,2 (29,4–31,5) ms | +3,6% | nie (w rozrzucie) |
| FileStorage nofsync reopen | 45,2 (44,5–46,2) ms | 45,5 (45,2–45,8) ms | +0,7% | nie (w rozrzucie) |

### native – workload – flatsql-native

| Metryka | Przed | Po | Zmiana mediany | 95% CI średnich rozłączne |
|---|---:|---:|---:|---|
| insert | 158 (157–160) ms | 159 (158–160) ms | +0,4% | nie (w rozrzucie) |
| point_lookup | 0,97 (0,94–0,98) µs | 0,93 (0,91–0,94) µs | -4,2% | tak, szybciej/mniej |
| index_lookup | 0,71 (0,70–0,71) µs | 0,70 (0,69–0,71) µs | -1,6% | nie (w rozrzucie) |
| range_scan | 50,0 (49,9–51,6) µs | 49,6 (49,1–50,0) µs | -0,9% | nie (w rozrzucie) |
| full_scan | 5,42 (5,38–5,50) ms | 5,44 (5,33–5,47) ms | +0,5% | nie (w rozrzucie) |
| group_by | 10,9 (10,9–11,0) ms | 10,8 (10,8–10,9) ms | -0,7% | nie (w rozrzucie) |
| top_k | 6,88 (6,85–7,01) ms | 6,81 (6,80–6,90) ms | -1,0% | nie (w rozrzucie) |
| materialize_10k | 2,91 (2,88–2,94) ms | 2,87 (2,86–2,91) ms | -1,2% | nie (w rozrzucie) |
| update_10pct | 14,8 (14,4–15,1) ms | 14,8 (14,5–14,9) ms | +0,1% | nie (w rozrzucie) |
| delete_10pct | 14,6 (14,3–14,7) ms | 14,6 (14,2–14,8) ms | +0,1% | nie (w rozrzucie) |
| snapshot | 16,8 (16,7–16,9) ms | 17,3 (17,2–17,8) ms | +3,1% | nie (w rozrzucie) |
| snapshot (rozmiar) | 3,71 MiB | 3,71 MiB | +0,0% | nie (w rozrzucie) |
| restore | 40,8 (40,3–41,1) ms | 40,9 (40,8–41,1) ms | +0,2% | nie (w rozrzucie) |
| insert_many | 142 (141–142) ms | 143 (142–144) ms | +0,8% | nie (w rozrzucie) |

Wyniki zgodne w obu wersjach (140 porównań sum kontrolnych i stanu tabeli).

### node – fsync – flatsql

| Metryka | Przed | Po | Zmiana mediany | 95% CI średnich rozłączne |
|---|---:|---:|---:|---|
| FsStore fsync insert | 413 (404–416) ms | 421 (408–433) ms | +1,9% | nie (w rozrzucie) |
| FsStore fsync commit_single | 822 (745–877) µs | 801 (792–859) µs | -2,5% | nie (w rozrzucie) |
| FsStore fsync commit_group | 5,84 (5,72–6,21) ms | 5,72 (5,65–5,96) ms | -2,0% | nie (w rozrzucie) |
| FsStore fsync checkpoint | 37,7 (36,3–39,5) ms | 39,8 (38,9–42,4) ms | +5,5% | nie (w rozrzucie) |
| FsStore fsync reopen | 53,7 (52,7–55,5) ms | 52,6 (52,1–53,9) ms | -2,0% | nie (w rozrzucie) |
| FsStore nofsync insert | 350 (344–356) ms | 349 (345–351) ms | -0,3% | nie (w rozrzucie) |
| FsStore nofsync commit_single | 82,8 (78,9–87,9) µs | 84,5 (81,7–86,7) µs | +2,1% | nie (w rozrzucie) |
| FsStore nofsync commit_group | 3,89 (3,88–4,09) ms | 3,92 (3,89–4,03) ms | +0,6% | nie (w rozrzucie) |
| FsStore nofsync checkpoint | 34,3 (33,7–36,6) ms | 35,7 (34,9–36,8) ms | +4,2% | nie (w rozrzucie) |
| FsStore nofsync reopen | 41,6 (41,1–42,0) ms | 41,5 (41,3–41,9) ms | -0,3% | nie (w rozrzucie) |
| sync round durableServer | 655 (633–697) µs | 691 (638–726) µs | +5,6% | nie (w rozrzucie) |
| sync round memoryServer | 554 (538–580) µs | 527 (511–538) µs | -4,8% | nie (w rozrzucie) |

### node – workload – flatsql

| Metryka | Przed | Po | Zmiana mediany | 95% CI średnich rozłączne |
|---|---:|---:|---:|---|
| insert | 292 (289–294) ms | 293 (289–297) ms | +0,2% | nie (w rozrzucie) |
| point_lookup | 2,62 (2,52–3,13) µs | 2,54 (2,49–2,63) µs | -3,1% | nie (w rozrzucie) |
| index_lookup | 2,27 (1,83–2,46) µs | 1,83 (1,74–2,44) µs | -19,6% | nie (w rozrzucie) |
| range_scan | 41,9 (41,4–43,1) µs | 42,8 (40,8–45,3) µs | +2,2% | nie (w rozrzucie) |
| full_scan | 4,56 (4,53–4,63) ms | 4,58 (4,52–4,65) ms | +0,6% | nie (w rozrzucie) |
| group_by | 9,60 (9,54–9,65) ms | 9,67 (9,62–9,77) ms | +0,7% | nie (w rozrzucie) |
| top_k | 5,35 (5,28–5,43) ms | 5,28 (5,26–5,44) ms | -1,3% | nie (w rozrzucie) |
| materialize_10k | 1,90 (1,86–1,95) ms | 1,89 (1,78–1,97) ms | -0,4% | nie (w rozrzucie) |
| update_10pct | 15,1 (14,9–16,0) ms | 15,7 (15,4–16,4) ms | +4,0% | nie (w rozrzucie) |
| delete_10pct | 18,2 (17,8–18,4) ms | 18,4 (18,0–18,9) ms | +1,1% | nie (w rozrzucie) |
| snapshot | 22,3 (22,2–22,8) ms | 22,7 (22,5–23,3) ms | +1,7% | nie (w rozrzucie) |
| snapshot (rozmiar) | 3,71 MiB | 3,71 MiB | +0,0% | nie (w rozrzucie) |
| restore | 48,9 (48,0–49,7) ms | 48,9 (48,2–49,6) ms | -0,0% | nie (w rozrzucie) |
| insert_many | 154 (153–156) ms | 152 (151–155) ms | -1,1% | nie (w rozrzucie) |

Wyniki zgodne w obu wersjach (160 porównań sum kontrolnych i stanu tabeli).

### node – workload – flatsql-json

| Metryka | Przed | Po | Zmiana mediany | 95% CI średnich rozłączne |
|---|---:|---:|---:|---|
| insert | 290 (288–295) ms | 292 (286–293) ms | +0,6% | nie (w rozrzucie) |
| point_lookup | 4,14 (4,13–4,21) µs | 4,24 (4,10–4,70) µs | +2,4% | nie (w rozrzucie) |
| index_lookup | 3,20 (3,08–3,43) µs | 3,25 (3,08–3,55) µs | +1,3% | nie (w rozrzucie) |
| range_scan | 45,5 (45,1–47,9) µs | 43,9 (43,6–44,8) µs | -3,4% | nie (w rozrzucie) |
| full_scan | 4,57 (4,52–4,62) ms | 4,55 (4,53–4,60) ms | -0,4% | nie (w rozrzucie) |
| group_by | 10,2 (9,97–10,3) ms | 10,2 (9,99–10,3) ms | +0,8% | nie (w rozrzucie) |
| top_k | 5,63 (5,57–5,78) ms | 5,71 (5,39–5,84) ms | +1,5% | nie (w rozrzucie) |
| materialize_10k | 9,32 (8,97–9,61) ms | 9,08 (8,73–9,34) ms | -2,6% | nie (w rozrzucie) |
| update_10pct | 14,9 (14,7–15,0) ms | 15,4 (15,2–16,1) ms | +3,8% | nie (w rozrzucie) |
| delete_10pct | 18,0 (17,7–19,0) ms | 17,9 (17,7–19,6) ms | -0,7% | nie (w rozrzucie) |
| snapshot | 22,7 (22,1–23,4) ms | 22,3 (21,9–23,8) ms | -1,8% | nie (w rozrzucie) |
| snapshot (rozmiar) | 3,71 MiB | 3,71 MiB | +0,0% | nie (w rozrzucie) |
| restore | 48,4 (48,0–48,9) ms | 49,0 (47,8–50,1) ms | +1,1% | nie (w rozrzucie) |
| insert_many | 154 (152–156) ms | 152 (151–154) ms | -1,3% | nie (w rozrzucie) |

Wyniki zgodne w obu wersjach (160 porównań sum kontrolnych i stanu tabeli).

