### browser – memory (n = runs)

| Operation | flatsql | flatsql-json |
|---|---:|---:|
| insert | 266.55 ms [265.94 ms–268.71 ms] n=10 | 268.90 ms [266.42 ms–271.32 ms] n=10 |
| point_lookup | 3.28 µs [3.15 µs–3.30 µs] n=10 | 4.16 µs [4.13 µs–4.25 µs] n=10 |
| index_lookup | 1.90 µs [1.84 µs–1.93 µs] n=10 | 2.88 µs [2.87 µs–2.90 µs] n=10 |
| range_scan | 45.5 µs [43.7 µs–47.1 µs] n=10 | 47.4 µs [45.9 µs–49.0 µs] n=10 |
| full_scan | 4.40 ms [4.39 ms–4.42 ms] n=10 | 4.45 ms [4.43 ms–4.54 ms] n=10 |
| group_by | 9.38 ms [9.28 ms–9.53 ms] n=10 | 9.77 ms [9.71 ms–9.80 ms] n=10 |
| top_k | 5.33 ms [5.29 ms–5.39 ms] n=10 | 5.35 ms [5.33 ms–5.36 ms] n=10 |
| materialize_10k | 1.55 ms [1.49 ms–1.60 ms] n=10 | 7.06 ms [7.01 ms–7.11 ms] n=10 |
| update_10pct | 15.38 ms [15.13 ms–15.76 ms] n=10 | 15.00 ms [14.76 ms–15.35 ms] n=10 |
| delete_10pct | 17.46 ms [17.03 ms–18.17 ms] n=10 | 17.58 ms [17.01 ms–17.62 ms] n=10 |
| snapshot | 22.62 ms [22.20 ms–23.03 ms] n=10 | 21.61 ms [21.21 ms–21.90 ms] n=10 |
| restore | 46.35 ms [46.11 ms–46.87 ms] n=10 | 46.69 ms [46.37 ms–47.08 ms] n=10 |

### browser – durable (n = runs)

| Operation | flatsql-idb | flatsql-idb-strict |
|---|---:|---:|
| insert | 324.04 ms [322.69 ms–365.72 ms] n=10 | 333.88 ms [331.16 ms–335.82 ms] n=10 |
| point_lookup | 3.71 µs [3.62 µs–3.88 µs] n=10 | 3.63 µs [3.57 µs–3.72 µs] n=10 |
| index_lookup | 2.36 µs [2.31 µs–2.38 µs] n=10 | 2.31 µs [2.28 µs–2.33 µs] n=10 |
| range_scan | 44.3 µs [43.9 µs–44.5 µs] n=10 | 44.0 µs [43.6 µs–46.0 µs] n=10 |
| full_scan | 4.42 ms [4.40 ms–4.45 ms] n=10 | 4.43 ms [4.40 ms–4.43 ms] n=10 |
| group_by | 9.47 ms [9.40 ms–9.49 ms] n=10 | 9.48 ms [9.41 ms–9.52 ms] n=10 |
| top_k | 5.33 ms [5.32 ms–5.34 ms] n=10 | 5.29 ms [5.26 ms–5.30 ms] n=10 |
| materialize_10k | 1.50 ms [1.39 ms–1.69 ms] n=10 | 1.40 ms [1.38 ms–1.56 ms] n=10 |
| update_10pct | 26.76 ms [21.98 ms–34.50 ms] n=10 | 31.75 ms [27.10 ms–36.66 ms] n=10 |
| delete_10pct | 21.47 ms [21.25 ms–22.28 ms] n=10 | 23.38 ms [22.87 ms–23.54 ms] n=10 |
| commit_single | 363.9 µs [354.5 µs–374.7 µs] n=10 | 979.2 µs [956.8 µs–1.03 ms] n=10 |
| commit_group | 5.07 ms [5.02 ms–5.13 ms] n=10 | 5.89 ms [5.69 ms–5.94 ms] n=10 |
| reopen | 81.43 ms [81.29 ms–81.56 ms] n=10 | 81.39 ms [81.13 ms–81.59 ms] n=10 |

### browser – worker (n = runs)

| Operation | flatsql-worker | flatsql-worker-idb |
|---|---:|---:|
| insert | 271.21 ms [269.07 ms–274.38 ms] n=10 | 331.73 ms [326.89 ms–336.52 ms] n=10 |
| point_lookup | 48.1 µs [46.1 µs–48.6 µs] n=10 | 49.2 µs [47.6 µs–50.5 µs] n=10 |
| index_lookup | 45.8 µs [43.2 µs–48.5 µs] n=10 | 46.8 µs [43.6 µs–48.4 µs] n=10 |
| range_scan | 96.7 µs [93.0 µs–100.3 µs] n=10 | 95.9 µs [89.8 µs–97.6 µs] n=10 |
| full_scan | 5.21 ms [5.09 ms–5.30 ms] n=10 | 5.47 ms [5.31 ms–5.52 ms] n=10 |
| group_by | 10.65 ms [10.48 ms–10.77 ms] n=10 | 10.67 ms [10.63 ms–10.75 ms] n=10 |
| top_k | 6.13 ms [5.99 ms–6.21 ms] n=10 | 6.39 ms [6.20 ms–6.56 ms] n=10 |
| materialize_10k | 13.54 ms [13.41 ms–13.62 ms] n=10 | 13.83 ms [13.54 ms–13.94 ms] n=10 |
| update_10pct | 15.75 ms [15.55 ms–15.88 ms] n=10 | 25.70 ms [20.88 ms–32.35 ms] n=10 |
| delete_10pct | 17.69 ms [17.35 ms–18.05 ms] n=10 | 21.57 ms [21.07 ms–22.06 ms] n=10 |
| commit_single | – | 448.1 µs [423.8 µs–465.4 µs] n=10 |
| commit_group | – | 5.21 ms [5.15 ms–5.27 ms] n=10 |
| reopen | – | 88.75 ms [86.92 ms–89.50 ms] n=10 |

### native – native (n = runs)

| Operation | flatsql-native |
|---|---:|
| insert | 161.52 ms [160.95 ms–163.13 ms] n=10 |
| point_lookup | 0.91 µs [0.90 µs–0.93 µs] n=10 |
| index_lookup | 0.71 µs [0.70 µs–0.71 µs] n=10 |
| range_scan | 48.7 µs [48.0 µs–49.5 µs] n=10 |
| full_scan | 5.35 ms [5.32 ms–5.40 ms] n=10 |
| group_by | 10.82 ms [10.78 ms–10.88 ms] n=10 |
| top_k | 6.82 ms [6.79 ms–6.85 ms] n=10 |
| materialize_10k | 2.86 ms [2.84 ms–2.88 ms] n=10 |
| update_10pct | 14.64 ms [14.51 ms–14.78 ms] n=10 |
| delete_10pct | 14.26 ms [14.21 ms–14.46 ms] n=10 |
| snapshot | 17.28 ms [17.14 ms–17.36 ms] n=10 |
| restore | 40.42 ms [39.97 ms–40.87 ms] n=10 |

### node – memory (n = runs)

| Operation | flatsql | flatsql-json |
|---|---:|---:|
| insert | 285.54 ms [284.70 ms–287.10 ms] n=10 | 285.30 ms [284.42 ms–286.79 ms] n=10 |
| point_lookup | 2.45 µs [2.44 µs–2.47 µs] n=10 | 4.11 µs [4.07 µs–4.13 µs] n=10 |
| index_lookup | 2.09 µs [1.79 µs–2.46 µs] n=10 | 3.40 µs [3.06 µs–4.15 µs] n=10 |
| range_scan | 41.0 µs [40.7 µs–41.5 µs] n=10 | 44.1 µs [43.5 µs–44.8 µs] n=10 |
| full_scan | 4.49 ms [4.47 ms–4.52 ms] n=10 | 4.51 ms [4.48 ms–4.54 ms] n=10 |
| group_by | 9.58 ms [9.52 ms–9.87 ms] n=10 | 10.01 ms [9.88 ms–10.08 ms] n=10 |
| top_k | 5.27 ms [5.23 ms–5.52 ms] n=10 | 5.34 ms [5.29 ms–5.66 ms] n=10 |
| materialize_10k | 1.65 ms [1.64 ms–1.75 ms] n=10 | 8.76 ms [8.71 ms–8.86 ms] n=10 |
| update_10pct | 15.37 ms [15.25 ms–15.50 ms] n=10 | 15.22 ms [15.17 ms–15.51 ms] n=10 |
| delete_10pct | 17.62 ms [17.51 ms–17.98 ms] n=10 | 17.69 ms [17.50 ms–17.75 ms] n=10 |
| snapshot | 22.06 ms [21.96 ms–22.71 ms] n=10 | 21.53 ms [21.48 ms–22.38 ms] n=10 |
| restore | 48.20 ms [48.04 ms–48.50 ms] n=10 | 48.23 ms [47.92 ms–48.97 ms] n=10 |

