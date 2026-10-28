### node – memory (n = runs)

| Operation | flatsql | flatsql-json |
|---|---:|---:|
| insert | 287.78 ms [285.93 ms–288.73 ms] n=10 | 287.78 ms [286.78 ms–288.92 ms] n=10 |
| point_lookup | 2.50 µs [2.46 µs–2.53 µs] n=10 | 4.10 µs [4.06 µs–4.16 µs] n=10 |
| index_lookup | 2.16 µs [1.73 µs–2.24 µs] n=10 | 3.42 µs [3.09 µs–3.52 µs] n=10 |
| range_scan | 40.9 µs [40.7 µs–42.9 µs] n=10 | 43.8 µs [43.2 µs–44.7 µs] n=10 |
| full_scan | 4.50 ms [4.49 ms–4.52 ms] n=10 | 4.52 ms [4.47 ms–4.57 ms] n=10 |
| group_by | 9.55 ms [9.52 ms–9.59 ms] n=10 | 9.97 ms [9.89 ms–9.97 ms] n=10 |
| top_k | 5.30 ms [5.26 ms–5.32 ms] n=10 | 5.30 ms [5.28 ms–5.36 ms] n=10 |
| materialize_10k | 1.74 ms [1.70 ms–1.78 ms] n=10 | 8.81 ms [8.78 ms–8.87 ms] n=10 |
| update_10pct | 15.37 ms [15.26 ms–15.76 ms] n=10 | 15.29 ms [15.14 ms–15.47 ms] n=10 |
| delete_10pct | 17.89 ms [17.74 ms–18.07 ms] n=10 | 17.61 ms [17.44 ms–17.65 ms] n=10 |
| snapshot | 22.12 ms [21.91 ms–22.59 ms] n=10 | 22.13 ms [21.85 ms–22.53 ms] n=10 |
| restore | 48.99 ms [48.57 ms–49.93 ms] n=10 | 47.99 ms [47.72 ms–48.15 ms] n=10 |

