//! Number of heap allocations per operation of Benchmark 1, counted by a
//! wrapper around the system allocator. The count does not depend on the
//! load of the machine, so it checks a change when timing cannot. A separate
//! program, so the counter does not slow down the timed runs of `cost-split`.
//!
//!   cargo run --release --manifest-path docs/research/data/benchmark-1/analysis/cost-split/Cargo.toml --target-dir target/analysis --bin allocations

use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicU64, Ordering::Relaxed};

use cost_split::load;
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

fn main() {
    let mut db = load();
    let star = db.prepare("SELECT * FROM items WHERE id = ?").unwrap();
    let one = db.prepare("SELECT id FROM items WHERE id = ?").unwrap();
    let scan = db.prepare("SELECT COUNT(*) FROM items WHERE price > 50 AND qty < 10").unwrap();
    let group = db.prepare("SELECT category, COUNT(*), AVG(price), MAX(qty) FROM items GROUP BY category").unwrap();

    count("point lookup SELECT *", || db.execute_prepared(&star, &[Value::Int(4242)]).unwrap());
    count("point lookup SELECT id", || db.execute_prepared(&one, &[Value::Int(4242)]).unwrap());
    count("full scan with filter", || db.execute_prepared(&scan, &[]).unwrap());
    count("GROUP BY (100 groups)", || db.execute_prepared(&group, &[]).unwrap());

    let with = db.snapshot().unwrap();
    let mut acked = Database::restore(Some(&with), &[], 1).unwrap();
    let upto = acked.stats().next_seq;
    acked.ack_outbox(upto);
    let without = acked.snapshot().unwrap();
    count("snapshot with outbox", || db.snapshot().unwrap());
    count("snapshot without outbox", || acked.snapshot().unwrap());
    count("restore with outbox", || Database::restore(Some(&with), &[], 1).unwrap());
    count("restore without outbox", || Database::restore(Some(&without), &[], 1).unwrap());
    count("UPDATE 10%", || db.execute("UPDATE items SET qty = qty + 1 WHERE id % 10 = 0").unwrap());
}
