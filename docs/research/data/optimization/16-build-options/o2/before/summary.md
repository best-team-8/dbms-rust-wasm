### node – memory (n = runs)

| Operation | flatsql | flatsql-json |
|---|---:|---:|
| insert | 288.80 ms [286.93 ms–290.70 ms] n=10 | 287.87 ms [286.99 ms–290.06 ms] n=10 |
| point_lookup | 2.52 µs [2.51 µs–2.61 µs] n=10 | 4.36 µs [4.24 µs–4.46 µs] n=10 |
| index_lookup | 1.95 µs [1.79 µs–2.52 µs] n=10 | 3.10 µs [3.01 µs–3.43 µs] n=10 |
| range_scan | 42.5 µs [41.1 µs–43.4 µs] n=10 | 44.3 µs [44.1 µs–53.2 µs] n=10 |
| full_scan | 4.52 ms [4.50 ms–4.53 ms] n=10 | 4.55 ms [4.52 ms–4.71 ms] n=10 |
| group_by | 9.55 ms [9.50 ms–9.83 ms] n=10 | 10.05 ms [9.90 ms–10.17 ms] n=10 |
| top_k | 5.34 ms [5.25 ms–5.41 ms] n=10 | 5.33 ms [5.31 ms–5.58 ms] n=10 |
| materialize_10k | 1.77 ms [1.73 ms–1.86 ms] n=10 | 9.05 ms [8.85 ms–9.17 ms] n=10 |
| update_10pct | 15.38 ms [15.24 ms–15.61 ms] n=10 | 15.97 ms [15.51 ms–16.45 ms] n=10 |
| delete_10pct | 18.01 ms [17.76 ms–18.26 ms] n=10 | 18.05 ms [17.90 ms–18.26 ms] n=10 |
| snapshot | 23.22 ms [22.17 ms–24.33 ms] n=10 | 22.24 ms [21.93 ms–22.65 ms] n=10 |
| restore | 49.10 ms [48.38 ms–50.43 ms] n=10 | 48.56 ms [48.25 ms–48.92 ms] n=10 |

