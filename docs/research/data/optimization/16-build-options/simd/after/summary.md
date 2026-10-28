### node – memory (n = runs)

| Operation | flatsql | flatsql-json |
|---|---:|---:|
| insert | 287.00 ms [284.40 ms–289.14 ms] n=10 | 286.59 ms [285.52 ms–289.04 ms] n=10 |
| point_lookup | 2.47 µs [2.45 µs–2.49 µs] n=10 | 4.09 µs [4.06 µs–4.16 µs] n=10 |
| index_lookup | 2.12 µs [1.75 µs–2.28 µs] n=10 | 3.36 µs [3.06 µs–3.68 µs] n=10 |
| range_scan | 41.5 µs [41.3 µs–41.9 µs] n=10 | 45.2 µs [43.4 µs–46.5 µs] n=10 |
| full_scan | 4.51 ms [4.49 ms–4.53 ms] n=10 | 4.56 ms [4.56 ms–4.59 ms] n=10 |
| group_by | 10.02 ms [9.87 ms–10.19 ms] n=10 | 10.08 ms [10.01 ms–10.14 ms] n=10 |
| top_k | 5.26 ms [5.24 ms–5.29 ms] n=10 | 5.26 ms [5.25 ms–5.26 ms] n=10 |
| materialize_10k | 1.73 ms [1.72 ms–1.81 ms] n=10 | 8.92 ms [8.83 ms–9.01 ms] n=10 |
| update_10pct | 15.62 ms [15.49 ms–15.86 ms] n=10 | 15.49 ms [15.31 ms–15.61 ms] n=10 |
| delete_10pct | 17.67 ms [17.46 ms–17.77 ms] n=10 | 17.71 ms [17.47 ms–17.80 ms] n=10 |
| snapshot | 21.82 ms [21.52 ms–22.25 ms] n=10 | 21.76 ms [21.67 ms–21.94 ms] n=10 |
| restore | 46.10 ms [45.89 ms–46.58 ms] n=10 | 45.81 ms [45.61 ms–45.91 ms] n=10 |

