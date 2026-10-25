### native – native (n = runs)

| Operation | flatsql-native |
|---|---:|
| insert | 207.13 ms [205.95 ms–207.49 ms] n=10 |
| point_lookup | 1.19 µs [1.18 µs–1.20 µs] n=10 |
| index_lookup | 0.70 µs [0.70 µs–0.72 µs] n=10 |
| range_scan | 51.5 µs [50.1 µs–53.9 µs] n=10 |
| full_scan | 7.41 ms [7.35 ms–7.49 ms] n=10 |
| group_by | 15.28 ms [15.20 ms–15.31 ms] n=10 |
| top_k | 5.77 ms [5.76 ms–5.78 ms] n=10 |
| materialize_10k | 3.68 ms [3.68 ms–3.71 ms] n=10 |
| update_10pct | 32.16 ms [31.53 ms–32.59 ms] n=10 |
| delete_10pct | 25.54 ms [25.02 ms–26.39 ms] n=10 |
| snapshot | 25.99 ms [25.69 ms–26.21 ms] n=10 |
| restore | 111.48 ms [110.70 ms–112.48 ms] n=10 |

### node – memory (n = runs)

| Operation | flatsql | flatsql-json |
|---|---:|---:|
| insert | 380.38 ms [376.99 ms–392.06 ms] n=10 | 385.43 ms [376.41 ms–396.50 ms] n=10 |
| point_lookup | 8.52 µs [8.42 µs–8.64 µs] n=10 | 4.96 µs [4.94 µs–4.97 µs] n=10 |
| index_lookup | 5.92 µs [5.90 µs–6.06 µs] n=10 | 4.04 µs [3.53 µs–4.10 µs] n=10 |
| range_scan | 57.3 µs [55.6 µs–58.4 µs] n=10 | 55.4 µs [54.7 µs–56.3 µs] n=10 |
| full_scan | 8.30 ms [8.27 ms–8.42 ms] n=10 | 8.35 ms [8.24 ms–8.49 ms] n=10 |
| group_by | 16.83 ms [16.54 ms–16.99 ms] n=10 | 16.41 ms [16.21 ms–16.76 ms] n=10 |
| top_k | 5.77 ms [5.63 ms–5.96 ms] n=10 | 5.93 ms [5.76 ms–5.99 ms] n=10 |
| materialize_10k | 10.00 ms [9.93 ms–10.32 ms] n=10 | 10.16 ms [9.87 ms–10.77 ms] n=10 |
| update_10pct | 42.60 ms [37.20 ms–49.86 ms] n=10 | 37.68 ms [37.37 ms–45.59 ms] n=10 |
| delete_10pct | 33.57 ms [30.99 ms–35.64 ms] n=10 | 31.60 ms [30.79 ms–34.31 ms] n=10 |
| snapshot | 37.36 ms [36.85 ms–37.93 ms] n=10 | 37.89 ms [37.07 ms–38.18 ms] n=10 |
| restore | 122.48 ms [121.91 ms–123.17 ms] n=10 | 125.12 ms [122.63 ms–127.46 ms] n=10 |

