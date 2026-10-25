//! Expressions compiled into closures when a statement is compiled.
//!
//! The interpreter (`expr::eval`, `expr::eval_pred`) walks the expression
//! tree for every row: a call per node, a `Result<Cow<Value>>` per node and a
//! comparison through the general order of values. For the shapes that
//! dominate filters and updates (a column compared with a literal, arithmetic
//! on INT and FLOAT columns, AND, OR and NOT of such terms) the compiler
//! specialises the evaluation to the type of the column: the literal is
//! converted once (`50` compared with a FLOAT column becomes `50.0`) and a row
//! is checked with one match on the column value. A column compared with a
//! literal, the most frequent conjunct, is a variant of `Pred` evaluated
//! without a call through a pointer; composite nodes are closures. Every other
//! node is evaluated by the interpreter inside a closure, so every expression
//! compiles. The semantics are the interpreter's: three-valued logic, the
//! total order of values (NaN after every number and equal to itself),
//! checked integer arithmetic, and the same errors for division by zero,
//! overflow and values that cannot be compared.

use std::cmp::Ordering;
use std::fmt;
use std::rc::Rc;

use crate::error::{Error, Result};
use crate::expr::{eval, eval_pred, BExpr, Cx};
use crate::sql::BinaryOp;
use crate::table::TableSchema;
use crate::value::{cmp_f64, DataType, Value};

type PredFn = dyn Fn(&[Value], &[Value]) -> Result<Option<bool>>;
type ValueFn = dyn Fn(&[Value], &[Value]) -> Result<Value>;
type IntFn = Box<dyn Fn(&[Value], &[Value]) -> Result<Option<i64>>>;
type FloatFn = Box<dyn Fn(&[Value], &[Value]) -> Result<Option<f64>>>;

/// A compiled predicate: `(row, parameters) -> TRUE / FALSE / NULL`.
#[derive(Clone)]
pub struct Pred(Node);

/// Column `c` compared with a literal converted to the column's type, or a
/// closure for anything else.
#[derive(Clone)]
enum Node {
    /// The literal is NULL: the comparison is always NULL.
    Null,
    /// FLOAT column with a number.
    Float {
        c: usize,
        op: BinaryOp,
        y: f64,
    },
    /// INT column with an INT literal.
    Int {
        c: usize,
        op: BinaryOp,
        y: i64,
    },
    /// INT column with a FLOAT literal (compared as floats).
    IntFloat {
        c: usize,
        op: BinaryOp,
        y: f64,
    },
    Text {
        c: usize,
        op: BinaryOp,
        y: Rc<str>,
    },
    Bool {
        c: usize,
        op: BinaryOp,
        y: bool,
    },
    /// BOOL column with a number: the boolean counts as 0 or 1.
    BoolNum {
        c: usize,
        op: BinaryOp,
        y: f64,
    },
    Closure(Rc<PredFn>),
}

#[cold]
#[inline(never)]
fn wrong_type(column: &str, v: &Value) -> Error {
    Error::Type(format!("{column} column holds {}", v.type_name()))
}

impl Pred {
    #[inline]
    pub fn test(&self, row: &[Value], params: &[Value]) -> Result<Option<bool>> {
        match &self.0 {
            Node::Null => Ok(None),
            Node::Float { c, op, y } => match &row[*c] {
                Value::Float(x) => Ok(Some(holds(*op, cmp_f64(*x, *y)))),
                Value::Null => Ok(None),
                v => Err(wrong_type("FLOAT", v)),
            },
            Node::Int { c, op, y } => match &row[*c] {
                Value::Int(x) => Ok(Some(holds(*op, x.cmp(y)))),
                Value::Null => Ok(None),
                v => Err(wrong_type("INT", v)),
            },
            Node::IntFloat { c, op, y } => match &row[*c] {
                Value::Int(x) => Ok(Some(holds(*op, cmp_f64(*x as f64, *y)))),
                Value::Null => Ok(None),
                v => Err(wrong_type("INT", v)),
            },
            Node::Text { c, op, y } => match &row[*c] {
                Value::Text(x) => Ok(Some(holds(*op, x.as_str().cmp(y)))),
                Value::Null => Ok(None),
                v => Err(wrong_type("TEXT", v)),
            },
            Node::Bool { c, op, y } => match &row[*c] {
                Value::Bool(x) => Ok(Some(holds(*op, x.cmp(y)))),
                Value::Null => Ok(None),
                v => Err(wrong_type("BOOL", v)),
            },
            Node::BoolNum { c, op, y } => match &row[*c] {
                Value::Bool(x) => Ok(Some(holds(*op, cmp_f64(*x as i64 as f64, *y)))),
                Value::Null => Ok(None),
                v => Err(wrong_type("BOOL", v)),
            },
            Node::Closure(f) => f(row, params),
        }
    }

    fn closure(f: impl Fn(&[Value], &[Value]) -> Result<Option<bool>> + 'static) -> Pred {
        Pred(Node::Closure(Rc::new(f)))
    }
}

impl fmt::Debug for Pred {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("Pred")
    }
}

/// A compiled value expression: `(row, parameters) -> value`.
#[derive(Clone)]
pub struct ValueExpr(Rc<ValueFn>);

impl ValueExpr {
    #[inline]
    pub fn eval(&self, row: &[Value], params: &[Value]) -> Result<Value> {
        (self.0)(row, params)
    }
}

impl fmt::Debug for ValueExpr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("ValueExpr")
    }
}

fn division_by_zero() -> Error {
    Error::Eval("division by zero".into())
}

#[cold]
fn overflow() -> Error {
    Error::Eval("integer overflow".into())
}

/// Whether `ord` satisfies comparison `op`.
#[inline(always)]
fn holds(op: BinaryOp, ord: Ordering) -> bool {
    match op {
        BinaryOp::Eq => ord == Ordering::Equal,
        BinaryOp::NotEq => ord != Ordering::Equal,
        BinaryOp::Lt => ord == Ordering::Less,
        BinaryOp::LtEq => ord != Ordering::Greater,
        BinaryOp::Gt => ord == Ordering::Greater,
        BinaryOp::GtEq => ord != Ordering::Less,
        _ => unreachable!("not a comparison"),
    }
}

fn and3(a: Option<bool>, b: Option<bool>) -> Option<bool> {
    match (a, b) {
        (Some(false), _) | (_, Some(false)) => Some(false),
        (Some(true), Some(true)) => Some(true),
        _ => None,
    }
}

/// A numeric expression of known type (INT or FLOAT columns and literals,
/// arithmetic of them); `None` from the closure is NULL.
enum Num {
    Int(IntFn),
    Float(FloatFn),
}

impl Num {
    fn into_float(self) -> FloatFn {
        match self {
            Num::Float(f) => f,
            Num::Int(f) => Box::new(move |row, params| Ok(f(row, params)?.map(|i| i as f64))),
        }
    }
}

fn column_type(schema: &TableSchema, col: usize) -> DataType {
    schema.columns[col].data_type
}

/// Compiles a numeric expression, or `None` when its type is not a number
/// known at compile time.
fn num(e: &BExpr, schema: &TableSchema) -> Option<Num> {
    match e {
        BExpr::Col(c) => match column_type(schema, *c) {
            DataType::Int => {
                let c = *c;
                Some(Num::Int(Box::new(move |row, _| match &row[c] {
                    Value::Int(x) => Ok(Some(*x)),
                    Value::Null => Ok(None),
                    other => Err(Error::Type(format!("INT column holds {}", other.type_name()))),
                })))
            }
            DataType::Float => {
                let c = *c;
                Some(Num::Float(Box::new(move |row, _| match &row[c] {
                    Value::Float(x) => Ok(Some(*x)),
                    Value::Null => Ok(None),
                    other => Err(Error::Type(format!("FLOAT column holds {}", other.type_name()))),
                })))
            }
            _ => None,
        },
        BExpr::Lit(Value::Int(i)) => {
            let i = *i;
            Some(Num::Int(Box::new(move |_, _| Ok(Some(i)))))
        }
        BExpr::Lit(Value::Float(f)) => {
            let f = *f;
            Some(Num::Float(Box::new(move |_, _| Ok(Some(f)))))
        }
        BExpr::Neg(x) => match num(x, schema)? {
            Num::Int(f) => Some(Num::Int(Box::new(move |row, params| match f(row, params)? {
                Some(i) => i.checked_neg().map(Some).ok_or_else(overflow),
                None => Ok(None),
            }))),
            Num::Float(f) => Some(Num::Float(Box::new(move |row, params| Ok(f(row, params)?.map(|x| -x))))),
        },
        BExpr::Bin(op @ (BinaryOp::Add | BinaryOp::Sub | BinaryOp::Mul | BinaryOp::Div | BinaryOp::Mod), l, r) => {
            let op = *op;
            match (num(l, schema)?, num(r, schema)?) {
                (Num::Int(a), Num::Int(b)) => {
                    // A literal right operand (`id % 10`) is read once.
                    if let BExpr::Lit(Value::Int(y)) = **r {
                        return Some(Num::Int(Box::new(move |row, params| match a(row, params)? {
                            Some(x) => int_arith(op, x, y).map(Some),
                            None => Ok(None),
                        })));
                    }
                    Some(Num::Int(Box::new(move |row, params| match (a(row, params)?, b(row, params)?) {
                        (Some(x), Some(y)) => int_arith(op, x, y).map(Some),
                        _ => Ok(None),
                    })))
                }
                (a, b) => {
                    let (a, b) = (a.into_float(), b.into_float());
                    Some(Num::Float(Box::new(move |row, params| match (a(row, params)?, b(row, params)?) {
                        (Some(x), Some(y)) => float_arith(op, x, y).map(Some),
                        _ => Ok(None),
                    })))
                }
            }
        }
        _ => None,
    }
}

#[inline]
fn int_arith(op: BinaryOp, x: i64, y: i64) -> Result<i64> {
    let r = match op {
        BinaryOp::Add => x.checked_add(y),
        BinaryOp::Sub => x.checked_sub(y),
        BinaryOp::Mul => x.checked_mul(y),
        BinaryOp::Div | BinaryOp::Mod if y == 0 => return Err(division_by_zero()),
        BinaryOp::Div => x.checked_div(y),
        BinaryOp::Mod => x.checked_rem(y),
        _ => unreachable!("not arithmetic"),
    };
    r.ok_or_else(overflow)
}

#[inline]
fn float_arith(op: BinaryOp, x: f64, y: f64) -> Result<f64> {
    if matches!(op, BinaryOp::Div | BinaryOp::Mod) && y == 0.0 {
        return Err(division_by_zero());
    }
    Ok(match op {
        BinaryOp::Add => x + y,
        BinaryOp::Sub => x - y,
        BinaryOp::Mul => x * y,
        BinaryOp::Div => x / y,
        BinaryOp::Mod => x % y,
        _ => unreachable!("not arithmetic"),
    })
}

/// The interpreter inside a closure, for nodes without a specialised form.
fn interpreted(e: &BExpr) -> Pred {
    let e = e.clone();
    Pred::closure(move |row, params| eval_pred(&e, row, Cx::params(params)))
}

/// `column op literal` specialised to the type of the column, or `None` when
/// the pair has no specialised form (the interpreter then reports an
/// incomparable pair when it meets a row).
fn column_vs_literal(schema: &TableSchema, c: usize, op: BinaryOp, lit: &Value) -> Option<Pred> {
    let node = match (column_type(schema, c), lit) {
        (_, Value::Null) => Node::Null,
        (DataType::Float, Value::Int(_) | Value::Float(_)) => Node::Float { c, op, y: lit.as_f64().unwrap() },
        (DataType::Int, Value::Int(y)) => Node::Int { c, op, y: *y },
        (DataType::Int, Value::Float(y)) => Node::IntFloat { c, op, y: *y },
        (DataType::Text, Value::Text(y)) => Node::Text { c, op, y: Rc::from(y.as_str()) },
        (DataType::Bool, Value::Bool(y)) => Node::Bool { c, op, y: *y },
        // A boolean compares with a number as 0 or 1 (an INT literal is exact
        // as a float in the range a boolean can be equal to).
        (DataType::Bool, Value::Int(y)) => Node::BoolNum { c, op, y: *y as f64 },
        (DataType::Bool, Value::Float(y)) => Node::BoolNum { c, op, y: *y },
        _ => return None,
    };
    Some(Pred(node))
}

/// Compiles a comparison `l op r`.
fn comparison(op: BinaryOp, l: &BExpr, r: &BExpr, schema: &TableSchema) -> Option<Pred> {
    match (l, r) {
        (BExpr::Col(c), BExpr::Lit(v)) => return column_vs_literal(schema, *c, op, v),
        (BExpr::Lit(v), BExpr::Col(c)) => return column_vs_literal(schema, *c, op.flipped(), v),
        _ => {}
    }
    match (num(l, schema)?, num(r, schema)?) {
        (Num::Int(a), Num::Int(b)) => {
            if let BExpr::Lit(Value::Int(y)) = *r {
                return Some(Pred::closure(move |row, params| Ok(a(row, params)?.map(|x| holds(op, x.cmp(&y))))));
            }
            Some(Pred::closure(move |row, params| match (a(row, params)?, b(row, params)?) {
                (Some(x), Some(y)) => Ok(Some(holds(op, x.cmp(&y)))),
                _ => Ok(None),
            }))
        }
        (a, b) => {
            let (a, b) = (a.into_float(), b.into_float());
            Some(Pred::closure(move |row, params| match (a(row, params)?, b(row, params)?) {
                (Some(x), Some(y)) => Ok(Some(holds(op, cmp_f64(x, y)))),
                _ => Ok(None),
            }))
        }
    }
}

/// Compiles a predicate (a WHERE conjunct) bound against `schema`.
pub fn predicate(e: &BExpr, schema: &TableSchema) -> Pred {
    let compiled = match e {
        BExpr::Bin(BinaryOp::And, l, r) => {
            let (a, b) = (predicate(l, schema), predicate(r, schema));
            Some(Pred::closure(move |row, params| {
                let x = a.test(row, params)?;
                if x == Some(false) {
                    return Ok(Some(false));
                }
                Ok(and3(x, b.test(row, params)?))
            }))
        }
        BExpr::Bin(BinaryOp::Or, l, r) => {
            let (a, b) = (predicate(l, schema), predicate(r, schema));
            Some(Pred::closure(move |row, params| {
                let x = a.test(row, params)?;
                if x == Some(true) {
                    return Ok(Some(true));
                }
                Ok(match (x, b.test(row, params)?) {
                    (_, Some(true)) => Some(true),
                    (Some(false), Some(false)) => Some(false),
                    _ => None,
                })
            }))
        }
        BExpr::Not(x) => {
            let a = predicate(x, schema);
            Some(Pred::closure(move |row, params| Ok(a.test(row, params)?.map(|b| !b))))
        }
        BExpr::IsNull(x, negated) => match **x {
            BExpr::Col(c) => {
                let negated = *negated;
                Some(Pred::closure(move |row, _| Ok(Some(row[c].is_null() != negated))))
            }
            _ => None,
        },
        BExpr::Bin(op, l, r) if op.is_comparison() => comparison(*op, l, r, schema),
        BExpr::Between(x, lo, hi, negated) => {
            match (comparison(BinaryOp::GtEq, x, lo, schema), comparison(BinaryOp::LtEq, x, hi, schema)) {
                (Some(ge), Some(le)) => {
                    let negated = *negated;
                    Some(Pred::closure(move |row, params| {
                        let a = ge.test(row, params)?;
                        let b = le.test(row, params)?;
                        Ok(and3(a, b).map(|v| v != negated))
                    }))
                }
                _ => None,
            }
        }
        _ => None,
    };
    compiled.unwrap_or_else(|| interpreted(e))
}

/// Compiles a value expression (an assignment of UPDATE).
pub fn value(e: &BExpr, schema: &TableSchema) -> ValueExpr {
    match num(e, schema) {
        // A bare column or literal is copied, as the interpreter does.
        Some(_) if matches!(e, BExpr::Col(_) | BExpr::Lit(_)) => interpreted_value(e),
        Some(Num::Int(f)) => ValueExpr(Rc::new(move |row, params| Ok(f(row, params)?.map_or(Value::Null, Value::Int)))),
        Some(Num::Float(f)) => {
            ValueExpr(Rc::new(move |row, params| Ok(f(row, params)?.map_or(Value::Null, Value::Float))))
        }
        None => interpreted_value(e),
    }
}

fn interpreted_value(e: &BExpr) -> ValueExpr {
    let e = e.clone();
    ValueExpr(Rc::new(move |row, params| Ok(eval(&e, row, Cx::params(params))?.into_owned())))
}
