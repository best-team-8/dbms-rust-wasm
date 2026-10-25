### native – native (n = runs)

| Operation | flatsql-native |
|---|---:|
| insert | 232.76 ms [221.44 ms–239.71 ms] n=20 |
| point_lookup | 1.35 µs [1.27 µs–1.42 µs] n=20 |
| index_lookup | 0.72 µs [0.71 µs–0.73 µs] n=20 |
| range_scan | 52.8 µs [51.9 µs–58.1 µs] n=20 |
| full_scan | 8.19 ms [8.01 ms–8.30 ms] n=20 |
| group_by | 17.33 ms [16.82 ms–18.71 ms] n=20 |
| top_k | 6.36 ms [6.17 ms–6.52 ms] n=20 |
| materialize_10k | 4.17 ms [3.79 ms–4.65 ms] n=20 |
| update_10pct | 34.73 ms [34.13 ms–37.44 ms] n=20 |
| delete_10pct | 27.99 ms [27.07 ms–30.11 ms] n=20 |
| snapshot | 27.81 ms [26.69 ms–29.17 ms] n=20 |
| restore | 124.76 ms [117.63 ms–132.07 ms] n=20 |

