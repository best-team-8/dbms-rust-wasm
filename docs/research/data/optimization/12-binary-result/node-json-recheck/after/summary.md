### node – memory (n = runs)

| Operation | flatsql-json |
|---|---:|
| insert | 296.44 ms [294.93 ms–304.61 ms] n=10 |
| point_lookup | 4.62 µs [4.45 µs–4.75 µs] n=10 |
| index_lookup | 3.13 µs [3.03 µs–3.42 µs] n=10 |
| range_scan | 66.7 µs [63.3 µs–68.4 µs] n=10 |
| full_scan | 5.14 ms [5.10 ms–5.18 ms] n=10 |
| group_by | 11.03 ms [10.92 ms–11.37 ms] n=10 |
| top_k | 5.60 ms [5.56 ms–5.66 ms] n=10 |
| materialize_10k | 10.96 ms [10.79 ms–11.18 ms] n=10 |
| update_10pct | 17.08 ms [16.56 ms–17.49 ms] n=10 |
| delete_10pct | 21.84 ms [21.59 ms–22.32 ms] n=10 |
| snapshot | 26.88 ms [26.81 ms–28.74 ms] n=10 |
| restore | 58.58 ms [57.72 ms–60.19 ms] n=10 |

