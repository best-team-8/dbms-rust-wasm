### node – memory (n = runs)

| Operation | flatsql | flatsql-json |
|---|---:|---:|
| insert | 288.07 ms [285.47 ms–291.05 ms] n=10 | 287.59 ms [286.67 ms–289.66 ms] n=10 |
| point_lookup | 2.47 µs [2.44 µs–2.68 µs] n=10 | 4.17 µs [4.11 µs–4.51 µs] n=10 |
| index_lookup | 1.88 µs [1.74 µs–2.45 µs] n=10 | 3.11 µs [3.03 µs–3.60 µs] n=10 |
| range_scan | 41.5 µs [41.0 µs–43.7 µs] n=10 | 43.9 µs [43.6 µs–44.9 µs] n=10 |
| full_scan | 4.54 ms [4.51 ms–4.61 ms] n=10 | 4.52 ms [4.50 ms–4.54 ms] n=10 |
| group_by | 9.62 ms [9.47 ms–9.82 ms] n=10 | 10.19 ms [10.03 ms–10.24 ms] n=10 |
| top_k | 5.26 ms [5.25 ms–5.48 ms] n=10 | 5.37 ms [5.33 ms–5.80 ms] n=10 |
| materialize_10k | 1.76 ms [1.68 ms–1.83 ms] n=10 | 8.87 ms [8.80 ms–8.91 ms] n=10 |
| update_10pct | 15.61 ms [15.39 ms–16.05 ms] n=10 | 15.26 ms [15.15 ms–15.45 ms] n=10 |
| delete_10pct | 17.75 ms [17.63 ms–18.30 ms] n=10 | 17.69 ms [17.59 ms–18.12 ms] n=10 |
| snapshot | 22.69 ms [22.47 ms–23.06 ms] n=10 | 21.75 ms [21.66 ms–22.11 ms] n=10 |
| restore | 48.19 ms [48.03 ms–49.30 ms] n=10 | 48.66 ms [48.24 ms–48.91 ms] n=10 |

