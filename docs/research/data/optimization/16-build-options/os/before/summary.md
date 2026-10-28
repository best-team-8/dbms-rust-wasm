### node – memory (n = runs)

| Operation | flatsql | flatsql-json |
|---|---:|---:|
| insert | 285.78 ms [284.81 ms–288.17 ms] n=10 | 286.18 ms [283.20 ms–286.94 ms] n=10 |
| point_lookup | 2.47 µs [2.46 µs–2.49 µs] n=10 | 4.13 µs [4.09 µs–4.57 µs] n=10 |
| index_lookup | 1.78 µs [1.76 µs–2.27 µs] n=10 | 3.06 µs [3.02 µs–3.32 µs] n=10 |
| range_scan | 41.1 µs [41.1 µs–42.2 µs] n=10 | 43.7 µs [43.3 µs–44.3 µs] n=10 |
| full_scan | 4.51 ms [4.48 ms–4.56 ms] n=10 | 4.52 ms [4.51 ms–4.54 ms] n=10 |
| group_by | 9.76 ms [9.50 ms–9.94 ms] n=10 | 10.10 ms [10.06 ms–10.42 ms] n=10 |
| top_k | 5.32 ms [5.27 ms–5.50 ms] n=10 | 5.68 ms [5.31 ms–5.89 ms] n=10 |
| materialize_10k | 1.76 ms [1.67 ms–1.85 ms] n=10 | 8.81 ms [8.60 ms–8.98 ms] n=10 |
| update_10pct | 15.65 ms [15.25 ms–16.23 ms] n=10 | 15.38 ms [15.14 ms–15.87 ms] n=10 |
| delete_10pct | 17.89 ms [17.69 ms–18.00 ms] n=10 | 17.77 ms [17.62 ms–17.93 ms] n=10 |
| snapshot | 22.60 ms [22.26 ms–22.91 ms] n=10 | 21.92 ms [21.69 ms–22.32 ms] n=10 |
| restore | 48.69 ms [48.31 ms–49.43 ms] n=10 | 48.39 ms [47.96 ms–49.37 ms] n=10 |

