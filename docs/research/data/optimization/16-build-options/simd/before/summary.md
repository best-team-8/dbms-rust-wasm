### node – memory (n = runs)

| Operation | flatsql | flatsql-json |
|---|---:|---:|
| insert | 286.98 ms [285.12 ms–288.17 ms] n=10 | 285.80 ms [284.24 ms–286.79 ms] n=10 |
| point_lookup | 2.50 µs [2.45 µs–2.78 µs] n=10 | 4.08 µs [4.07 µs–4.11 µs] n=10 |
| index_lookup | 1.80 µs [1.75 µs–2.19 µs] n=10 | 3.04 µs [2.98 µs–3.59 µs] n=10 |
| range_scan | 41.2 µs [41.0 µs–42.2 µs] n=10 | 43.8 µs [43.4 µs–44.2 µs] n=10 |
| full_scan | 4.52 ms [4.51 ms–4.54 ms] n=10 | 4.51 ms [4.50 ms–4.54 ms] n=10 |
| group_by | 9.56 ms [9.51 ms–9.87 ms] n=10 | 10.00 ms [9.83 ms–10.14 ms] n=10 |
| top_k | 5.29 ms [5.25 ms–5.48 ms] n=10 | 5.36 ms [5.30 ms–5.70 ms] n=10 |
| materialize_10k | 1.70 ms [1.67 ms–1.75 ms] n=10 | 8.79 ms [8.67 ms–8.87 ms] n=10 |
| update_10pct | 15.78 ms [15.36 ms–15.84 ms] n=10 | 15.30 ms [15.18 ms–15.63 ms] n=10 |
| delete_10pct | 17.89 ms [17.66 ms–18.25 ms] n=10 | 17.81 ms [17.55 ms–18.17 ms] n=10 |
| snapshot | 22.07 ms [21.90 ms–22.34 ms] n=10 | 21.56 ms [21.50 ms–21.65 ms] n=10 |
| restore | 48.56 ms [48.37 ms–48.91 ms] n=10 | 48.45 ms [48.22 ms–49.36 ms] n=10 |

