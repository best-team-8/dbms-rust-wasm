### browser – memory (n = runs)

| Operation | flatsql | flatsql-json |
|---|---:|---:|
| insert | 282.88 ms [281.34 ms–283.52 ms] n=10 | 275.71 ms [273.08 ms–278.56 ms] n=10 |
| point_lookup | 3.36 µs [3.28 µs–3.51 µs] n=10 | 4.48 µs [4.43 µs–4.59 µs] n=10 |
| index_lookup | 1.92 µs [1.89 µs–1.98 µs] n=10 | 2.94 µs [2.88 µs–3.04 µs] n=10 |
| range_scan | 52.6 µs [51.4 µs–54.8 µs] n=10 | 54.8 µs [54.4 µs–57.4 µs] n=10 |
| full_scan | 5.08 ms [5.00 ms–5.13 ms] n=10 | 5.02 ms [4.97 ms–5.11 ms] n=10 |
| group_by | 11.46 ms [11.18 ms–11.53 ms] n=10 | 10.76 ms [10.62 ms–10.89 ms] n=10 |
| top_k | 5.56 ms [5.52 ms–5.60 ms] n=10 | 5.82 ms [5.62 ms–6.09 ms] n=10 |
| materialize_10k | 1.61 ms [1.55 ms–1.62 ms] n=10 | 9.20 ms [9.04 ms–9.46 ms] n=10 |
| update_10pct | 16.65 ms [16.19 ms–17.11 ms] n=10 | 17.54 ms [17.12 ms–18.07 ms] n=10 |
| delete_10pct | 19.95 ms [19.09 ms–20.65 ms] n=10 | 21.14 ms [20.44 ms–21.49 ms] n=10 |
| snapshot | 25.01 ms [24.56 ms–25.53 ms] n=10 | 28.77 ms [28.01 ms–29.96 ms] n=10 |
| restore | 51.50 ms [50.61 ms–51.85 ms] n=10 | 57.92 ms [56.86 ms–59.00 ms] n=10 |

