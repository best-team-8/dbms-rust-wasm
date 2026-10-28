### browser – memory (n = runs)

| Operation | flatsql | flatsql-json |
|---|---:|---:|
| insert | 357.24 ms [354.82 ms–360.83 ms] n=10 | 334.16 ms [332.27 ms–336.46 ms] n=10 |
| point_lookup | 8.78 µs [8.72 µs–8.90 µs] n=10 | 5.50 µs [5.47 µs–5.54 µs] n=10 |
| index_lookup | 437.2 µs [428.4 µs–449.1 µs] n=10 | 426.7 µs [422.7 µs–440.5 µs] n=10 |
| range_scan | 77.6 µs [77.4 µs–79.5 µs] n=10 | 77.3 µs [76.6 µs–77.6 µs] n=10 |
| full_scan | 7.83 ms [7.78 ms–7.90 ms] n=10 | 7.79 ms [7.75 ms–7.81 ms] n=10 |
| group_by | 15.61 ms [15.36 ms–15.73 ms] n=10 | 15.41 ms [15.25 ms–15.46 ms] n=10 |
| top_k | 5.44 ms [5.38 ms–5.56 ms] n=10 | 5.42 ms [5.35 ms–5.50 ms] n=10 |
| materialize_10k | 11.38 ms [11.26 ms–11.43 ms] n=10 | 7.80 ms [7.76 ms–7.84 ms] n=10 |
| update_10pct | 30.86 ms [30.70 ms–31.07 ms] n=10 | 30.69 ms [30.47 ms–30.92 ms] n=10 |
| delete_10pct | 26.92 ms [26.58 ms–27.54 ms] n=10 | 26.83 ms [26.60 ms–27.11 ms] n=10 |
| snapshot | 32.21 ms [31.81 ms–32.33 ms] n=10 | 32.48 ms [32.34 ms–33.10 ms] n=10 |
| restore | 114.71 ms [114.03 ms–115.68 ms] n=10 | 115.21 ms [114.27 ms–115.78 ms] n=10 |

### browser – durable (n = runs)

| Operation | flatsql-idb | flatsql-idb-strict |
|---|---:|---:|
| insert | 441.66 ms [436.26 ms–451.55 ms] n=10 | 453.16 ms [445.20 ms–454.21 ms] n=10 |
| point_lookup | 9.47 µs [9.44 µs–9.54 µs] n=10 | 9.59 µs [9.45 µs–9.70 µs] n=10 |
| index_lookup | 437.5 µs [433.2 µs–450.9 µs] n=10 | 442.0 µs [437.2 µs–453.2 µs] n=10 |
| range_scan | 83.0 µs [82.0 µs–85.5 µs] n=10 | 82.5 µs [80.5 µs–83.4 µs] n=10 |
| full_scan | 8.92 ms [8.87 ms–8.97 ms] n=10 | 8.94 ms [8.89 ms–9.05 ms] n=10 |
| group_by | 16.34 ms [16.17 ms–16.94 ms] n=10 | 16.16 ms [16.04 ms–16.44 ms] n=10 |
| top_k | 5.47 ms [5.41 ms–5.53 ms] n=10 | 5.46 ms [5.43 ms–5.48 ms] n=10 |
| materialize_10k | 13.91 ms [13.84 ms–14.06 ms] n=10 | 13.91 ms [13.86 ms–13.97 ms] n=10 |
| update_10pct | 37.87 ms [37.05 ms–39.36 ms] n=10 | 40.06 ms [38.84 ms–44.72 ms] n=10 |
| delete_10pct | 32.91 ms [32.49 ms–33.19 ms] n=10 | 34.26 ms [33.98 ms–34.66 ms] n=10 |
| commit_single | 376.1 µs [363.8 µs–398.6 µs] n=10 | 974.7 µs [956.2 µs–1.05 ms] n=10 |
| commit_group | 5.86 ms [5.78 ms–6.02 ms] n=10 | 6.62 ms [6.35 ms–6.81 ms] n=10 |
| reopen | 173.31 ms [172.08 ms–174.09 ms] n=10 | 173.04 ms [171.58 ms–174.59 ms] n=10 |

### browser – worker (n = runs)

| Operation | flatsql-worker | flatsql-worker-idb |
|---|---:|---:|
| insert | 349.67 ms [348.27 ms–352.93 ms] n=10 | 449.03 ms [448.03 ms–454.48 ms] n=10 |
| point_lookup | 59.2 µs [57.3 µs–60.1 µs] n=10 | 59.4 µs [58.2 µs–61.2 µs] n=10 |
| index_lookup | 566.0 µs [558.0 µs–569.2 µs] n=10 | 560.9 µs [545.1 µs–571.4 µs] n=10 |
| range_scan | 141.5 µs [135.1 µs–146.6 µs] n=10 | 139.4 µs [135.9 µs–155.3 µs] n=10 |
| full_scan | 8.52 ms [8.45 ms–8.57 ms] n=10 | 9.50 ms [9.46 ms–9.54 ms] n=10 |
| group_by | 18.05 ms [17.98 ms–18.36 ms] n=10 | 17.27 ms [16.80 ms–18.54 ms] n=10 |
| top_k | 6.05 ms [5.95 ms–6.11 ms] n=10 | 6.02 ms [5.94 ms–6.13 ms] n=10 |
| materialize_10k | 23.18 ms [23.05 ms–23.38 ms] n=10 | 25.66 ms [25.59 ms–26.14 ms] n=10 |
| update_10pct | 34.03 ms [33.76 ms–34.75 ms] n=10 | 43.91 ms [41.94 ms–53.56 ms] n=10 |
| delete_10pct | 27.12 ms [26.58 ms–27.88 ms] n=10 | 32.60 ms [32.39 ms–33.42 ms] n=10 |
| commit_single | – | 418.8 µs [397.3 µs–427.4 µs] n=10 |
| commit_group | – | 6.15 ms [6.03 ms–6.27 ms] n=10 |
| reopen | – | 178.53 ms [177.34 ms–179.92 ms] n=10 |

### native – native (n = runs)

| Operation | flatsql-native |
|---|---:|
| insert | 207.54 ms [206.68 ms–208.19 ms] n=10 |
| point_lookup | 2.28 µs [2.27 µs–2.31 µs] n=10 |
| index_lookup | 435.0 µs [431.5 µs–458.5 µs] n=10 |
| range_scan | 83.9 µs [81.7 µs–86.5 µs] n=10 |
| full_scan | 8.06 ms [8.03 ms–8.12 ms] n=10 |
| group_by | 14.64 ms [14.54 ms–14.77 ms] n=10 |
| top_k | 5.63 ms [5.61 ms–5.72 ms] n=10 |
| materialize_10k | 3.72 ms [3.70 ms–3.76 ms] n=10 |
| update_10pct | 30.93 ms [30.85 ms–31.06 ms] n=10 |
| delete_10pct | 24.85 ms [24.76 ms–25.37 ms] n=10 |
| snapshot | 25.27 ms [25.15 ms–25.45 ms] n=10 |
| restore | 111.26 ms [110.98 ms–112.28 ms] n=10 |

### node – memory (n = runs)

| Operation | flatsql | flatsql-json |
|---|---:|---:|
| insert | 371.34 ms [368.93 ms–375.67 ms] n=10 | 369.11 ms [367.72 ms–373.82 ms] n=10 |
| point_lookup | 8.94 µs [8.83 µs–9.04 µs] n=10 | 5.36 µs [5.33 µs–5.41 µs] n=10 |
| index_lookup | 426.5 µs [422.3 µs–436.5 µs] n=10 | 416.4 µs [413.1 µs–424.1 µs] n=10 |
| range_scan | 80.8 µs [79.8 µs–82.6 µs] n=10 | 82.2 µs [81.2 µs–83.2 µs] n=10 |
| full_scan | 7.74 ms [7.71 ms–7.77 ms] n=10 | 7.79 ms [7.74 ms–7.83 ms] n=10 |
| group_by | 15.53 ms [15.42 ms–16.12 ms] n=10 | 16.43 ms [16.34 ms–16.79 ms] n=10 |
| top_k | 5.24 ms [5.22 ms–5.35 ms] n=10 | 5.27 ms [5.23 ms–5.37 ms] n=10 |
| materialize_10k | 9.54 ms [9.49 ms–9.73 ms] n=10 | 9.48 ms [9.45 ms–9.55 ms] n=10 |
| update_10pct | 41.38 ms [40.64 ms–41.76 ms] n=10 | 31.90 ms [31.77 ms–32.17 ms] n=10 |
| delete_10pct | 27.68 ms [27.44 ms–28.09 ms] n=10 | 28.93 ms [27.57 ms–29.05 ms] n=10 |
| snapshot | 33.81 ms [33.62 ms–34.26 ms] n=10 | 33.59 ms [33.23 ms–33.89 ms] n=10 |
| restore | 115.03 ms [113.68 ms–116.47 ms] n=10 | 113.93 ms [113.06 ms–114.31 ms] n=10 |

