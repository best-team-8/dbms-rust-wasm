### node – memory (n = runs)

| Operation | flatsql | flatsql-json |
|---|---:|---:|
| insert | 288.20 ms [284.66 ms–290.02 ms] n=10 | 286.83 ms [285.06 ms–287.60 ms] n=10 |
| point_lookup | 2.47 µs [2.45 µs–2.56 µs] n=10 | 4.14 µs [4.05 µs–4.21 µs] n=10 |
| index_lookup | 1.81 µs [1.75 µs–2.14 µs] n=10 | 3.46 µs [3.18 µs–3.56 µs] n=10 |
| range_scan | 42.4 µs [41.0 µs–43.0 µs] n=10 | 44.3 µs [43.4 µs–47.7 µs] n=10 |
| full_scan | 4.52 ms [4.49 ms–4.55 ms] n=10 | 4.54 ms [4.50 ms–4.55 ms] n=10 |
| group_by | 10.01 ms [9.73 ms–10.33 ms] n=10 | 9.76 ms [9.69 ms–9.88 ms] n=10 |
| top_k | 5.25 ms [5.23 ms–5.28 ms] n=10 | 5.35 ms [5.28 ms–5.53 ms] n=10 |
| materialize_10k | 1.72 ms [1.67 ms–1.78 ms] n=10 | 8.83 ms [8.80 ms–8.97 ms] n=10 |
| update_10pct | 15.32 ms [15.23 ms–15.47 ms] n=10 | 15.54 ms [15.35 ms–15.59 ms] n=10 |
| delete_10pct | 17.62 ms [17.44 ms–18.00 ms] n=10 | 17.61 ms [17.51 ms–18.14 ms] n=10 |
| snapshot | 21.69 ms [21.59 ms–22.10 ms] n=10 | 22.05 ms [21.76 ms–22.75 ms] n=10 |
| restore | 46.74 ms [46.49 ms–47.10 ms] n=10 | 46.36 ms [46.02 ms–46.81 ms] n=10 |

