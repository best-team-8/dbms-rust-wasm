### native – native (n = runs)

| Operation | flatsql-native |
|---|---:|
| insert | 229.26 ms [225.95 ms–234.59 ms] n=10 |
| point_lookup | 1.27 µs [1.22 µs–1.31 µs] n=10 |
| index_lookup | 0.70 µs [0.68 µs–0.72 µs] n=10 |
| range_scan | 51.7 µs [49.9 µs–57.1 µs] n=10 |
| full_scan | 8.56 ms [8.50 ms–8.62 ms] n=10 |
| group_by | 17.15 ms [16.75 ms–17.36 ms] n=10 |
| top_k | 6.23 ms [6.10 ms–6.41 ms] n=10 |
| materialize_10k | 4.15 ms [4.05 ms–4.39 ms] n=10 |
| update_10pct | 37.42 ms [35.01 ms–39.21 ms] n=10 |
| delete_10pct | 27.57 ms [26.71 ms–28.73 ms] n=10 |
| snapshot | 27.21 ms [26.81 ms–28.24 ms] n=10 |
| restore | 122.38 ms [115.87 ms–131.41 ms] n=10 |

### node – memory (n = runs)

| Operation | flatsql | flatsql-json |
|---|---:|---:|
| insert | 458.11 ms [446.90 ms–468.81 ms] n=10 | 477.93 ms [442.64 ms–482.86 ms] n=10 |
| point_lookup | 9.13 µs [8.87 µs–9.40 µs] n=10 | 5.23 µs [5.01 µs–6.09 µs] n=10 |
| index_lookup | 6.58 µs [6.03 µs–8.66 µs] n=10 | 3.35 µs [3.10 µs–4.42 µs] n=10 |
| range_scan | 58.7 µs [56.6 µs–63.7 µs] n=10 | 56.3 µs [56.2 µs–63.3 µs] n=10 |
| full_scan | 8.89 ms [8.74 ms–9.46 ms] n=10 | 8.76 ms [8.41 ms–9.11 ms] n=10 |
| group_by | 19.83 ms [17.46 ms–21.59 ms] n=10 | 17.91 ms [17.38 ms–19.46 ms] n=10 |
| top_k | 6.33 ms [5.98 ms–8.81 ms] n=10 | 6.25 ms [6.14 ms–6.58 ms] n=10 |
| materialize_10k | 12.50 ms [11.56 ms–14.11 ms] n=10 | 11.48 ms [10.95 ms–11.80 ms] n=10 |
| update_10pct | 40.20 ms [38.55 ms–49.42 ms] n=10 | 48.36 ms [40.03 ms–49.73 ms] n=10 |
| delete_10pct | 34.95 ms [32.76 ms–36.77 ms] n=10 | 32.99 ms [32.00 ms–35.63 ms] n=10 |
| snapshot | 38.27 ms [37.87 ms–40.00 ms] n=10 | 38.56 ms [37.56 ms–39.79 ms] n=10 |
| restore | 132.56 ms [126.72 ms–141.46 ms] n=10 | 129.86 ms [127.33 ms–136.97 ms] n=10 |

