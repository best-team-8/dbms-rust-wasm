### native – native (n = runs)

| Operation | flatsql-native |
|---|---:|
| insert | 168.75 ms [167.09 ms–169.91 ms] n=10 |
| point_lookup | 1.37 µs [1.30 µs–1.42 µs] n=10 |
| index_lookup | 0.73 µs [0.73 µs–0.84 µs] n=10 |
| range_scan | 54.5 µs [52.7 µs–56.3 µs] n=10 |
| full_scan | 5.88 ms [5.82 ms–5.99 ms] n=10 |
| group_by | 16.74 ms [16.34 ms–16.81 ms] n=10 |
| top_k | 6.47 ms [6.38 ms–6.48 ms] n=10 |
| materialize_10k | 3.76 ms [3.72 ms–3.91 ms] n=10 |
| update_10pct | 16.76 ms [16.15 ms–17.06 ms] n=10 |
| delete_10pct | 17.10 ms [16.49 ms–17.71 ms] n=10 |
| snapshot | 18.43 ms [18.26 ms–19.37 ms] n=10 |
| restore | 49.76 ms [48.28 ms–51.17 ms] n=10 |

### node – memory (n = runs)

| Operation | flatsql | flatsql-json |
|---|---:|---:|
| insert | 312.63 ms [311.19 ms–318.79 ms] n=10 | 317.28 ms [314.95 ms–328.16 ms] n=10 |
| point_lookup | 9.11 µs [8.92 µs–9.28 µs] n=10 | 5.27 µs [5.07 µs–5.92 µs] n=10 |
| index_lookup | 12.4 µs [9.64 µs–14.3 µs] n=10 | 3.29 µs [3.15 µs–4.27 µs] n=10 |
| range_scan | 62.5 µs [60.5 µs–72.8 µs] n=10 | 65.9 µs [61.0 µs–67.1 µs] n=10 |
| full_scan | 5.64 ms [5.57 ms–5.78 ms] n=10 | 5.58 ms [5.51 ms–5.68 ms] n=10 |
| group_by | 19.23 ms [18.62 ms–20.19 ms] n=10 | 18.92 ms [18.46 ms–19.56 ms] n=10 |
| top_k | 5.99 ms [5.94 ms–6.10 ms] n=10 | 5.97 ms [5.90 ms–6.20 ms] n=10 |
| materialize_10k | 11.06 ms [10.95 ms–11.77 ms] n=10 | 11.63 ms [11.44 ms–12.22 ms] n=10 |
| update_10pct | 18.39 ms [18.06 ms–19.37 ms] n=10 | 17.78 ms [17.62 ms–18.31 ms] n=10 |
| delete_10pct | 24.76 ms [23.81 ms–25.19 ms] n=10 | 23.01 ms [22.01 ms–23.24 ms] n=10 |
| snapshot | 30.36 ms [29.52 ms–32.17 ms] n=10 | 29.69 ms [28.62 ms–32.22 ms] n=10 |
| restore | 62.81 ms [61.84 ms–67.86 ms] n=10 | 61.94 ms [60.48 ms–62.85 ms] n=10 |

