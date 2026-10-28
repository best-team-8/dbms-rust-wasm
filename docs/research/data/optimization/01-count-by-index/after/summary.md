### native – native (n = runs)

| Operation | flatsql-native |
|---|---:|
| insert | 219.18 ms [217.36 ms–222.14 ms] n=10 |
| point_lookup | 2.40 µs [2.38 µs–2.46 µs] n=10 |
| index_lookup | 1.66 µs [1.65 µs–1.68 µs] n=10 |
| range_scan | 86.4 µs [83.6 µs–91.9 µs] n=10 |
| full_scan | 8.27 ms [8.18 ms–8.37 ms] n=10 |
| group_by | 15.32 ms [15.23 ms–15.42 ms] n=10 |
| top_k | 5.79 ms [5.70 ms–5.92 ms] n=10 |
| materialize_10k | 3.88 ms [3.83 ms–3.96 ms] n=10 |
| update_10pct | 31.98 ms [31.81 ms–33.44 ms] n=10 |
| delete_10pct | 25.78 ms [25.69 ms–26.32 ms] n=10 |
| snapshot | 25.93 ms [25.79 ms–26.57 ms] n=10 |
| restore | 115.40 ms [114.00 ms–117.66 ms] n=10 |

### node – memory (n = runs)

| Operation | flatsql | flatsql-json |
|---|---:|---:|
| insert | 401.57 ms [399.50 ms–408.46 ms] n=10 | 412.68 ms [406.45 ms–427.55 ms] n=10 |
| point_lookup | 9.76 µs [9.65 µs–10.0 µs] n=10 | 6.40 µs [6.22 µs–6.56 µs] n=10 |
| index_lookup | 9.47 µs [8.67 µs–10.1 µs] n=10 | 5.58 µs [4.61 µs–6.76 µs] n=10 |
| range_scan | 92.3 µs [89.5 µs–93.6 µs] n=10 | 87.8 µs [81.8 µs–90.2 µs] n=10 |
| full_scan | 8.36 ms [8.22 ms–8.48 ms] n=10 | 8.16 ms [8.11 ms–8.24 ms] n=10 |
| group_by | 17.07 ms [16.15 ms–17.71 ms] n=10 | 16.59 ms [16.10 ms–17.79 ms] n=10 |
| top_k | 5.71 ms [5.62 ms–6.21 ms] n=10 | 5.65 ms [5.59 ms–5.70 ms] n=10 |
| materialize_10k | 10.06 ms [9.96 ms–10.43 ms] n=10 | 10.29 ms [9.98 ms–10.76 ms] n=10 |
| update_10pct | 43.30 ms [38.31 ms–49.26 ms] n=10 | 37.52 ms [36.78 ms–39.20 ms] n=10 |
| delete_10pct | 31.36 ms [30.87 ms–33.46 ms] n=10 | 31.73 ms [30.74 ms–33.42 ms] n=10 |
| snapshot | 38.24 ms [37.94 ms–38.62 ms] n=10 | 37.68 ms [37.33 ms–38.63 ms] n=10 |
| restore | 124.91 ms [123.86 ms–136.03 ms] n=10 | 121.69 ms [121.27 ms–124.20 ms] n=10 |

