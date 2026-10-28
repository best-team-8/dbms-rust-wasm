### native – native (n = runs)

| Operation | flatsql-native |
|---|---:|
| insert | 157.84 ms [157.51 ms–158.91 ms] n=10 |
| point_lookup | 1.15 µs [1.14 µs–1.16 µs] n=10 |
| index_lookup | 0.76 µs [0.76 µs–0.76 µs] n=10 |
| range_scan | 47.3 µs [46.4 µs–48.5 µs] n=10 |
| full_scan | 5.31 ms [5.27 ms–5.40 ms] n=10 |
| group_by | 10.03 ms [9.97 ms–10.08 ms] n=10 |
| top_k | 5.82 ms [5.81 ms–5.86 ms] n=10 |
| materialize_10k | 3.38 ms [3.36 ms–3.42 ms] n=10 |
| update_10pct | 15.00 ms [14.97 ms–15.09 ms] n=10 |
| delete_10pct | 15.76 ms [15.68 ms–16.02 ms] n=10 |
| snapshot | 17.54 ms [17.50 ms–17.58 ms] n=10 |
| restore | 44.43 ms [44.13 ms–44.98 ms] n=10 |

### node – memory (n = runs)

| Operation | flatsql | flatsql-json |
|---|---:|---:|
| insert | 286.50 ms [285.91 ms–288.58 ms] n=10 | 289.22 ms [287.54 ms–290.98 ms] n=10 |
| point_lookup | 2.78 µs [2.49 µs–3.09 µs] n=10 | 4.26 µs [4.22 µs–4.84 µs] n=10 |
| index_lookup | 1.77 µs [1.74 µs–2.16 µs] n=10 | 3.21 µs [3.04 µs–3.47 µs] n=10 |
| range_scan | 48.7 µs [48.2 µs–49.5 µs] n=10 | 51.1 µs [50.3 µs–51.6 µs] n=10 |
| full_scan | 4.91 ms [4.87 ms–5.01 ms] n=10 | 4.92 ms [4.91 ms–4.94 ms] n=10 |
| group_by | 10.32 ms [10.25 ms–10.38 ms] n=10 | 10.94 ms [10.49 ms–11.14 ms] n=10 |
| top_k | 5.33 ms [5.31 ms–5.34 ms] n=10 | 5.49 ms [5.42 ms–5.81 ms] n=10 |
| materialize_10k | 1.66 ms [1.61 ms–1.78 ms] n=10 | 10.74 ms [10.67 ms–11.10 ms] n=10 |
| update_10pct | 15.95 ms [15.89 ms–16.18 ms] n=10 | 17.16 ms [16.70 ms–17.27 ms] n=10 |
| delete_10pct | 19.12 ms [18.87 ms–19.44 ms] n=10 | 21.78 ms [21.04 ms–22.00 ms] n=10 |
| snapshot | 25.58 ms [25.39 ms–26.23 ms] n=10 | 27.38 ms [26.35 ms–30.31 ms] n=10 |
| restore | 51.02 ms [50.71 ms–51.45 ms] n=10 | 59.02 ms [58.37 ms–60.16 ms] n=10 |

