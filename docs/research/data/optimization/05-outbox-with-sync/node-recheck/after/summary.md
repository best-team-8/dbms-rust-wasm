### node – memory (n = runs)

| Operation | flatsql | flatsql-json |
|---|---:|---:|
| insert | 295.30 ms [291.93 ms–307.90 ms] n=10 | 296.80 ms [293.41 ms–301.94 ms] n=10 |
| point_lookup | 8.31 µs [8.21 µs–8.86 µs] n=10 | 4.46 µs [4.34 µs–4.75 µs] n=10 |
| index_lookup | 7.82 µs [7.57 µs–8.74 µs] n=10 | 3.25 µs [3.05 µs–3.57 µs] n=10 |
| range_scan | 59.9 µs [55.4 µs–63.0 µs] n=10 | 57.5 µs [54.8 µs–65.7 µs] n=10 |
| full_scan | 5.18 ms [5.12 ms–5.28 ms] n=10 | 5.18 ms [5.15 ms–5.24 ms] n=10 |
| group_by | 17.36 ms [17.06 ms–18.32 ms] n=10 | 17.09 ms [16.90 ms–17.73 ms] n=10 |
| top_k | 5.76 ms [5.58 ms–5.80 ms] n=10 | 5.56 ms [5.49 ms–5.72 ms] n=10 |
| materialize_10k | 10.67 ms [10.48 ms–11.48 ms] n=10 | 10.76 ms [10.67 ms–11.11 ms] n=10 |
| update_10pct | 35.71 ms [26.76 ms–40.76 ms] n=10 | 30.33 ms [26.31 ms–35.90 ms] n=10 |
| delete_10pct | 25.21 ms [23.56 ms–26.48 ms] n=10 | 25.20 ms [22.86 ms–25.85 ms] n=10 |
| snapshot | 31.23 ms [30.19 ms–31.50 ms] n=10 | 30.99 ms [29.60 ms–31.51 ms] n=10 |
| restore | 65.27 ms [64.24 ms–67.30 ms] n=10 | 64.24 ms [62.22 ms–65.19 ms] n=10 |

