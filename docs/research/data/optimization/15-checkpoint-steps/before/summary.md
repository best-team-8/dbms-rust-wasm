### native – native (n = runs)

| Operation | flatsql-native |
|---|---:|
| insert | 158.02 ms [157.38 ms–159.53 ms] n=10 |
| point_lookup | 0.97 µs [0.94 µs–0.98 µs] n=10 |
| index_lookup | 0.71 µs [0.70 µs–0.71 µs] n=10 |
| range_scan | 50.0 µs [49.9 µs–51.6 µs] n=10 |
| full_scan | 5.42 ms [5.38 ms–5.50 ms] n=10 |
| group_by | 10.91 ms [10.87 ms–10.96 ms] n=10 |
| top_k | 6.88 ms [6.85 ms–7.01 ms] n=10 |
| materialize_10k | 2.91 ms [2.88 ms–2.94 ms] n=10 |
| update_10pct | 14.76 ms [14.42 ms–15.11 ms] n=10 |
| delete_10pct | 14.56 ms [14.25 ms–14.73 ms] n=10 |
| snapshot | 16.79 ms [16.73 ms–16.88 ms] n=10 |
| restore | 40.76 ms [40.30 ms–41.06 ms] n=10 |

### node – memory (n = runs)

| Operation | flatsql | flatsql-json |
|---|---:|---:|
| insert | 292.09 ms [289.01 ms–293.61 ms] n=10 | 289.95 ms [287.62 ms–294.81 ms] n=10 |
| point_lookup | 2.62 µs [2.52 µs–3.13 µs] n=10 | 4.14 µs [4.13 µs–4.21 µs] n=10 |
| index_lookup | 2.27 µs [1.83 µs–2.46 µs] n=10 | 3.20 µs [3.08 µs–3.43 µs] n=10 |
| range_scan | 41.9 µs [41.4 µs–43.1 µs] n=10 | 45.5 µs [45.1 µs–47.9 µs] n=10 |
| full_scan | 4.56 ms [4.53 ms–4.63 ms] n=10 | 4.57 ms [4.52 ms–4.62 ms] n=10 |
| group_by | 9.60 ms [9.54 ms–9.65 ms] n=10 | 10.16 ms [9.97 ms–10.29 ms] n=10 |
| top_k | 5.35 ms [5.28 ms–5.43 ms] n=10 | 5.63 ms [5.57 ms–5.78 ms] n=10 |
| materialize_10k | 1.90 ms [1.86 ms–1.95 ms] n=10 | 9.32 ms [8.97 ms–9.61 ms] n=10 |
| update_10pct | 15.07 ms [14.87 ms–15.99 ms] n=10 | 14.86 ms [14.75 ms–14.97 ms] n=10 |
| delete_10pct | 18.16 ms [17.81 ms–18.45 ms] n=10 | 18.03 ms [17.71 ms–18.97 ms] n=10 |
| snapshot | 22.34 ms [22.24 ms–22.76 ms] n=10 | 22.68 ms [22.10 ms–23.45 ms] n=10 |
| restore | 48.94 ms [47.97 ms–49.72 ms] n=10 | 48.41 ms [48.04 ms–48.88 ms] n=10 |

