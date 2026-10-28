### browser – memory (n = runs)

| Operation | flatsql | flatsql-json |
|---|---:|---:|
| insert | 286.16 ms [283.77 ms–294.41 ms] n=10 | 281.25 ms [274.28 ms–284.12 ms] n=10 |
| point_lookup | 3.95 µs [3.86 µs–4.02 µs] n=10 | 4.61 µs [4.51 µs–4.84 µs] n=10 |
| index_lookup | 2.00 µs [1.93 µs–2.14 µs] n=10 | 2.99 µs [2.90 µs–3.11 µs] n=10 |
| range_scan | 53.5 µs [52.2 µs–54.2 µs] n=10 | 53.6 µs [52.8 µs–54.0 µs] n=10 |
| full_scan | 5.35 ms [5.29 ms–5.51 ms] n=10 | 5.26 ms [5.23 ms–5.47 ms] n=10 |
| group_by | 11.74 ms [10.73 ms–12.60 ms] n=10 | 12.42 ms [11.56 ms–12.74 ms] n=10 |
| top_k | 5.64 ms [5.60 ms–5.74 ms] n=10 | 5.76 ms [5.73 ms–5.90 ms] n=10 |
| materialize_10k | 13.94 ms [13.86 ms–14.57 ms] n=10 | 9.18 ms [9.08 ms–9.20 ms] n=10 |
| update_10pct | 17.34 ms [16.85 ms–17.63 ms] n=10 | 16.87 ms [16.55 ms–17.21 ms] n=10 |
| delete_10pct | 22.35 ms [20.82 ms–23.69 ms] n=10 | 20.46 ms [20.23 ms–21.51 ms] n=10 |
| snapshot | 27.99 ms [26.95 ms–28.77 ms] n=10 | 28.24 ms [27.44 ms–29.04 ms] n=10 |
| restore | 57.50 ms [55.42 ms–58.40 ms] n=10 | 56.11 ms [55.10 ms–56.28 ms] n=10 |

