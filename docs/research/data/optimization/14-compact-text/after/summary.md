### native – native (n = runs)

| Operation | flatsql-native |
|---|---:|
| insert | 158.22 ms [156.65 ms–159.22 ms] n=10 |
| point_lookup | 0.96 µs [0.93 µs–1.11 µs] n=10 |
| index_lookup | 0.69 µs [0.69 µs–0.71 µs] n=10 |
| range_scan | 49.2 µs [48.7 µs–50.9 µs] n=10 |
| full_scan | 5.43 ms [5.38 ms–5.49 ms] n=10 |
| group_by | 10.80 ms [10.73 ms–10.86 ms] n=10 |
| top_k | 6.87 ms [6.80 ms–6.88 ms] n=10 |
| materialize_10k | 2.87 ms [2.85 ms–2.88 ms] n=10 |
| update_10pct | 14.65 ms [14.38 ms–14.75 ms] n=10 |
| delete_10pct | 14.50 ms [14.27 ms–14.63 ms] n=10 |
| snapshot | 16.72 ms [16.62 ms–16.81 ms] n=10 |
| restore | 40.71 ms [39.72 ms–41.03 ms] n=10 |

### node – memory (n = runs)

| Operation | flatsql | flatsql-json |
|---|---:|---:|
| insert | 288.24 ms [287.47 ms–289.86 ms] n=10 | 288.90 ms [286.98 ms–291.04 ms] n=10 |
| point_lookup | 2.48 µs [2.44 µs–2.53 µs] n=10 | 4.20 µs [4.14 µs–4.42 µs] n=10 |
| index_lookup | 1.83 µs [1.75 µs–2.21 µs] n=10 | 3.30 µs [3.05 µs–3.68 µs] n=10 |
| range_scan | 40.7 µs [40.3 µs–41.6 µs] n=10 | 44.0 µs [43.7 µs–45.4 µs] n=10 |
| full_scan | 4.53 ms [4.51 ms–4.54 ms] n=10 | 4.53 ms [4.52 ms–4.54 ms] n=10 |
| group_by | 9.60 ms [9.50 ms–9.77 ms] n=10 | 9.79 ms [9.72 ms–10.05 ms] n=10 |
| top_k | 5.30 ms [5.27 ms–5.34 ms] n=10 | 5.66 ms [5.62 ms–5.70 ms] n=10 |
| materialize_10k | 1.88 ms [1.77 ms–2.04 ms] n=10 | 9.23 ms [9.00 ms–9.49 ms] n=10 |
| update_10pct | 15.20 ms [14.79 ms–15.99 ms] n=10 | 15.47 ms [15.36 ms–15.78 ms] n=10 |
| delete_10pct | 18.25 ms [17.96 ms–18.78 ms] n=10 | 19.95 ms [19.80 ms–20.87 ms] n=10 |
| snapshot | 22.54 ms [22.37 ms–22.77 ms] n=10 | 23.62 ms [23.07 ms–27.91 ms] n=10 |
| restore | 48.44 ms [48.19 ms–48.95 ms] n=10 | 55.24 ms [55.06 ms–57.03 ms] n=10 |

