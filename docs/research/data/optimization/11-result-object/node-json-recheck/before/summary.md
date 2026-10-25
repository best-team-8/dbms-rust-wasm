### node – memory (n = runs)

| Operation | flatsql-json |
|---|---:|
| insert | 299.93 ms [296.52 ms–302.20 ms] n=10 |
| point_lookup | 5.02 µs [4.58 µs–5.52 µs] n=10 |
| index_lookup | 3.04 µs [3.01 µs–4.10 µs] n=10 |
| range_scan | 54.7 µs [54.0 µs–61.2 µs] n=10 |
| full_scan | 5.26 ms [5.11 ms–5.34 ms] n=10 |
| group_by | 12.10 ms [11.29 ms–12.87 ms] n=10 |
| top_k | 5.58 ms [5.53 ms–5.87 ms] n=10 |
| materialize_10k | 11.15 ms [10.69 ms–11.82 ms] n=10 |
| update_10pct | 17.47 ms [17.14 ms–18.84 ms] n=10 |
| delete_10pct | 22.04 ms [21.50 ms–22.60 ms] n=10 |
| snapshot | 27.04 ms [26.45 ms–30.32 ms] n=10 |
| restore | 60.95 ms [58.77 ms–63.70 ms] n=10 |

