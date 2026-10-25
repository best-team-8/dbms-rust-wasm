### native – native (n = runs)

| Operation | flatsql-native |
|---|---:|
| insert | 238.55 ms [230.79 ms–244.89 ms] n=20 |
| point_lookup | 2.69 µs [2.53 µs–2.90 µs] n=20 |
| index_lookup | 1.78 µs [1.67 µs–2.12 µs] n=20 |
| range_scan | 53.1 µs [50.5 µs–59.0 µs] n=20 |
| full_scan | 8.64 ms [8.18 ms–9.06 ms] n=20 |
| group_by | 17.24 ms [16.75 ms–17.86 ms] n=20 |
| top_k | 6.37 ms [6.13 ms–6.96 ms] n=20 |
| materialize_10k | 4.14 ms [3.98 ms–4.63 ms] n=20 |
| update_10pct | 35.48 ms [34.17 ms–36.68 ms] n=20 |
| delete_10pct | 27.82 ms [27.04 ms–29.92 ms] n=20 |
| snapshot | 28.03 ms [26.69 ms–30.64 ms] n=20 |
| restore | 125.53 ms [120.89 ms–134.67 ms] n=20 |

