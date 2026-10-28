### browser – durable (n = runs)

| Operation | flatsql-idb |
|---|---:|
| insert | 386.30 ms [377.71 ms–394.79 ms] n=10 |
| point_lookup | 10.0 µs [9.81 µs–10.2 µs] n=10 |
| index_lookup | 6.34 µs [6.15 µs–6.49 µs] n=10 |
| range_scan | 58.2 µs [56.0 µs–59.6 µs] n=10 |
| full_scan | 5.30 ms [5.23 ms–5.54 ms] n=10 |
| group_by | 17.01 ms [16.75 ms–18.17 ms] n=10 |
| top_k | 5.71 ms [5.64 ms–5.89 ms] n=10 |
| materialize_10k | 14.21 ms [13.46 ms–14.73 ms] n=10 |
| update_10pct | 29.51 ms [27.41 ms–35.99 ms] n=10 |
| delete_10pct | 26.34 ms [26.02 ms–26.90 ms] n=10 |
| commit_single | 497.3 µs [479.7 µs–503.0 µs] n=10 |
| commit_group | 5.66 ms [5.51 ms–5.74 ms] n=10 |
| reopen | 102.16 ms [100.13 ms–105.12 ms] n=10 |

