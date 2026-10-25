### native – native (n = runs)

| Operation | flatsql-native |
|---|---:|
| insert | 205.46 ms [204.58 ms–207.60 ms] n=20 |
| point_lookup | 1.20 µs [1.18 µs–1.21 µs] n=20 |
| index_lookup | 0.73 µs [0.72 µs–0.74 µs] n=20 |
| range_scan | 49.7 µs [48.2 µs–51.7 µs] n=20 |
| full_scan | 7.62 ms [7.45 ms–7.73 ms] n=20 |
| group_by | 15.38 ms [15.26 ms–15.54 ms] n=20 |
| top_k | 5.83 ms [5.77 ms–5.91 ms] n=20 |
| materialize_10k | 3.76 ms [3.69 ms–3.81 ms] n=20 |
| update_10pct | 31.80 ms [31.45 ms–32.34 ms] n=20 |
| delete_10pct | 25.79 ms [25.17 ms–26.24 ms] n=20 |
| snapshot | 25.80 ms [25.39 ms–26.15 ms] n=20 |
| restore | 111.81 ms [110.45 ms–112.96 ms] n=20 |

