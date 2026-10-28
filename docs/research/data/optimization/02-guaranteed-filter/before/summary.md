### native – native (n = runs)

| Operation | flatsql-native |
|---|---:|
| insert | 234.95 ms [225.62 ms–280.24 ms] n=10 |
| point_lookup | 2.78 µs [2.63 µs–2.81 µs] n=10 |
| index_lookup | 1.68 µs [1.67 µs–1.72 µs] n=10 |
| range_scan | 96.8 µs [85.8 µs–105.5 µs] n=10 |
| full_scan | 9.08 ms [8.73 ms–9.23 ms] n=10 |
| group_by | 17.36 ms [17.20 ms–17.67 ms] n=10 |
| top_k | 6.07 ms [5.86 ms–6.33 ms] n=10 |
| materialize_10k | 4.33 ms [4.13 ms–4.69 ms] n=10 |
| update_10pct | 35.10 ms [33.41 ms–36.41 ms] n=10 |
| delete_10pct | 27.97 ms [26.56 ms–31.35 ms] n=10 |
| snapshot | 27.59 ms [27.40 ms–27.88 ms] n=10 |
| restore | 128.37 ms [126.65 ms–131.85 ms] n=10 |

### node – memory (n = runs)

| Operation | flatsql | flatsql-json |
|---|---:|---:|
| insert | 486.66 ms [467.80 ms–526.59 ms] n=10 | 472.83 ms [454.80 ms–513.87 ms] n=10 |
| point_lookup | 10.3 µs [10.1 µs–12.2 µs] n=10 | 6.80 µs [6.44 µs–6.97 µs] n=10 |
| index_lookup | 9.39 µs [8.89 µs–9.88 µs] n=10 | 5.79 µs [5.09 µs–6.07 µs] n=10 |
| range_scan | 91.0 µs [83.5 µs–100.4 µs] n=10 | 96.2 µs [91.6 µs–99.4 µs] n=10 |
| full_scan | 8.98 ms [8.82 ms–9.61 ms] n=10 | 8.80 ms [8.56 ms–9.43 ms] n=10 |
| group_by | 18.38 ms [16.83 ms–24.99 ms] n=10 | 17.92 ms [17.25 ms–18.55 ms] n=10 |
| top_k | 5.96 ms [5.66 ms–7.49 ms] n=10 | 6.05 ms [5.84 ms–6.27 ms] n=10 |
| materialize_10k | 11.81 ms [11.22 ms–13.50 ms] n=10 | 10.83 ms [10.55 ms–12.04 ms] n=10 |
| update_10pct | 48.32 ms [47.42 ms–51.76 ms] n=10 | 42.82 ms [38.69 ms–47.28 ms] n=10 |
| delete_10pct | 33.64 ms [31.85 ms–34.74 ms] n=10 | 32.44 ms [31.51 ms–33.78 ms] n=10 |
| snapshot | 39.68 ms [38.33 ms–40.70 ms] n=10 | 39.02 ms [37.65 ms–42.08 ms] n=10 |
| restore | 140.64 ms [129.97 ms–145.81 ms] n=10 | 144.94 ms [132.66 ms–155.93 ms] n=10 |

