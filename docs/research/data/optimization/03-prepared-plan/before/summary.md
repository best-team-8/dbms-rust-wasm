### native – native (n = runs)

| Operation | flatsql-native |
|---|---:|
| insert | 246.50 ms [232.70 ms–253.05 ms] n=10 |
| point_lookup | 2.71 µs [2.59 µs–2.87 µs] n=10 |
| index_lookup | 1.73 µs [1.68 µs–2.25 µs] n=10 |
| range_scan | 58.3 µs [52.8 µs–62.2 µs] n=10 |
| full_scan | 8.57 ms [8.36 ms–8.78 ms] n=10 |
| group_by | 17.79 ms [17.08 ms–20.40 ms] n=10 |
| top_k | 6.17 ms [5.96 ms–6.78 ms] n=10 |
| materialize_10k | 3.99 ms [3.84 ms–4.24 ms] n=10 |
| update_10pct | 34.57 ms [32.70 ms–36.09 ms] n=10 |
| delete_10pct | 27.37 ms [26.78 ms–29.31 ms] n=10 |
| snapshot | 29.38 ms [28.37 ms–31.70 ms] n=10 |
| restore | 127.16 ms [125.90 ms–130.50 ms] n=10 |

### node – memory (n = runs)

| Operation | flatsql | flatsql-json |
|---|---:|---:|
| insert | 459.46 ms [443.02 ms–501.18 ms] n=10 | 475.08 ms [464.08 ms–490.92 ms] n=10 |
| point_lookup | 10.1 µs [9.96 µs–10.6 µs] n=10 | 6.62 µs [6.46 µs–6.86 µs] n=10 |
| index_lookup | 9.79 µs [9.72 µs–9.99 µs] n=10 | 5.38 µs [4.70 µs–6.25 µs] n=10 |
| range_scan | 57.5 µs [55.4 µs–63.3 µs] n=10 | 54.2 µs [52.4 µs–56.1 µs] n=10 |
| full_scan | 9.45 ms [8.80 ms–10.52 ms] n=10 | 8.49 ms [8.32 ms–8.54 ms] n=10 |
| group_by | 19.09 ms [18.23 ms–25.04 ms] n=10 | 17.71 ms [17.39 ms–18.76 ms] n=10 |
| top_k | 6.31 ms [5.96 ms–6.52 ms] n=10 | 6.32 ms [5.84 ms–6.77 ms] n=10 |
| materialize_10k | 11.77 ms [11.04 ms–13.60 ms] n=10 | 11.42 ms [10.97 ms–12.40 ms] n=10 |
| update_10pct | 41.89 ms [38.13 ms–52.90 ms] n=10 | 44.25 ms [42.31 ms–46.24 ms] n=10 |
| delete_10pct | 33.76 ms [32.23 ms–36.21 ms] n=10 | 33.20 ms [31.26 ms–36.29 ms] n=10 |
| snapshot | 39.47 ms [38.42 ms–41.10 ms] n=10 | 39.04 ms [38.58 ms–39.69 ms] n=10 |
| restore | 131.98 ms [126.99 ms–134.99 ms] n=10 | 136.67 ms [133.95 ms–138.79 ms] n=10 |

