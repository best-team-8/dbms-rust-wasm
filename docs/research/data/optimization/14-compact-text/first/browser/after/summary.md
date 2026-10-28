### browser – memory (n = runs)

| Operation | flatsql | flatsql-json |
|---|---:|---:|
| insert | 292.78 ms [288.48 ms–296.21 ms] n=10 | 289.63 ms [287.06 ms–295.15 ms] n=10 |
| point_lookup | 3.08 µs [3.03 µs–3.19 µs] n=10 | 4.24 µs [4.21 µs–4.30 µs] n=10 |
| index_lookup | 1.82 µs [1.81 µs–1.91 µs] n=10 | 2.86 µs [2.79 µs–2.91 µs] n=10 |
| range_scan | 42.9 µs [42.4 µs–44.6 µs] n=10 | 44.1 µs [43.7 µs–44.9 µs] n=10 |
| full_scan | 4.53 ms [4.51 ms–4.61 ms] n=10 | 4.63 ms [4.60 ms–4.64 ms] n=10 |
| group_by | 9.26 ms [9.17 ms–9.29 ms] n=10 | 10.49 ms [10.44 ms–10.56 ms] n=10 |
| top_k | 5.40 ms [5.38 ms–5.42 ms] n=10 | 5.34 ms [5.33 ms–5.38 ms] n=10 |
| materialize_10k | 1.56 ms [1.52 ms–1.57 ms] n=10 | 7.19 ms [7.13 ms–7.22 ms] n=10 |
| update_10pct | 15.58 ms [15.14 ms–16.06 ms] n=10 | 15.00 ms [14.66 ms–15.41 ms] n=10 |
| delete_10pct | 18.27 ms [18.12 ms–18.83 ms] n=10 | 18.21 ms [18.12 ms–18.58 ms] n=10 |
| snapshot | 22.12 ms [21.73 ms–22.45 ms] n=10 | 21.61 ms [21.33 ms–21.80 ms] n=10 |
| restore | 44.87 ms [44.47 ms–45.85 ms] n=10 | 44.99 ms [44.80 ms–45.14 ms] n=10 |

