### browser – memory (n = runs)

| Operation | flatsql | flatsql-json |
|---|---:|---:|
| insert | 267.88 ms [266.35 ms–270.86 ms] n=10 | 270.88 ms [266.01 ms–273.61 ms] n=10 |
| point_lookup | 3.11 µs [3.02 µs–3.18 µs] n=10 | 4.17 µs [4.14 µs–4.21 µs] n=10 |
| index_lookup | 1.89 µs [1.85 µs–1.91 µs] n=10 | 2.94 µs [2.88 µs–2.97 µs] n=10 |
| range_scan | 45.4 µs [43.7 µs–46.7 µs] n=10 | 45.8 µs [45.7 µs–46.1 µs] n=10 |
| full_scan | 4.45 ms [4.40 ms–4.47 ms] n=10 | 4.44 ms [4.42 ms–4.49 ms] n=10 |
| group_by | 9.36 ms [9.33 ms–9.42 ms] n=10 | 9.71 ms [9.67 ms–9.76 ms] n=10 |
| top_k | 5.35 ms [5.34 ms–5.40 ms] n=10 | 5.53 ms [5.49 ms–5.57 ms] n=10 |
| materialize_10k | 1.55 ms [1.41 ms–1.58 ms] n=10 | 7.31 ms [7.18 ms–7.48 ms] n=10 |
| update_10pct | 14.83 ms [14.74 ms–15.03 ms] n=10 | 15.22 ms [14.75 ms–15.44 ms] n=10 |
| delete_10pct | 17.28 ms [17.05 ms–17.39 ms] n=10 | 19.24 ms [18.87 ms–19.76 ms] n=10 |
| snapshot | 21.93 ms [21.74 ms–22.27 ms] n=10 | 24.96 ms [22.54 ms–25.42 ms] n=10 |
| restore | 46.25 ms [45.98 ms–46.47 ms] n=10 | 51.65 ms [49.92 ms–52.29 ms] n=10 |

