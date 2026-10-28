### node – memory (n = runs)

| Operation | flatsql | flatsql-json |
|---|---:|---:|
| insert | 286.43 ms [285.51 ms–288.69 ms] n=10 | 285.88 ms [285.25 ms–286.87 ms] n=10 |
| point_lookup | 2.51 µs [2.46 µs–2.59 µs] n=10 | 4.26 µs [4.15 µs–4.36 µs] n=10 |
| index_lookup | 2.33 µs [1.99 µs–2.81 µs] n=10 | 3.15 µs [3.03 µs–3.94 µs] n=10 |
| range_scan | 42.0 µs [41.3 µs–43.1 µs] n=10 | 44.6 µs [43.9 µs–45.7 µs] n=10 |
| full_scan | 4.53 ms [4.51 ms–4.54 ms] n=10 | 4.51 ms [4.49 ms–4.56 ms] n=10 |
| group_by | 9.87 ms [9.66 ms–9.91 ms] n=10 | 10.06 ms [9.96 ms–10.08 ms] n=10 |
| top_k | 5.42 ms [5.25 ms–5.61 ms] n=10 | 5.30 ms [5.28 ms–5.40 ms] n=10 |
| materialize_10k | 1.71 ms [1.68 ms–1.84 ms] n=10 | 8.98 ms [8.95 ms–9.04 ms] n=10 |
| update_10pct | 15.72 ms [15.27 ms–15.85 ms] n=10 | 15.47 ms [15.31 ms–15.58 ms] n=10 |
| delete_10pct | 18.24 ms [17.71 ms–18.65 ms] n=10 | 17.69 ms [17.56 ms–18.02 ms] n=10 |
| snapshot | 22.27 ms [22.08 ms–22.69 ms] n=10 | 22.38 ms [21.84 ms–22.93 ms] n=10 |
| restore | 48.35 ms [48.01 ms–48.79 ms] n=10 | 48.28 ms [48.15 ms–49.48 ms] n=10 |

