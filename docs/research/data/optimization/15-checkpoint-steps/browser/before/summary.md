### browser – memory (n = runs)

| Operation | flatsql | flatsql-json |
|---|---:|---:|
| insert | 268.25 ms [264.71 ms–270.80 ms] n=10 | 266.67 ms [265.40 ms–269.78 ms] n=10 |
| point_lookup | 3.06 µs [3.03 µs–3.08 µs] n=10 | 4.21 µs [4.17 µs–4.24 µs] n=10 |
| index_lookup | 1.89 µs [1.86 µs–1.91 µs] n=10 | 2.90 µs [2.87 µs–2.93 µs] n=10 |
| range_scan | 45.8 µs [43.3 µs–47.9 µs] n=10 | 46.9 µs [46.0 µs–47.6 µs] n=10 |
| full_scan | 4.41 ms [4.37 ms–4.48 ms] n=10 | 4.45 ms [4.43 ms–4.48 ms] n=10 |
| group_by | 9.40 ms [9.39 ms–9.42 ms] n=10 | 9.70 ms [9.64 ms–9.73 ms] n=10 |
| top_k | 5.36 ms [5.35 ms–5.37 ms] n=10 | 5.54 ms [5.51 ms–5.57 ms] n=10 |
| materialize_10k | 1.49 ms [1.44 ms–1.57 ms] n=10 | 7.04 ms [7.00 ms–7.10 ms] n=10 |
| update_10pct | 14.54 ms [14.31 ms–14.92 ms] n=10 | 15.25 ms [14.93 ms–15.46 ms] n=10 |
| delete_10pct | 17.56 ms [17.10 ms–17.81 ms] n=10 | 17.61 ms [17.44 ms–17.81 ms] n=10 |
| snapshot | 22.15 ms [21.45 ms–22.69 ms] n=10 | 21.57 ms [21.21 ms–21.90 ms] n=10 |
| restore | 46.27 ms [45.98 ms–47.29 ms] n=10 | 46.22 ms [45.90 ms–47.13 ms] n=10 |

### browser – durable (n = runs)

| Operation | flatsql-idb | flatsql-idb-strict |
|---|---:|---:|
| insert | 334.81 ms [331.43 ms–402.60 ms] n=10 | 349.27 ms [335.21 ms–352.76 ms] n=10 |
| point_lookup | 3.64 µs [3.62 µs–3.75 µs] n=10 | 3.75 µs [3.68 µs–3.78 µs] n=10 |
| index_lookup | 2.32 µs [2.29 µs–2.43 µs] n=10 | 2.32 µs [2.29 µs–2.35 µs] n=10 |
| range_scan | 44.8 µs [43.8 µs–46.9 µs] n=10 | 44.2 µs [43.9 µs–44.7 µs] n=10 |
| full_scan | 4.43 ms [4.41 ms–4.47 ms] n=10 | 4.41 ms [4.40 ms–4.43 ms] n=10 |
| group_by | 9.29 ms [9.25 ms–9.32 ms] n=10 | 9.29 ms [9.25 ms–9.32 ms] n=10 |
| top_k | 5.32 ms [5.30 ms–5.35 ms] n=10 | 5.32 ms [5.29 ms–5.34 ms] n=10 |
| materialize_10k | 1.59 ms [1.33 ms–1.69 ms] n=10 | 1.46 ms [1.34 ms–1.57 ms] n=10 |
| update_10pct | 25.23 ms [24.01 ms–28.31 ms] n=10 | 22.54 ms [21.60 ms–30.37 ms] n=10 |
| delete_10pct | 22.10 ms [21.75 ms–23.35 ms] n=10 | 23.17 ms [22.96 ms–23.67 ms] n=10 |
| commit_single | 358.8 µs [352.6 µs–373.2 µs] n=10 | 991.6 µs [971.7 µs–1.02 ms] n=10 |
| commit_group | 5.11 ms [4.93 ms–5.23 ms] n=10 | 5.88 ms [5.78 ms–6.09 ms] n=10 |
| reopen | 85.46 ms [85.01 ms–85.99 ms] n=10 | 86.36 ms [84.74 ms–87.05 ms] n=10 |

