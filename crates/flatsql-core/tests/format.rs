//! Deterministic binary format: equal data gives equal bytes.

use flatsql_core::sync::decode_batch;
use flatsql_core::{Database, Value};

/// Pure logical clock: makes stamps deterministic.
fn logical() -> u64 {
    0
}

/// NaN bit patterns: quiet, negative (the result of 0/0 on x86), signalling
/// and with a payload. WebAssembly arithmetic may produce any of them.
const NANS: [u64; 4] = [0x7FF8_0000_0000_0000, 0xFFF8_0000_0000_0000, 0x7FF0_0000_0000_0001, 0x7FFF_FFFF_FFFF_FFFF];

/// Every byte image of a database holding one NaN, written through `nan`.
fn images(nan: f64) -> Vec<(&'static str, Vec<u8>)> {
    let mut db = Database::new(7);
    db.set_clock(logical);
    db.enable_sync().unwrap();
    db.execute("CREATE TABLE m (id INT PRIMARY KEY, x FLOAT)").unwrap();
    let insert = db.prepare("INSERT INTO m VALUES (?, ?)").unwrap();
    db.execute_prepared(&insert, &[Value::Int(1), Value::Float(nan)]).unwrap();
    db.execute_prepared(&insert, &[Value::Int(2), Value::Float(1.5)]).unwrap();
    let (outbox, _) = db.export_outbox(usize::MAX).unwrap();
    vec![
        ("WAL", db.take_wal()),
        ("snapshot", db.snapshot().unwrap()),
        ("outbox batch", outbox),
        ("state batch", db.export_state().unwrap()),
    ]
}

#[test]
fn equal_data_encodes_to_identical_bytes_whatever_the_nan_bits() {
    let reference = images(f64::from_bits(NANS[0]));
    for bits in &NANS[1..] {
        let nan = f64::from_bits(*bits);
        assert!(nan.is_nan());
        for ((what, want), (_, got)) in reference.iter().zip(images(nan)) {
            assert_eq!(&got, want, "{what} differs for NaN bits {bits:#018x}");
        }
    }
}

#[test]
fn nan_computed_in_sql_is_stored_canonically() {
    // inf - inf and its negation give NaNs with different sign bits.
    let image = |sql: &str| {
        let mut db = Database::new(7);
        db.set_clock(logical);
        db.execute("CREATE TABLE m (id INT PRIMARY KEY, x FLOAT)").unwrap();
        db.execute(sql).unwrap();
        db.snapshot().unwrap()
    };
    let a = image("INSERT INTO m VALUES (1, 1e308 * 10 - 1e308 * 10)");
    let b = image("INSERT INTO m VALUES (1, -(1e308 * 10 - 1e308 * 10))");
    assert_eq!(a, b);
    // The value still reads back as NaN.
    let mut db = Database::restore(Some(&a), &[], 0).unwrap();
    match &db.execute("SELECT x FROM m").unwrap().rows[0][0] {
        Value::Float(f) => assert!(f.is_nan()),
        other => panic!("{other:?}"),
    }
    let state = db.export_state().unwrap();
    assert_eq!(decode_batch(&state).unwrap().len(), 2);
}
