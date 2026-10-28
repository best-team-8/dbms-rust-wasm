### native – native (n = runs)

| Operation | flatsql-native |
|---|---:|
| insert | 210.28 ms [206.92 ms–213.16 ms] n=10 |
| point_lookup | 1.19 µs [1.18 µs–1.24 µs] n=10 |
| index_lookup | 0.75 µs [0.72 µs–0.76 µs] n=10 |
| range_scan | 50.8 µs [48.7 µs–51.4 µs] n=10 |
| full_scan | 5.52 ms [5.37 ms–5.61 ms] n=10 |
| group_by | 15.68 ms [15.58 ms–15.86 ms] n=10 |
| top_k | 5.95 ms [5.90 ms–6.02 ms] n=10 |
| materialize_10k | 3.63 ms [3.62 ms–3.75 ms] n=10 |
| update_10pct | 30.46 ms [30.18 ms–31.38 ms] n=10 |
| delete_10pct | 23.80 ms [23.62 ms–24.12 ms] n=10 |
| snapshot | 25.52 ms [25.20 ms–25.94 ms] n=10 |
| restore | 112.39 ms [111.62 ms–113.52 ms] n=10 |

### node – memory (n = runs)

| Operation | flatsql | flatsql-json |
|---|---:|---:|
| insert | 373.64 ms [371.97 ms–379.44 ms] n=10 | 374.90 ms [372.97 ms–378.09 ms] n=10 |
| point_lookup | 8.14 µs [8.06 µs–8.41 µs] n=10 | 4.83 µs [4.78 µs–4.87 µs] n=10 |
| index_lookup | 6.25 µs [5.96 µs–8.41 µs] n=10 | 3.44 µs [3.05 µs–4.13 µs] n=10 |
| range_scan | 53.5 µs [52.4 µs–58.2 µs] n=10 | 56.9 µs [55.9 µs–58.1 µs] n=10 |
| full_scan | 5.38 ms [5.13 ms–5.62 ms] n=10 | 5.23 ms [5.13 ms–5.33 ms] n=10 |
| group_by | 16.98 ms [16.69 ms–17.77 ms] n=10 | 17.11 ms [16.30 ms–17.37 ms] n=10 |
| top_k | 5.78 ms [5.61 ms–5.93 ms] n=10 | 5.92 ms [5.81 ms–6.51 ms] n=10 |
| materialize_10k | 10.03 ms [9.83 ms–10.68 ms] n=10 | 9.87 ms [9.77 ms–9.99 ms] n=10 |
| update_10pct | 43.43 ms [35.81 ms–47.30 ms] n=10 | 34.60 ms [32.26 ms–41.42 ms] n=10 |
| delete_10pct | 32.29 ms [31.42 ms–34.11 ms] n=10 | 30.70 ms [30.57 ms–32.55 ms] n=10 |
| snapshot | 37.46 ms [37.12 ms–38.12 ms] n=10 | 37.33 ms [36.90 ms–38.83 ms] n=10 |
| restore | 123.49 ms [122.12 ms–124.78 ms] n=10 | 122.74 ms [120.51 ms–126.69 ms] n=10 |

