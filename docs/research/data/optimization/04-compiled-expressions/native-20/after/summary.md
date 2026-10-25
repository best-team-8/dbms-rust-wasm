### native – native (n = runs)

| Operation | flatsql-native |
|---|---:|
| insert | 207.56 ms [205.88 ms–209.24 ms] n=20 |
| point_lookup | 1.20 µs [1.17 µs–1.22 µs] n=20 |
| index_lookup | 0.71 µs [0.70 µs–0.74 µs] n=20 |
| range_scan | 49.3 µs [48.0 µs–51.2 µs] n=20 |
| full_scan | 5.47 ms [5.38 ms–5.54 ms] n=20 |
| group_by | 15.76 ms [15.56 ms–15.91 ms] n=20 |
| top_k | 5.95 ms [5.92 ms–6.02 ms] n=20 |
| materialize_10k | 3.65 ms [3.60 ms–3.68 ms] n=20 |
| update_10pct | 30.35 ms [30.00 ms–30.79 ms] n=20 |
| delete_10pct | 23.99 ms [23.61 ms–24.45 ms] n=20 |
| snapshot | 25.58 ms [25.28 ms–25.93 ms] n=20 |
| restore | 111.34 ms [110.41 ms–113.51 ms] n=20 |

