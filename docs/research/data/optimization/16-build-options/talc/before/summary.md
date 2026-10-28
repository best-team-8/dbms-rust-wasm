### node – memory (n = runs)

| Operation | flatsql | flatsql-json |
|---|---:|---:|
| insert | 287.98 ms [287.15 ms–288.29 ms] n=10 | 284.31 ms [283.76 ms–286.43 ms] n=10 |
| point_lookup | 2.47 µs [2.44 µs–2.51 µs] n=10 | 4.13 µs [4.06 µs–4.50 µs] n=10 |
| index_lookup | 2.37 µs [1.87 µs–2.78 µs] n=10 | 3.53 µs [3.06 µs–3.79 µs] n=10 |
| range_scan | 41.1 µs [40.9 µs–41.8 µs] n=10 | 44.0 µs [43.5 µs–44.3 µs] n=10 |
| full_scan | 4.54 ms [4.52 ms–4.56 ms] n=10 | 4.51 ms [4.49 ms–4.55 ms] n=10 |
| group_by | 9.65 ms [9.53 ms–10.02 ms] n=10 | 10.09 ms [9.80 ms–10.34 ms] n=10 |
| top_k | 5.36 ms [5.28 ms–5.54 ms] n=10 | 5.36 ms [5.28 ms–5.86 ms] n=10 |
| materialize_10k | 1.66 ms [1.63 ms–1.72 ms] n=10 | 8.92 ms [8.87 ms–8.98 ms] n=10 |
| update_10pct | 15.58 ms [15.31 ms–15.88 ms] n=10 | 15.39 ms [15.28 ms–15.58 ms] n=10 |
| delete_10pct | 18.02 ms [17.66 ms–18.66 ms] n=10 | 17.94 ms [17.68 ms–18.02 ms] n=10 |
| snapshot | 22.30 ms [22.10 ms–22.76 ms] n=10 | 21.61 ms [21.43 ms–22.02 ms] n=10 |
| restore | 48.09 ms [47.98 ms–48.59 ms] n=10 | 48.18 ms [47.83 ms–48.69 ms] n=10 |

