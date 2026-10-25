### native – native (n = runs)

| Operation | flatsql-native |
|---|---:|
| insert | 159.87 ms [158.45 ms–161.39 ms] n=10 |
| point_lookup | 1.20 µs [1.19 µs–1.23 µs] n=10 |
| index_lookup | 0.74 µs [0.73 µs–0.76 µs] n=10 |
| range_scan | 50.0 µs [48.7 µs–52.0 µs] n=10 |
| full_scan | 5.44 ms [5.43 ms–5.50 ms] n=10 |
| group_by | 15.92 ms [15.81 ms–15.98 ms] n=10 |
| top_k | 6.00 ms [5.96 ms–6.04 ms] n=10 |
| materialize_10k | 3.80 ms [3.77 ms–3.83 ms] n=10 |
| update_10pct | 23.08 ms [22.75 ms–23.44 ms] n=10 |
| delete_10pct | 16.56 ms [16.19 ms–17.15 ms] n=10 |
| snapshot | 18.26 ms [18.11 ms–18.80 ms] n=10 |
| restore | 54.94 ms [54.56 ms–55.55 ms] n=10 |

### node – memory (n = runs)

| Operation | flatsql | flatsql-json |
|---|---:|---:|
| insert | 298.81 ms [296.74 ms–305.03 ms] n=10 | 295.56 ms [293.49 ms–300.72 ms] n=10 |
| point_lookup | 8.33 µs [8.20 µs–8.58 µs] n=10 | 4.63 µs [4.46 µs–5.28 µs] n=10 |
| index_lookup | 7.48 µs [7.01 µs–9.14 µs] n=10 | 3.13 µs [3.05 µs–3.44 µs] n=10 |
| range_scan | 56.6 µs [55.3 µs–58.6 µs] n=10 | 62.1 µs [53.5 µs–69.4 µs] n=10 |
| full_scan | 5.24 ms [5.20 ms–5.26 ms] n=10 | 5.14 ms [5.11 ms–5.22 ms] n=10 |
| group_by | 17.01 ms [16.81 ms–17.63 ms] n=10 | 16.96 ms [16.84 ms–17.51 ms] n=10 |
| top_k | 5.75 ms [5.68 ms–6.93 ms] n=10 | 5.57 ms [5.52 ms–5.63 ms] n=10 |
| materialize_10k | 10.56 ms [10.46 ms–10.93 ms] n=10 | 10.87 ms [10.68 ms–11.00 ms] n=10 |
| update_10pct | 33.84 ms [28.10 ms–38.83 ms] n=10 | 28.09 ms [26.39 ms–35.49 ms] n=10 |
| delete_10pct | 25.17 ms [23.45 ms–26.85 ms] n=10 | 23.81 ms [22.79 ms–25.84 ms] n=10 |
| snapshot | 30.77 ms [30.25 ms–31.82 ms] n=10 | 31.40 ms [30.44 ms–31.66 ms] n=10 |
| restore | 65.73 ms [64.84 ms–66.58 ms] n=10 | 63.39 ms [62.78 ms–65.04 ms] n=10 |

