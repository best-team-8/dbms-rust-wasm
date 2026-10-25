### native – native (n = runs)

| Operation | flatsql-native |
|---|---:|
| insert | 205.51 ms [204.72 ms–207.83 ms] n=10 |
| point_lookup | 1.21 µs [1.17 µs–1.25 µs] n=10 |
| index_lookup | 0.70 µs [0.70 µs–0.73 µs] n=10 |
| range_scan | 49.5 µs [48.6 µs–51.1 µs] n=10 |
| full_scan | 5.45 ms [5.41 ms–5.59 ms] n=10 |
| group_by | 15.48 ms [15.43 ms–15.65 ms] n=10 |
| top_k | 5.97 ms [5.94 ms–6.04 ms] n=10 |
| materialize_10k | 3.70 ms [3.65 ms–3.80 ms] n=10 |
| update_10pct | 29.75 ms [29.66 ms–30.28 ms] n=10 |
| delete_10pct | 24.02 ms [23.83 ms–24.34 ms] n=10 |
| snapshot | 25.74 ms [25.46 ms–26.11 ms] n=10 |
| restore | 111.10 ms [110.03 ms–112.82 ms] n=10 |

### node – memory (n = runs)

| Operation | flatsql | flatsql-json |
|---|---:|---:|
| insert | 377.20 ms [372.41 ms–383.81 ms] n=10 | 378.76 ms [375.41 ms–385.66 ms] n=10 |
| point_lookup | 8.33 µs [8.19 µs–8.46 µs] n=10 | 4.91 µs [4.78 µs–5.03 µs] n=10 |
| index_lookup | 5.98 µs [5.88 µs–6.27 µs] n=10 | 3.03 µs [2.99 µs–3.83 µs] n=10 |
| range_scan | 58.2 µs [54.5 µs–64.1 µs] n=10 | 53.7 µs [52.9 µs–54.7 µs] n=10 |
| full_scan | 5.17 ms [5.15 ms–5.21 ms] n=10 | 5.08 ms [5.04 ms–5.14 ms] n=10 |
| group_by | 17.67 ms [16.73 ms–18.04 ms] n=10 | 16.08 ms [16.04 ms–16.34 ms] n=10 |
| top_k | 5.71 ms [5.60 ms–5.99 ms] n=10 | 5.80 ms [5.67 ms–6.12 ms] n=10 |
| materialize_10k | 9.92 ms [9.83 ms–10.09 ms] n=10 | 10.15 ms [9.93 ms–10.31 ms] n=10 |
| update_10pct | 38.86 ms [33.06 ms–45.29 ms] n=10 | 40.49 ms [35.24 ms–43.02 ms] n=10 |
| delete_10pct | 31.24 ms [30.64 ms–34.35 ms] n=10 | 30.37 ms [30.05 ms–31.40 ms] n=10 |
| snapshot | 37.31 ms [36.39 ms–38.45 ms] n=10 | 38.65 ms [37.48 ms–39.97 ms] n=10 |
| restore | 123.69 ms [121.17 ms–125.13 ms] n=10 | 124.81 ms [123.11 ms–138.76 ms] n=10 |

