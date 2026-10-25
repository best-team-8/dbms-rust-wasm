### native – native (n = runs)

| Operation | flatsql-native |
|---|---:|
| insert | 206.59 ms [206.03 ms–208.29 ms] n=20 |
| point_lookup | 1.23 µs [1.18 µs–1.27 µs] n=20 |
| index_lookup | 0.72 µs [0.70 µs–0.74 µs] n=20 |
| range_scan | 50.3 µs [48.1 µs–51.8 µs] n=20 |
| full_scan | 6.29 ms [6.23 ms–6.41 ms] n=20 |
| group_by | 15.72 ms [15.55 ms–15.84 ms] n=20 |
| top_k | 5.92 ms [5.89 ms–5.98 ms] n=20 |
| materialize_10k | 3.90 ms [3.86 ms–3.95 ms] n=20 |
| update_10pct | 30.72 ms [30.30 ms–31.21 ms] n=20 |
| delete_10pct | 24.16 ms [23.82 ms–24.50 ms] n=20 |
| snapshot | 25.69 ms [25.31 ms–26.79 ms] n=20 |
| restore | 112.64 ms [111.18 ms–114.60 ms] n=20 |

