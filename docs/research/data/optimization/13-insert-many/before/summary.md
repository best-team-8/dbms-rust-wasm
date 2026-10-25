### native – native (n = runs)

| Operation | flatsql-native |
|---|---:|
| insert | 163.42 ms [161.52 ms–164.95 ms] n=10 |
| point_lookup | 1.22 µs [1.20 µs–1.23 µs] n=10 |
| index_lookup | 0.77 µs [0.77 µs–0.80 µs] n=10 |
| range_scan | 49.2 µs [48.3 µs–52.8 µs] n=10 |
| full_scan | 5.50 ms [5.46 ms–5.54 ms] n=10 |
| group_by | 10.48 ms [10.35 ms–10.65 ms] n=10 |
| top_k | 5.89 ms [5.83 ms–6.02 ms] n=10 |
| materialize_10k | 3.73 ms [3.70 ms–3.79 ms] n=10 |
| update_10pct | 15.53 ms [15.42 ms–15.73 ms] n=10 |
| delete_10pct | 15.90 ms [15.78 ms–16.81 ms] n=10 |
| snapshot | 18.21 ms [17.82 ms–19.05 ms] n=10 |
| restore | 45.46 ms [44.43 ms–46.69 ms] n=10 |

### node – memory (n = runs)

| Operation | flatsql | flatsql-json |
|---|---:|---:|
| insert | 298.17 ms [296.61 ms–301.86 ms] n=10 | 299.48 ms [294.09 ms–304.32 ms] n=10 |
| point_lookup | 2.65 µs [2.57 µs–2.72 µs] n=10 | 4.57 µs [4.48 µs–4.80 µs] n=10 |
| index_lookup | 1.87 µs [1.74 µs–2.01 µs] n=10 | 3.14 µs [3.08 µs–3.26 µs] n=10 |
| range_scan | 50.3 µs [49.8 µs–52.6 µs] n=10 | 67.1 µs [66.4 µs–68.6 µs] n=10 |
| full_scan | 5.12 ms [5.09 ms–5.15 ms] n=10 | 5.12 ms [5.11 ms–5.20 ms] n=10 |
| group_by | 10.78 ms [10.60 ms–10.87 ms] n=10 | 11.22 ms [11.12 ms–11.66 ms] n=10 |
| top_k | 5.58 ms [5.55 ms–5.69 ms] n=10 | 5.56 ms [5.51 ms–5.63 ms] n=10 |
| materialize_10k | 1.97 ms [1.87 ms–2.07 ms] n=10 | 11.30 ms [10.82 ms–11.52 ms] n=10 |
| update_10pct | 16.93 ms [16.75 ms–17.32 ms] n=10 | 17.48 ms [17.29 ms–17.66 ms] n=10 |
| delete_10pct | 20.44 ms [19.53 ms–21.58 ms] n=10 | 21.98 ms [21.37 ms–22.35 ms] n=10 |
| snapshot | 26.12 ms [26.08 ms–27.17 ms] n=10 | 29.06 ms [27.55 ms–30.53 ms] n=10 |
| restore | 54.16 ms [52.86 ms–54.71 ms] n=10 | 59.41 ms [58.55 ms–60.86 ms] n=10 |

