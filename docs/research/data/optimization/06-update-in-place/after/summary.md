### native – native (n = runs)

| Operation | flatsql-native |
|---|---:|
| insert | 162.05 ms [160.92 ms–163.70 ms] n=10 |
| point_lookup | 1.25 µs [1.22 µs–1.27 µs] n=10 |
| index_lookup | 0.75 µs [0.74 µs–0.75 µs] n=10 |
| range_scan | 48.7 µs [47.9 µs–49.5 µs] n=10 |
| full_scan | 5.60 ms [5.51 ms–5.64 ms] n=10 |
| group_by | 15.84 ms [15.64 ms–15.96 ms] n=10 |
| top_k | 6.00 ms [5.95 ms–6.09 ms] n=10 |
| materialize_10k | 3.61 ms [3.56 ms–3.71 ms] n=10 |
| update_10pct | 15.68 ms [15.24 ms–16.15 ms] n=10 |
| delete_10pct | 15.89 ms [15.77 ms–16.24 ms] n=10 |
| snapshot | 18.09 ms [17.64 ms–19.07 ms] n=10 |
| restore | 54.45 ms [54.32 ms–54.61 ms] n=10 |

### node – memory (n = runs)

| Operation | flatsql | flatsql-json |
|---|---:|---:|
| insert | 298.28 ms [296.27 ms–300.36 ms] n=10 | 297.34 ms [295.61 ms–298.61 ms] n=10 |
| point_lookup | 8.15 µs [8.10 µs–8.31 µs] n=10 | 4.54 µs [4.47 µs–4.87 µs] n=10 |
| index_lookup | 7.69 µs [7.52 µs–10.8 µs] n=10 | 3.63 µs [3.08 µs–4.09 µs] n=10 |
| range_scan | 57.3 µs [56.0 µs–58.1 µs] n=10 | 56.0 µs [54.4 µs–61.0 µs] n=10 |
| full_scan | 5.21 ms [5.17 ms–5.30 ms] n=10 | 5.39 ms [5.20 ms–5.51 ms] n=10 |
| group_by | 17.45 ms [17.37 ms–18.19 ms] n=10 | 17.65 ms [17.11 ms–18.45 ms] n=10 |
| top_k | 5.66 ms [5.60 ms–5.81 ms] n=10 | 5.53 ms [5.52 ms–5.76 ms] n=10 |
| materialize_10k | 10.50 ms [10.38 ms–10.95 ms] n=10 | 10.83 ms [10.65 ms–11.08 ms] n=10 |
| update_10pct | 17.54 ms [17.45 ms–17.70 ms] n=10 | 17.41 ms [17.15 ms–17.71 ms] n=10 |
| delete_10pct | 22.40 ms [21.66 ms–22.85 ms] n=10 | 21.71 ms [21.19 ms–22.72 ms] n=10 |
| snapshot | 27.74 ms [26.96 ms–29.30 ms] n=10 | 30.20 ms [27.25 ms–31.03 ms] n=10 |
| restore | 64.03 ms [62.99 ms–66.06 ms] n=10 | 63.37 ms [62.46 ms–65.16 ms] n=10 |

