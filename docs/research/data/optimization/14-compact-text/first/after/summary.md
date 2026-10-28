### native – native (n = runs)

| Operation | flatsql-native |
|---|---:|
| insert | 178.19 ms [176.59 ms–180.39 ms] n=10 |
| point_lookup | 0.97 µs [0.94 µs–1.02 µs] n=10 |
| index_lookup | 0.70 µs [0.70 µs–0.70 µs] n=10 |
| range_scan | 51.4 µs [49.3 µs–52.4 µs] n=10 |
| full_scan | 5.44 ms [5.34 ms–5.46 ms] n=10 |
| group_by | 10.96 ms [10.85 ms–11.11 ms] n=10 |
| top_k | 6.92 ms [6.85 ms–7.01 ms] n=10 |
| materialize_10k | 3.07 ms [3.05 ms–3.10 ms] n=10 |
| update_10pct | 15.02 ms [14.87 ms–15.26 ms] n=10 |
| delete_10pct | 15.02 ms [14.83 ms–15.14 ms] n=10 |
| snapshot | 17.25 ms [17.16 ms–17.35 ms] n=10 |
| restore | 42.41 ms [42.21 ms–42.85 ms] n=10 |

### node – memory (n = runs)

| Operation | flatsql | flatsql-json |
|---|---:|---:|
| insert | 316.01 ms [312.34 ms–317.76 ms] n=10 | 310.78 ms [310.01 ms–315.23 ms] n=10 |
| point_lookup | 2.54 µs [2.52 µs–2.55 µs] n=10 | 4.27 µs [4.21 µs–4.72 µs] n=10 |
| index_lookup | 1.74 µs [1.71 µs–1.84 µs] n=10 | 2.98 µs [2.96 µs–3.13 µs] n=10 |
| range_scan | 42.9 µs [42.5 µs–43.6 µs] n=10 | 45.7 µs [45.1 µs–47.7 µs] n=10 |
| full_scan | 4.59 ms [4.54 ms–4.71 ms] n=10 | 4.54 ms [4.53 ms–4.55 ms] n=10 |
| group_by | 9.67 ms [9.55 ms–9.82 ms] n=10 | 9.82 ms [9.65 ms–10.00 ms] n=10 |
| top_k | 5.60 ms [5.48 ms–5.80 ms] n=10 | 5.58 ms [5.50 ms–5.66 ms] n=10 |
| materialize_10k | 1.76 ms [1.72 ms–1.91 ms] n=10 | 8.98 ms [8.86 ms–9.22 ms] n=10 |
| update_10pct | 15.88 ms [15.64 ms–16.34 ms] n=10 | 15.58 ms [15.38 ms–15.68 ms] n=10 |
| delete_10pct | 19.87 ms [19.14 ms–20.57 ms] n=10 | 19.12 ms [18.78 ms–19.60 ms] n=10 |
| snapshot | 23.69 ms [22.68 ms–24.57 ms] n=10 | 22.36 ms [22.14 ms–22.68 ms] n=10 |
| restore | 46.88 ms [46.76 ms–48.99 ms] n=10 | 47.18 ms [46.78 ms–47.53 ms] n=10 |

