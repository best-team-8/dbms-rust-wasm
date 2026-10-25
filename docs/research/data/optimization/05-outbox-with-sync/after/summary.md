### native – native (n = runs)

| Operation | flatsql-native |
|---|---:|
| insert | 159.87 ms [157.04 ms–160.08 ms] n=10 |
| point_lookup | 1.24 µs [1.20 µs–1.24 µs] n=10 |
| index_lookup | 0.74 µs [0.73 µs–0.75 µs] n=10 |
| range_scan | 48.2 µs [47.5 µs–51.7 µs] n=10 |
| full_scan | 5.52 ms [5.46 ms–5.58 ms] n=10 |
| group_by | 15.74 ms [15.59 ms–15.90 ms] n=10 |
| top_k | 5.95 ms [5.92 ms–6.02 ms] n=10 |
| materialize_10k | 3.80 ms [3.78 ms–3.86 ms] n=10 |
| update_10pct | 22.88 ms [22.79 ms–23.22 ms] n=10 |
| delete_10pct | 16.36 ms [16.20 ms–16.60 ms] n=10 |
| snapshot | 18.18 ms [18.10 ms–18.27 ms] n=10 |
| restore | 54.69 ms [54.34 ms–55.08 ms] n=10 |

### node – memory (n = runs)

| Operation | flatsql | flatsql-json |
|---|---:|---:|
| insert | 294.06 ms [292.26 ms–295.85 ms] n=10 | 296.14 ms [293.89 ms–298.07 ms] n=10 |
| point_lookup | 8.17 µs [8.03 µs–8.33 µs] n=10 | 4.47 µs [4.37 µs–4.58 µs] n=10 |
| index_lookup | 7.47 µs [7.32 µs–10.6 µs] n=10 | 3.25 µs [3.13 µs–3.85 µs] n=10 |
| range_scan | 57.4 µs [54.9 µs–60.6 µs] n=10 | 62.2 µs [53.4 µs–68.5 µs] n=10 |
| full_scan | 5.11 ms [5.07 ms–5.18 ms] n=10 | 5.14 ms [5.09 ms–5.21 ms] n=10 |
| group_by | 17.96 ms [17.03 ms–18.71 ms] n=10 | 17.10 ms [16.90 ms–18.56 ms] n=10 |
| top_k | 5.51 ms [5.46 ms–5.80 ms] n=10 | 5.52 ms [5.51 ms–5.55 ms] n=10 |
| materialize_10k | 10.59 ms [10.42 ms–10.74 ms] n=10 | 10.84 ms [10.57 ms–11.32 ms] n=10 |
| update_10pct | 36.89 ms [29.10 ms–38.10 ms] n=10 | 26.87 ms [26.29 ms–27.73 ms] n=10 |
| delete_10pct | 25.12 ms [23.39 ms–27.76 ms] n=10 | 25.94 ms [25.59 ms–26.41 ms] n=10 |
| snapshot | 31.19 ms [30.11 ms–32.36 ms] n=10 | 31.07 ms [30.73 ms–31.47 ms] n=10 |
| restore | 68.50 ms [66.26 ms–74.63 ms] n=10 | 64.61 ms [62.86 ms–66.77 ms] n=10 |

