### native – native (n = runs)

| Operation | flatsql-native |
|---|---:|
| insert | 165.90 ms [162.93 ms–167.67 ms] n=10 |
| point_lookup | 1.26 µs [1.24 µs–1.31 µs] n=10 |
| index_lookup | 0.78 µs [0.75 µs–0.81 µs] n=10 |
| range_scan | 49.9 µs [47.9 µs–51.4 µs] n=10 |
| full_scan | 5.55 ms [5.49 ms–5.70 ms] n=10 |
| group_by | 10.44 ms [10.38 ms–10.49 ms] n=10 |
| top_k | 6.06 ms [6.01 ms–6.09 ms] n=10 |
| materialize_10k | 3.86 ms [3.83 ms–3.91 ms] n=10 |
| update_10pct | 15.78 ms [15.37 ms–16.02 ms] n=10 |
| delete_10pct | 16.67 ms [16.01 ms–16.76 ms] n=10 |
| snapshot | 17.72 ms [17.63 ms–17.99 ms] n=10 |
| restore | 44.71 ms [44.13 ms–45.43 ms] n=10 |

### node – memory (n = runs)

| Operation | flatsql | flatsql-json |
|---|---:|---:|
| insert | 297.83 ms [294.96 ms–299.02 ms] n=10 | 295.69 ms [295.22 ms–300.64 ms] n=10 |
| point_lookup | 2.60 µs [2.56 µs–2.65 µs] n=10 | 4.44 µs [4.39 µs–4.52 µs] n=10 |
| index_lookup | 1.97 µs [1.76 µs–2.19 µs] n=10 | 3.35 µs [3.08 µs–3.98 µs] n=10 |
| range_scan | 52.0 µs [49.7 µs–53.8 µs] n=10 | 68.0 µs [66.4 µs–72.6 µs] n=10 |
| full_scan | 5.08 ms [5.05 ms–5.09 ms] n=10 | 5.14 ms [5.09 ms–5.32 ms] n=10 |
| group_by | 10.70 ms [10.55 ms–10.92 ms] n=10 | 11.00 ms [10.90 ms–11.07 ms] n=10 |
| top_k | 5.62 ms [5.52 ms–5.69 ms] n=10 | 5.75 ms [5.62 ms–5.83 ms] n=10 |
| materialize_10k | 1.93 ms [1.83 ms–1.94 ms] n=10 | 10.83 ms [10.74 ms–10.96 ms] n=10 |
| update_10pct | 16.64 ms [16.39 ms–17.19 ms] n=10 | 17.53 ms [17.16 ms–18.28 ms] n=10 |
| delete_10pct | 20.09 ms [19.88 ms–20.48 ms] n=10 | 22.58 ms [21.94 ms–23.08 ms] n=10 |
| snapshot | 26.15 ms [25.74 ms–26.59 ms] n=10 | 28.99 ms [26.88 ms–30.51 ms] n=10 |
| restore | 51.94 ms [51.86 ms–52.47 ms] n=10 | 58.24 ms [56.77 ms–59.69 ms] n=10 |

