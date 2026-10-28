### native – native (n = runs)

| Operation | flatsql-native |
|---|---:|
| insert | 162.81 ms [160.66 ms–166.71 ms] n=10 |
| point_lookup | 1.22 µs [1.20 µs–1.27 µs] n=10 |
| index_lookup | 0.74 µs [0.73 µs–0.75 µs] n=10 |
| range_scan | 48.8 µs [48.5 µs–50.4 µs] n=10 |
| full_scan | 5.43 ms [5.43 ms–5.46 ms] n=10 |
| group_by | 15.69 ms [15.62 ms–15.83 ms] n=10 |
| top_k | 6.03 ms [5.99 ms–6.10 ms] n=10 |
| materialize_10k | 3.53 ms [3.48 ms–3.54 ms] n=10 |
| update_10pct | 15.57 ms [15.40 ms–15.76 ms] n=10 |
| delete_10pct | 16.07 ms [15.92 ms–16.63 ms] n=10 |
| snapshot | 17.81 ms [17.64 ms–18.01 ms] n=10 |
| restore | 46.22 ms [45.88 ms–47.73 ms] n=10 |

### node – memory (n = runs)

| Operation | flatsql | flatsql-json |
|---|---:|---:|
| insert | 300.34 ms [296.20 ms–302.29 ms] n=10 | 298.19 ms [294.39 ms–302.90 ms] n=10 |
| point_lookup | 8.27 µs [8.12 µs–8.39 µs] n=10 | 4.87 µs [4.50 µs–4.94 µs] n=10 |
| index_lookup | 9.83 µs [8.34 µs–11.3 µs] n=10 | 3.89 µs [3.17 µs–4.17 µs] n=10 |
| range_scan | 57.4 µs [56.4 µs–60.7 µs] n=10 | 54.0 µs [53.1 µs–56.6 µs] n=10 |
| full_scan | 5.19 ms [5.14 ms–5.33 ms] n=10 | 5.17 ms [5.15 ms–5.25 ms] n=10 |
| group_by | 17.89 ms [17.30 ms–19.01 ms] n=10 | 18.07 ms [17.77 ms–18.33 ms] n=10 |
| top_k | 5.63 ms [5.61 ms–5.82 ms] n=10 | 5.60 ms [5.56 ms–5.76 ms] n=10 |
| materialize_10k | 10.69 ms [10.48 ms–10.82 ms] n=10 | 10.85 ms [10.77 ms–10.93 ms] n=10 |
| update_10pct | 17.69 ms [17.11 ms–18.74 ms] n=10 | 17.02 ms [16.92 ms–17.35 ms] n=10 |
| delete_10pct | 21.96 ms [21.44 ms–23.80 ms] n=10 | 21.99 ms [21.47 ms–22.21 ms] n=10 |
| snapshot | 30.82 ms [27.02 ms–31.24 ms] n=10 | 27.65 ms [26.40 ms–30.12 ms] n=10 |
| restore | 61.15 ms [59.34 ms–63.15 ms] n=10 | 59.09 ms [58.42 ms–59.84 ms] n=10 |

