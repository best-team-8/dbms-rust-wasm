### native – native (n = runs)

| Operation | flatsql-native |
|---|---:|
| insert | 205.95 ms [204.53 ms–206.89 ms] n=10 |
| point_lookup | 1.19 µs [1.17 µs–1.23 µs] n=10 |
| index_lookup | 0.69 µs [0.69 µs–0.70 µs] n=10 |
| range_scan | 48.4 µs [47.6 µs–49.7 µs] n=10 |
| full_scan | 7.58 ms [7.56 ms–7.66 ms] n=10 |
| group_by | 15.34 ms [15.26 ms–15.37 ms] n=10 |
| top_k | 5.81 ms [5.75 ms–5.83 ms] n=10 |
| materialize_10k | 3.78 ms [3.71 ms–3.83 ms] n=10 |
| update_10pct | 32.20 ms [31.92 ms–33.60 ms] n=10 |
| delete_10pct | 25.82 ms [25.31 ms–26.22 ms] n=10 |
| snapshot | 26.22 ms [25.57 ms–26.49 ms] n=10 |
| restore | 110.91 ms [110.03 ms–112.01 ms] n=10 |

### node – memory (n = runs)

| Operation | flatsql | flatsql-json |
|---|---:|---:|
| insert | 382.77 ms [380.15 ms–391.16 ms] n=10 | 379.56 ms [373.54 ms–384.49 ms] n=10 |
| point_lookup | 8.82 µs [8.66 µs–9.06 µs] n=10 | 4.96 µs [4.78 µs–5.02 µs] n=10 |
| index_lookup | 6.84 µs [6.18 µs–7.45 µs] n=10 | 3.36 µs [3.09 µs–3.78 µs] n=10 |
| range_scan | 57.2 µs [55.7 µs–60.4 µs] n=10 | 56.7 µs [55.2 µs–61.0 µs] n=10 |
| full_scan | 8.46 ms [8.37 ms–8.56 ms] n=10 | 8.32 ms [8.21 ms–8.33 ms] n=10 |
| group_by | 16.11 ms [16.01 ms–17.08 ms] n=10 | 16.62 ms [16.32 ms–17.35 ms] n=10 |
| top_k | 5.65 ms [5.56 ms–6.13 ms] n=10 | 5.89 ms [5.72 ms–5.98 ms] n=10 |
| materialize_10k | 9.93 ms [9.78 ms–10.05 ms] n=10 | 9.84 ms [9.65 ms–10.02 ms] n=10 |
| update_10pct | 38.20 ms [37.35 ms–46.46 ms] n=10 | 46.76 ms [44.29 ms–47.67 ms] n=10 |
| delete_10pct | 32.07 ms [31.08 ms–33.73 ms] n=10 | 31.53 ms [30.96 ms–32.46 ms] n=10 |
| snapshot | 37.77 ms [36.16 ms–38.72 ms] n=10 | 37.97 ms [36.75 ms–38.61 ms] n=10 |
| restore | 121.57 ms [120.25 ms–131.88 ms] n=10 | 121.63 ms [120.39 ms–123.30 ms] n=10 |

