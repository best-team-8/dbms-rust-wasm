### node – memory (n = runs)

| Operation | flatsql | flatsql-json |
|---|---:|---:|
| insert | 378.79 ms [377.25 ms–386.94 ms] n=10 | 400.02 ms [385.54 ms–413.11 ms] n=10 |
| point_lookup | 8.15 µs [7.99 µs–8.49 µs] n=10 | 4.92 µs [4.78 µs–5.02 µs] n=10 |
| index_lookup | 5.90 µs [5.89 µs–5.96 µs] n=10 | 3.17 µs [3.04 µs–4.01 µs] n=10 |
| range_scan | 54.3 µs [53.2 µs–56.9 µs] n=10 | 54.0 µs [53.5 µs–58.8 µs] n=10 |
| full_scan | 5.16 ms [5.11 ms–5.19 ms] n=10 | 5.12 ms [5.09 ms–5.16 ms] n=10 |
| group_by | 17.29 ms [16.38 ms–18.11 ms] n=10 | 16.25 ms [16.05 ms–16.96 ms] n=10 |
| top_k | 5.83 ms [5.58 ms–6.06 ms] n=10 | 5.88 ms [5.78 ms–6.24 ms] n=10 |
| materialize_10k | 10.00 ms [9.88 ms–10.11 ms] n=10 | 9.84 ms [9.73 ms–9.96 ms] n=10 |
| update_10pct | 39.37 ms [33.40 ms–46.33 ms] n=10 | 34.50 ms [33.15 ms–40.82 ms] n=10 |
| delete_10pct | 31.76 ms [30.60 ms–33.60 ms] n=10 | 31.87 ms [30.63 ms–33.61 ms] n=10 |
| snapshot | 38.12 ms [37.91 ms–39.44 ms] n=10 | 37.52 ms [35.20 ms–38.84 ms] n=10 |
| restore | 123.24 ms [121.58 ms–124.71 ms] n=10 | 123.80 ms [122.24 ms–127.15 ms] n=10 |

