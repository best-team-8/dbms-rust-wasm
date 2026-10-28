### native – native (n = runs)

| Operation | flatsql-native |
|---|---:|
| insert | 232.47 ms [227.19 ms–238.19 ms] n=10 |
| point_lookup | 2.77 µs [2.68 µs–3.18 µs] n=10 |
| index_lookup | 1.77 µs [1.75 µs–1.94 µs] n=10 |
| range_scan | 56.1 µs [54.4 µs–57.8 µs] n=10 |
| full_scan | 8.36 ms [8.12 ms–8.73 ms] n=10 |
| group_by | 16.72 ms [16.53 ms–17.11 ms] n=10 |
| top_k | 6.39 ms [6.03 ms–6.81 ms] n=10 |
| materialize_10k | 4.14 ms [4.06 ms–4.26 ms] n=10 |
| update_10pct | 33.74 ms [32.61 ms–34.65 ms] n=10 |
| delete_10pct | 29.13 ms [27.74 ms–31.21 ms] n=10 |
| snapshot | 27.21 ms [26.40 ms–28.23 ms] n=10 |
| restore | 125.80 ms [119.76 ms–140.37 ms] n=10 |

### node – memory (n = runs)

| Operation | flatsql | flatsql-json |
|---|---:|---:|
| insert | 472.27 ms [463.59 ms–506.91 ms] n=10 | 504.17 ms [469.15 ms–564.73 ms] n=10 |
| point_lookup | 10.3 µs [10.0 µs–10.6 µs] n=10 | 6.61 µs [6.41 µs–7.18 µs] n=10 |
| index_lookup | 9.38 µs [8.79 µs–9.82 µs] n=10 | 5.60 µs [4.89 µs–6.01 µs] n=10 |
| range_scan | 58.7 µs [54.4 µs–62.0 µs] n=10 | 56.5 µs [54.5 µs–63.6 µs] n=10 |
| full_scan | 9.05 ms [8.66 ms–9.19 ms] n=10 | 8.53 ms [8.42 ms–9.03 ms] n=10 |
| group_by | 17.66 ms [17.43 ms–17.92 ms] n=10 | 18.31 ms [17.63 ms–19.96 ms] n=10 |
| top_k | 6.08 ms [5.85 ms–6.21 ms] n=10 | 6.37 ms [6.27 ms–6.68 ms] n=10 |
| materialize_10k | 10.94 ms [10.71 ms–11.72 ms] n=10 | 11.57 ms [10.84 ms–12.23 ms] n=10 |
| update_10pct | 42.11 ms [38.67 ms–54.83 ms] n=10 | 42.31 ms [37.24 ms–46.83 ms] n=10 |
| delete_10pct | 33.89 ms [31.92 ms–37.91 ms] n=10 | 33.88 ms [32.19 ms–34.72 ms] n=10 |
| snapshot | 37.52 ms [36.88 ms–38.82 ms] n=10 | 38.72 ms [37.46 ms–39.41 ms] n=10 |
| restore | 129.72 ms [127.61 ms–162.78 ms] n=10 | 131.30 ms [124.27 ms–135.73 ms] n=10 |

