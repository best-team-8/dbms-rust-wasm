### native – native (n = runs)

| Operation | flatsql-native |
|---|---:|
| insert | 163.23 ms [162.30 ms–164.63 ms] n=10 |
| point_lookup | 1.21 µs [1.19 µs–1.29 µs] n=10 |
| index_lookup | 0.76 µs [0.75 µs–0.77 µs] n=10 |
| range_scan | 48.3 µs [47.9 µs–49.2 µs] n=10 |
| full_scan | 5.56 ms [5.53 ms–5.60 ms] n=10 |
| group_by | 10.32 ms [10.27 ms–10.51 ms] n=10 |
| top_k | 6.01 ms [5.94 ms–6.12 ms] n=10 |
| materialize_10k | 3.78 ms [3.74 ms–3.83 ms] n=10 |
| update_10pct | 16.06 ms [15.74 ms–16.62 ms] n=10 |
| delete_10pct | 17.01 ms [16.25 ms–17.55 ms] n=10 |
| snapshot | 17.92 ms [17.75 ms–18.39 ms] n=10 |
| restore | 45.14 ms [44.77 ms–46.05 ms] n=10 |

### node – memory (n = runs)

| Operation | flatsql | flatsql-json |
|---|---:|---:|
| insert | 297.94 ms [294.14 ms–302.21 ms] n=10 | 295.32 ms [293.29 ms–300.51 ms] n=10 |
| point_lookup | 3.04 µs [2.96 µs–3.07 µs] n=10 | 4.46 µs [4.42 µs–4.60 µs] n=10 |
| index_lookup | 1.95 µs [1.90 µs–2.11 µs] n=10 | 3.21 µs [3.08 µs–3.42 µs] n=10 |
| range_scan | 52.9 µs [52.5 µs–54.2 µs] n=10 | 53.0 µs [52.2 µs–55.0 µs] n=10 |
| full_scan | 5.23 ms [5.11 ms–5.30 ms] n=10 | 5.18 ms [5.08 ms–5.35 ms] n=10 |
| group_by | 11.46 ms [11.36 ms–11.50 ms] n=10 | 12.57 ms [12.36 ms–12.83 ms] n=10 |
| top_k | 5.57 ms [5.52 ms–5.68 ms] n=10 | 5.52 ms [5.48 ms–5.62 ms] n=10 |
| materialize_10k | 10.49 ms [10.34 ms–11.00 ms] n=10 | 10.95 ms [10.79 ms–11.15 ms] n=10 |
| update_10pct | 17.37 ms [17.15 ms–17.81 ms] n=10 | 17.04 ms [16.80 ms–17.32 ms] n=10 |
| delete_10pct | 22.00 ms [21.81 ms–22.29 ms] n=10 | 21.53 ms [20.98 ms–22.55 ms] n=10 |
| snapshot | 27.77 ms [26.90 ms–30.11 ms] n=10 | 30.65 ms [30.22 ms–31.32 ms] n=10 |
| restore | 59.31 ms [56.59 ms–59.62 ms] n=10 | 59.61 ms [59.40 ms–60.14 ms] n=10 |

