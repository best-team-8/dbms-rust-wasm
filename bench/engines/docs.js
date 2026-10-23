// Shared parts of the adapters of engines without SQL (Dexie, PouchDB, RxDB):
// row ↔ document conversion and the answers of the SQL queries computed from
// documents in JavaScript (for operations the engine's API cannot express,
// e.g. GROUP BY). Every adapter documents which parts run in the engine.

export const pad = (id) => String(id).padStart(8, '0');
export const toDoc = (r) => ({ id: r[0], category: r[1], price: r[2], qty: r[3], name: r[4] });
export const toRow = (d) => [d.id, d.category, d.price, d.qty, d.name];

export function sumPrice(docs) {
  let sum = 0;
  for (const d of docs) sum += d.price;
  return [[sum]];
}

export function countFiltered(docs) {
  let n = 0;
  for (const d of docs) if (d.price > 50 && d.qty < 10) n++;
  return [[n]];
}

export function groupBy(docs) {
  const g = new Map();
  for (const d of docs) {
    let a = g.get(d.category);
    if (!a) g.set(d.category, (a = { n: 0, sum: 0, max: -Infinity }));
    a.n++;
    a.sum += d.price;
    if (d.qty > a.max) a.max = d.qty;
  }
  return [...g].map(([k, a]) => [k, a.n, a.sum / a.n, a.max]);
}

/** price DESC, id ASC, first 10 – from any set that contains the answer. */
export function top10(docs) {
  return docs
    .map((d) => [d.id, d.price])
    .sort((x, y) => y[1] - x[1] || x[0] - y[0])
    .slice(0, 10);
}

export function verifyDocs(docs) {
  let qty = 0;
  let price = 0;
  let n = 0;
  for (const d of docs) {
    n++;
    qty += d.qty;
    price += d.price;
  }
  return [n, qty, price];
}
