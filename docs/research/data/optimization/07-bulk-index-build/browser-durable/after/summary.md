### browser – durable (n = runs)

| Operation | flatsql-idb |
|---|---:|
| insert | 384.18 ms [381.11 ms–391.60 ms] n=10 |
| point_lookup | 9.75 µs [9.59 µs–9.87 µs] n=10 |
| index_lookup | 6.11 µs [6.06 µs–6.27 µs] n=10 |
| range_scan | 57.5 µs [54.5 µs–58.8 µs] n=10 |
| full_scan | 5.21 ms [5.20 ms–5.26 ms] n=10 |
| group_by | 17.96 ms [17.38 ms–18.66 ms] n=10 |
| top_k | 5.82 ms [5.65 ms–6.24 ms] n=10 |
| materialize_10k | 13.45 ms [13.32 ms–13.56 ms] n=10 |
| update_10pct | 31.60 ms [28.47 ms–34.43 ms] n=10 |
| delete_10pct | 28.16 ms [27.65 ms–28.57 ms] n=10 |
| commit_single | 490.3 µs [472.5 µs–494.9 µs] n=10 |
| commit_group | 5.74 ms [5.49 ms–5.88 ms] n=10 |
| reopen | 98.40 ms [96.91 ms–100.81 ms] n=10 |

