//! End-to-end SQL behaviour of the engine.

use flatsql_core::{Database, Error, QueryResult, Value};

fn db() -> Database {
    let mut db = Database::new(1);
    db.execute(
        "CREATE TABLE items (id INT PRIMARY KEY, name TEXT NOT NULL, category TEXT, price FLOAT, qty INT);
         INSERT INTO items VALUES
           (1, 'sword',   'weapon', 120.0, 1),
           (2, 'shield',  'armor',   80.5, 1),
           (3, 'potion',  'consumable', 5.0, 12),
           (4, 'bow',     'weapon',  95.0, 1),
           (5, 'helmet',  'armor',   40.0, 2),
           (6, 'elixir',  'consumable', 25.0, NULL);",
    )
    .unwrap();
    db
}

fn q(db: &mut Database, sql: &str) -> QueryResult {
    db.execute(sql).unwrap_or_else(|e| panic!("{sql}\n{e}"))
}

fn col(r: &QueryResult, i: usize) -> Vec<String> {
    r.rows.iter().map(|row| row[i].to_string()).collect()
}

#[test]
fn select_projection_filter_and_order() {
    let mut db = db();
    let r = q(&mut db, "SELECT name, price * 2 AS double FROM items WHERE category = 'weapon' ORDER BY price DESC");
    assert_eq!(r.columns, vec!["name", "double"]);
    assert_eq!(col(&r, 0), vec!["sword", "bow"]);
    assert_eq!(r.rows[0][1], Value::Float(240.0));
    assert_eq!(r.message, "SELECT 2");
}

#[test]
fn wildcard_limit_offset_and_pk_order() {
    let mut db = db();
    let r = q(&mut db, "SELECT * FROM items ORDER BY id DESC LIMIT 2 OFFSET 1");
    assert_eq!(r.columns.len(), 5);
    assert_eq!(col(&r, 0), vec!["5", "4"]);
    // PK order needs no sort and stops early: only 3 rows are visited.
    assert_eq!(r.rows_scanned, 3);
}

#[test]
fn null_semantics_in_where() {
    let mut db = db();
    assert_eq!(q(&mut db, "SELECT id FROM items WHERE qty > 0").rows.len(), 5);
    assert_eq!(col(&q(&mut db, "SELECT id FROM items WHERE qty IS NULL"), 0), vec!["6"]);
    assert_eq!(q(&mut db, "SELECT id FROM items WHERE qty <> 1").rows.len(), 2);
}

#[test]
fn aggregates_group_by_having() {
    let mut db = db();
    let r = q(
        &mut db,
        "SELECT category, COUNT(*) AS n, SUM(price) AS total, AVG(price), MIN(name), MAX(qty)
         FROM items GROUP BY category HAVING COUNT(*) >= 2 ORDER BY total DESC",
    );
    assert_eq!(col(&r, 0), vec!["weapon", "armor", "consumable"]);
    assert_eq!(r.rows[0][1], Value::Int(2));
    assert_eq!(r.rows[0][2], Value::Float(215.0));
    assert_eq!(r.rows[2][4], Value::from("elixir"));
    assert_eq!(r.rows[2][5], Value::Int(12)); // NULL qty ignored by MAX

    let r = q(&mut db, "SELECT COUNT(*), COUNT(qty), SUM(qty), COUNT(DISTINCT category) FROM items");
    assert_eq!(r.rows[0], vec![Value::Int(6), Value::Int(5), Value::Int(17), Value::Int(3)]);

    // Aggregates over no rows: one row, COUNT = 0, SUM = NULL.
    let r = q(&mut db, "SELECT COUNT(*), SUM(price) FROM items WHERE id > 100");
    assert_eq!(r.rows, vec![vec![Value::Int(0), Value::Null]]);
    // ... but GROUP BY over no rows returns nothing.
    assert!(q(&mut db, "SELECT category, COUNT(*) FROM items WHERE id > 100 GROUP BY category").rows.is_empty());
}

#[test]
fn group_by_alias_position_and_expression() {
    let mut db = db();
    let r = q(&mut db, "SELECT qty > 1 AS bulk, COUNT(*) FROM items WHERE qty IS NOT NULL GROUP BY bulk ORDER BY 1");
    assert_eq!(r.rows, vec![vec![Value::Bool(false), Value::Int(3)], vec![Value::Bool(true), Value::Int(2)]]);
}

#[test]
fn distinct_like_in_between() {
    let mut db = db();
    let r = q(&mut db, "SELECT DISTINCT category FROM items ORDER BY category");
    assert_eq!(col(&r, 0), vec!["armor", "consumable", "weapon"]);
    assert_eq!(
        col(&q(&mut db, "SELECT name FROM items WHERE name LIKE '%i%' ORDER BY id"), 0),
        vec!["shield", "potion", "elixir"]
    );
    assert_eq!(q(&mut db, "SELECT id FROM items WHERE name ILIKE 'S%'").rows.len(), 2);
    assert_eq!(col(&q(&mut db, "SELECT id FROM items WHERE id IN (2, 4, 99)"), 0), vec!["2", "4"]);
    assert_eq!(
        col(&q(&mut db, "SELECT id FROM items WHERE price BETWEEN 40 AND 95 ORDER BY id"), 0),
        vec!["2", "4", "5"]
    );
}

#[test]
fn select_without_from() {
    let mut db = Database::new(1);
    let r = q(&mut db, "SELECT 1 + 1 AS two, UPPER('flat') || '-sql'");
    assert_eq!(r.rows, vec![vec![Value::Int(2), Value::from("FLAT-sql")]]);
    assert!(q(&mut db, "SELECT 1 WHERE FALSE").rows.is_empty());
}

#[test]
fn insert_update_delete() {
    let mut db = db();
    let r = q(&mut db, "INSERT INTO items (id, name) VALUES (7, 'ring'), (8, 'amulet')");
    assert_eq!(r.rows_affected, 2);
    let r = q(&mut db, "SELECT price, qty FROM items WHERE id = 7");
    assert_eq!(r.rows[0], vec![Value::Null, Value::Null]);

    let r = q(&mut db, "UPDATE items SET price = price * 1.1, qty = qty + 1 WHERE category = 'weapon'");
    assert_eq!(r.rows_affected, 2);
    let r = q(&mut db, "SELECT qty FROM items WHERE category = 'weapon'");
    assert_eq!(col(&r, 0), vec!["2", "2"]);

    let r = q(&mut db, "DELETE FROM items WHERE qty IS NULL");
    assert_eq!(r.rows_affected, 3);
    assert_eq!(q(&mut db, "SELECT COUNT(*) FROM items").rows[0][0], Value::Int(5));
}

#[test]
fn update_can_change_primary_key() {
    let mut db = db();
    q(&mut db, "UPDATE items SET id = 100 WHERE id = 1");
    assert!(q(&mut db, "SELECT * FROM items WHERE id = 1").rows.is_empty());
    assert_eq!(q(&mut db, "SELECT name FROM items WHERE id = 100").rows[0][0], Value::from("sword"));
    let err = db.execute("UPDATE items SET id = 2 WHERE id = 3").unwrap_err();
    assert!(matches!(err, Error::Constraint(_)));
}

#[test]
fn constraints_and_type_checks() {
    let mut db = db();
    assert!(matches!(db.execute("INSERT INTO items VALUES (1, 'dup', NULL, NULL, NULL)"), Err(Error::Constraint(_))));
    assert!(matches!(db.execute("INSERT INTO items (id) VALUES (50)"), Err(Error::Constraint(_))));
    assert!(matches!(db.execute("INSERT INTO items (id, name, qty) VALUES (51, 'x', 'many')"), Err(Error::Type(_))));
    assert!(matches!(db.execute("INSERT INTO items (id, name, qty) VALUES (52, 'x', 1.5)"), Err(Error::Type(_))));
    // An integral float is accepted by an INT column.
    q(&mut db, "INSERT INTO items (id, name, qty) VALUES (53, 'x', 2.0)");
    assert!(matches!(db.execute("SELECT * FROM items WHERE name = 5"), Err(Error::Type(_))));
    assert!(matches!(db.execute("SELECT nope FROM items"), Err(Error::Schema(_))));
    assert!(matches!(db.execute("SELECT * FROM nope"), Err(Error::Schema(_))));
}

#[test]
fn insert_or_replace_and_ignore() {
    let mut db = db();
    let r = q(&mut db, "INSERT OR IGNORE INTO items (id, name) VALUES (1, 'ignored'), (9, 'new')");
    assert_eq!(r.rows_affected, 1);
    assert_eq!(q(&mut db, "SELECT name FROM items WHERE id = 1").rows[0][0], Value::from("sword"));
    q(&mut db, "REPLACE INTO items (id, name) VALUES (1, 'replaced')");
    let r = q(&mut db, "SELECT name, price FROM items WHERE id = 1");
    assert_eq!(r.rows[0], vec![Value::from("replaced"), Value::Null]);
}

#[test]
fn statements_are_atomic() {
    let mut db = db();
    // The second row violates the primary key, so the first must not stay.
    assert!(db.execute("INSERT INTO items (id, name) VALUES (20, 'a'), (1, 'b')").is_err());
    assert!(q(&mut db, "SELECT * FROM items WHERE id = 20").rows.is_empty());
    // Division by zero in the middle of an UPDATE leaves every row untouched.
    assert!(db.execute("UPDATE items SET qty = 10 / (qty - 2)").is_err());
    assert_eq!(q(&mut db, "SELECT qty FROM items WHERE id = 1").rows[0][0], Value::Int(1));
    assert!(!db.take_wal().is_empty()); // only the initial script was logged
    assert_eq!(db.wal_pending(), 0);
}

#[test]
fn explicit_transactions() {
    let mut db = db();
    q(&mut db, "BEGIN");
    q(&mut db, "INSERT INTO items (id, name) VALUES (30, 'temp')");
    q(&mut db, "DELETE FROM items WHERE id = 1");
    assert_eq!(q(&mut db, "SELECT COUNT(*) FROM items").rows[0][0], Value::Int(6));
    q(&mut db, "ROLLBACK");
    assert_eq!(q(&mut db, "SELECT COUNT(*) FROM items").rows[0][0], Value::Int(6));
    assert!(q(&mut db, "SELECT * FROM items WHERE id = 30").rows.is_empty());
    assert_eq!(q(&mut db, "SELECT name FROM items WHERE id = 1").rows[0][0], Value::from("sword"));

    db.take_wal();
    q(&mut db, "BEGIN; INSERT INTO items (id, name) VALUES (31, 'kept');");
    // A failing statement inside a transaction only rolls back itself.
    assert!(db.execute("INSERT INTO items (id, name) VALUES (31, 'dup')").is_err());
    assert!(db.in_transaction());
    assert_eq!(db.wal_pending(), 0, "nothing is logged before COMMIT");
    q(&mut db, "COMMIT");
    assert!(db.wal_pending() > 0);
    assert_eq!(q(&mut db, "SELECT name FROM items WHERE id = 31").rows[0][0], Value::from("kept"));

    assert!(matches!(db.execute("COMMIT"), Err(Error::Transaction(_))));
    q(&mut db, "BEGIN");
    assert!(matches!(db.execute("BEGIN"), Err(Error::Transaction(_))));
    assert!(db.snapshot().is_err());
    q(&mut db, "ROLLBACK");
}

#[test]
fn ddl_rollback_restores_schema() {
    let mut db = db();
    q(&mut db, "CREATE INDEX idx_cat ON items (category)");
    q(&mut db, "BEGIN; DROP INDEX idx_cat; DROP TABLE items; CREATE TABLE other (k TEXT PRIMARY KEY);");
    assert!(db.execute("SELECT * FROM items").is_err());
    q(&mut db, "ROLLBACK");
    assert_eq!(q(&mut db, "SELECT COUNT(*) FROM items").rows[0][0], Value::Int(6));
    assert!(db.execute("SELECT * FROM other").is_err());
    let plan = q(&mut db, "EXPLAIN SELECT * FROM items WHERE category = 'armor'");
    assert!(plan.rows[1][0].to_string().contains("INDEX LOOKUP idx_cat"));
}

#[test]
fn planner_uses_keys_and_indexes() {
    let mut db = Database::new(1);
    q(&mut db, "CREATE TABLE t (id INT PRIMARY KEY, grp INT, v FLOAT)");
    let mut sql = String::from("INSERT INTO t VALUES ");
    for i in 0..1000 {
        if i > 0 {
            sql.push(',');
        }
        sql.push_str(&format!("({i}, {}, {}.5)", i % 10, i));
    }
    q(&mut db, &sql);

    let plan = |db: &mut Database, sql: &str| {
        let line = q(db, &format!("EXPLAIN {sql}")).rows[1][0].to_string();
        line.trim().trim_start_matches("ACCESS:").trim().to_string()
    };

    assert!(plan(&mut db, "SELECT * FROM t WHERE id = 5").starts_with("PRIMARY KEY LOOKUP"));
    assert!(plan(&mut db, "SELECT * FROM t WHERE 5 = id AND v > 1").starts_with("PRIMARY KEY LOOKUP"));
    assert!(plan(&mut db, "SELECT * FROM t WHERE id IN (3, 1, 3)").starts_with("PRIMARY KEY MULTI-LOOKUP"));
    assert!(plan(&mut db, "SELECT * FROM t WHERE id > 10 AND id <= 20").starts_with("PRIMARY KEY RANGE"));
    assert!(plan(&mut db, "SELECT * FROM t WHERE id BETWEEN 1 AND 3").starts_with("PRIMARY KEY RANGE"));
    assert!(plan(&mut db, "SELECT * FROM t WHERE grp = 3").starts_with("FULL TABLE SCAN"));
    assert!(plan(&mut db, "SELECT * FROM t WHERE id > 5 AND id < 5").starts_with("EMPTY"));
    assert!(plan(&mut db, "SELECT * FROM t WHERE id = NULL").starts_with("EMPTY"));

    q(&mut db, "CREATE INDEX t_grp ON t (grp)");
    assert!(plan(&mut db, "SELECT * FROM t WHERE grp = 3").starts_with("INDEX LOOKUP t_grp"));
    assert!(plan(&mut db, "SELECT * FROM t WHERE grp IN (1, 2)").starts_with("INDEX MULTI-LOOKUP"));
    assert!(plan(&mut db, "SELECT * FROM t WHERE grp >= 8").starts_with("INDEX RANGE SCAN"));

    // Same answers whatever the access path.
    let r = q(&mut db, "SELECT COUNT(*) FROM t WHERE grp = 3");
    assert_eq!(r.rows[0][0], Value::Int(100));
    assert_eq!(r.rows_scanned, 100);
    let r = q(&mut db, "SELECT COUNT(*) FROM t WHERE id > 10 AND id <= 20");
    assert_eq!(r.rows[0][0], Value::Int(10));
    assert_eq!(r.rows_scanned, 10);
    assert_eq!(q(&mut db, "SELECT COUNT(*) FROM t WHERE grp >= 8 AND v < 100").rows[0][0], Value::Int(20));
    assert_eq!(q(&mut db, "SELECT COUNT(*) FROM t WHERE id IN (1, 2, 2000)").rows[0][0], Value::Int(2));
    let r = q(&mut db, "SELECT v FROM t WHERE id = 5");
    assert_eq!((r.rows[0][0].clone(), r.rows_scanned), (Value::Float(5.5), 1));
}

#[test]
fn top_k_ordering_matches_full_sort() {
    let mut db = Database::new(1);
    q(&mut db, "CREATE TABLE t (id INT PRIMARY KEY, score INT)");
    let mut sql = String::from("INSERT INTO t VALUES ");
    for i in 0..500 {
        if i > 0 {
            sql.push(',');
        }
        sql.push_str(&format!("({i}, {})", (i * 7919) % 101));
    }
    q(&mut db, &sql);
    let full = q(&mut db, "SELECT id FROM t ORDER BY score DESC, id");
    let top = q(&mut db, "SELECT id FROM t ORDER BY score DESC, id LIMIT 10 OFFSET 5");
    assert_eq!(top.rows, full.rows[5..15].to_vec());
}

#[test]
fn prepared_statements_with_params() {
    let mut db = db();
    let p = db.prepare("SELECT name FROM items WHERE category = ? AND price > ? ORDER BY id").unwrap();
    assert_eq!(p.param_count(), 2);
    let r = db.execute_prepared(&p, &[Value::from("weapon"), Value::Float(100.0)]).unwrap();
    assert_eq!(col(&r, 0), vec!["sword"]);
    assert!(db.execute_prepared(&p, &[Value::from("weapon")]).is_err());

    let ins = db.prepare("INSERT INTO items (id, name, qty) VALUES (?, ?, ?)").unwrap();
    for i in 100..110 {
        db.execute_prepared(&ins, &[Value::Int(i), Value::from("bulk"), Value::Int(i)]).unwrap();
    }
    assert_eq!(q(&mut db, "SELECT COUNT(*) FROM items WHERE name = 'bulk'").rows[0][0], Value::Int(10));
}

#[test]
fn show_tables_and_describe() {
    let mut db = db();
    q(&mut db, "CREATE INDEX idx_cat ON items (category)");
    let r = q(&mut db, "SHOW TABLES");
    assert_eq!(r.rows[0][0], Value::from("items"));
    assert_eq!(r.rows[0][1], Value::Int(6));
    let r = q(&mut db, "DESCRIBE items");
    assert_eq!(r.rows[0][3], Value::from("PRIMARY KEY"));
    assert_eq!(r.rows[2][3], Value::from("INDEX idx_cat"));
    assert_eq!(r.rows[1][2], Value::Bool(false)); // name is NOT NULL
}

#[test]
fn flat_sql_model_rules() {
    let mut db = Database::new(1);
    assert!(matches!(db.execute("CREATE TABLE t (a INT, b TEXT)"), Err(Error::Schema(_))));
    assert!(matches!(db.execute("SELECT * FROM a JOIN b ON a.x = b.x"), Err(Error::Unsupported(_))));
    q(&mut db, "CREATE TABLE \"Mixed Case\" (Key TEXT, PRIMARY KEY (Key))");
    q(&mut db, "INSERT INTO \"mixed case\" (KEY) VALUES ('a')");
    assert_eq!(q(&mut db, "SELECT key FROM \"MIXED CASE\"").rows.len(), 1);
}

#[test]
fn json_output() {
    let mut db = db();
    let r = q(&mut db, "SELECT id, name FROM items WHERE id = 1");
    assert_eq!(
        r.to_json(),
        r#"{"columns":["id","name"],"rows":[[1,"sword"]],"rowsAffected":0,"rowsScanned":1,"message":"SELECT 1"}"#
    );
}
