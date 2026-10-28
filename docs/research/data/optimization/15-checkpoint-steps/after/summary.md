### native – native (n = runs)

| Operation | flatsql-native |
|---|---:|
| insert | 158.70 ms [158.15 ms–160.20 ms] n=10 |
| point_lookup | 0.93 µs [0.91 µs–0.94 µs] n=10 |
| index_lookup | 0.70 µs [0.69 µs–0.71 µs] n=10 |
| range_scan | 49.6 µs [49.1 µs–50.0 µs] n=10 |
| full_scan | 5.44 ms [5.33 ms–5.47 ms] n=10 |
| group_by | 10.83 ms [10.76 ms–10.89 ms] n=10 |
| top_k | 6.81 ms [6.80 ms–6.90 ms] n=10 |
| materialize_10k | 2.87 ms [2.86 ms–2.91 ms] n=10 |
| update_10pct | 14.78 ms [14.54 ms–14.86 ms] n=10 |
| delete_10pct | 14.57 ms [14.24 ms–14.79 ms] n=10 |
| snapshot | 17.32 ms [17.23 ms–17.80 ms] n=10 |
| restore | 40.86 ms [40.77 ms–41.13 ms] n=10 |

### node – memory (n = runs)

| Operation | flatsql | flatsql-json |
|---|---:|---:|
| insert | 292.64 ms [289.27 ms–296.92 ms] n=10 | 291.60 ms [285.79 ms–292.94 ms] n=10 |
| point_lookup | 2.54 µs [2.49 µs–2.63 µs] n=10 | 4.24 µs [4.10 µs–4.70 µs] n=10 |
| index_lookup | 1.83 µs [1.74 µs–2.44 µs] n=10 | 3.25 µs [3.08 µs–3.55 µs] n=10 |
| range_scan | 42.8 µs [40.8 µs–45.3 µs] n=10 | 43.9 µs [43.6 µs–44.8 µs] n=10 |
| full_scan | 4.58 ms [4.52 ms–4.65 ms] n=10 | 4.55 ms [4.53 ms–4.60 ms] n=10 |
| group_by | 9.67 ms [9.62 ms–9.77 ms] n=10 | 10.24 ms [9.99 ms–10.26 ms] n=10 |
| top_k | 5.28 ms [5.26 ms–5.44 ms] n=10 | 5.71 ms [5.39 ms–5.84 ms] n=10 |
| materialize_10k | 1.89 ms [1.78 ms–1.97 ms] n=10 | 9.08 ms [8.73 ms–9.34 ms] n=10 |
| update_10pct | 15.67 ms [15.43 ms–16.41 ms] n=10 | 15.43 ms [15.24 ms–16.09 ms] n=10 |
| delete_10pct | 18.36 ms [18.02 ms–18.89 ms] n=10 | 17.89 ms [17.68 ms–19.58 ms] n=10 |
| snapshot | 22.73 ms [22.54 ms–23.26 ms] n=10 | 22.26 ms [21.86 ms–23.82 ms] n=10 |
| restore | 48.93 ms [48.21 ms–49.60 ms] n=10 | 48.96 ms [47.81 ms–50.10 ms] n=10 |

