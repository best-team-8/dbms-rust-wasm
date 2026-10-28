### node – memory (n = runs)

| Operation | flatsql | flatsql-json |
|---|---:|---:|
| insert | 286.13 ms [285.14 ms–286.97 ms] n=10 | 285.00 ms [283.09 ms–286.63 ms] n=10 |
| point_lookup | 2.59 µs [2.57 µs–3.25 µs] n=10 | 4.51 µs [4.25 µs–4.80 µs] n=10 |
| index_lookup | 1.90 µs [1.83 µs–2.13 µs] n=10 | 3.22 µs [3.15 µs–3.85 µs] n=10 |
| range_scan | 42.6 µs [41.6 µs–43.7 µs] n=10 | 45.4 µs [44.4 µs–46.4 µs] n=10 |
| full_scan | 4.50 ms [4.49 ms–4.51 ms] n=10 | 4.51 ms [4.49 ms–4.54 ms] n=10 |
| group_by | 10.08 ms [9.90 ms–10.14 ms] n=10 | 10.05 ms [9.99 ms–10.07 ms] n=10 |
| top_k | 5.26 ms [5.23 ms–5.35 ms] n=10 | 5.32 ms [5.32 ms–5.38 ms] n=10 |
| materialize_10k | 1.71 ms [1.68 ms–1.75 ms] n=10 | 9.10 ms [9.06 ms–9.19 ms] n=10 |
| update_10pct | 15.54 ms [15.32 ms–16.22 ms] n=10 | 15.46 ms [15.29 ms–15.77 ms] n=10 |
| delete_10pct | 17.63 ms [17.54 ms–18.10 ms] n=10 | 17.32 ms [17.18 ms–17.68 ms] n=10 |
| snapshot | 22.05 ms [21.85 ms–22.38 ms] n=10 | 22.00 ms [21.82 ms–22.61 ms] n=10 |
| restore | 48.57 ms [48.16 ms–48.90 ms] n=10 | 48.34 ms [47.94 ms–49.00 ms] n=10 |

