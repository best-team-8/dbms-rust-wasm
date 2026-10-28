### node – memory (n = runs)

| Operation | flatsql-json |
|---|---:|
| insert | 298.75 ms [296.72 ms–300.05 ms] n=10 |
| point_lookup | 4.52 µs [4.43 µs–4.80 µs] n=10 |
| index_lookup | 3.13 µs [3.09 µs–3.29 µs] n=10 |
| range_scan | 54.7 µs [52.5 µs–58.2 µs] n=10 |
| full_scan | 5.12 ms [5.10 ms–5.27 ms] n=10 |
| group_by | 12.63 ms [12.49 ms–12.98 ms] n=10 |
| top_k | 5.56 ms [5.53 ms–5.67 ms] n=10 |
| materialize_10k | 11.00 ms [10.91 ms–11.69 ms] n=10 |
| update_10pct | 17.48 ms [16.92 ms–18.16 ms] n=10 |
| delete_10pct | 21.86 ms [21.28 ms–22.61 ms] n=10 |
| snapshot | 30.40 ms [29.03 ms–30.81 ms] n=10 |
| restore | 59.14 ms [58.54 ms–59.53 ms] n=10 |

