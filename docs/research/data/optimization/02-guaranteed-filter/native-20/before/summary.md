### native – native (n = runs)

| Operation | flatsql-native |
|---|---:|
| insert | 234.70 ms [223.93 ms–251.85 ms] n=20 |
| point_lookup | 2.65 µs [2.54 µs–2.75 µs] n=20 |
| index_lookup | 1.82 µs [1.69 µs–2.45 µs] n=20 |
| range_scan | 92.3 µs [86.3 µs–97.5 µs] n=20 |
| full_scan | 8.99 ms [8.61 ms–9.43 ms] n=20 |
| group_by | 16.72 ms [16.19 ms–17.99 ms] n=20 |
| top_k | 6.00 ms [5.83 ms–6.46 ms] n=20 |
| materialize_10k | 4.18 ms [4.10 ms–4.39 ms] n=20 |
| update_10pct | 34.58 ms [33.19 ms–37.47 ms] n=20 |
| delete_10pct | 27.49 ms [26.71 ms–29.39 ms] n=20 |
| snapshot | 27.82 ms [26.38 ms–29.07 ms] n=20 |
| restore | 127.66 ms [123.87 ms–134.09 ms] n=20 |

