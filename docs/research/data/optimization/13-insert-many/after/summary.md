### native – native (n = runs)

| Operation | flatsql-native |
|---|---:|
| insert | 163.36 ms [162.74 ms–165.82 ms] n=10 |
| point_lookup | 1.18 µs [1.17 µs–1.21 µs] n=10 |
| index_lookup | 0.77 µs [0.76 µs–0.78 µs] n=10 |
| range_scan | 50.0 µs [48.7 µs–50.4 µs] n=10 |
| full_scan | 5.53 ms [5.45 ms–5.64 ms] n=10 |
| group_by | 10.29 ms [10.25 ms–10.36 ms] n=10 |
| top_k | 5.97 ms [5.91 ms–6.04 ms] n=10 |
| materialize_10k | 3.50 ms [3.47 ms–3.60 ms] n=10 |
| update_10pct | 15.56 ms [15.29 ms–15.95 ms] n=10 |
| delete_10pct | 16.14 ms [15.99 ms–16.62 ms] n=10 |
| snapshot | 17.89 ms [17.64 ms–18.20 ms] n=10 |
| restore | 45.99 ms [44.87 ms–46.73 ms] n=10 |

### node – memory (n = runs)

| Operation | flatsql | flatsql-json |
|---|---:|---:|
| insert | 301.55 ms [297.95 ms–303.00 ms] n=10 | 301.40 ms [298.31 ms–303.88 ms] n=10 |
| point_lookup | 2.61 µs [2.55 µs–2.71 µs] n=10 | 4.45 µs [4.35 µs–4.50 µs] n=10 |
| index_lookup | 1.82 µs [1.75 µs–2.05 µs] n=10 | 3.08 µs [3.04 µs–3.13 µs] n=10 |
| range_scan | 51.9 µs [50.9 µs–54.2 µs] n=10 | 53.1 µs [52.7 µs–55.5 µs] n=10 |
| full_scan | 5.12 ms [5.04 ms–5.17 ms] n=10 | 5.18 ms [5.15 ms–5.21 ms] n=10 |
| group_by | 10.77 ms [10.61 ms–10.82 ms] n=10 | 10.93 ms [10.87 ms–11.86 ms] n=10 |
| top_k | 5.51 ms [5.47 ms–5.56 ms] n=10 | 5.56 ms [5.53 ms–5.62 ms] n=10 |
| materialize_10k | 1.85 ms [1.72 ms–1.96 ms] n=10 | 11.10 ms [10.99 ms–11.35 ms] n=10 |
| update_10pct | 17.13 ms [16.30 ms–18.83 ms] n=10 | 17.16 ms [16.68 ms–17.85 ms] n=10 |
| delete_10pct | 19.52 ms [19.32 ms–19.89 ms] n=10 | 21.69 ms [21.30 ms–22.23 ms] n=10 |
| snapshot | 26.16 ms [25.82 ms–27.33 ms] n=10 | 30.15 ms [29.07 ms–30.76 ms] n=10 |
| restore | 54.09 ms [52.26 ms–54.84 ms] n=10 | 58.55 ms [57.47 ms–60.03 ms] n=10 |

