### native – native (n = runs)

| Operation | flatsql-native |
|---|---:|
| insert | 208.35 ms [207.10 ms–209.40 ms] n=10 |
| point_lookup | 2.32 µs [2.28 µs–2.36 µs] n=10 |
| index_lookup | 1.68 µs [1.66 µs–1.68 µs] n=10 |
| range_scan | 82.2 µs [81.4 µs–84.6 µs] n=10 |
| full_scan | 8.01 ms [7.98 ms–8.11 ms] n=10 |
| group_by | 14.79 ms [14.72 ms–14.96 ms] n=10 |
| top_k | 5.52 ms [5.50 ms–5.59 ms] n=10 |
| materialize_10k | 3.72 ms [3.69 ms–3.88 ms] n=10 |
| update_10pct | 31.04 ms [30.74 ms–31.94 ms] n=10 |
| delete_10pct | 25.40 ms [24.95 ms–26.12 ms] n=10 |
| snapshot | 25.45 ms [25.23 ms–26.88 ms] n=10 |
| restore | 112.15 ms [111.03 ms–113.74 ms] n=10 |

### node – memory (n = runs)

| Operation | flatsql | flatsql-json |
|---|---:|---:|
| insert | 378.07 ms [368.82 ms–386.37 ms] n=10 | 375.19 ms [367.73 ms–388.27 ms] n=10 |
| point_lookup | 9.67 µs [9.23 µs–9.91 µs] n=10 | 6.20 µs [5.93 µs–6.53 µs] n=10 |
| index_lookup | 9.32 µs [8.77 µs–9.61 µs] n=10 | 5.52 µs [4.86 µs–5.66 µs] n=10 |
| range_scan | 87.8 µs [81.1 µs–90.9 µs] n=10 | 83.8 µs [79.5 µs–87.9 µs] n=10 |
| full_scan | 8.20 ms [8.05 ms–8.62 ms] n=10 | 7.90 ms [7.87 ms–7.94 ms] n=10 |
| group_by | 15.68 ms [15.40 ms–16.70 ms] n=10 | 15.60 ms [15.45 ms–15.82 ms] n=10 |
| top_k | 5.37 ms [5.33 ms–5.50 ms] n=10 | 5.36 ms [5.34 ms–5.41 ms] n=10 |
| materialize_10k | 10.04 ms [9.85 ms–10.22 ms] n=10 | 9.66 ms [9.46 ms–10.15 ms] n=10 |
| update_10pct | 37.47 ms [35.79 ms–43.13 ms] n=10 | 40.64 ms [36.42 ms–43.83 ms] n=10 |
| delete_10pct | 30.76 ms [29.60 ms–31.54 ms] n=10 | 29.92 ms [29.20 ms–31.78 ms] n=10 |
| snapshot | 37.07 ms [36.26 ms–38.20 ms] n=10 | 37.61 ms [36.72 ms–38.20 ms] n=10 |
| restore | 118.72 ms [117.93 ms–120.14 ms] n=10 | 120.02 ms [117.70 ms–124.28 ms] n=10 |

