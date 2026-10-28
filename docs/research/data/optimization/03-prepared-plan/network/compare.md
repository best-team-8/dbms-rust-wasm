Przed: po-2.2 (core: skip the conjuncts of WHERE that the access path guarantees); po: po-2.2-dirty (core: skip the conjuncts of WHERE that the access path guarantees).
Obciążenie procesora przed pomiarem: 7,4%. Uruchomienia naprzemienne, 10 na wersję.
Moduł WebAssembly: 590964 → 608941 B, gzip -9: 223497 → 232549 B (+9052 B).

### node – network – flatsql

| Metryka | Przed | Po | Zmiana mediany | 95% CI średnich rozłączne |
|---|---:|---:|---:|---|
| local (WASM, in-process) | 25,9 (25,0–27,8) µs | 28,1 (24,9–30,7) µs | +8,7% | nie (w rozrzucie) |
| cloud API (localhost HTTP) | 1,03 (0,99–1,06) ms | 950 (934–1227) µs | -7,5% | nie (w rozrzucie) |
| cloud API (+20 ms RTT) | 21,3 (21,2–21,3) ms | 21,3 (21,2–21,5) ms | +0,2% | nie (w rozrzucie) |
| cloud API (+50 ms RTT) | 51,8 (51,7–52,0) ms | 51,7 (51,7–51,9) ms | -0,1% | nie (w rozrzucie) |
| cloud API (+100 ms RTT) | 102 (102–102) ms | 102 (102–102) ms | +0,1% | nie (w rozrzucie) |

