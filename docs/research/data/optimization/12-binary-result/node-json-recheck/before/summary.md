### node – memory (n = runs)

| Operation | flatsql-json |
|---|---:|
| insert | 298.06 ms [295.90 ms–301.29 ms] n=10 |
| point_lookup | 4.41 µs [4.37 µs–4.51 µs] n=10 |
| index_lookup | 3.22 µs [3.09 µs–3.67 µs] n=10 |
| range_scan | 53.2 µs [52.4 µs–55.2 µs] n=10 |
| full_scan | 5.17 ms [5.10 ms–5.19 ms] n=10 |
| group_by | 12.54 ms [12.47 ms–12.74 ms] n=10 |
| top_k | 5.57 ms [5.51 ms–5.90 ms] n=10 |
| materialize_10k | 10.92 ms [10.68 ms–10.98 ms] n=10 |
| update_10pct | 17.25 ms [16.92 ms–18.13 ms] n=10 |
| delete_10pct | 22.56 ms [22.10 ms–22.91 ms] n=10 |
| snapshot | 30.67 ms [30.08 ms–30.94 ms] n=10 |
| restore | 58.12 ms [57.40 ms–59.95 ms] n=10 |

