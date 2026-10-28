### browser – memory (n = runs)

| Operation | flatsql | flatsql-json |
|---|---:|---:|
| insert | 275.14 ms [272.14 ms–277.22 ms] n=10 | 271.91 ms [271.14 ms–274.02 ms] n=10 |
| point_lookup | 3.09 µs [3.02 µs–3.12 µs] n=10 | 4.32 µs [4.29 µs–4.35 µs] n=10 |
| index_lookup | 1.89 µs [1.86 µs–1.92 µs] n=10 | 2.85 µs [2.81 µs–2.90 µs] n=10 |
| range_scan | 51.6 µs [50.8 µs–53.5 µs] n=10 | 54.1 µs [52.4 µs–54.6 µs] n=10 |
| full_scan | 4.91 ms [4.83 ms–4.95 ms] n=10 | 4.89 ms [4.86 ms–4.93 ms] n=10 |
| group_by | 10.82 ms [10.26 ms–11.08 ms] n=10 | 11.42 ms [11.35 ms–11.45 ms] n=10 |
| top_k | 5.41 ms [5.39 ms–5.45 ms] n=10 | 5.74 ms [5.70 ms–5.77 ms] n=10 |
| materialize_10k | 1.59 ms [1.56 ms–1.64 ms] n=10 | 8.92 ms [8.78 ms–8.99 ms] n=10 |
| update_10pct | 15.90 ms [15.56 ms–16.65 ms] n=10 | 15.88 ms [15.78 ms–16.21 ms] n=10 |
| delete_10pct | 19.14 ms [18.85 ms–19.50 ms] n=10 | 19.10 ms [18.87 ms–19.72 ms] n=10 |
| snapshot | 24.60 ms [23.95 ms–25.17 ms] n=10 | 25.40 ms [25.33 ms–25.73 ms] n=10 |
| restore | 50.02 ms [49.39 ms–51.62 ms] n=10 | 50.04 ms [49.76 ms–50.83 ms] n=10 |

