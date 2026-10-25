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
//! The access path returns a superset of the matching rows. [`choose_terms`]
//! also tells which conjuncts of the WHERE clause every returned row
//! satisfies: the executor evaluates only the others, and a query that needs
//! nothing but the number of matching rows counts them without reading a row
//! when the access path guarantees the whole clause.

use std::borrow::Borrow;
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
    let terms = filter.map(BExpr::conjuncts).unwrap_or_default();
    Ok(choose_terms(table, &terms)?.0)
}

/// Picks the access path and tells whether it guarantees the whole WHERE
/// clause (see [`choose_terms`]).
pub fn choose_covered(table: &Table, filter: Option<&BExpr>) -> Result<(Access, bool)> {
    let terms = filter.map(BExpr::conjuncts).unwrap_or_default();
    let (access, absorbed) = choose_terms(table, &terms)?;
    Ok((access, covers_all(absorbed, terms.len())))
}

/// Mask of the first `n` conjuncts.
fn all_terms(n: usize) -> u64 {
    if n >= 64 {
        u64::MAX
    } else {
        (1 << n) - 1
    }
}

/// Whether `absorbed` contains every one of `n` conjuncts (only the first 64
/// can be absorbed).
pub fn covers_all(absorbed: u64, n: usize) -> bool {
    n <= 64 && absorbed == all_terms(n)
}

/// Picks the access path for a WHERE clause given as its top-level `AND`
/// terms and returns the conjuncts that every row of the path satisfies
/// (bit `i` set: conjunct `i`), so they need not be evaluated again.
///
/// A conjunct `col = lit` or `col IN (lit, ...)` that drives a lookup is
/// guaranteed, and so is every range conjunct on the column of a range scan,
/// because the bounds of the scan are the tightest of them. The key passed
/// to the tree has the type the column stores (`key_for`), and the tree
/// orders values of one type the way the comparison operators do, so a
/// returned row compares the same way as in the evaluator. A range scan of a
/// secondary index without a lower bound starts after NULL, which the index
/// stores but which never satisfies a comparison. A literal TRUE is
/// guaranteed as well; when the access path is `Empty` nothing is evaluated.
pub fn choose_terms<T: Borrow<BExpr>>(table: &Table, terms: &[T]) -> Result<(Access, u64)> {
    let empty = Ok((Access::Empty, all_terms(terms.len())));
    let bit = |pos: usize| if pos < 64 { 1u64 << pos } else { 0 };

    // Each candidate remembers the conjuncts it guarantees.
    let mut eq: Vec<(usize, Value, usize)> = Vec::new();
    let mut ins: Vec<(usize, Vec<Value>, usize)> = Vec::new();
    let mut lower: BTreeMap<usize, Bound<Value>> = BTreeMap::new();
    let mut upper: BTreeMap<usize, Bound<Value>> = BTreeMap::new();
    let mut ranged: BTreeMap<usize, u64> = BTreeMap::new();
    let mut always: u64 = 0;

    for (pos, term) in terms.iter().map(Borrow::borrow).enumerate() {
        match term {
            BExpr::Lit(v) => match truth(v) {
                Ok(Some(true)) => always |= bit(pos),
                Ok(_) => return empty,
                // A WHERE clause that is only a non-boolean literal is an
                // error; next to other terms it is reported by the evaluator.
                Err(e) if terms.len() == 1 => return Err(e),
                Err(_) => {}
            },
            BExpr::Bin(op, l, r) if op.is_comparison() && *op != BinaryOp::NotEq => {
                let (col, v, op) = match (&**l, &**r) {
                    (BExpr::Col(c), BExpr::Lit(v)) => (*c, v, *op),
                    (BExpr::Lit(v), BExpr::Col(c)) => (*c, v, op.flipped()),
                    _ => continue,
                };
                if v.is_null() {
                    return empty;
                }
                let Some(v) = key_for(table, col, v)? else { continue };
                match op {
                    BinaryOp::Eq => {
                        eq.push((col, v, pos));
                        continue;
                    }
                    BinaryOp::Lt => tighten(&mut upper, col, Bound::Excluded(v), false),
                    BinaryOp::LtEq => tighten(&mut upper, col, Bound::Included(v), false),
                    BinaryOp::Gt => tighten(&mut lower, col, Bound::Excluded(v), true),
                    BinaryOp::GtEq => tighten(&mut lower, col, Bound::Included(v), true),
                    _ => continue,
                }
                *ranged.entry(col).or_default() |= bit(pos);
            }
            BExpr::Between(x, lo, hi, false) => {
                let (BExpr::Col(c), BExpr::Lit(a), BExpr::Lit(b)) = (&**x, &**lo, &**hi) else { continue };
                if a.is_null() || b.is_null() {
                    return empty;
                }
                if let (Some(a), Some(b)) = (key_for(table, *c, a)?, key_for(table, *c, b)?) {
                    tighten(&mut lower, *c, Bound::Included(a), true);
                    tighten(&mut upper, *c, Bound::Included(b), false);
                    *ranged.entry(*c).or_default() |= bit(pos);
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
                    return empty;
                }
                values.sort();
                values.dedup();
                ins.push((*c, values, pos));
            }
            _ => {}
        }
    }

    let pk = table.schema.pk;
    if let Some((_, v, pos)) = eq.iter().find(|(c, ..)| *c == pk) {
        return Ok((Access::PkPoint(v.clone()), always | bit(*pos)));
    }
    if let Some((_, vs, pos)) = ins.iter().find(|(c, ..)| *c == pk) {
        return Ok((Access::PkMulti(vs.clone()), always | bit(*pos)));
    }
    for (c, v, pos) in &eq {
        if let Some(i) = table.index_on(*c) {
            return Ok((Access::IndexPoint(i, v.clone()), always | bit(*pos)));
        }
    }
    for (c, vs, pos) in &ins {
        if let Some(i) = table.index_on(*c) {
            return Ok((Access::IndexMulti(i, vs.clone()), always | bit(*pos)));
        }
    }
    let bounds = |c: usize| {
        let lo = lower.get(&c).cloned().unwrap_or(Bound::Unbounded);
        let hi = upper.get(&c).cloned().unwrap_or(Bound::Unbounded);
        (lo, hi)
    };
    if lower.contains_key(&pk) || upper.contains_key(&pk) {
        let (lo, hi) = bounds(pk);
        if range_empty(&lo, &hi) {
            return empty;
        }
        return Ok((Access::PkRange(lo, hi), always | ranged[&pk]));
    }
    for c in lower.keys().chain(upper.keys()) {
        if let Some(i) = table.index_on(*c) {
            let (lo, hi) = bounds(*c);
            if range_empty(&lo, &hi) {
                return empty;
            }
            let lo = match lo {
                Bound::Unbounded => Bound::Excluded(Value::Null),
                lo => lo,
            };
            return Ok((Access::IndexRange(i, lo, hi), always | ranged[c]));
        }
    }
    Ok((Access::FullScan, always))
}

/// Number of rows an access path returns, counted without reading them: a
/// secondary index keeps the set of keys of every value.
pub fn count(t: &Table, access: &Access) -> u64 {
    let n = match access {
        Access::Empty => 0,
        Access::PkPoint(k) => usize::from(t.rows.contains_key(k)),
        Access::PkMulti(keys) => keys.iter().filter(|k| t.rows.contains_key(*k)).count(),
        Access::PkRange(lo, hi) => t.rows.range::<Value, _>((lo.as_ref(), hi.as_ref())).count(),
        Access::IndexPoint(i, v) => t.indexes[*i].map.get(v).map_or(0, |keys| keys.len()),
        Access::IndexMulti(i, vs) => vs.iter().filter_map(|v| t.indexes[*i].map.get(v)).map(|keys| keys.len()).sum(),
        Access::IndexRange(i, lo, hi) => {
            t.indexes[*i].map.range::<Value, _>((lo.as_ref(), hi.as_ref())).map(|(_, keys)| keys.len()).sum()
        }
        Access::FullScan => t.rows.len(),
    };
    n as u64
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
