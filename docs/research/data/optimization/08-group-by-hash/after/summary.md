### native – native (n = runs)

| Operation | flatsql-native |
|---|---:|
| insert | 173.00 ms [171.28 ms–174.72 ms] n=10 |
| point_lookup | 1.29 µs [1.27 µs–1.35 µs] n=10 |
| index_lookup | 0.76 µs [0.76 µs–0.77 µs] n=10 |
| range_scan | 53.3 µs [51.9 µs–57.7 µs] n=10 |
| full_scan | 5.84 ms [5.76 ms–5.91 ms] n=10 |
| group_by | 10.99 ms [10.79 ms–11.17 ms] n=10 |
| top_k | 6.53 ms [6.38 ms–6.55 ms] n=10 |
| materialize_10k | 4.00 ms [3.96 ms–4.05 ms] n=10 |
| update_10pct | 16.17 ms [15.97 ms–16.59 ms] n=10 |
| delete_10pct | 17.84 ms [17.47 ms–18.08 ms] n=10 |
| snapshot | 18.74 ms [18.46 ms–19.32 ms] n=10 |
| restore | 47.76 ms [46.22 ms–48.63 ms] n=10 |

### node – memory (n = runs)

| Operation | flatsql | flatsql-json |
|---|---:|---:|
| insert | 318.21 ms [313.62 ms–325.71 ms] n=10 | 318.35 ms [316.92 ms–326.59 ms] n=10 |
| point_lookup | 8.73 µs [8.64 µs–9.07 µs] n=10 | 5.35 µs [4.98 µs–5.59 µs] n=10 |
| index_lookup | 9.82 µs [9.44 µs–13.1 µs] n=10 | 3.25 µs [3.12 µs–3.48 µs] n=10 |
| range_scan | 67.2 µs [65.6 µs–70.7 µs] n=10 | 63.4 µs [60.0 µs–67.0 µs] n=10 |
| full_scan | 5.71 ms [5.54 ms–5.96 ms] n=10 | 5.57 ms [5.49 ms–6.11 ms] n=10 |
| group_by | 11.74 ms [11.59 ms–12.34 ms] n=10 | 12.13 ms [11.88 ms–12.40 ms] n=10 |
| top_k | 6.00 ms [5.93 ms–6.09 ms] n=10 | 6.05 ms [5.97 ms–6.21 ms] n=10 |
| materialize_10k | 11.15 ms [10.96 ms–12.26 ms] n=10 | 11.63 ms [11.46 ms–11.88 ms] n=10 |
| update_10pct | 18.82 ms [17.85 ms–19.34 ms] n=10 | 19.02 ms [17.99 ms–21.05 ms] n=10 |
| delete_10pct | 23.59 ms [22.39 ms–24.66 ms] n=10 | 23.21 ms [22.66 ms–24.17 ms] n=10 |
| snapshot | 31.91 ms [31.52 ms–32.26 ms] n=10 | 31.34 ms [30.90 ms–31.74 ms] n=10 |
| restore | 62.73 ms [61.43 ms–64.42 ms] n=10 | 63.00 ms [62.09 ms–64.84 ms] n=10 |

