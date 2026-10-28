### node – memory (n = runs)

| Operation | flatsql | flatsql-json |
|---|---:|---:|
| insert | 316.25 ms [312.85 ms–320.15 ms] n=10 | 316.91 ms [315.17 ms–321.21 ms] n=10 |
| point_lookup | 2.51 µs [2.49 µs–2.54 µs] n=10 | 4.26 µs [4.18 µs–4.65 µs] n=10 |
| index_lookup | 1.77 µs [1.70 µs–2.55 µs] n=10 | 3.18 µs [3.04 µs–4.17 µs] n=10 |
| range_scan | 47.0 µs [46.4 µs–47.8 µs] n=10 | 50.7 µs [49.1 µs–52.3 µs] n=10 |
| full_scan | 4.58 ms [4.56 ms–4.62 ms] n=10 | 4.62 ms [4.58 ms–4.64 ms] n=10 |
| group_by | 9.84 ms [9.72 ms–10.00 ms] n=10 | 9.94 ms [9.82 ms–10.08 ms] n=10 |
| top_k | 5.48 ms [5.44 ms–5.57 ms] n=10 | 5.42 ms [5.37 ms–5.44 ms] n=10 |
| materialize_10k | 1.80 ms [1.77 ms–1.89 ms] n=10 | 9.31 ms [9.14 ms–9.50 ms] n=10 |
| update_10pct | 15.81 ms [15.69 ms–15.92 ms] n=10 | 16.29 ms [15.97 ms–16.43 ms] n=10 |
| delete_10pct | 19.14 ms [19.01 ms–19.89 ms] n=10 | 19.10 ms [18.88 ms–20.11 ms] n=10 |
| snapshot | 19.83 ms [19.45 ms–20.54 ms] n=10 | 20.44 ms [20.04 ms–20.93 ms] n=10 |
| restore | 47.50 ms [47.13 ms–48.56 ms] n=10 | 47.80 ms [47.47 ms–49.13 ms] n=10 |

