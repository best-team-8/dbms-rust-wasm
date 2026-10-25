//! Number of heap allocations per operation of Benchmark 1, counted by a
//! wrapper around the system allocator. The count does not depend on the
//! load of the machine, so it checks a change when timing cannot. A separate
//! program, so the counter does not slow down the timed runs of `cost-split`.
//!
//!   cargo run --release --manifest-path docs/research/data/benchmark-1/analysis/cost-split/Cargo.toml --target-dir target/analysis --bin allocations

use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicU64, Ordering::Relaxed};

use cost_split::{load, row, with_outbox};
use flatsql_core::{Database, Value};

struct Counting;

static ALLOCS: AtomicU64 = AtomicU64::new(0);
static BYTES: AtomicU64 = AtomicU64::new(0);

unsafe impl GlobalAlloc for Counting {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        ALLOCS.fetch_add(1, Relaxed);
        BYTES.fetch_add(layout.size() as u64, Relaxed);
        System.alloc(layout)
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        System.dealloc(ptr, layout)
    }

    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        ALLOCS.fetch_add(1, Relaxed);
        BYTES.fetch_add(new_size as u64, Relaxed);
        System.realloc(ptr, layout, new_size)
    }
}

#[global_allocator]
static COUNTING: Counting = Counting;

/// Allocations (including reallocations) and requested bytes of `f`; the
/// result is dropped inside, as in the benchmark loop.
fn count<T>(name: &str, f: impl FnOnce() -> T) {
    let (a, b) = (ALLOCS.load(Relaxed), BYTES.load(Relaxed));
    drop(f());
    println!("{name}: {} allocations, {} bytes", ALLOCS.load(Relaxed) - a, BYTES.load(Relaxed) - b);
}

/// 1000 new rows in one transaction.
fn insert_rows(db: &mut Database) {
    let first = db.row_count("items").unwrap() as i64 + 1_000_000;
    let ins = db.prepare("INSERT INTO items VALUES (?, ?, ?, ?, ?)").unwrap();
    db.execute("BEGIN").unwrap();
    for i in first..first + 1000 {
        db.execute_prepared(&ins, &row(i)).unwrap();
    }
    db.execute("COMMIT").unwrap();
    db.take_wal();
}

fn main() {
    let mut db = load();
    let star = db.prepare("SELECT * FROM items WHERE id = ?").unwrap();
    let one = db.prepare("SELECT id FROM items WHERE id = ?").unwrap();
    let by_index = db.prepare("SELECT COUNT(*) FROM items WHERE category = ?").unwrap();
    let range = db.prepare("SELECT SUM(price) FROM items WHERE id BETWEEN ? AND ?").unwrap();
    let scan = db.prepare("SELECT COUNT(*) FROM items WHERE price > 50 AND qty < 10").unwrap();
    let group = db.prepare("SELECT category, COUNT(*), AVG(price), MAX(qty) FROM items GROUP BY category").unwrap();

    count("point lookup SELECT *", || db.execute_prepared(&star, &[Value::Int(4242)]).unwrap());
    count("point lookup SELECT id", || db.execute_prepared(&one, &[Value::Int(4242)]).unwrap());
    count("COUNT(*) by index", || db.execute_prepared(&by_index, &[Value::from("c42")]).unwrap());
    count("key range (1000 rows) + SUM", || db.execute_prepared(&range, &[Value::Int(4242), Value::Int(5241)]).unwrap());
    count("prepare point lookup", || db.prepare("SELECT * FROM items WHERE id = ?").unwrap());
    count("full scan with filter", || db.execute_prepared(&scan, &[]).unwrap());
    count("GROUP BY (100 groups)", || db.execute_prepared(&group, &[]).unwrap());
    #[cfg(core_encode)]
    {
        // The buffer is kept between results, as in the WebAssembly binding.
        let many = db.prepare("SELECT * FROM items WHERE id < 10000").unwrap();
        let mut out = Vec::new();
        db.execute_prepared_encoded(&many, &[], &mut out).unwrap();
        count("10 000 rows collected", || db.execute_prepared(&many, &[]).unwrap());
        count("10 000 rows encoded from the table", || db.execute_prepared_encoded(&many, &[], &mut out).unwrap());
        count("point lookup SELECT * encoded from the table", || {
            db.execute_prepared_encoded(&star, &[Value::Int(4242)], &mut out).unwrap()
        });
    }

    let with = with_outbox(&db);
    let mut outboxed = Database::restore(Some(&with), &[], 1).unwrap();
    let mut acked = Database::restore(Some(&with), &[], 1).unwrap();
    let upto = acked.stats().next_seq;
    acked.ack_outbox(upto);
    let without = acked.snapshot().unwrap();
    let created = db.snapshot().unwrap();
    count("snapshot, benchmark database", || db.snapshot().unwrap());
    count("restore, benchmark database", || Database::restore(Some(&created), &[], 1).unwrap());
    count("snapshot with outbox", || outboxed.snapshot().unwrap());
    count("snapshot without outbox", || acked.snapshot().unwrap());
    count("restore with outbox", || Database::restore(Some(&with), &[], 1).unwrap());
    count("restore without outbox", || Database::restore(Some(&without), &[], 1).unwrap());
    count("UPDATE 10%", || db.execute("UPDATE items SET qty = qty + 1 WHERE id % 10 = 0").unwrap());
    count("insert 1000 rows, benchmark database", || insert_rows(&mut db));
    count("insert 1000 rows, outbox kept", || insert_rows(&mut outboxed));
    #[cfg(core_many)]
    count("insert 1000 rows in one call, benchmark database", || {
        let first = db.row_count("items").unwrap() as i64 + 2_000_000;
        let ins = db.prepare("INSERT INTO items VALUES (?, ?, ?, ?, ?)").unwrap();
        db.execute("BEGIN").unwrap();
        let mut next = first;
        db.execute_many(&ins, |params| {
            if next == first + 1000 {
                return Ok(false);
            }
            params.extend(row(next));
            next += 1;
            Ok(true)
        })
        .unwrap();
        db.execute("COMMIT").unwrap();
        db.take_wal();
    });
}
