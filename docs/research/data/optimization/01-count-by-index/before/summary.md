### native – native (n = runs)

| Operation | flatsql-native |
|---|---:|
| insert | 220.30 ms [218.36 ms–223.13 ms] n=10 |
| point_lookup | 2.47 µs [2.39 µs–2.60 µs] n=10 |
| index_lookup | 485.7 µs [478.4 µs–487.6 µs] n=10 |
| range_scan | 88.7 µs [86.1 µs–95.1 µs] n=10 |
| full_scan | 8.45 ms [8.38 ms–8.53 ms] n=10 |
| group_by | 15.33 ms [15.08 ms–15.44 ms] n=10 |
| top_k | 5.91 ms [5.84 ms–5.99 ms] n=10 |
| materialize_10k | 3.96 ms [3.91 ms–4.08 ms] n=10 |
| update_10pct | 33.13 ms [31.96 ms–34.88 ms] n=10 |
| delete_10pct | 25.87 ms [25.67 ms–26.34 ms] n=10 |
| snapshot | 26.72 ms [26.32 ms–27.17 ms] n=10 |
| restore | 116.92 ms [115.36 ms–119.73 ms] n=10 |

### node – memory (n = runs)

| Operation | flatsql | flatsql-json |
|---|---:|---:|
| insert | 426.60 ms [411.58 ms–438.84 ms] n=10 | 415.99 ms [411.72 ms–461.67 ms] n=10 |
| point_lookup | 10.6 µs [10.3 µs–10.8 µs] n=10 | 6.30 µs [6.19 µs–6.72 µs] n=10 |
| index_lookup | 497.8 µs [482.2 µs–536.9 µs] n=10 | 484.1 µs [460.9 µs–519.6 µs] n=10 |
| range_scan | 96.5 µs [90.8 µs–98.9 µs] n=10 | 93.4 µs [88.6 µs–96.6 µs] n=10 |
| full_scan | 8.32 ms [8.19 ms–8.89 ms] n=10 | 8.23 ms [8.14 ms–8.38 ms] n=10 |
| group_by | 16.85 ms [16.69 ms–17.49 ms] n=10 | 17.75 ms [17.23 ms–18.41 ms] n=10 |
| top_k | 5.73 ms [5.64 ms–6.01 ms] n=10 | 5.76 ms [5.58 ms–6.67 ms] n=10 |
| materialize_10k | 9.94 ms [9.81 ms–10.33 ms] n=10 | 10.25 ms [10.14 ms–10.35 ms] n=10 |
| update_10pct | 43.99 ms [36.77 ms–51.62 ms] n=10 | 44.93 ms [37.60 ms–49.15 ms] n=10 |
| delete_10pct | 31.99 ms [31.10 ms–35.09 ms] n=10 | 30.72 ms [30.12 ms–31.87 ms] n=10 |
| snapshot | 37.60 ms [37.03 ms–38.38 ms] n=10 | 37.54 ms [37.15 ms–38.15 ms] n=10 |
| restore | 122.41 ms [121.23 ms–126.50 ms] n=10 | 122.16 ms [119.78 ms–123.53 ms] n=10 |

