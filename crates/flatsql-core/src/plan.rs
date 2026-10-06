//! Access path selection.
//!
//! The planner looks at the top-level `AND` terms of a WHERE clause and picks
//! the cheapest way to find candidate rows, in this order of preference:
//!
//! 1. primary key point lookup (`pk = c`) / multi lookup (`pk IN (...)`),
//! 2. secondary index lookup (`col = c`, `col IN (...)`),
//! 3. primary key range scan (`pk > a AND pk <= b`, `pk BETWEEN a AND b`),
//! 4. secondary index range scan,
//! 5. full table scan.
//!
//! The complete WHERE clause is still evaluated on every candidate row, so
//! the access path only has to return a superset of the matching rows.

use std::collections::BTreeMap;
use std::ops::Bound;

use crate::error::{Error, Result};
use crate::expr::{describe, truth, BExpr};
use crate::sql::BinaryOp;
use crate::table::{RowEntry, Table};
use crate::value::{DataType, Value};

#[derive(Debug, Clone, PartialEq)]
pub enum Access {
    /// The condition can never be true.
    Empty,
    PkPoint(Value),
    PkMulti(Vec<Value>),
    PkRange(Bound<Value>, Bound<Value>),
    IndexPoint(usize, Value),
    IndexMulti(usize, Vec<Value>),
    IndexRange(usize, Bound<Value>, Bound<Value>),
    FullScan,
}

impl Access {
    /// Whether rows come out in primary key order.
    pub fn pk_ordered(&self) -> bool {
        matches!(self, Access::Empty | Access::PkPoint(_) | Access::PkMulti(_) | Access::PkRange(..) | Access::FullScan)
    }

    pub fn describe(&self, t: &Table) -> String {
        let pk = &t.schema.pk_column().name;
        let list = |vs: &[Value]| vs.iter().map(Value::to_sql).collect::<Vec<_>>().join(", ");
        let range = |col: &str, lo: &Bound<Value>, hi: &Bound<Value>| {
            let mut parts = Vec::new();
            match lo {
                Bound::Included(v) => parts.push(format!("{col} >= {}", v.to_sql())),
                Bound::Excluded(v) => parts.push(format!("{col} > {}", v.to_sql())),
                Bound::Unbounded => {}
            }
            match hi {
                Bound::Included(v) => parts.push(format!("{col} <= {}", v.to_sql())),
                Bound::Excluded(v) => parts.push(format!("{col} < {}", v.to_sql())),
                Bound::Unbounded => {}
            }
            parts.join(" AND ")
        };
        let index = |i: usize| {
            let idx = &t.indexes[i];
            (idx.name.clone(), t.schema.columns[idx.column].name.clone())
        };
        match self {
            Access::Empty => "EMPTY (condition is never true)".into(),
            Access::PkPoint(v) => format!("PRIMARY KEY LOOKUP ({pk} = {})", v.to_sql()),
            Access::PkMulti(vs) => format!("PRIMARY KEY MULTI-LOOKUP ({pk} IN ({}))", list(vs)),
            Access::PkRange(lo, hi) => format!("PRIMARY KEY RANGE SCAN ({})", range(pk, lo, hi)),
            Access::IndexPoint(i, v) => {
                let (name, col) = index(*i);
                format!("INDEX LOOKUP {name} ({col} = {})", v.to_sql())
            }
            Access::IndexMulti(i, vs) => {
                let (name, col) = index(*i);
                format!("INDEX MULTI-LOOKUP {name} ({col} IN ({}))", list(vs))
            }
            Access::IndexRange(i, lo, hi) => {
                let (name, col) = index(*i);
                format!("INDEX RANGE SCAN {name} ({})", range(&col, lo, hi))
            }
            Access::FullScan => format!("FULL TABLE SCAN ({} rows)", t.rows.len()),
        }
    }
}

/// Picks the access path for a (bound) WHERE clause.
pub fn choose(table: &Table, filter: Option<&BExpr>) -> Result<Access> {
    let Some(filter) = filter else { return Ok(Access::FullScan) };
    if let BExpr::Lit(v) = filter {
        return Ok(if truth(v)? == Some(true) { Access::FullScan } else { Access::Empty });
    }

    let mut eq: Vec<(usize, Value)> = Vec::new();
    let mut ins: Vec<(usize, Vec<Value>)> = Vec::new();
    let mut lower: BTreeMap<usize, Bound<Value>> = BTreeMap::new();
    let mut upper: BTreeMap<usize, Bound<Value>> = BTreeMap::new();

    for term in filter.conjuncts() {
        match term {
            BExpr::Bin(op, l, r) if op.is_comparison() && *op != BinaryOp::NotEq => {
                let (col, v, op) = match (&**l, &**r) {
                    (BExpr::Col(c), BExpr::Lit(v)) => (*c, v, *op),
                    (BExpr::Lit(v), BExpr::Col(c)) => (*c, v, op.flipped()),
                    _ => continue,
                };
                if v.is_null() {
                    return Ok(Access::Empty);
                }
                let Some(v) = key_for(table, col, v)? else { continue };
                match op {
                    BinaryOp::Eq => eq.push((col, v)),
                    BinaryOp::Lt => tighten(&mut upper, col, Bound::Excluded(v), false),
                    BinaryOp::LtEq => tighten(&mut upper, col, Bound::Included(v), false),
                    BinaryOp::Gt => tighten(&mut lower, col, Bound::Excluded(v), true),
                    BinaryOp::GtEq => tighten(&mut lower, col, Bound::Included(v), true),
                    _ => {}
                }
            }
            BExpr::Between(x, lo, hi, false) => {
                let (BExpr::Col(c), BExpr::Lit(a), BExpr::Lit(b)) = (&**x, &**lo, &**hi) else { continue };
                if a.is_null() || b.is_null() {
                    return Ok(Access::Empty);
                }
                if let (Some(a), Some(b)) = (key_for(table, *c, a)?, key_for(table, *c, b)?) {
                    tighten(&mut lower, *c, Bound::Included(a), true);
                    tighten(&mut upper, *c, Bound::Included(b), false);
                }
            }
            BExpr::InList(x, list, false) => {
                let BExpr::Col(c) = &**x else { continue };
                let mut values = Vec::with_capacity(list.len());
                let mut usable = true;
                for item in list {
                    match item {
                        BExpr::Lit(v) if v.is_null() => {}
                        BExpr::Lit(v) => match key_for(table, *c, v)? {
                            Some(k) => values.push(k),
                            None => usable = false,
                        },
                        _ => usable = false,
                    }
                }
                if !usable {
                    continue;
                }
                if values.is_empty() {
                    return Ok(Access::Empty);
                }
                values.sort();
                values.dedup();
                ins.push((*c, values));
            }
            _ => {}
        }
    }

    let pk = table.schema.pk;
    if let Some((_, v)) = eq.iter().find(|(c, _)| *c == pk) {
        return Ok(Access::PkPoint(v.clone()));
    }
    if let Some((_, vs)) = ins.iter().find(|(c, _)| *c == pk) {
        return Ok(Access::PkMulti(vs.clone()));
    }
    for (c, v) in &eq {
        if let Some(i) = table.index_on(*c) {
            return Ok(Access::IndexPoint(i, v.clone()));
        }
    }
    for (c, vs) in &ins {
        if let Some(i) = table.index_on(*c) {
            return Ok(Access::IndexMulti(i, vs.clone()));
        }
    }
    let bounds = |c: usize| {
        let lo = lower.get(&c).cloned().unwrap_or(Bound::Unbounded);
        let hi = upper.get(&c).cloned().unwrap_or(Bound::Unbounded);
        (lo, hi)
    };
    if lower.contains_key(&pk) || upper.contains_key(&pk) {
        let (lo, hi) = bounds(pk);
        return Ok(if range_empty(&lo, &hi) { Access::Empty } else { Access::PkRange(lo, hi) });
    }
    for c in lower.keys().chain(upper.keys()) {
        if let Some(i) = table.index_on(*c) {
            let (lo, hi) = bounds(*c);
            return Ok(if range_empty(&lo, &hi) { Access::Empty } else { Access::IndexRange(i, lo, hi) });
        }
    }
    Ok(Access::FullScan)
}

/// Converts a literal into a key comparable with the column, or `None` if the
/// predicate should not drive the access path. Incompatible types are an
/// error (the same one the evaluator would raise).
fn key_for(table: &Table, col: usize, v: &Value) -> Result<Option<Value>> {
    let column = &table.schema.columns[col];
    let ok = match (column.data_type, v) {
        (DataType::Text, Value::Text(_)) => Some(v.clone()),
        (DataType::Int | DataType::Float, Value::Int(_) | Value::Float(_)) => Some(v.clone()),
        (DataType::Int | DataType::Float, Value::Bool(_)) => None,
        (DataType::Bool, Value::Bool(_)) => Some(v.clone()),
        (DataType::Bool, Value::Int(i)) if *i == 0 || *i == 1 => Some(Value::Bool(*i == 1)),
        (DataType::Bool, Value::Int(_) | Value::Float(_)) => None,
        (ty, v) => {
            return Err(Error::Type(format!("cannot compare {} column '{}' with {}", ty, column.name, v.type_name())))
        }
    };
    Ok(ok)
}

fn bound_value(b: &Bound<Value>) -> Option<&Value> {
    match b {
        Bound::Included(v) | Bound::Excluded(v) => Some(v),
        Bound::Unbounded => None,
    }
}

/// Keeps the more restrictive of the existing and the new bound.
fn tighten(map: &mut BTreeMap<usize, Bound<Value>>, col: usize, new: Bound<Value>, is_lower: bool) {
    let replace = match map.get(&col).and_then(bound_value) {
        None => true,
        Some(old) => {
            let nv = bound_value(&new).unwrap();
            match nv.cmp(old) {
                std::cmp::Ordering::Equal => matches!(new, Bound::Excluded(_)),
                std::cmp::Ordering::Greater => is_lower,
                std::cmp::Ordering::Less => !is_lower,
            }
        }
    };
    if replace {
        map.insert(col, new);
    }
}

fn range_empty(lo: &Bound<Value>, hi: &Bound<Value>) -> bool {
    match (lo, hi) {
        (Bound::Unbounded, _) | (_, Bound::Unbounded) => false,
        (l, h) => {
            let (a, b) = (bound_value(l).unwrap(), bound_value(h).unwrap());
            a > b || (a == b && (matches!(l, Bound::Excluded(_)) || matches!(h, Bound::Excluded(_))))
        }
    }
}

type RowIter<'a> = Box<dyn Iterator<Item = &'a RowEntry> + 'a>;

/// Iterates over the candidate rows of an access path.
pub fn scan<'a>(t: &'a Table, access: &'a Access, reverse: bool) -> RowIter<'a> {
    fn dir<'a, I>(it: I, reverse: bool) -> RowIter<'a>
    where
        I: DoubleEndedIterator<Item = &'a RowEntry> + 'a,
    {
        if reverse {
            Box::new(it.rev())
        } else {
            Box::new(it)
        }
    }
    match access {
        Access::Empty => Box::new(std::iter::empty()),
        Access::PkPoint(k) => Box::new(t.rows.get(k).into_iter()),
        Access::PkMulti(keys) => dir(keys.iter().filter_map(move |k| t.rows.get(k)), reverse),
        Access::PkRange(lo, hi) => dir(t.rows.range::<Value, _>((lo.as_ref(), hi.as_ref())).map(|(_, e)| e), reverse),
        Access::IndexPoint(i, v) => Box::new(
            t.indexes[*i].map.get(v).into_iter().flat_map(move |keys| keys.iter().filter_map(move |k| t.rows.get(k))),
        ),
        Access::IndexMulti(i, vs) => Box::new(
            vs.iter()
                .filter_map(move |v| t.indexes[*i].map.get(v))
                .flat_map(move |keys| keys.iter().filter_map(move |k| t.rows.get(k))),
        ),
        Access::IndexRange(i, lo, hi) => Box::new(
            t.indexes[*i]
                .map
                .range::<Value, _>((lo.as_ref(), hi.as_ref()))
                .flat_map(move |(_, keys)| keys.iter().filter_map(move |k| t.rows.get(k))),
        ),
        Access::FullScan => dir(t.rows.values(), reverse),
    }
}

/// Text of the WHERE clause for EXPLAIN.
pub fn describe_filter(t: Option<&Table>, filter: &BExpr) -> String {
    describe(filter, t.map(|t| &*t.schema), &[])
}
