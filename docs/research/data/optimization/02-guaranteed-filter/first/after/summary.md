### native – native (n = runs)

| Operation | flatsql-native |
|---|---:|
| insert | 208.32 ms [204.81 ms–210.40 ms] n=10 |
| point_lookup | 2.25 µs [2.22 µs–2.26 µs] n=10 |
| index_lookup | 1.65 µs [1.64 µs–1.73 µs] n=10 |
| range_scan | 51.5 µs [50.9 µs–52.1 µs] n=10 |
| full_scan | 7.57 ms [7.45 ms–7.66 ms] n=10 |
| group_by | 15.25 ms [15.17 ms–15.42 ms] n=10 |
| top_k | 5.70 ms [5.66 ms–5.74 ms] n=10 |
| materialize_10k | 3.47 ms [3.44 ms–3.50 ms] n=10 |
| update_10pct | 31.21 ms [31.01 ms–31.57 ms] n=10 |
| delete_10pct | 25.16 ms [25.12 ms–25.82 ms] n=10 |
| snapshot | 25.38 ms [25.26 ms–25.71 ms] n=10 |
| restore | 109.78 ms [109.39 ms–110.17 ms] n=10 |

### node – memory (n = runs)

| Operation | flatsql | flatsql-json |
|---|---:|---:|
| insert | 367.68 ms [366.58 ms–371.79 ms] n=10 | 367.45 ms [364.99 ms–370.91 ms] n=10 |
| point_lookup | 9.13 µs [9.03 µs–9.17 µs] n=10 | 6.06 µs [5.92 µs–6.20 µs] n=10 |
| index_lookup | 9.15 µs [8.61 µs–9.30 µs] n=10 | 4.89 µs [4.87 µs–5.52 µs] n=10 |
| range_scan | 57.6 µs [57.1 µs–62.4 µs] n=10 | 55.1 µs [54.3 µs–56.9 µs] n=10 |
| full_scan | 7.99 ms [7.86 ms–8.06 ms] n=10 | 7.94 ms [7.91 ms–8.08 ms] n=10 |
| group_by | 16.02 ms [15.92 ms–16.18 ms] n=10 | 16.15 ms [15.83 ms–16.55 ms] n=10 |
| top_k | 5.96 ms [5.79 ms–6.01 ms] n=10 | 5.79 ms [5.71 ms–5.90 ms] n=10 |
| materialize_10k | 9.81 ms [9.67 ms–9.92 ms] n=10 | 9.59 ms [9.54 ms–9.74 ms] n=10 |
| update_10pct | 43.06 ms [36.21 ms–47.23 ms] n=10 | 36.34 ms [35.96 ms–37.47 ms] n=10 |
| delete_10pct | 31.15 ms [29.92 ms–31.98 ms] n=10 | 30.43 ms [29.71 ms–31.56 ms] n=10 |
| snapshot | 37.26 ms [37.01 ms–38.06 ms] n=10 | 37.78 ms [37.23 ms–38.26 ms] n=10 |
| restore | 121.55 ms [118.66 ms–123.73 ms] n=10 | 120.70 ms [118.79 ms–125.08 ms] n=10 |

