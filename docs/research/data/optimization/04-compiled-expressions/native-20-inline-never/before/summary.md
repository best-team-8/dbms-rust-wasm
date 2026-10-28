### native – native (n = runs)

| Operation | flatsql-native |
|---|---:|
| insert | 207.72 ms [205.71 ms–209.83 ms] n=20 |
| point_lookup | 1.20 µs [1.17 µs–1.23 µs] n=20 |
| index_lookup | 0.73 µs [0.72 µs–0.74 µs] n=20 |
| range_scan | 48.8 µs [48.1 µs–49.9 µs] n=20 |
| full_scan | 7.51 ms [7.46 ms–7.63 ms] n=20 |
| group_by | 15.38 ms [15.27 ms–15.45 ms] n=20 |
| top_k | 5.82 ms [5.76 ms–5.85 ms] n=20 |
| materialize_10k | 3.73 ms [3.70 ms–3.77 ms] n=20 |
| update_10pct | 32.49 ms [31.78 ms–33.07 ms] n=20 |
| delete_10pct | 25.40 ms [25.13 ms–25.76 ms] n=20 |
| snapshot | 25.95 ms [25.61 ms–26.89 ms] n=20 |
| restore | 111.02 ms [110.30 ms–112.66 ms] n=20 |

