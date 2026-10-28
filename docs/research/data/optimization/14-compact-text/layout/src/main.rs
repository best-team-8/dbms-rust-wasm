//! Row layouts for the same 100 000 rows: live heap bytes and allocations,
//! build time (as restore does: decoded values into the structure), full
//! scan with `price > 50 AND qty < 10` in key order, and key lookups.

use std::alloc::{GlobalAlloc, Layout, System};
use std::collections::BTreeMap;
use std::hint::black_box;
use std::sync::atomic::{AtomicI64, AtomicU64, Ordering::Relaxed};
use std::time::Instant;

use flatsql_core::{Stamp, Value};

struct Counting;
static LIVE: AtomicI64 = AtomicI64::new(0);
static ALLOCS: AtomicU64 = AtomicU64::new(0);
unsafe impl GlobalAlloc for Counting {
    unsafe fn alloc(&self, l: Layout) -> *mut u8 {
        LIVE.fetch_add(l.size() as i64, Relaxed);
        ALLOCS.fetch_add(1, Relaxed);
        System.alloc(l)
    }
    unsafe fn dealloc(&self, p: *mut u8, l: Layout) {
        LIVE.fetch_sub(l.size() as i64, Relaxed);
        System.dealloc(p, l)
    }
    unsafe fn realloc(&self, p: *mut u8, l: Layout, n: usize) -> *mut u8 {
        LIVE.fetch_add(n as i64 - l.size() as i64, Relaxed);
        ALLOCS.fetch_add(1, Relaxed);
        System.realloc(p, l, n)
    }
}
#[global_allocator]
static A: Counting = Counting;

const ROWS: i64 = 100_000;

fn row(i: i64) -> Vec<Value> {
    vec![
        Value::Int(i),
        Value::Text(format!("c{}", i % 100).into()),
        Value::Float(((i * 7919) % 10000) as f64 / 100.0),
        Value::Int(i % 50),
        Value::Text(format!("item-{i}").into()),
    ]
}

fn pred(price: &Value, qty: &Value) -> bool {
    matches!(price, Value::Float(p) if *p > 50.0) && matches!(qty, Value::Int(q) if *q < 10)
}

/// Inline capacity: the value stays as large as `Value` (24 B on 64-bit, 16 B on wasm32).
const INLINE: usize = if cfg!(target_pointer_width = "64") { 22 } else { 14 };

/// Short texts inline, longer ones on the heap.
#[derive(Clone)]
#[allow(dead_code)]
enum V {
    Null,
    Int(i64),
    Float(f64),
    Small(u8, [u8; INLINE]),
    Text(Box<str>),
}

fn small(v: &Value) -> V {
    match v {
        Value::Int(i) => V::Int(*i),
        Value::Float(f) => V::Float(*f),
        Value::Text(s) if s.len() <= INLINE => {
            let mut b = [0; INLINE];
            b[..s.len()].copy_from_slice(s.as_bytes());
            V::Small(s.len() as u8, b)
        }
        Value::Text(s) => V::Text(s.as_str().into()),
        _ => V::Null,
    }
}

fn small_pred(r: &[V]) -> bool {
    matches!(r[2], V::Float(p) if p > 50.0) && matches!(r[3], V::Int(q) if q < 10)
}

/// A row as bytes: per value a tag, then i64/f64 or u8 length + UTF-8.
fn encode(vs: &[Value]) -> Box<[u8]> {
    let mut b = Vec::with_capacity(40);
    for v in vs {
        match v {
            Value::Int(i) => {
                b.push(1);
                b.extend_from_slice(&i.to_le_bytes());
            }
            Value::Float(f) => {
                b.push(2);
                b.extend_from_slice(&f.to_le_bytes());
            }
            Value::Text(s) => {
                b.push(3);
                b.push(s.len() as u8);
                b.extend_from_slice(s.as_bytes());
            }
            _ => b.push(0),
        }
    }
    b.into_boxed_slice()
}

/// Column `col` of an encoded row: tag and the 8 bytes of a number.
fn field(b: &[u8], col: usize) -> (u8, [u8; 8]) {
    let mut p = 0;
    for c in 0..=col {
        let tag = b[p];
        p += 1;
        let len = match tag {
            1 | 2 => 8,
            3 => 1 + b[p] as usize,
            _ => 0,
        };
        if c == col {
            let mut x = [0; 8];
            if len == 8 {
                x.copy_from_slice(&b[p..p + 8]);
            }
            return (tag, x);
        }
        p += len;
    }
    unreachable!()
}

fn encoded_pred(b: &[u8]) -> bool {
    let (t, p) = field(b, 2);
    let (u, q) = field(b, 3);
    t == 2 && f64::from_le_bytes(p) > 50.0 && u == 1 && i64::from_le_bytes(q) < 10
}

fn measure<T>(
    name: &str,
    input: &[Vec<Value>],
    build: impl Fn(&[Vec<Value>]) -> T,
    scan: impl Fn(&T) -> usize,
    get: impl Fn(&T, i64) -> bool,
) {
    let mut best = (f64::MAX, f64::MAX, f64::MAX, 0i64, 0u64);
    for _ in 0..5 {
        let (l0, a0) = (LIVE.load(Relaxed), ALLOCS.load(Relaxed));
        let t = Instant::now();
        let s = build(input);
        let b = t.elapsed().as_secs_f64() * 1e3;
        let (live, allocs) = (LIVE.load(Relaxed) - l0, ALLOCS.load(Relaxed) - a0);
        let t = Instant::now();
        let mut n = 0;
        for _ in 0..10 {
            n = black_box(scan(&s));
        }
        assert_eq!(n, 9_990);
        let sc = t.elapsed().as_secs_f64() * 1e3 / 10.0;
        let mut k = 12345i64;
        let t = Instant::now();
        for _ in 0..200_000 {
            k = (k.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407)) & 0x7fff_ffff_ffff_ffff;
            black_box(get(&s, (k >> 20) % ROWS));
        }
        let g = t.elapsed().as_secs_f64() * 1e9 / 200_000.0;
        best = (best.0.min(b), best.1.min(sc), best.2.min(g), live, allocs);
        drop(s);
    }
    println!(
        "{name:30} | {:5.1} MiB | {:7} | {:6.1} ms | {:5.2} ms | {:4.0} ns",
        best.3 as f64 / 1048576.0,
        best.4,
        best.0,
        best.1,
        best.2
    );
}

struct Entry {
    values: Vec<Value>,
    _stamp: Stamp,
}

struct Arena<T> {
    map: BTreeMap<Value, (u32, Stamp)>,
    values: Vec<T>,
}

fn main() {
    println!(
        "size_of Value {} B, inline-text value {} B, Stamp {} B",
        std::mem::size_of::<Value>(),
        std::mem::size_of::<V>(),
        std::mem::size_of::<Stamp>()
    );
    let input: Vec<Vec<Value>> = (0..ROWS).map(row).collect();
    println!("layout                         | live heap | allocs  | build     | full scan | key lookup (best of 5)");
    measure(
        "Vec<Value> per row (today)",
        &input,
        |rows| rows.iter().map(|r| (r[0].clone(), Entry { values: r.clone(), _stamp: Stamp::default() })).collect::<BTreeMap<_, _>>(),
        |m| m.values().filter(|e| pred(&e.values[2], &e.values[3])).count(),
        |m, k| m.get(&Value::Int(k)).is_some_and(|e| !e.values.is_empty()),
    );
    measure(
        "Box<[Value]> per row",
        &input,
        |rows| rows.iter().map(|r| (r[0].clone(), (r.clone().into_boxed_slice(), Stamp::default()))).collect::<BTreeMap<_, _>>(),
        |m| m.values().filter(|e| pred(&e.0[2], &e.0[3])).count(),
        |m, k| m.get(&Value::Int(k)).is_some_and(|e| !e.0.is_empty()),
    );
    measure(
        "slot + one Vec<Value> block",
        &input,
        |rows| {
            let mut a = Arena { map: BTreeMap::new(), values: Vec::with_capacity(rows.len() * 5) };
            for (i, r) in rows.iter().enumerate() {
                a.values.extend(r.iter().cloned());
                a.map.insert(r[0].clone(), (i as u32, Stamp::default()));
            }
            a
        },
        |a| {
            a.map
                .values()
                .filter(|(s, _)| {
                    let r = &a.values[*s as usize * 5..*s as usize * 5 + 5];
                    pred(&r[2], &r[3])
                })
                .count()
        },
        |a, k| a.map.get(&Value::Int(k)).is_some_and(|(s, _)| !a.values[*s as usize * 5..].is_empty()),
    );
    measure(
        "slot + block, inline texts",
        &input,
        |rows| {
            let mut a = Arena { map: BTreeMap::new(), values: Vec::with_capacity(rows.len() * 5) };
            for (i, r) in rows.iter().enumerate() {
                a.values.extend(r.iter().map(small));
                a.map.insert(r[0].clone(), (i as u32, Stamp::default()));
            }
            a
        },
        |a| a.map.values().filter(|(s, _)| small_pred(&a.values[*s as usize * 5..*s as usize * 5 + 5])).count(),
        |a, k| a.map.get(&Value::Int(k)).is_some_and(|(s, _)| !a.values[*s as usize * 5..].is_empty()),
    );
    measure(
        "Vec per row, inline texts",
        &input,
        |rows| rows.iter().map(|r| (r[0].clone(), (r.iter().map(small).collect::<Vec<V>>(), Stamp::default()))).collect::<BTreeMap<_, _>>(),
        |m| m.values().filter(|e| small_pred(&e.0)).count(),
        |m, k| m.get(&Value::Int(k)).is_some_and(|e| !e.0.is_empty()),
    );
    measure(
        "encoded bytes per row",
        &input,
        |rows| rows.iter().map(|r| (r[0].clone(), (encode(r), Stamp::default()))).collect::<BTreeMap<_, _>>(),
        |m| m.values().filter(|e| encoded_pred(&e.0)).count(),
        |m, k| m.get(&Value::Int(k)).is_some_and(|e| !e.0.is_empty()),
    );
}
