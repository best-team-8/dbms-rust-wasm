Przed: po-2.15 (core: group rows by one column through a hash table); po: po-2.15-dirty (core: group rows by one column through a hash table).
Obciążenie procesora przed pomiarem: 12,1%. Uruchomienia naprzemienne, 10 na wersję.
Moduł WebAssembly: 690050 → 690130 B, gzip -9: 259478 → 259491 B (+13 B).

### node – fsync – flatsql

| Metryka | Przed | Po | Zmiana mediany | 95% CI średnich rozłączne |
|---|---:|---:|---:|---|
| FsStore fsync insert | 431 (424–435) ms | 425 (422–431) ms | -1,4% | nie (w rozrzucie) |
| FsStore fsync commit_single | 1,42 (1,40–1,42) ms | 720 (716–782) µs | -49,3% | tak, szybciej/mniej |
| FsStore fsync commit_group | 6,58 (6,47–6,76) ms | 5,96 (5,76–6,17) ms | -9,4% | nie (w rozrzucie) |
| FsStore fsync checkpoint | 38,6 (36,4–39,9) ms | 38,5 (36,6–39,3) ms | -0,2% | nie (w rozrzucie) |
| FsStore fsync reopen | 49,9 (49,6–51,2) ms | 52,3 (52,1–52,9) ms | +4,9% | tak, wolniej/więcej |
| FsStore nofsync insert | 353 (346–357) ms | 354 (351–363) ms | +0,3% | nie (w rozrzucie) |
| FsStore nofsync commit_single | 595 (589–607) µs | 85,4 (82,8–87,3) µs | -85,6% | tak, szybciej/mniej |
| FsStore nofsync commit_group | 5,33 (4,72–5,57) ms | 3,92 (3,87–3,96) ms | -26,4% | tak, szybciej/mniej |
| FsStore nofsync checkpoint | 38,0 (36,1–41,2) ms | 36,6 (36,0–38,8) ms | -3,8% | nie (w rozrzucie) |
| FsStore nofsync reopen | 39,4 (38,8–40,2) ms | 40,0 (39,5–41,4) ms | +1,4% | nie (w rozrzucie) |
| sync round durableServer | 720 (688–751) µs | 693 (667–708) µs | -3,7% | nie (w rozrzucie) |
| sync round memoryServer | 565 (553–583) µs | 564 (544–611) µs | -0,2% | nie (w rozrzucie) |

