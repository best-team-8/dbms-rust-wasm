### browser – memory (n = runs)

| Operation | flatsql | flatsql-json |
|---|---:|---:|
| insert | 291.62 ms [289.75 ms–295.01 ms] n=10 | 277.19 ms [275.60 ms–280.97 ms] n=10 |
| point_lookup | 3.99 µs [3.83 µs–4.05 µs] n=10 | 4.70 µs [4.57 µs–4.84 µs] n=10 |
| index_lookup | 1.99 µs [1.97 µs–2.02 µs] n=10 | 2.88 µs [2.83 µs–2.91 µs] n=10 |
| range_scan | 51.9 µs [51.1 µs–54.8 µs] n=10 | 53.3 µs [52.0 µs–54.6 µs] n=10 |
| full_scan | 5.30 ms [5.25 ms–5.34 ms] n=10 | 5.39 ms [5.23 ms–5.53 ms] n=10 |
| group_by | 12.48 ms [11.40 ms–12.80 ms] n=10 | 12.04 ms [11.30 ms–12.39 ms] n=10 |
| top_k | 5.69 ms [5.60 ms–6.73 ms] n=10 | 5.84 ms [5.72 ms–6.16 ms] n=10 |
| materialize_10k | 14.00 ms [13.61 ms–14.37 ms] n=10 | 9.08 ms [8.98 ms–9.14 ms] n=10 |
| update_10pct | 18.49 ms [17.50 ms–19.98 ms] n=10 | 16.72 ms [16.17 ms–17.04 ms] n=10 |
| delete_10pct | 23.15 ms [22.28 ms–24.53 ms] n=10 | 20.59 ms [20.33 ms–20.95 ms] n=10 |
| snapshot | 28.43 ms [27.50 ms–28.79 ms] n=10 | 27.88 ms [27.00 ms–28.67 ms] n=10 |
| restore | 56.10 ms [55.73 ms–57.58 ms] n=10 | 55.17 ms [53.55 ms–56.21 ms] n=10 |

