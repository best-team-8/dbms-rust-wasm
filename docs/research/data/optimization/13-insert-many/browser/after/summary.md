### browser – memory (n = runs)

| Operation | flatsql | flatsql-json |
|---|---:|---:|
| insert | 281.09 ms [279.25 ms–286.31 ms] n=10 | 278.85 ms [274.88 ms–284.26 ms] n=10 |
| point_lookup | 3.32 µs [3.27 µs–3.36 µs] n=10 | 4.51 µs [4.48 µs–4.52 µs] n=10 |
| index_lookup | 1.90 µs [1.86 µs–1.91 µs] n=10 | 2.90 µs [2.87 µs–2.96 µs] n=10 |
| range_scan | 54.2 µs [52.4 µs–55.2 µs] n=10 | 56.1 µs [54.0 µs–57.0 µs] n=10 |
| full_scan | 5.03 ms [5.02 ms–5.06 ms] n=10 | 5.05 ms [4.99 ms–5.11 ms] n=10 |
| group_by | 11.42 ms [11.35 ms–11.51 ms] n=10 | 12.01 ms [11.91 ms–12.16 ms] n=10 |
| top_k | 5.54 ms [5.51 ms–5.57 ms] n=10 | 6.10 ms [5.94 ms–6.17 ms] n=10 |
| materialize_10k | 1.60 ms [1.53 ms–1.64 ms] n=10 | 9.27 ms [9.12 ms–10.00 ms] n=10 |
| update_10pct | 16.05 ms [15.67 ms–16.39 ms] n=10 | 17.16 ms [16.33 ms–17.99 ms] n=10 |
| delete_10pct | 19.17 ms [19.02 ms–19.58 ms] n=10 | 21.73 ms [20.45 ms–22.86 ms] n=10 |
| snapshot | 24.98 ms [24.78 ms–26.06 ms] n=10 | 28.49 ms [27.68 ms–29.30 ms] n=10 |
| restore | 51.81 ms [50.82 ms–52.07 ms] n=10 | 56.98 ms [56.43 ms–57.75 ms] n=10 |

