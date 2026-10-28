### browser – memory (n = runs)

| Operation | flatsql | flatsql-json |
|---|---:|---:|
| insert | 268.24 ms [267.18 ms–269.10 ms] n=10 | 268.70 ms [268.10 ms–269.97 ms] n=10 |
| point_lookup | 3.11 µs [3.05 µs–3.17 µs] n=10 | 4.10 µs [4.09 µs–4.20 µs] n=10 |
| index_lookup | 1.87 µs [1.83 µs–1.87 µs] n=10 | 2.88 µs [2.86 µs–2.93 µs] n=10 |
| range_scan | 45.4 µs [43.8 µs–46.0 µs] n=10 | 49.5 µs [46.4 µs–50.9 µs] n=10 |
| full_scan | 4.42 ms [4.39 ms–4.46 ms] n=10 | 4.43 ms [4.42 ms–4.46 ms] n=10 |
| group_by | 9.32 ms [9.25 ms–9.46 ms] n=10 | 9.77 ms [9.74 ms–9.79 ms] n=10 |
| top_k | 5.33 ms [5.30 ms–5.38 ms] n=10 | 5.34 ms [5.33 ms–5.38 ms] n=10 |
| materialize_10k | 1.48 ms [1.44 ms–1.54 ms] n=10 | 7.02 ms [7.00 ms–7.17 ms] n=10 |
| update_10pct | 15.64 ms [15.29 ms–15.78 ms] n=10 | 15.20 ms [14.99 ms–15.62 ms] n=10 |
| delete_10pct | 17.26 ms [17.05 ms–17.55 ms] n=10 | 17.46 ms [17.32 ms–17.80 ms] n=10 |
| snapshot | 21.77 ms [21.27 ms–22.37 ms] n=10 | 21.56 ms [20.94 ms–21.96 ms] n=10 |
| restore | 46.42 ms [46.29 ms–47.12 ms] n=10 | 46.05 ms [45.97 ms–47.60 ms] n=10 |

### browser – durable (n = runs)

| Operation | flatsql-idb | flatsql-idb-strict |
|---|---:|---:|
| insert | 331.24 ms [323.72 ms–372.95 ms] n=10 | 339.43 ms [335.64 ms–342.82 ms] n=10 |
| point_lookup | 3.76 µs [3.54 µs–3.84 µs] n=10 | 3.61 µs [3.58 µs–3.64 µs] n=10 |
| index_lookup | 2.33 µs [2.29 µs–2.36 µs] n=10 | 2.29 µs [2.28 µs–2.38 µs] n=10 |
| range_scan | 44.7 µs [44.2 µs–46.8 µs] n=10 | 44.1 µs [43.9 µs–45.1 µs] n=10 |
| full_scan | 4.40 ms [4.40 ms–4.46 ms] n=10 | 4.43 ms [4.41 ms–4.45 ms] n=10 |
| group_by | 9.47 ms [9.42 ms–9.50 ms] n=10 | 9.43 ms [9.38 ms–9.46 ms] n=10 |
| top_k | 5.29 ms [5.28 ms–5.34 ms] n=10 | 5.32 ms [5.30 ms–5.34 ms] n=10 |
| materialize_10k | 1.47 ms [1.34 ms–1.64 ms] n=10 | 1.38 ms [1.37 ms–1.41 ms] n=10 |
| update_10pct | 25.95 ms [23.56 ms–33.89 ms] n=10 | 25.95 ms [22.63 ms–31.99 ms] n=10 |
| delete_10pct | 22.20 ms [21.87 ms–22.52 ms] n=10 | 23.43 ms [23.06 ms–24.28 ms] n=10 |
| commit_single | 376.3 µs [367.2 µs–423.5 µs] n=10 | 1.03 ms [980.0 µs–1.07 ms] n=10 |
| commit_group | 4.98 ms [4.88 ms–5.15 ms] n=10 | 5.94 ms [5.72 ms–6.05 ms] n=10 |
| reopen | 81.62 ms [81.11 ms–81.96 ms] n=10 | 81.40 ms [80.71 ms–82.12 ms] n=10 |

