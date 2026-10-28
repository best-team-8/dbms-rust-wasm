### native – native (n = runs)

| Operation | flatsql-native |
|---|---:|
| insert | 238.03 ms [231.46 ms–262.62 ms] n=20 |
| point_lookup | 2.63 µs [2.54 µs–2.73 µs] n=20 |
| index_lookup | 1.76 µs [1.70 µs–2.03 µs] n=20 |
| range_scan | 51.5 µs [50.4 µs–55.2 µs] n=20 |
| full_scan | 8.52 ms [8.24 ms–9.70 ms] n=20 |
| group_by | 16.82 ms [16.32 ms–17.85 ms] n=20 |
| top_k | 6.13 ms [6.10 ms–6.29 ms] n=20 |
| materialize_10k | 3.86 ms [3.78 ms–4.08 ms] n=20 |
| update_10pct | 36.66 ms [34.79 ms–37.23 ms] n=20 |
| delete_10pct | 27.31 ms [26.52 ms–30.40 ms] n=20 |
| snapshot | 28.47 ms [26.96 ms–30.10 ms] n=20 |
| restore | 138.73 ms [125.24 ms–148.34 ms] n=20 |

