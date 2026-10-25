### browser – memory (n = runs)

| Operation | flatsql | flatsql-json |
|---|---:|---:|
| insert | 289.15 ms [286.24 ms–293.46 ms] n=10 | 276.26 ms [274.66 ms–278.31 ms] n=10 |
| point_lookup | 8.99 µs [8.84 µs–9.05 µs] n=10 | 4.66 µs [4.51 µs–4.71 µs] n=10 |
| index_lookup | 5.65 µs [5.46 µs–6.34 µs] n=10 | 2.94 µs [2.88 µs–3.00 µs] n=10 |
| range_scan | 55.2 µs [54.6 µs–56.8 µs] n=10 | 52.2 µs [52.0 µs–54.3 µs] n=10 |
| full_scan | 5.37 ms [5.24 ms–5.47 ms] n=10 | 5.21 ms [5.17 ms–5.27 ms] n=10 |
| group_by | 10.99 ms [10.71 ms–11.69 ms] n=10 | 12.04 ms [10.98 ms–12.59 ms] n=10 |
| top_k | 5.58 ms [5.55 ms–5.68 ms] n=10 | 6.19 ms [5.81 ms–6.62 ms] n=10 |
| materialize_10k | 13.91 ms [13.88 ms–14.37 ms] n=10 | 8.98 ms [8.83 ms–9.30 ms] n=10 |
| update_10pct | 16.59 ms [16.22 ms–17.79 ms] n=10 | 17.27 ms [17.14 ms–18.16 ms] n=10 |
| delete_10pct | 20.62 ms [20.31 ms–20.80 ms] n=10 | 22.03 ms [21.65 ms–22.72 ms] n=10 |
| snapshot | 28.43 ms [28.24 ms–29.38 ms] n=10 | 28.54 ms [27.38 ms–29.45 ms] n=10 |
| restore | 55.55 ms [54.55 ms–59.46 ms] n=10 | 56.24 ms [55.54 ms–57.14 ms] n=10 |

