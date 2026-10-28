### node – memory (n = runs)

| Operation | flatsql | flatsql-json |
|---|---:|---:|
| insert | 356.39 ms [352.74 ms–360.41 ms] n=10 | 354.30 ms [352.87 ms–355.86 ms] n=10 |
| point_lookup | 3.04 µs [3.00 µs–3.07 µs] n=10 | 5.35 µs [5.26 µs–5.51 µs] n=10 |
| index_lookup | 2.64 µs [2.51 µs–3.09 µs] n=10 | 4.15 µs [4.01 µs–4.61 µs] n=10 |
| range_scan | 74.6 µs [74.3 µs–75.8 µs] n=10 | 73.7 µs [72.8 µs–75.4 µs] n=10 |
| full_scan | 6.72 ms [6.61 ms–6.79 ms] n=10 | 6.64 ms [6.60 ms–6.74 ms] n=10 |
| group_by | 14.42 ms [14.39 ms–14.48 ms] n=10 | 14.01 ms [13.97 ms–14.12 ms] n=10 |
| top_k | 8.45 ms [8.38 ms–8.58 ms] n=10 | 9.69 ms [8.81 ms–9.83 ms] n=10 |
| materialize_10k | 2.61 ms [2.53 ms–2.79 ms] n=10 | 11.38 ms [11.27 ms–11.52 ms] n=10 |
| update_10pct | 21.00 ms [20.55 ms–21.17 ms] n=10 | 20.92 ms [20.57 ms–21.04 ms] n=10 |
| delete_10pct | 24.01 ms [23.65 ms–24.60 ms] n=10 | 23.46 ms [22.92 ms–24.07 ms] n=10 |
| snapshot | 26.76 ms [26.34 ms–27.21 ms] n=10 | 26.67 ms [26.32 ms–27.01 ms] n=10 |
| restore | 70.35 ms [69.74 ms–73.22 ms] n=10 | 69.89 ms [69.56 ms–70.36 ms] n=10 |

