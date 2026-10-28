### native – native (n = runs)

| Operation | flatsql-native |
|---|---:|
| insert | 209.39 ms [207.30 ms–211.57 ms] n=10 |
| point_lookup | 1.22 µs [1.18 µs–1.31 µs] n=10 |
| index_lookup | 0.70 µs [0.69 µs–0.72 µs] n=10 |
| range_scan | 48.6 µs [48.2 µs–49.6 µs] n=10 |
| full_scan | 5.38 ms [5.34 ms–5.43 ms] n=10 |
| group_by | 15.57 ms [15.43 ms–15.80 ms] n=10 |
| top_k | 6.00 ms [5.98 ms–6.24 ms] n=10 |
| materialize_10k | 3.62 ms [3.61 ms–3.70 ms] n=10 |
| update_10pct | 30.69 ms [30.10 ms–31.12 ms] n=10 |
| delete_10pct | 24.31 ms [23.75 ms–24.86 ms] n=10 |
| snapshot | 25.33 ms [25.27 ms–25.81 ms] n=10 |
| restore | 111.27 ms [110.07 ms–112.79 ms] n=10 |

### node – memory (n = runs)

| Operation | flatsql | flatsql-json |
|---|---:|---:|
| insert | 381.38 ms [378.83 ms–383.25 ms] n=10 | 385.03 ms [375.88 ms–391.54 ms] n=10 |
| point_lookup | 8.44 µs [8.31 µs–8.67 µs] n=10 | 4.84 µs [4.71 µs–4.92 µs] n=10 |
| index_lookup | 6.72 µs [6.29 µs–7.44 µs] n=10 | 3.10 µs [3.01 µs–3.39 µs] n=10 |
| range_scan | 57.0 µs [53.7 µs–66.1 µs] n=10 | 56.7 µs [52.1 µs–58.5 µs] n=10 |
| full_scan | 5.20 ms [5.18 ms–5.25 ms] n=10 | 5.18 ms [5.11 ms–5.20 ms] n=10 |
| group_by | 17.64 ms [16.42 ms–18.13 ms] n=10 | 16.28 ms [16.14 ms–16.83 ms] n=10 |
| top_k | 5.63 ms [5.52 ms–5.91 ms] n=10 | 5.90 ms [5.66 ms–6.30 ms] n=10 |
| materialize_10k | 10.08 ms [9.93 ms–10.31 ms] n=10 | 9.97 ms [9.84 ms–10.04 ms] n=10 |
| update_10pct | 36.37 ms [32.88 ms–42.02 ms] n=10 | 40.38 ms [34.58 ms–43.39 ms] n=10 |
| delete_10pct | 33.09 ms [31.00 ms–35.92 ms] n=10 | 32.77 ms [30.87 ms–34.26 ms] n=10 |
| snapshot | 37.75 ms [37.24 ms–38.89 ms] n=10 | 38.63 ms [37.15 ms–39.35 ms] n=10 |
| restore | 121.81 ms [120.43 ms–123.37 ms] n=10 | 122.55 ms [121.37 ms–124.05 ms] n=10 |

