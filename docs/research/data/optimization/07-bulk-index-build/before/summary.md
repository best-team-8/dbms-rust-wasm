### native – native (n = runs)

| Operation | flatsql-native |
|---|---:|
| insert | 162.52 ms [161.58 ms–164.33 ms] n=10 |
| point_lookup | 1.22 µs [1.20 µs–1.32 µs] n=10 |
| index_lookup | 0.74 µs [0.73 µs–0.75 µs] n=10 |
| range_scan | 50.3 µs [49.7 µs–52.2 µs] n=10 |
| full_scan | 5.64 ms [5.56 ms–5.72 ms] n=10 |
| group_by | 15.79 ms [15.71 ms–15.93 ms] n=10 |
| top_k | 6.01 ms [5.96 ms–6.15 ms] n=10 |
| materialize_10k | 3.62 ms [3.54 ms–3.66 ms] n=10 |
| update_10pct | 15.49 ms [15.34 ms–15.77 ms] n=10 |
| delete_10pct | 16.43 ms [15.93 ms–16.71 ms] n=10 |
| snapshot | 18.05 ms [17.71 ms–18.70 ms] n=10 |
| restore | 53.93 ms [53.72 ms–54.88 ms] n=10 |

### node – memory (n = runs)

| Operation | flatsql | flatsql-json |
|---|---:|---:|
| insert | 294.67 ms [294.37 ms–305.94 ms] n=10 | 295.39 ms [293.98 ms–298.47 ms] n=10 |
| point_lookup | 8.34 µs [8.23 µs–8.73 µs] n=10 | 4.60 µs [4.51 µs–4.97 µs] n=10 |
| index_lookup | 8.02 µs [7.30 µs–9.35 µs] n=10 | 3.21 µs [2.99 µs–3.68 µs] n=10 |
| range_scan | 59.7 µs [56.6 µs–61.4 µs] n=10 | 54.2 µs [53.1 µs–55.1 µs] n=10 |
| full_scan | 5.27 ms [5.26 ms–5.32 ms] n=10 | 5.46 ms [5.20 ms–5.57 ms] n=10 |
| group_by | 18.77 ms [18.32 ms–19.20 ms] n=10 | 17.33 ms [17.22 ms–17.81 ms] n=10 |
| top_k | 5.93 ms [5.64 ms–6.53 ms] n=10 | 5.78 ms [5.64 ms–5.99 ms] n=10 |
| materialize_10k | 10.81 ms [10.57 ms–11.16 ms] n=10 | 10.89 ms [10.69 ms–11.65 ms] n=10 |
| update_10pct | 17.44 ms [17.03 ms–17.78 ms] n=10 | 17.83 ms [17.00 ms–18.60 ms] n=10 |
| delete_10pct | 22.05 ms [21.21 ms–22.82 ms] n=10 | 22.43 ms [21.96 ms–22.84 ms] n=10 |
| snapshot | 30.85 ms [27.43 ms–31.77 ms] n=10 | 27.64 ms [27.19 ms–29.94 ms] n=10 |
| restore | 63.72 ms [62.47 ms–66.34 ms] n=10 | 66.76 ms [63.25 ms–73.78 ms] n=10 |

