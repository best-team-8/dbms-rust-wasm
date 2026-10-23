// Statistics over repeated process runs (docs/research/benchmark-scenario.md):
// median with quartiles as the main value, mean with a 95% confidence
// interval (Student's t) for comparisons between versions.

// Two-sided 97.5% quantiles of Student's t distribution, df = 1..30.
const T975 = [
  12.706, 4.303, 3.182, 2.776, 2.571, 2.447, 2.365, 2.306, 2.262, 2.228, 2.201, 2.179, 2.16, 2.145, 2.131, 2.12, 2.11, 2.101, 2.093,
  2.086, 2.08, 2.074, 2.069, 2.064, 2.06, 2.056, 2.052, 2.048, 2.045, 2.042,
];

/** Quantile with linear interpolation between order statistics (type 7). */
export function quantile(sorted, p) {
  if (sorted.length === 0) return NaN;
  const h = (sorted.length - 1) * p;
  const lo = Math.floor(h);
  return sorted[lo] + (h - lo) * ((sorted[Math.min(lo + 1, sorted.length - 1)] ?? sorted[lo]) - sorted[lo]);
}

/** Summary of one metric measured once per run. */
export function summarize(values) {
  const v = values.filter((x) => Number.isFinite(x)).sort((a, b) => a - b);
  const n = v.length;
  if (n === 0) return null;
  const mean = v.reduce((a, b) => a + b, 0) / n;
  const sd = n > 1 ? Math.sqrt(v.reduce((a, b) => a + (b - mean) ** 2, 0) / (n - 1)) : 0;
  const t = n > 1 ? (T975[n - 2] ?? 1.96) : 0;
  const half = n > 1 ? (t * sd) / Math.sqrt(n) : 0;
  const median = quantile(v, 0.5);
  const q1 = quantile(v, 0.25);
  const q3 = quantile(v, 0.75);
  return { n, median, q1, q3, iqr: q3 - q1, min: v[0], max: v[n - 1], mean, sd, ci95: [mean - half, mean + half], cv: mean ? sd / mean : 0 };
}
