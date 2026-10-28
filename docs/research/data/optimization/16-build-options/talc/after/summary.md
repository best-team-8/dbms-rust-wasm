### node – memory (n = runs)

| Operation | flatsql | flatsql-json |
|---|---:|---:|
| insert | 288.53 ms [286.78 ms–289.13 ms] n=10 | 286.22 ms [285.91 ms–288.77 ms] n=10 |
| point_lookup | 2.51 µs [2.48 µs–2.58 µs] n=10 | 4.20 µs [4.14 µs–4.79 µs] n=10 |
| index_lookup | 2.08 µs [1.79 µs–2.58 µs] n=10 | 3.08 µs [2.98 µs–3.51 µs] n=10 |
| range_scan | 42.2 µs [41.6 µs–43.8 µs] n=10 | 44.4 µs [43.8 µs–44.7 µs] n=10 |
| full_scan | 4.60 ms [4.58 ms–4.62 ms] n=10 | 4.62 ms [4.61 ms–4.63 ms] n=10 |
| group_by | 9.95 ms [9.80 ms–10.13 ms] n=10 | 9.89 ms [9.85 ms–9.95 ms] n=10 |
| top_k | 5.51 ms [5.48 ms–5.53 ms] n=10 | 5.55 ms [5.48 ms–6.19 ms] n=10 |
| materialize_10k | 1.82 ms [1.71 ms–1.82 ms] n=10 | 8.94 ms [8.78 ms–9.03 ms] n=10 |
| update_10pct | 15.86 ms [15.51 ms–16.09 ms] n=10 | 15.54 ms [15.46 ms–15.62 ms] n=10 |
| delete_10pct | 17.57 ms [17.33 ms–17.73 ms] n=10 | 17.90 ms [17.60 ms–18.16 ms] n=10 |
| snapshot | 22.07 ms [21.80 ms–22.41 ms] n=10 | 22.44 ms [22.12 ms–22.88 ms] n=10 |
| restore | 47.32 ms [47.15 ms–47.79 ms] n=10 | 47.06 ms [46.71 ms–47.33 ms] n=10 |

