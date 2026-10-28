### native – native (n = runs)

| Operation | flatsql-native |
|---|---:|
| insert | 160.23 ms [158.85 ms–163.41 ms] n=10 |
| point_lookup | 1.18 µs [1.17 µs–1.21 µs] n=10 |
| index_lookup | 0.77 µs [0.76 µs–0.78 µs] n=10 |
| range_scan | 49.6 µs [47.8 µs–51.2 µs] n=10 |
| full_scan | 5.41 ms [5.36 ms–5.53 ms] n=10 |
| group_by | 10.13 ms [10.00 ms–10.24 ms] n=10 |
| top_k | 5.82 ms [5.78 ms–5.85 ms] n=10 |
| materialize_10k | 3.42 ms [3.39 ms–3.50 ms] n=10 |
| update_10pct | 15.22 ms [15.03 ms–15.36 ms] n=10 |
| delete_10pct | 16.24 ms [15.68 ms–16.44 ms] n=10 |
| snapshot | 17.58 ms [17.43 ms–17.70 ms] n=10 |
| restore | 44.52 ms [44.35 ms–45.16 ms] n=10 |

### node – memory (n = runs)

| Operation | flatsql | flatsql-json |
|---|---:|---:|
| insert | 297.90 ms [292.06 ms–301.30 ms] n=10 | 292.20 ms [288.83 ms–301.57 ms] n=10 |
| point_lookup | 2.60 µs [2.55 µs–3.00 µs] n=10 | 4.55 µs [4.34 µs–4.78 µs] n=10 |
| index_lookup | 2.41 µs [1.75 µs–2.75 µs] n=10 | 3.18 µs [3.12 µs–3.51 µs] n=10 |
| range_scan | 50.6 µs [49.8 µs–52.3 µs] n=10 | 52.8 µs [51.4 µs–57.2 µs] n=10 |
| full_scan | 5.08 ms [4.95 ms–5.19 ms] n=10 | 5.05 ms [5.01 ms–5.12 ms] n=10 |
| group_by | 10.51 ms [10.45 ms–10.60 ms] n=10 | 10.81 ms [10.60 ms–10.98 ms] n=10 |
| top_k | 5.45 ms [5.39 ms–5.61 ms] n=10 | 5.54 ms [5.43 ms–5.71 ms] n=10 |
| materialize_10k | 1.72 ms [1.65 ms–1.74 ms] n=10 | 10.89 ms [10.77 ms–11.22 ms] n=10 |
| update_10pct | 16.20 ms [16.04 ms–16.62 ms] n=10 | 16.28 ms [15.75 ms–17.36 ms] n=10 |
| delete_10pct | 19.40 ms [19.28 ms–19.73 ms] n=10 | 19.41 ms [19.01 ms–20.39 ms] n=10 |
| snapshot | 26.62 ms [26.07 ms–27.26 ms] n=10 | 26.25 ms [25.94 ms–27.82 ms] n=10 |
| restore | 52.08 ms [51.27 ms–53.40 ms] n=10 | 54.06 ms [51.95 ms–59.65 ms] n=10 |

