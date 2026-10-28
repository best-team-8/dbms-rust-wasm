### node – memory (n = runs)

| Operation | flatsql | flatsql-json |
|---|---:|---:|
| insert | 358.37 ms [353.14 ms–361.31 ms] n=10 | 354.74 ms [351.97 ms–358.29 ms] n=10 |
| point_lookup | 2.87 µs [2.82 µs–2.90 µs] n=10 | 4.64 µs [4.51 µs–4.75 µs] n=10 |
| index_lookup | 2.36 µs [2.23 µs–2.58 µs] n=10 | 3.65 µs [3.53 µs–4.15 µs] n=10 |
| range_scan | 57.4 µs [55.6 µs–59.1 µs] n=10 | 57.4 µs [53.8 µs–58.2 µs] n=10 |
| full_scan | 5.16 ms [5.00 ms–5.22 ms] n=10 | 5.12 ms [4.98 ms–5.30 ms] n=10 |
| group_by | 10.58 ms [10.43 ms–10.82 ms] n=10 | 10.87 ms [10.68 ms–11.05 ms] n=10 |
| top_k | 5.98 ms [5.56 ms–6.03 ms] n=10 | 5.48 ms [5.45 ms–5.51 ms] n=10 |
| materialize_10k | 2.36 ms [2.30 ms–2.44 ms] n=10 | 9.93 ms [9.76 ms–10.08 ms] n=10 |
| update_10pct | 16.48 ms [16.32 ms–16.65 ms] n=10 | 16.78 ms [16.59 ms–16.90 ms] n=10 |
| delete_10pct | 20.68 ms [20.48 ms–20.97 ms] n=10 | 20.52 ms [20.30 ms–21.10 ms] n=10 |
| snapshot | 22.12 ms [21.96 ms–22.53 ms] n=10 | 22.01 ms [21.79 ms–22.41 ms] n=10 |
| restore | 66.02 ms [65.24 ms–67.37 ms] n=10 | 66.60 ms [65.78 ms–66.98 ms] n=10 |

