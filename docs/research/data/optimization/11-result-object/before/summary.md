### native – native (n = runs)

| Operation | flatsql-native |
|---|---:|
| insert | 162.83 ms [162.65 ms–164.36 ms] n=10 |
| point_lookup | 1.22 µs [1.19 µs–1.26 µs] n=10 |
| index_lookup | 0.75 µs [0.75 µs–0.75 µs] n=10 |
| range_scan | 50.0 µs [48.7 µs–52.7 µs] n=10 |
| full_scan | 5.48 ms [5.43 ms–5.66 ms] n=10 |
| group_by | 10.36 ms [10.29 ms–10.43 ms] n=10 |
| top_k | 6.04 ms [6.02 ms–6.08 ms] n=10 |
| materialize_10k | 3.84 ms [3.80 ms–3.91 ms] n=10 |
| update_10pct | 15.55 ms [15.30 ms–15.78 ms] n=10 |
| delete_10pct | 17.27 ms [16.63 ms–17.87 ms] n=10 |
| snapshot | 18.13 ms [18.02 ms–18.29 ms] n=10 |
| restore | 44.95 ms [44.45 ms–45.69 ms] n=10 |

### node – memory (n = runs)

| Operation | flatsql | flatsql-json |
|---|---:|---:|
| insert | 299.43 ms [294.78 ms–305.35 ms] n=10 | 296.81 ms [296.09 ms–297.08 ms] n=10 |
| point_lookup | 8.20 µs [8.09 µs–8.42 µs] n=10 | 4.76 µs [4.40 µs–5.42 µs] n=10 |
| index_lookup | 7.41 µs [7.18 µs–7.51 µs] n=10 | 3.15 µs [3.00 µs–3.71 µs] n=10 |
| range_scan | 57.6 µs [55.9 µs–59.3 µs] n=10 | 54.7 µs [52.8 µs–56.9 µs] n=10 |
| full_scan | 5.19 ms [5.16 ms–5.26 ms] n=10 | 5.20 ms [5.15 ms–5.31 ms] n=10 |
| group_by | 10.99 ms [10.93 ms–11.43 ms] n=10 | 11.83 ms [11.52 ms–12.18 ms] n=10 |
| top_k | 5.67 ms [5.55 ms–5.73 ms] n=10 | 5.62 ms [5.50 ms–6.04 ms] n=10 |
| materialize_10k | 10.46 ms [10.38 ms–10.51 ms] n=10 | 10.98 ms [10.76 ms–11.06 ms] n=10 |
| update_10pct | 17.39 ms [17.26 ms–17.49 ms] n=10 | 17.02 ms [16.88 ms–17.29 ms] n=10 |
| delete_10pct | 22.11 ms [21.32 ms–22.98 ms] n=10 | 22.31 ms [21.80 ms–22.52 ms] n=10 |
| snapshot | 29.97 ms [28.94 ms–30.81 ms] n=10 | 27.32 ms [26.71 ms–29.76 ms] n=10 |
| restore | 59.04 ms [57.97 ms–60.51 ms] n=10 | 60.02 ms [58.57 ms–60.40 ms] n=10 |

