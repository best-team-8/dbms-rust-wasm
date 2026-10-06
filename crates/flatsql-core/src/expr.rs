//! Expression binding (name resolution, constant folding, aggregate
//! extraction) and evaluation with SQL three-valued logic.

use std::borrow::Cow;
use std::cmp::Ordering;
use std::collections::BTreeSet;

use crate::error::{Error, Result};
use crate::sql::{BinaryOp, Expr, UnaryOp, AGGREGATES};
use crate::table::TableSchema;
use crate::value::{format_float, Value};

/// Expression with column names resolved to row offsets.
#[derive(Debug, Clone, PartialEq)]
pub enum BExpr {
    Lit(Value),
    Col(usize),
    /// Result of the n-th aggregate of the current group.
    Agg(usize),
    Neg(Box<BExpr>),
    Not(Box<BExpr>),
    Bin(BinaryOp, Box<BExpr>, Box<BExpr>),
    IsNull(Box<BExpr>, bool),
    InList(Box<BExpr>, Vec<BExpr>, bool),
    Between(Box<BExpr>, Box<BExpr>, Box<BExpr>, bool),
    Like {
        expr: Box<BExpr>,
        pattern: Box<BExpr>,
        negated: bool,
        ci: bool,
    },
    Func(ScalarFn, Vec<BExpr>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScalarFn {
    Lower,
    Upper,
    Length,
    Abs,
    Round,
    Coalesce,
    Substr,
    Typeof,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AggFunc {
    CountStar,
    Count,
    Sum,
    Avg,
    Min,
    Max,
}

#[derive(Debug, Clone, PartialEq)]
pub struct AggSpec {
    pub func: AggFunc,
    pub arg: Option<BExpr>,
    pub distinct: bool,
    /// Original SQL text, used by EXPLAIN.
    pub label: String,
}

impl BExpr {
    fn is_lit(&self) -> bool {
        matches!(self, BExpr::Lit(_))
    }

    /// Splits a predicate into its top-level `AND` terms.
    pub fn conjuncts(&self) -> Vec<&BExpr> {
        let mut out = Vec::new();
        fn walk<'a>(e: &'a BExpr, out: &mut Vec<&'a BExpr>) {
            match e {
                BExpr::Bin(BinaryOp::And, l, r) => {
                    walk(l, out);
                    walk(r, out);
                }
                other => out.push(other),
            }
        }
        walk(self, &mut out);
        out
    }
}

/// Resolves names of an AST expression against a table schema.
pub struct Binder<'a> {
    schema: Option<&'a TableSchema>,
    params: &'a [Value],
}

impl<'a> Binder<'a> {
    pub fn new(schema: Option<&'a TableSchema>, params: &'a [Value]) -> Self {
        Binder { schema, params }
    }

    /// Binds an expression in which aggregate functions are not allowed.
    pub fn bind(&self, e: &Expr, clause: &str) -> Result<BExpr> {
        self.bind_inner(e, None, false, clause)
    }

    /// Binds an expression that may contain aggregates; each aggregate call is
    /// appended to `aggs` and replaced by `BExpr::Agg(i)`.
    pub fn bind_agg(&self, e: &Expr, aggs: &mut Vec<AggSpec>) -> Result<BExpr> {
        self.bind_inner(e, Some(aggs), false, "")
    }

    fn bind_inner(&self, e: &Expr, mut aggs: Option<&mut Vec<AggSpec>>, in_agg: bool, clause: &str) -> Result<BExpr> {
        let b = match e {
            Expr::Literal(v) => return Ok(BExpr::Lit(v.clone())),
            Expr::Param(i) => {
                let v = self.params.get(*i).ok_or_else(|| {
                    Error::Eval(format!("missing value for parameter ?{} ({} supplied)", i + 1, self.params.len()))
                })?;
                return Ok(BExpr::Lit(v.clone()));
            }
            Expr::Column(name) => {
                let schema = self
                    .schema
                    .ok_or_else(|| Error::Schema(format!("column '{name}' used in a query without FROM")))?;
                return Ok(BExpr::Col(schema.require_column(name)?));
            }
            Expr::Unary { op, expr } => {
                let inner = self.bind_inner(expr, aggs, in_agg, clause)?;
                match op {
                    UnaryOp::Neg => BExpr::Neg(Box::new(inner)),
                    UnaryOp::Not => BExpr::Not(Box::new(inner)),
                }
            }
            Expr::Binary { op, left, right } => {
                let l = self.bind_inner(left, aggs.as_deref_mut(), in_agg, clause)?;
                let r = self.bind_inner(right, aggs, in_agg, clause)?;
                BExpr::Bin(*op, Box::new(l), Box::new(r))
            }
            Expr::IsNull { expr, negated } => {
                BExpr::IsNull(Box::new(self.bind_inner(expr, aggs, in_agg, clause)?), *negated)
            }
            Expr::InList { expr, list, negated } => {
                let x = self.bind_inner(expr, aggs.as_deref_mut(), in_agg, clause)?;
                let mut items = Vec::with_capacity(list.len());
                for item in list {
                    items.push(self.bind_inner(item, aggs.as_deref_mut(), in_agg, clause)?);
                }
                BExpr::InList(Box::new(x), items, *negated)
            }
            Expr::Between { expr, low, high, negated } => {
                let x = self.bind_inner(expr, aggs.as_deref_mut(), in_agg, clause)?;
                let lo = self.bind_inner(low, aggs.as_deref_mut(), in_agg, clause)?;
                let hi = self.bind_inner(high, aggs, in_agg, clause)?;
                BExpr::Between(Box::new(x), Box::new(lo), Box::new(hi), *negated)
            }
            Expr::Like { expr, pattern, negated, case_insensitive } => {
                let x = self.bind_inner(expr, aggs.as_deref_mut(), in_agg, clause)?;
                let p = self.bind_inner(pattern, aggs, in_agg, clause)?;
                BExpr::Like { expr: Box::new(x), pattern: Box::new(p), negated: *negated, ci: *case_insensitive }
            }
            Expr::Function { name, args, star, distinct } => {
                if AGGREGATES.contains(&name.as_str()) {
                    return self.bind_aggregate(e, name, args, *star, *distinct, aggs, in_agg, clause);
                }
                if *distinct {
                    return Err(Error::Parse {
                        pos: 0,
                        msg: format!("DISTINCT is only valid inside aggregates, not {name}()"),
                    });
                }
                let f = scalar_fn(name, args.len())?;
                let mut bound = Vec::with_capacity(args.len());
                for a in args {
                    bound.push(self.bind_inner(a, aggs.as_deref_mut(), in_agg, clause)?);
                }
                BExpr::Func(f, bound)
            }
        };
        fold(b)
    }

    #[allow(clippy::too_many_arguments)]
    fn bind_aggregate(
        &self,
        e: &Expr,
        name: &str,
        args: &[Expr],
        star: bool,
        distinct: bool,
        aggs: Option<&mut Vec<AggSpec>>,
        in_agg: bool,
        clause: &str,
    ) -> Result<BExpr> {
        let Some(aggs) = aggs else {
            let where_ = if clause.is_empty() { "here".to_string() } else { format!("in {clause}") };
            return Err(Error::Schema(format!("aggregate function {name}() is not allowed {where_}")));
        };
        if in_agg {
            return Err(Error::Schema(format!("aggregate function {name}() cannot be nested")));
        }
        let func = match (name, star) {
            ("COUNT", true) => AggFunc::CountStar,
            ("COUNT", false) => AggFunc::Count,
            ("SUM", _) => AggFunc::Sum,
            ("AVG", _) => AggFunc::Avg,
            ("MIN", _) => AggFunc::Min,
            ("MAX", _) => AggFunc::Max,
            _ => unreachable!("not an aggregate"),
        };
        let arg = if func == AggFunc::CountStar {
            None
        } else {
            if args.len() != 1 {
                return Err(Error::Schema(format!("{name}() takes exactly one argument")));
            }
            Some(self.bind_inner(&args[0], None, true, clause).map_err(|err| match err {
                Error::Schema(m) if m.contains("not allowed") => {
                    Error::Schema(format!("aggregate function {name}() cannot be nested"))
                }
                other => other,
            })?)
        };
        let spec = AggSpec { func, arg, distinct, label: e.to_string() };
        if let Some(i) = aggs.iter().position(|a| *a == spec) {
            return Ok(BExpr::Agg(i));
        }
        aggs.push(spec);
        Ok(BExpr::Agg(aggs.len() - 1))
    }
}

fn scalar_fn(name: &str, argc: usize) -> Result<ScalarFn> {
    let (f, ok) = match name {
        "LOWER" => (ScalarFn::Lower, argc == 1),
        "UPPER" => (ScalarFn::Upper, argc == 1),
        "LENGTH" | "LEN" => (ScalarFn::Length, argc == 1),
        "ABS" => (ScalarFn::Abs, argc == 1),
        "ROUND" => (ScalarFn::Round, argc == 1 || argc == 2),
        "COALESCE" | "IFNULL" => (ScalarFn::Coalesce, argc >= 1),
        "SUBSTR" | "SUBSTRING" => (ScalarFn::Substr, argc == 2 || argc == 3),
        "TYPEOF" => (ScalarFn::Typeof, argc == 1),
        _ => return Err(Error::Schema(format!("unknown function {name}()"))),
    };
    if !ok {
        return Err(Error::Schema(format!("wrong number of arguments for {name}()")));
    }
    Ok(f)
}

/// Evaluates nodes whose operands are all literals at bind time.
fn fold(b: BExpr) -> Result<BExpr> {
    let constant = match &b {
        BExpr::Neg(x) | BExpr::Not(x) | BExpr::IsNull(x, _) => x.is_lit(),
        BExpr::Bin(_, l, r) => l.is_lit() && r.is_lit(),
        BExpr::InList(x, list, _) => x.is_lit() && list.iter().all(BExpr::is_lit),
        BExpr::Between(x, l, h, _) => x.is_lit() && l.is_lit() && h.is_lit(),
        BExpr::Like { expr, pattern, .. } => expr.is_lit() && pattern.is_lit(),
        BExpr::Func(_, args) => args.iter().all(BExpr::is_lit),
        BExpr::Lit(_) | BExpr::Col(_) | BExpr::Agg(_) => false,
    };
    if constant {
        let v = eval(&b, &[], &[])?.into_owned();
        Ok(BExpr::Lit(v))
    } else {
        Ok(b)
    }
}

/// Evaluates an expression against a row (and the aggregate values of the
/// current group, if any).
///
/// Column references and literals – the leaves of almost every expression –
/// are resolved inline without a function call; only composite nodes go
/// through `eval_node`. Keeping the hot path out of the big `match` avoids a
/// heavy stack frame per leaf, which made filters several times slower.
#[inline(always)]
pub fn eval<'a>(e: &'a BExpr, row: &'a [Value], aggs: &'a [Value]) -> Result<Cow<'a, Value>> {
    match e {
        BExpr::Lit(v) => Ok(Cow::Borrowed(v)),
        BExpr::Col(i) => Ok(Cow::Borrowed(&row[*i])),
        BExpr::Agg(i) => Ok(Cow::Borrowed(&aggs[*i])),
        other => eval_node(other, row, aggs).map(Cow::Owned),
    }
}

#[inline(never)]
fn eval_node(e: &BExpr, row: &[Value], aggs: &[Value]) -> Result<Value> {
    match e {
        BExpr::Lit(_) | BExpr::Col(_) | BExpr::Agg(_) => Ok(eval(e, row, aggs)?.into_owned()),
        BExpr::Bin(BinaryOp::And | BinaryOp::Or, ..) | BExpr::Not(_) => Ok(opt_bool(eval_pred(e, row, aggs)?)),
        BExpr::Bin(op, l, r) if op.is_comparison() => {
            Ok(opt_bool(compare(*op, &*eval(l, row, aggs)?, &*eval(r, row, aggs)?)?))
        }
        BExpr::Bin(BinaryOp::Concat, l, r) => {
            let (a, b) = (eval(l, row, aggs)?, eval(r, row, aggs)?);
            if a.is_null() || b.is_null() {
                return Ok(Value::Null);
            }
            Ok(Value::Text(format!("{a}{b}")))
        }
        BExpr::Bin(op, l, r) => arith(*op, &*eval(l, row, aggs)?, &*eval(r, row, aggs)?),
        BExpr::Neg(x) => match &*eval(x, row, aggs)? {
            Value::Null => Ok(Value::Null),
            Value::Int(i) => i.checked_neg().map(Value::Int).ok_or_else(overflow),
            Value::Float(f) => Ok(Value::Float(-f)),
            other => Err(type_error(format!("cannot negate {} value", other.type_name()))),
        },
        BExpr::IsNull(x, negated) => Ok(Value::Bool(eval(x, row, aggs)?.is_null() != *negated)),
        BExpr::InList(..) | BExpr::Between(..) | BExpr::Like { .. } => Ok(opt_bool(eval_pred(e, row, aggs)?)),
        BExpr::Func(f, args) => eval_func(*f, args, row, aggs),
    }
}

/// Evaluates a boolean expression to SQL three-valued logic (`None` = NULL)
/// without materialising intermediate `Value::Bool`s.
pub fn eval_pred(e: &BExpr, row: &[Value], aggs: &[Value]) -> Result<Option<bool>> {
    match e {
        BExpr::Bin(BinaryOp::And, l, r) => {
            let a = eval_pred(l, row, aggs)?;
            if a == Some(false) {
                return Ok(Some(false));
            }
            Ok(and3(a, eval_pred(r, row, aggs)?))
        }
        BExpr::Bin(BinaryOp::Or, l, r) => {
            let a = eval_pred(l, row, aggs)?;
            if a == Some(true) {
                return Ok(Some(true));
            }
            Ok(match (a, eval_pred(r, row, aggs)?) {
                (_, Some(true)) => Some(true),
                (Some(false), Some(false)) => Some(false),
                _ => None,
            })
        }
        BExpr::Bin(op, l, r) if op.is_comparison() => compare(*op, &*eval(l, row, aggs)?, &*eval(r, row, aggs)?),
        BExpr::Not(x) => Ok(eval_pred(x, row, aggs)?.map(|b| !b)),
        BExpr::IsNull(x, negated) => Ok(Some(eval(x, row, aggs)?.is_null() != *negated)),
        BExpr::InList(x, list, negated) => eval_in_list(x, list, *negated, row, aggs),
        BExpr::Between(x, lo, hi, negated) => eval_between(x, lo, hi, *negated, row, aggs),
        BExpr::Like { expr, pattern, negated, ci } => eval_like(expr, pattern, *negated, *ci, row, aggs),
        other => truth(&*eval(other, row, aggs)?),
    }
}

#[inline(never)]
fn eval_between(
    x: &BExpr,
    lo: &BExpr,
    hi: &BExpr,
    negated: bool,
    row: &[Value],
    aggs: &[Value],
) -> Result<Option<bool>> {
    let v = eval(x, row, aggs)?;
    let ge = compare(BinaryOp::GtEq, &v, &*eval(lo, row, aggs)?)?;
    let le = compare(BinaryOp::LtEq, &v, &*eval(hi, row, aggs)?)?;
    Ok(and3(ge, le).map(|b| b != negated))
}

#[inline(never)]
fn eval_like(
    x: &BExpr,
    pattern: &BExpr,
    negated: bool,
    ci: bool,
    row: &[Value],
    aggs: &[Value],
) -> Result<Option<bool>> {
    let v = eval(x, row, aggs)?;
    let p = eval(pattern, row, aggs)?;
    match (&*v, &*p) {
        (Value::Null, _) | (_, Value::Null) => Ok(None),
        (Value::Text(s), Value::Text(pat)) => Ok(Some(like_match(s, pat, ci) != negated)),
        (a, _) => Err(type_error(format!("LIKE requires TEXT, got {}", a.type_name()))),
    }
}

#[inline(never)]
fn eval_in_list(x: &BExpr, list: &[BExpr], negated: bool, row: &[Value], aggs: &[Value]) -> Result<Option<bool>> {
    let v = eval(x, row, aggs)?;
    if v.is_null() {
        return Ok(None);
    }
    let mut saw_null = false;
    for item in list {
        let iv = eval(item, row, aggs)?;
        if iv.is_null() {
            saw_null = true;
        } else if sql_cmp(&v, &iv)? == Ordering::Equal {
            return Ok(Some(!negated));
        }
    }
    Ok(if saw_null { None } else { Some(negated) })
}

#[cold]
#[inline(never)]
fn type_error(msg: String) -> Error {
    Error::Type(msg)
}

#[cold]
#[inline(never)]
fn overflow() -> Error {
    Error::Eval("integer overflow".into())
}

fn and3(a: Option<bool>, b: Option<bool>) -> Option<bool> {
    match (a, b) {
        (Some(false), _) | (_, Some(false)) => Some(false),
        (Some(true), Some(true)) => Some(true),
        _ => None,
    }
}

fn opt_bool(b: Option<bool>) -> Value {
    b.map_or(Value::Null, Value::Bool)
}

/// SQL truthiness: NULL is "unknown" (`None`).
pub fn truth(v: &Value) -> Result<Option<bool>> {
    match v {
        Value::Null => Ok(None),
        Value::Bool(b) => Ok(Some(*b)),
        Value::Int(i) => Ok(Some(*i != 0)),
        Value::Float(f) => Ok(Some(*f != 0.0)),
        Value::Text(_) => Err(type_error("expected a boolean condition, got TEXT".into())),
    }
}

/// Evaluates a predicate; rows for which it is NULL do not qualify.
#[inline]
pub fn matches(filter: Option<&BExpr>, row: &[Value]) -> Result<bool> {
    match filter {
        None => Ok(true),
        Some(f) => Ok(eval_pred(f, row, &[])? == Some(true)),
    }
}

/// Ordering of two non-NULL values for comparison operators.
#[inline]
pub fn sql_cmp(a: &Value, b: &Value) -> Result<Ordering> {
    match (a, b) {
        (Value::Int(x), Value::Int(y)) => Ok(x.cmp(y)),
        (Value::Bool(x), Value::Int(_) | Value::Float(_)) => Ok(Value::Int(*x as i64).cmp(b)),
        (Value::Int(_) | Value::Float(_), Value::Bool(y)) => Ok(a.cmp(&Value::Int(*y as i64))),
        _ if a.comparable(b) => Ok(a.cmp(b)),
        _ => Err(type_error(format!("cannot compare {} with {}", a.type_name(), b.type_name()))),
    }
}

/// Comparison operator in three-valued logic.
#[inline]
fn compare(op: BinaryOp, a: &Value, b: &Value) -> Result<Option<bool>> {
    if a.is_null() || b.is_null() {
        return Ok(None);
    }
    let ord = sql_cmp(a, b)?;
    Ok(Some(match op {
        BinaryOp::Eq => ord == Ordering::Equal,
        BinaryOp::NotEq => ord != Ordering::Equal,
        BinaryOp::Lt => ord == Ordering::Less,
        BinaryOp::LtEq => ord != Ordering::Greater,
        BinaryOp::Gt => ord == Ordering::Greater,
        BinaryOp::GtEq => ord != Ordering::Less,
        _ => unreachable!(),
    }))
}

fn arith(op: BinaryOp, a: &Value, b: &Value) -> Result<Value> {
    if a.is_null() || b.is_null() {
        return Ok(Value::Null);
    }
    match (a, b) {
        (Value::Int(x), Value::Int(y)) => {
            let (x, y) = (*x, *y);
            let r = match op {
                BinaryOp::Add => x.checked_add(y),
                BinaryOp::Sub => x.checked_sub(y),
                BinaryOp::Mul => x.checked_mul(y),
                BinaryOp::Div | BinaryOp::Mod if y == 0 => return Err(Error::Eval("division by zero".into())),
                BinaryOp::Div => x.checked_div(y),
                BinaryOp::Mod => x.checked_rem(y),
                _ => unreachable!(),
            };
            r.map(Value::Int).ok_or_else(overflow)
        }
        _ if a.is_numeric() && b.is_numeric() => {
            let (x, y) = (a.as_f64().unwrap(), b.as_f64().unwrap());
            if matches!(op, BinaryOp::Div | BinaryOp::Mod) && y == 0.0 {
                return Err(Error::Eval("division by zero".into()));
            }
            Ok(Value::Float(match op {
                BinaryOp::Add => x + y,
                BinaryOp::Sub => x - y,
                BinaryOp::Mul => x * y,
                BinaryOp::Div => x / y,
                BinaryOp::Mod => x % y,
                _ => unreachable!(),
            }))
        }
        _ => Err(Error::Type(format!(
            "operator {} cannot be applied to {} and {}",
            op.symbol(),
            a.type_name(),
            b.type_name()
        ))),
    }
}

fn eval_func(f: ScalarFn, args: &[BExpr], row: &[Value], aggs: &[Value]) -> Result<Value> {
    if f == ScalarFn::Coalesce {
        for a in args {
            let v = eval(a, row, aggs)?;
            if !v.is_null() {
                return Ok(v.into_owned());
            }
        }
        return Ok(Value::Null);
    }
    let first = eval(&args[0], row, aggs)?;
    if f == ScalarFn::Typeof {
        return Ok(Value::Text(first.type_name().to_ascii_lowercase()));
    }
    if first.is_null() {
        return Ok(Value::Null);
    }
    let text = |v: &Value| match v {
        Value::Text(s) => s.clone(),
        other => other.to_string(),
    };
    match f {
        ScalarFn::Lower => Ok(Value::Text(text(&first).to_lowercase())),
        ScalarFn::Upper => Ok(Value::Text(text(&first).to_uppercase())),
        ScalarFn::Length => Ok(Value::Int(text(&first).chars().count() as i64)),
        ScalarFn::Abs => match &*first {
            Value::Int(i) => i.checked_abs().map(Value::Int).ok_or_else(overflow),
            Value::Float(x) => Ok(Value::Float(x.abs())),
            other => Err(Error::Type(format!("ABS() requires a number, got {}", other.type_name()))),
        },
        ScalarFn::Round => {
            let digits = match args.get(1) {
                Some(d) => match &*eval(d, row, aggs)? {
                    Value::Int(n) => *n,
                    Value::Null => return Ok(Value::Null),
                    _ => return Err(Error::Type("ROUND() precision must be an INT".into())),
                },
                None => 0,
            };
            match &*first {
                Value::Int(i) => Ok(Value::Int(*i)),
                Value::Float(x) => {
                    let m = 10f64.powi(digits.clamp(-15, 15) as i32);
                    Ok(Value::Float((x * m).round() / m))
                }
                other => Err(Error::Type(format!("ROUND() requires a number, got {}", other.type_name()))),
            }
        }
        ScalarFn::Substr => {
            let s: Vec<char> = text(&first).chars().collect();
            let int_arg = |i: usize| -> Result<Option<i64>> {
                match args.get(i) {
                    None => Ok(None),
                    Some(a) => match &*eval(a, row, aggs)? {
                        Value::Int(n) => Ok(Some(*n)),
                        _ => Err(Error::Type("SUBSTR() positions must be INT".into())),
                    },
                }
            };
            let start = int_arg(1)?.unwrap_or(1).max(1) as usize - 1;
            let len = int_arg(2)?.map(|n| n.max(0) as usize).unwrap_or(usize::MAX);
            Ok(Value::Text(s.iter().skip(start).take(len).collect()))
        }
        ScalarFn::Coalesce | ScalarFn::Typeof => unreachable!(),
    }
}

/// SQL `LIKE` with `%` (any run) and `_` (single character) wildcards.
pub fn like_match(text: &str, pattern: &str, case_insensitive: bool) -> bool {
    let (t, p): (Vec<char>, Vec<char>) = if case_insensitive {
        (text.to_lowercase().chars().collect(), pattern.to_lowercase().chars().collect())
    } else {
        (text.chars().collect(), pattern.chars().collect())
    };
    let (mut ti, mut pi) = (0usize, 0usize);
    let mut backtrack: Option<(usize, usize)> = None;
    while ti < t.len() {
        if pi < p.len() && (p[pi] == '_' || (p[pi] != '%' && p[pi] == t[ti])) {
            ti += 1;
            pi += 1;
        } else if pi < p.len() && p[pi] == '%' {
            backtrack = Some((pi, ti));
            pi += 1;
        } else if let Some((bp, bt)) = backtrack {
            pi = bp + 1;
            ti = bt + 1;
            backtrack = Some((bp, bt + 1));
        } else {
            return false;
        }
    }
    p[pi..].iter().all(|c| *c == '%')
}

/// Running state of one aggregate within one group.
#[derive(Debug, Clone)]
pub struct AggAcc {
    count: i64,
    int_sum: i128,
    float_sum: f64,
    saw_float: bool,
    best: Option<Value>,
    seen: Option<BTreeSet<Value>>,
}

impl AggAcc {
    pub fn new(spec: &AggSpec) -> Self {
        AggAcc {
            count: 0,
            int_sum: 0,
            float_sum: 0.0,
            saw_float: false,
            best: None,
            seen: spec.distinct.then(BTreeSet::new),
        }
    }

    pub fn update(&mut self, spec: &AggSpec, row: &[Value]) -> Result<()> {
        let Some(arg) = &spec.arg else {
            self.count += 1;
            return Ok(());
        };
        let v = eval(arg, row, &[])?;
        if v.is_null() {
            return Ok(());
        }
        if let Some(seen) = &mut self.seen {
            if !seen.insert(v.clone().into_owned()) {
                return Ok(());
            }
        }
        match spec.func {
            AggFunc::Count | AggFunc::CountStar => self.count += 1,
            AggFunc::Sum | AggFunc::Avg => {
                match &*v {
                    Value::Int(i) => self.int_sum += *i as i128,
                    Value::Float(f) => {
                        self.float_sum += f;
                        self.saw_float = true;
                    }
                    other => {
                        return Err(Error::Type(format!(
                            "{} requires numeric values, got {}",
                            spec.label,
                            other.type_name()
                        )))
                    }
                }
                self.count += 1;
            }
            AggFunc::Min | AggFunc::Max => {
                let want = if spec.func == AggFunc::Min { Ordering::Less } else { Ordering::Greater };
                let replace = match &self.best {
                    None => true,
                    Some(b) => sql_cmp(&v, b)? == want,
                };
                if replace {
                    self.best = Some(v.into_owned());
                }
            }
        }
        Ok(())
    }

    pub fn finish(&self, spec: &AggSpec) -> Value {
        match spec.func {
            AggFunc::CountStar | AggFunc::Count => Value::Int(self.count),
            AggFunc::Sum if self.count == 0 => Value::Null,
            AggFunc::Sum if self.saw_float => Value::Float(self.int_sum as f64 + self.float_sum),
            AggFunc::Sum => match i64::try_from(self.int_sum) {
                Ok(i) => Value::Int(i),
                Err(_) => Value::Float(self.int_sum as f64),
            },
            AggFunc::Avg if self.count == 0 => Value::Null,
            AggFunc::Avg => Value::Float((self.int_sum as f64 + self.float_sum) / self.count as f64),
            AggFunc::Min | AggFunc::Max => self.best.clone().unwrap_or(Value::Null),
        }
    }
}

/// Human readable form of a bound expression (EXPLAIN output).
pub fn describe(e: &BExpr, schema: Option<&TableSchema>, aggs: &[AggSpec]) -> String {
    let d = |x: &BExpr| {
        let s = describe(x, schema, aggs);
        if matches!(x, BExpr::Bin(..) | BExpr::Between(..) | BExpr::InList(..) | BExpr::Like { .. } | BExpr::IsNull(..))
        {
            format!("({s})")
        } else {
            s
        }
    };
    match e {
        BExpr::Lit(v) => match v {
            Value::Float(f) => format_float(*f),
            other => other.to_sql(),
        },
        BExpr::Col(i) => schema.map_or_else(|| format!("#{i}"), |s| s.columns[*i].name.clone()),
        BExpr::Agg(i) => aggs.get(*i).map_or_else(|| format!("agg#{i}"), |a| a.label.clone()),
        BExpr::Neg(x) => format!("-{}", d(x)),
        BExpr::Not(x) => format!("NOT {}", d(x)),
        BExpr::Bin(op, l, r) => format!("{} {} {}", d(l), op.symbol(), d(r)),
        BExpr::IsNull(x, n) => format!("{} IS {}NULL", d(x), if *n { "NOT " } else { "" }),
        BExpr::InList(x, list, n) => format!(
            "{} {}IN ({})",
            d(x),
            if *n { "NOT " } else { "" },
            list.iter().map(|i| describe(i, schema, aggs)).collect::<Vec<_>>().join(", ")
        ),
        BExpr::Between(x, lo, hi, n) => {
            format!("{} {}BETWEEN {} AND {}", d(x), if *n { "NOT " } else { "" }, d(lo), d(hi))
        }
        BExpr::Like { expr, pattern, negated, ci } => format!(
            "{} {}{} {}",
            d(expr),
            if *negated { "NOT " } else { "" },
            if *ci { "ILIKE" } else { "LIKE" },
            d(pattern)
        ),
        BExpr::Func(f, args) => {
            format!("{:?}({})", f, args.iter().map(|a| describe(a, schema, aggs)).collect::<Vec<_>>().join(", "))
                .to_uppercase()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sql::{parse_one, SelectItem, Statement};

    fn eval_sql(expr: &str) -> Result<Value> {
        let Statement::Select(s) = parse_one(&format!("SELECT {expr}")).unwrap() else { panic!() };
        let SelectItem::Expr { expr, .. } = &s.projection[0] else { panic!() };
        let b = Binder::new(None, &[]).bind(expr, "SELECT")?;
        Ok(eval(&b, &[], &[])?.into_owned())
    }

    #[test]
    fn arithmetic() {
        assert_eq!(eval_sql("1 + 2 * 3").unwrap(), Value::Int(7));
        assert_eq!(eval_sql("7 / 2").unwrap(), Value::Int(3));
        assert_eq!(eval_sql("7 / 2.0").unwrap(), Value::Float(3.5));
        assert_eq!(eval_sql("-(3 - 5)").unwrap(), Value::Int(2));
        assert_eq!(eval_sql("10 % 4").unwrap(), Value::Int(2));
        assert!(matches!(eval_sql("1 / 0"), Err(Error::Eval(_))));
        assert!(matches!(eval_sql("9223372036854775807 + 1"), Err(Error::Eval(_))));
        assert!(matches!(eval_sql("'a' + 1"), Err(Error::Type(_))));
    }

    #[test]
    fn three_valued_logic() {
        assert_eq!(eval_sql("NULL = NULL").unwrap(), Value::Null);
        assert_eq!(eval_sql("NULL AND FALSE").unwrap(), Value::Bool(false));
        assert_eq!(eval_sql("NULL OR TRUE").unwrap(), Value::Bool(true));
        assert_eq!(eval_sql("NULL OR FALSE").unwrap(), Value::Null);
        assert_eq!(eval_sql("NOT NULL").unwrap(), Value::Null);
        assert_eq!(eval_sql("1 IN (2, NULL)").unwrap(), Value::Null);
        assert_eq!(eval_sql("1 IN (1, NULL)").unwrap(), Value::Bool(true));
        assert_eq!(eval_sql("NULL IS NULL").unwrap(), Value::Bool(true));
        assert_eq!(eval_sql("3 BETWEEN 1 AND 5").unwrap(), Value::Bool(true));
        assert_eq!(eval_sql("3 NOT BETWEEN 1 AND 5").unwrap(), Value::Bool(false));
    }

    #[test]
    fn comparisons_and_types() {
        assert_eq!(eval_sql("1 = 1.0").unwrap(), Value::Bool(true));
        assert_eq!(eval_sql("TRUE = 1").unwrap(), Value::Bool(true));
        assert_eq!(eval_sql("'b' > 'a'").unwrap(), Value::Bool(true));
        assert!(matches!(eval_sql("'1' = 1"), Err(Error::Type(_))));
    }

    #[test]
    fn functions() {
        assert_eq!(eval_sql("UPPER('abc') || LOWER('DEF')").unwrap(), Value::from("ABCdef"));
        assert_eq!(eval_sql("LENGTH('zażółć')").unwrap(), Value::Int(6));
        assert_eq!(eval_sql("COALESCE(NULL, NULL, 3)").unwrap(), Value::Int(3));
        assert_eq!(eval_sql("ROUND(2.567, 2)").unwrap(), Value::Float(2.57));
        assert_eq!(eval_sql("ABS(-4)").unwrap(), Value::Int(4));
        assert_eq!(eval_sql("SUBSTR('database', 5)").unwrap(), Value::from("base"));
        assert_eq!(eval_sql("SUBSTR('database', 1, 4)").unwrap(), Value::from("data"));
        assert_eq!(eval_sql("TYPEOF(1.5)").unwrap(), Value::from("float"));
        assert!(eval_sql("NOPE(1)").is_err());
        assert!(eval_sql("COUNT(*)").is_err());
    }

    #[test]
    fn like_patterns() {
        assert!(like_match("sword of fire", "sword%", false));
        assert!(like_match("sword of fire", "%of%", false));
        assert!(like_match("abc", "a_c", false));
        assert!(!like_match("abc", "a_", false));
        assert!(like_match("ABC", "abc", true));
        assert!(!like_match("ABC", "abc", false));
        assert!(like_match("aaa", "%a%a", false));
        assert!(like_match("", "%", false));
        assert!(!like_match("abd", "%c%", false));
    }

    #[test]
    fn aggregates_are_collected_and_deduplicated() {
        let Statement::Select(s) = parse_one("SELECT COUNT(*) + COUNT(*), SUM(x)").unwrap() else { panic!() };
        let schema = TableSchema {
            name: "t".into(),
            columns: vec![crate::table::Column {
                name: "x".into(),
                data_type: crate::value::DataType::Int,
                not_null: false,
            }],
            pk: 0,
        };
        let binder = Binder::new(Some(&schema), &[]);
        let mut aggs = Vec::new();
        for item in &s.projection {
            let SelectItem::Expr { expr, .. } = item else { panic!() };
            binder.bind_agg(expr, &mut aggs).unwrap();
        }
        assert_eq!(aggs.len(), 2);
        let Statement::Select(s) = parse_one("SELECT SUM(COUNT(x))").unwrap() else { panic!() };
        let SelectItem::Expr { expr, .. } = &s.projection[0] else { panic!() };
        assert!(binder.bind_agg(expr, &mut Vec::new()).is_err());
    }
}
