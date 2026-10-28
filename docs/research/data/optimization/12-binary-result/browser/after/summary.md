### browser – memory (n = runs)

| Operation | flatsql | flatsql-json |
|---|---:|---:|
| insert | 280.65 ms [276.31 ms–283.13 ms] n=10 | 274.83 ms [273.71 ms–276.77 ms] n=10 |
| point_lookup | 3.29 µs [3.26 µs–3.39 µs] n=10 | 4.59 µs [4.41 µs–4.66 µs] n=10 |
| index_lookup | 1.87 µs [1.85 µs–1.94 µs] n=10 | 2.91 µs [2.84 µs–2.94 µs] n=10 |
| range_scan | 51.7 µs [51.1 µs–53.7 µs] n=10 | 55.3 µs [54.1 µs–56.7 µs] n=10 |
| full_scan | 5.11 ms [5.01 ms–5.16 ms] n=10 | 5.07 ms [4.99 ms–5.16 ms] n=10 |
| group_by | 11.49 ms [11.02 ms–11.71 ms] n=10 | 10.56 ms [10.49 ms–10.71 ms] n=10 |
| top_k | 5.62 ms [5.56 ms–5.63 ms] n=10 | 5.70 ms [5.65 ms–5.79 ms] n=10 |
| materialize_10k | 1.59 ms [1.49 ms–1.68 ms] n=10 | 9.01 ms [8.95 ms–9.07 ms] n=10 |
| update_10pct | 16.97 ms [16.68 ms–17.17 ms] n=10 | 16.72 ms [16.62 ms–17.48 ms] n=10 |
| delete_10pct | 19.94 ms [19.18 ms–20.85 ms] n=10 | 20.55 ms [20.31 ms–20.85 ms] n=10 |
| snapshot | 25.19 ms [25.03 ms–26.04 ms] n=10 | 28.47 ms [28.39 ms–29.06 ms] n=10 |
| restore | 50.82 ms [50.68 ms–51.23 ms] n=10 | 55.68 ms [55.27 ms–56.33 ms] n=10 |

