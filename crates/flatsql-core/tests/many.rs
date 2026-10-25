//! One prepared write executed for many sets of parameters: the same rows and
//! WAL as executing it for each set, applied together or not at all, with one
//! reading of the clock for all rows.

use std::collections::HashSet;

use flatsql_core::sync::{decode_batch, Mutation};
use flatsql_core::{Database, Error, QueryResult, Value};

const SCHEMA: &str = "CREATE TABLE t (id INT PRIMARY KEY, s TEXT, f FLOAT, b BOOL)";

fn run(db: &mut Database, sql: &str) -> QueryResult {
    db.execute(sql).unwrap_or_else(|e| panic!("{sql}\n{e}"))
}

fn database() -> Database {
    let mut db = Database::new(1);
    run(&mut db, SCHEMA);
    run(&mut db, "CREATE INDEX t_s ON t (s)");
    db
}

fn rows(db: &mut Database) -> Vec<Vec<Value>> {
    run(db, "SELECT * FROM t ORDER BY id").rows
}

/// Parameters of `n` rows from `first`, with every type and NULLs.
fn params(first: i64, n: i64) -> Vec<Vec<Value>> {
    (first..first + n)
        .map(|i| {
            vec![
                Value::Int(i),
                if i % 7 == 0 { Value::Null } else { Value::Text(format!("ś{}", i % 13)) },
                Value::Float(i as f64 / 4.0),
                Value::Bool(i % 2 == 0),
            ]
        })
        .collect()
}

/// `execute_many` with the sets of `sets` in order.
fn many(db: &mut Database, sql: &str, sets: &[Vec<Value>]) -> Result<u64, Error> {
    let p = db.prepare(sql).unwrap();
    let mut it = sets.iter();
    db.execute_many(&p, |out| {
        Ok(match it.next() {
            Some(set) => {
                out.extend(set.iter().cloned());
                true
            }
            None => false,
        })
    })
}

#[test]
fn executing_for_many_sets_gives_the_rows_and_wal_of_executing_for_each() {
    let mut one = database();
    let mut each = database();
    let writes: [(&str, Vec<Vec<Value>>); 3] = [
        ("INSERT INTO t VALUES (?, ?, ?, ?)", params(0, 500)),
        (
            "UPDATE t SET s = ?, f = f + ? WHERE id = ?",
            (0..500).step_by(3).map(|i| vec![Value::Text(format!("u{i}")), Value::Float(0.5), Value::Int(i)]).collect(),
        ),
        (
            "DELETE FROM t WHERE id = ? OR s = ?",
            (0..500).step_by(5).map(|i| vec![Value::Int(i), Value::from("ś3")]).collect(),
        ),
    ];
    for (sql, sets) in &writes {
        let affected = many(&mut one, sql, sets).unwrap();
        let p = each.prepare(sql).unwrap();
        let mut want = 0;
        for set in sets {
            want += each.execute_prepared(&p, set).unwrap().rows_affected;
        }
        assert_eq!(affected, want, "{sql}");
        assert_eq!(rows(&mut one), rows(&mut each), "{sql}");
    }
    // A restart from the WAL gives the same rows.
    let mut restored = Database::restore(None, &one.take_wal(), 1).unwrap();
    assert_eq!(rows(&mut restored), rows(&mut each));
    assert_eq!(
        run(&mut restored, "SELECT COUNT(*) FROM t WHERE s = 'ś5'").rows,
        run(&mut each, "SELECT COUNT(*) FROM t WHERE s = 'ś5'").rows
    );
}

#[test]
fn the_sets_are_one_statement() {
    let mut db = database();
    let insert = "INSERT INTO t VALUES (?, ?, ?, ?)";
    many(&mut db, insert, &params(0, 10)).unwrap();
    let before = rows(&mut db);
    // A duplicate key in the middle: none of the rows are inserted.
    let mut sets = params(100, 10);
    sets[5][0] = Value::Int(3);
    let err = many(&mut db, insert, &sets).unwrap_err();
    assert_eq!(
        err.to_string(),
        db.execute_prepared(&db.prepare(insert).unwrap(), &params(3, 1)[0]).unwrap_err().to_string()
    );
    assert_eq!(rows(&mut db), before);
    // A set with the wrong number of values.
    let mut sets = params(200, 3);
    sets[2].pop();
    assert!(matches!(many(&mut db, insert, &sets), Err(Error::Eval(_))));
    assert_eq!(rows(&mut db), before);
    // Inside a transaction only this statement is undone.
    run(&mut db, "BEGIN");
    many(&mut db, insert, &params(300, 2)).unwrap();
    let mut sets = params(400, 2);
    sets[1][0] = Value::Int(300);
    many(&mut db, insert, &sets).unwrap_err();
    run(&mut db, "COMMIT");
    assert_eq!(rows(&mut db).len(), before.len() + 2);
    // An error of the source of parameters also undoes the statement.
    let p = db.prepare(insert).unwrap();
    let mut n = 0;
    let err = db.execute_many(&p, |out| {
        n += 1;
        if n == 3 {
            return Err(Error::Eval("source failed".into()));
        }
        out.extend(params(500 + n, 1).remove(0));
        Ok(true)
    });
    assert_eq!(err.unwrap_err().to_string(), Error::Eval("source failed".into()).to_string());
    assert_eq!(rows(&mut db).len(), before.len() + 2);
    // No sets: nothing to do.
    assert_eq!(many(&mut db, insert, &[]).unwrap(), 0);
    // Only writes.
    for sql in ["SELECT * FROM t WHERE id = ?", "CREATE TABLE u (id INT PRIMARY KEY)"] {
        assert!(matches!(many(&mut db, sql, &[vec![Value::Int(1)]]), Err(Error::Unsupported(_))), "{sql}");
    }
}

#[test]
fn rows_of_all_sets_get_stamps_from_one_clock_reading() {
    let mut db = database();
    many(&mut db, "INSERT INTO t VALUES (?, ?, ?, ?)", &params(0, 1000)).unwrap();
    let stamps: Vec<_> = decode_batch(&db.export_state().unwrap())
        .unwrap()
        .into_iter()
        .filter(|c| matches!(c.mutation, Mutation::Put { .. }))
        .map(|c| c.stamp)
        .collect();
    assert_eq!(stamps.len(), 1000);
    assert_eq!(stamps.iter().collect::<HashSet<_>>().len(), 1000, "stamps are unique");
    assert_eq!(stamps.iter().map(|s| s.time >> 16).collect::<HashSet<_>>().len(), 1, "one physical time");
}
