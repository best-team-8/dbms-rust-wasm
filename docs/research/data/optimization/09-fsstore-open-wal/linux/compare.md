Przed: null (null); po: po-2.15-dirty (core: group rows by one column through a hash table).
Obciążenie procesora przed pomiarem: 0,2%. Uruchomienia naprzemienne, 10 na wersję.
Moduł WebAssembly: 690050 → 690130 B, gzip -9: 259478 → 259491 B (+13 B).

### node – fsync – flatsql

| Metryka | Przed | Po | Zmiana mediany | 95% CI średnich rozłączne |
|---|---:|---:|---:|---|
| FsStore fsync insert | 563 (556–574) ms | 561 (555–571) ms | -0,4% | nie (w rozrzucie) |
| FsStore fsync commit_single | 3,14 (3,10–3,17) ms | 2,83 (2,76–2,87) ms | -10,0% | tak, szybciej/mniej |
| FsStore fsync commit_group | 10,6 (9,88–10,8) ms | 9,86 (9,67–10,3) ms | -7,0% | nie (w rozrzucie) |
| FsStore fsync checkpoint | 64,7 (64,1–65,9) ms | 65,8 (64,9–66,8) ms | +1,7% | nie (w rozrzucie) |
| FsStore fsync reopen | 66,4 (64,5–67,6) ms | 66,6 (66,6–68,4) ms | +0,4% | nie (w rozrzucie) |
| FsStore nofsync insert | 430 (428–434) ms | 436 (426–439) ms | +1,4% | nie (w rozrzucie) |
| FsStore nofsync commit_single | 432 (416–458) µs | 177 (174–188) µs | -58,9% | tak, szybciej/mniej |
| FsStore nofsync commit_group | 4,81 (4,68–5,64) ms | 4,44 (4,21–4,48) ms | -7,8% | tak, szybciej/mniej |
| FsStore nofsync checkpoint | 56,6 (55,1–58,9) ms | 59,3 (56,5–60,8) ms | +4,7% | nie (w rozrzucie) |
| FsStore nofsync reopen | 48,8 (46,6–52,6) ms | 54,2 (52,2–55,2) ms | +11,0% | nie (w rozrzucie) |
| sync round durableServer | 870 (824–882) µs | 825 (806–859) µs | -5,1% | nie (w rozrzucie) |
| sync round memoryServer | 696 (686–723) µs | 681 (666–700) µs | -2,1% | nie (w rozrzucie) |

