### browser – memory (n = runs)

| Operation | flatsql | flatsql-json |
|---|---:|---:|
| insert | 272.10 ms [269.79 ms–274.63 ms] n=10 | 270.24 ms [268.41 ms–272.14 ms] n=10 |
| point_lookup | 3.17 µs [3.11 µs–3.27 µs] n=10 | 4.31 µs [4.28 µs–4.35 µs] n=10 |
| index_lookup | 1.96 µs [1.90 µs–2.00 µs] n=10 | 2.85 µs [2.81 µs–2.90 µs] n=10 |
| range_scan | 51.2 µs [50.7 µs–52.5 µs] n=10 | 53.4 µs [52.2 µs–54.4 µs] n=10 |
| full_scan | 4.87 ms [4.83 ms–4.93 ms] n=10 | 4.86 ms [4.83 ms–4.90 ms] n=10 |
| group_by | 10.93 ms [10.23 ms–11.13 ms] n=10 | 11.43 ms [11.40 ms–11.48 ms] n=10 |
| top_k | 5.43 ms [5.40 ms–5.47 ms] n=10 | 5.89 ms [5.79 ms–6.05 ms] n=10 |
| materialize_10k | 1.53 ms [1.46 ms–1.60 ms] n=10 | 9.12 ms [8.98 ms–9.66 ms] n=10 |
| update_10pct | 15.85 ms [15.61 ms–16.28 ms] n=10 | 16.00 ms [15.92 ms–16.69 ms] n=10 |
| delete_10pct | 19.69 ms [19.10 ms–19.87 ms] n=10 | 20.63 ms [20.13 ms–21.43 ms] n=10 |
| snapshot | 24.31 ms [24.21 ms–24.50 ms] n=10 | 28.27 ms [25.54 ms–28.75 ms] n=10 |
| restore | 49.14 ms [48.83 ms–49.36 ms] n=10 | 55.58 ms [53.04 ms–56.90 ms] n=10 |

