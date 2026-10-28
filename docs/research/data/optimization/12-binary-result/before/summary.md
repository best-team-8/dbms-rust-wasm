### native – native (n = runs)

| Operation | flatsql-native |
|---|---:|
| insert | 164.71 ms [163.42 ms–166.24 ms] n=10 |
| point_lookup | 1.21 µs [1.20 µs–1.32 µs] n=10 |
| index_lookup | 0.74 µs [0.73 µs–0.75 µs] n=10 |
| range_scan | 50.2 µs [48.9 µs–53.4 µs] n=10 |
| full_scan | 5.51 ms [5.48 ms–5.54 ms] n=10 |
| group_by | 10.43 ms [10.31 ms–10.58 ms] n=10 |
| top_k | 6.08 ms [6.05 ms–6.12 ms] n=10 |
| materialize_10k | 3.82 ms [3.79 ms–3.92 ms] n=10 |
| update_10pct | 15.46 ms [15.36 ms–16.13 ms] n=10 |
| delete_10pct | 16.91 ms [16.25 ms–17.61 ms] n=10 |
| snapshot | 17.81 ms [17.78 ms–17.93 ms] n=10 |
| restore | 45.34 ms [44.49 ms–46.09 ms] n=10 |

### node – memory (n = runs)

| Operation | flatsql | flatsql-json |
|---|---:|---:|
| insert | 299.23 ms [294.75 ms–306.34 ms] n=10 | 299.57 ms [294.12 ms–303.58 ms] n=10 |
| point_lookup | 3.12 µs [3.02 µs–3.24 µs] n=10 | 4.86 µs [4.55 µs–5.32 µs] n=10 |
| index_lookup | 2.03 µs [1.95 µs–2.09 µs] n=10 | 3.14 µs [3.02 µs–3.59 µs] n=10 |
| range_scan | 54.3 µs [53.8 µs–59.0 µs] n=10 | 52.9 µs [52.0 µs–55.8 µs] n=10 |
| full_scan | 5.22 ms [5.17 ms–5.40 ms] n=10 | 5.21 ms [5.12 ms–5.28 ms] n=10 |
| group_by | 11.29 ms [11.18 ms–11.57 ms] n=10 | 12.59 ms [12.54 ms–13.13 ms] n=10 |
| top_k | 5.55 ms [5.52 ms–5.80 ms] n=10 | 5.64 ms [5.61 ms–5.77 ms] n=10 |
| materialize_10k | 10.80 ms [10.33 ms–11.41 ms] n=10 | 11.02 ms [10.80 ms–11.09 ms] n=10 |
| update_10pct | 17.32 ms [17.10 ms–17.84 ms] n=10 | 17.40 ms [17.09 ms–17.91 ms] n=10 |
| delete_10pct | 22.25 ms [21.83 ms–22.73 ms] n=10 | 21.21 ms [20.78 ms–21.70 ms] n=10 |
| snapshot | 31.06 ms [30.73 ms–31.48 ms] n=10 | 30.96 ms [28.61 ms–31.35 ms] n=10 |
| restore | 58.34 ms [57.12 ms–61.14 ms] n=10 | 59.39 ms [57.44 ms–60.99 ms] n=10 |

