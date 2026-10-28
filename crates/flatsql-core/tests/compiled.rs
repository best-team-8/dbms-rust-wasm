//! Compiled expressions against the interpreter: random expressions over the
//! columns of a table, evaluated on random rows, must give the same result
//! (the same value of the same type, or an error in both).

use flatsql_core::compile;
use flatsql_core::expr::{eval, eval_pred, BExpr, Binder, Cx};
use flatsql_core::sql::{parse_one, SelectItem, Statement};
use flatsql_core::table::{Column, TableSchema};
use flatsql_core::{DataType, Value};

/// xorshift64* – deterministic randomness without dependencies.
struct Rng(u64);

impl Rng {
    fn next(&mut self) -> u64 {
        self.0 ^= self.0 >> 12;
        self.0 ^= self.0 << 25;
        self.0 ^= self.0 >> 27;
        self.0.wrapping_mul(0x2545_F491_4F6C_DD1D)
    }

    fn below(&mut self, n: u64) -> u64 {
        self.next() % n
    }

    fn pick<'a>(&mut self, items: &[&'a str]) -> &'a str {
        items[self.below(items.len() as u64) as usize]
    }
}

fn schema() -> TableSchema {
    let col = |name: &str, data_type| Column { name: name.into(), data_type, not_null: false };
    TableSchema {
        name: "t".into(),
        columns: vec![
            col("id", DataType::Int),
            col("a", DataType::Int),
            col("b", DataType::Text),
            col("c", DataType::Float),
            col("d", DataType::Bool),
        ],
        pk: 0,
    }
}

/// A row as the engine stores it: every value has the type of its column.
fn row(rng: &mut Rng) -> Vec<Value> {
    let null = |rng: &mut Rng| rng.below(6) == 0;
    let int = |rng: &mut Rng| match rng.below(8) {
        0 => i64::MAX,
        1 => i64::MIN,
        2 => 0,
        _ => rng.below(21) as i64 - 10,
    };
    let mut r = vec![Value::Int(int(rng))];
    r.push(if null(rng) { Value::Null } else { Value::Int(int(rng)) });
    r.push(if null(rng) { Value::Null } else { Value::Text(format!("k{}", rng.below(4)).into()) });
    r.push(if null(rng) {
        Value::Null
    } else {
        Value::Float(match rng.below(8) {
            0 => f64::NAN,
            1 => -0.0,
            2 => 1e308,
            3 => f64::INFINITY,
            _ => (rng.below(41) as f64 - 20.0) * 0.5,
        })
    });
    r.push(if null(rng) { Value::Null } else { Value::Bool(rng.below(2) == 0) });
    r
}

fn literal(rng: &mut Rng) -> String {
    rng.pick(&[
        "NULL",
        "0",
        "1",
        "2",
        "-3",
        "10",
        "0.0",
        "2.5",
        "-1.5",
        "9223372036854775807",
        "TRUE",
        "FALSE",
        "'k1'",
        "'k3'",
    ])
    .to_string()
}

/// A numeric expression (sometimes of another type, for the error paths).
fn numeric(rng: &mut Rng, depth: u32) -> String {
    if depth == 0 || rng.below(3) == 0 {
        return match rng.below(5) {
            0 | 1 => rng.pick(&["id", "a", "c", "c", "d", "b"]).to_string(),
            _ => literal(rng),
        };
    }
    match rng.below(6) {
        0 => format!("-({})", numeric(rng, depth - 1)),
        _ => format!(
            "({} {} {})",
            numeric(rng, depth - 1),
            rng.pick(&["+", "-", "*", "/", "%"]),
            numeric(rng, depth - 1)
        ),
    }
}

fn predicate(rng: &mut Rng, depth: u32) -> String {
    if depth == 0 || rng.below(3) == 0 {
        let op = rng.pick(&["=", "<>", "<", "<=", ">", ">="]);
        return match rng.below(6) {
            0 => format!("{} IS {}NULL", numeric(rng, 1), if rng.below(2) == 0 { "NOT " } else { "" }),
            1 => format!(
                "{} {}BETWEEN {} AND {}",
                numeric(rng, 1),
                if rng.below(4) == 0 { "NOT " } else { "" },
                literal(rng),
                literal(rng)
            ),
            2 => format!("b LIKE '{}'", rng.pick(&["k%", "_1", "%3"])),
            _ => format!("{} {op} {}", numeric(rng, 2), numeric(rng, 2)),
        };
    }
    match rng.below(4) {
        0 => format!("NOT ({})", predicate(rng, depth - 1)),
        1 => format!("({}) OR ({})", predicate(rng, depth - 1), predicate(rng, depth - 1)),
        _ => format!("({}) AND ({})", predicate(rng, depth - 1), predicate(rng, depth - 1)),
    }
}

/// Binds `SELECT <expr> FROM t`; `None` when binding fails (e.g. a constant
/// part divides by zero).
fn bind(schema: &TableSchema, expr: &str) -> Option<BExpr> {
    let Statement::Select(s) = parse_one(&format!("SELECT {expr} FROM t")).ok()? else { return None };
    let SelectItem::Expr { expr, .. } = &s.projection[0] else { return None };
    Binder::new(Some(schema), &[]).bind(expr, "SELECT").ok()
}

/// The result as text with its type (`Value` equality treats 3 and 3.0 as
/// equal, so it would hide a wrong type), or the error message.
fn shown<T: std::fmt::Debug>(r: flatsql_core::Result<T>) -> String {
    match r {
        Ok(v) => format!("{v:?}"),
        Err(e) => format!("error: {e}"),
    }
}

#[test]
fn compiled_expressions_give_the_interpreters_results() {
    let schema = schema();
    let mut rng = Rng(0x5DEE_CE66_D1CE_4E5B);
    let rows: Vec<Vec<Value>> = (0..60).map(|_| row(&mut rng)).collect();
    let (mut predicates, mut values) = (0, 0);
    for _ in 0..4000 {
        if let Some(e) = bind(&schema, &predicate(&mut rng, 3)) {
            let compiled = compile::predicate(&e, &schema);
            for r in &rows {
                let want = shown(eval_pred(&e, r, Cx::default()));
                assert_eq!(shown(compiled.test(r, &[])), want, "{e:?} on {r:?}");
            }
            predicates += 1;
        }
        if let Some(e) = bind(&schema, &numeric(&mut rng, 3)) {
            let compiled = compile::value(&e, &schema);
            for r in &rows {
                let want = shown(eval(&e, r, Cx::default()).map(|v| v.into_owned()));
                assert_eq!(shown(compiled.eval(r, &[])), want, "{e:?} on {r:?}");
            }
            values += 1;
        }
    }
    assert!(predicates > 2000 && values > 2000, "{predicates} predicates, {values} values");
}
