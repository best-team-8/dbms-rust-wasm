//! Statement execution.
//!
//! SELECT, INSERT, UPDATE and DELETE are first compiled against the catalog
//! ([`compile`]): names are resolved, `?` parameters stay as references and
//! the access path is chosen or prepared as a template. A prepared statement
//! keeps the compiled form and executes it with new parameter values; a
//! statement executed once is compiled and run right away. Other statements
//! run from their syntax tree.

use std::borrow::{Borrow, Cow};
use std::cmp::Ordering;
use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::sync::Arc;

use crate::compile::{self, Pred, ValueExpr};
use crate::db::{catalog_key, Database};
use crate::error::{Error, Result};
use crate::expr::{describe, eval, eval_pred, AggAcc, AggFunc, AggSpec, BExpr, Binder, Cx};
use crate::hash::{FxBuildHasher, ValueKey};
use crate::plan::{self, Access, AccessPlan};
use crate::result::QueryResult;
use crate::sql::{CreateTable, Delete, Expr, Insert, OnConflict, Select, SelectItem, Statement, Update};
use crate::sync::{Change, Mutation};
use crate::table::{Column, Table, TableSchema};
use crate::value::Value;

/// A statement compiled against the catalog (see the module documentation).
/// One per prepared statement, so the size of the largest variant does not
/// matter, while a box would add an indirection to every execution.
#[allow(clippy::large_enum_variant)]
#[derive(Debug, Clone)]
pub(crate) enum Compiled {
    Select(SelectPlan),
    Insert(InsertPlan),
    Update(UpdatePlan),
    Delete(DeletePlan),
}

/// Compiles SELECT, INSERT, UPDATE and DELETE; `None` for other statements.
pub(crate) fn compile(db: &Database, stmt: &Statement) -> Result<Option<Compiled>> {
    Ok(Some(match stmt {
        Statement::Select(s) => Compiled::Select(compile_select(db, s)?),
        Statement::Insert(i) => Compiled::Insert(compile_insert(db, i)?),
        Statement::Update(u) => Compiled::Update(compile_update(db, u)?),
        Statement::Delete(d) => Compiled::Delete(compile_delete(db, d)?),
        _ => return Ok(None),
    }))
}

pub(crate) fn run_select(db: &Database, p: &SelectPlan, params: &[Value]) -> Result<QueryResult> {
    let run = bind_select(db, p, params)?;
    let mut rows = Collect(Vec::new());
    let scanned = execute_select(&run, &mut rows)?;
    Ok(QueryResult {
        columns: p.names.clone(),
        rows: rows.0,
        rows_affected: 0,
        rows_scanned: scanned,
        message: String::new(),
    })
}

/// Runs a compiled SELECT with its rows written to `sink`; returns the
/// number of rows the access path visited.
pub(crate) fn run_select_into(db: &Database, p: &SelectPlan, params: &[Value], sink: &mut dyn RowSink) -> Result<u64> {
    execute_select(&bind_select(db, p, params)?, sink)
}

impl SelectPlan {
    pub(crate) fn names(&self) -> &Arc<[String]> {
        &self.names
    }
}

/// Receives the rows of a SELECT: collected as vectors of values for a
/// `QueryResult`, or encoded straight from the table rows into the binary
/// result format, without a copy of every value.
pub(crate) trait RowSink {
    /// A row projected from `row` (a table row, or the first row of a group).
    fn project(&mut self, proj: &[BExpr], row: &[Value], cx: Cx) -> Result<()>;
    /// Rows computed already (aggregates, sorted or distinct rows).
    fn computed(&mut self, rows: Vec<Vec<Value>>);
    /// Rows received so far.
    fn len(&self) -> usize;
}

struct Collect(Vec<Vec<Value>>);

impl RowSink for Collect {
    fn project(&mut self, proj: &[BExpr], row: &[Value], cx: Cx) -> Result<()> {
        self.0.push(project(proj, row, cx)?);
        Ok(())
    }

    fn computed(&mut self, rows: Vec<Vec<Value>>) {
        if self.0.is_empty() {
            self.0 = rows;
        } else {
            self.0.extend(rows);
        }
    }

    fn len(&self) -> usize {
        self.0.len()
    }
}

/// Runs a compiled INSERT, UPDATE or DELETE (inside a transaction).
pub(crate) fn run_compiled_write(db: &mut Database, c: &Compiled, params: &[Value]) -> Result<QueryResult> {
    match c {
        Compiled::Insert(p) => insert(db, p, params),
        Compiled::Update(p) => update(db, p, params),
        Compiled::Delete(p) => delete(db, p, params),
        Compiled::Select(_) => unreachable!("SELECT does not write"),
    }
}

/// Read-only statements without a compiled form.
pub(crate) fn run_read(db: &Database, stmt: &Statement, params: &[Value]) -> Result<QueryResult> {
    match stmt {
        Statement::Explain(inner) => explain(db, inner, params),
        Statement::ShowTables => Ok(show_tables(db)),
        Statement::Describe(name) => describe_table(db, name),
        _ => unreachable!("not a read-only statement without a compiled form"),
    }
}

/// Schema statements (inside a transaction).
pub(crate) fn run_write(db: &mut Database, stmt: &Statement) -> Result<QueryResult> {
    match stmt {
        Statement::CreateTable(ct) => create_table(db, ct),
        Statement::DropTable { name, if_exists } => drop_table(db, name, *if_exists),
        Statement::CreateIndex { name, table, column, if_not_exists } => {
            create_index(db, name, table, column, *if_not_exists)
        }
        Statement::DropIndex { name, if_exists } => drop_index(db, name, *if_exists),
        _ => unreachable!("not a schema statement"),
    }
}

fn table_of<'a>(db: &'a Database, key: &str) -> Result<&'a Table> {
    db.tables.get(key).ok_or_else(|| Error::Schema(format!("no such table '{key}'")))
}

// ----------------------------------------------------------------------
// SELECT
// ----------------------------------------------------------------------

/// A compiled SELECT.
#[derive(Debug, Clone)]
pub(crate) struct SelectPlan {
    /// Catalog key of the table (`None` without FROM).
    table: Option<Arc<str>>,
    /// Conjuncts of the WHERE clause, and each of them compiled.
    filter: Vec<BExpr>,
    preds: Vec<Pred>,
    access: Option<AccessPlan>,
    aggregate: bool,
    aggs: Vec<AggSpec>,
    proj: Vec<BExpr>,
    /// Column names of the result, shared by every result of the statement.
    names: Arc<[String]>,
    group: Vec<BExpr>,
    having: Option<BExpr>,
    order: Vec<(BExpr, bool)>,
    /// Every aggregate is `COUNT(*)` of one group and nothing else refers to
    /// a column: when the access path guarantees the WHERE clause, the rows
    /// are counted, not read.
    count_shape: bool,
    /// `ORDER BY` is the primary key alone (`Some(desc)`), so a path in key
    /// order needs no sort.
    pk_order: Option<bool>,
    distinct: bool,
    limit: Option<BExpr>,
    offset: Option<BExpr>,
}

/// A compiled SELECT with the parameter values of one execution.
struct Run<'a> {
    p: &'a SelectPlan,
    table: Option<&'a Table>,
    params: &'a [Value],
    access: Access,
    /// Conjuncts the access path guarantees (bit `i`: conjunct `i`); they are
    /// not evaluated again.
    skip: u64,
    count_only: bool,
    /// Rows come from the access path in the requested order already.
    presorted: bool,
    reverse: bool,
    limit: Option<usize>,
    offset: usize,
}

fn const_usize(e: &Option<BExpr>, params: &[Value], clause: &str) -> Result<Option<usize>> {
    let Some(e) = e else { return Ok(None) };
    match eval(e, &[], Cx::params(params))?.as_ref() {
        Value::Int(n) if *n >= 0 => Ok(Some(*n as usize)),
        Value::Null => Ok(None),
        other => Err(Error::Type(format!("{clause} must be a non-negative integer, got {}", other.to_sql()))),
    }
}

/// Whether any node of `e` satisfies `leaf` (checked on every node).
fn any_node(e: &BExpr, leaf: &impl Fn(&BExpr) -> bool) -> bool {
    leaf(e)
        || match e {
            BExpr::Agg(_) | BExpr::Lit(_) | BExpr::Col(_) | BExpr::Param(_) => false,
            BExpr::Neg(x) | BExpr::Not(x) | BExpr::IsNull(x, _) => any_node(x, leaf),
            BExpr::Bin(_, l, r) => any_node(l, leaf) || any_node(r, leaf),
            BExpr::InList(x, l, _) => any_node(x, leaf) || l.iter().any(|i| any_node(i, leaf)),
            BExpr::Between(x, l, h, _) => any_node(x, leaf) || any_node(l, leaf) || any_node(h, leaf),
            BExpr::Like { expr, pattern, .. } => any_node(expr, leaf) || any_node(pattern, leaf),
            BExpr::Func(_, args) => args.iter().any(|a| any_node(a, leaf)),
        }
}

fn contains_agg(e: &BExpr) -> bool {
    any_node(e, &|n| matches!(n, BExpr::Agg(_)))
}

fn uses_columns(e: &BExpr) -> bool {
    any_node(e, &|n| matches!(n, BExpr::Col(_)))
}

fn compile_select(db: &Database, s: &Select) -> Result<SelectPlan> {
    let table = match &s.from {
        Some(name) => Some(db.table(name)?),
        None => None,
    };
    let schema = table.map(|t| &*t.schema);
    let binder = Binder::deferred(schema);

    let filter = match &s.selection {
        Some(e) => binder.bind(e, "WHERE")?.into_conjuncts(),
        None => Vec::new(),
    };
    let no_columns = TableSchema { name: String::new(), columns: Vec::new(), pk: 0 };
    let preds = filter.iter().map(|e| compile::predicate(e, schema.unwrap_or(&no_columns))).collect();

    let aggregate = !s.group_by.is_empty()
        || s.having.is_some()
        || s.projection.iter().any(|i| matches!(i, SelectItem::Expr { expr, .. } if expr.contains_aggregate()));

    let mut aggs = Vec::new();
    let mut proj = Vec::new();
    let mut names = Vec::new();
    for item in &s.projection {
        match item {
            SelectItem::Wildcard => {
                let schema = schema.ok_or_else(|| Error::Schema("SELECT * requires a FROM clause".into()))?;
                for (i, c) in schema.columns.iter().enumerate() {
                    proj.push(BExpr::Col(i));
                    names.push(c.name.clone());
                }
            }
            SelectItem::Expr { expr, alias } => {
                let b = if aggregate { binder.bind_agg(expr, &mut aggs)? } else { binder.bind(expr, "SELECT")? };
                proj.push(b);
                names.push(match (alias, expr) {
                    (Some(a), _) => a.clone(),
                    (None, Expr::Column(c)) => schema
                        .and_then(|s| s.column_index(c).map(|i| s.columns[i].name.clone()))
                        .unwrap_or_else(|| c.clone()),
                    (None, e) => e.to_string(),
                });
            }
        }
    }
    let alias_of = |name: &str| names.iter().position(|n| n.eq_ignore_ascii_case(name));

    let mut group = Vec::new();
    for e in &s.group_by {
        let b = match e {
            Expr::Literal(Value::Int(n)) => position(&proj, *n, "GROUP BY")?,
            Expr::Column(c) if schema.and_then(|s| s.column_index(c)).is_none() && alias_of(c).is_some() => {
                proj[alias_of(c).unwrap()].clone()
            }
            e => binder.bind(e, "GROUP BY")?,
        };
        if contains_agg(&b) {
            return Err(Error::Schema("aggregate functions are not allowed in GROUP BY".into()));
        }
        group.push(b);
    }

    let having = match &s.having {
        Some(e) => Some(binder.bind_agg(e, &mut aggs)?),
        None => None,
    };

    let mut order = Vec::new();
    for item in &s.order_by {
        let b = match &item.expr {
            Expr::Literal(Value::Int(n)) => position(&proj, *n, "ORDER BY")?,
            Expr::Column(c) if alias_of(c).is_some() => proj[alias_of(c).unwrap()].clone(),
            e if aggregate => binder.bind_agg(e, &mut aggs)?,
            e => binder.bind(e, "ORDER BY")?,
        };
        order.push((b, item.desc));
    }

    let access = match table {
        Some(t) => Some(AccessPlan::new(t, &filter)?),
        None => None,
    };
    // SELECT COUNT(*) FROM t WHERE <guaranteed by the access path>: nothing
    // but the number of rows is needed, and an index stores it.
    let count_shape = aggregate
        && group.is_empty()
        && aggs.iter().all(|a| a.func == AggFunc::CountStar)
        && !proj.iter().chain(having.iter()).chain(order.iter().map(|(e, _)| e)).any(uses_columns);
    let pk_order = match (table, order.as_slice()) {
        (Some(t), [(BExpr::Col(c), desc)]) if !aggregate && !s.distinct && *c == t.schema.pk => Some(*desc),
        _ => None,
    };
    let limits = Binder::deferred(None);
    let limit = s.limit.as_ref().map(|e| limits.bind(e, "LIMIT")).transpose()?;
    let offset = s.offset.as_ref().map(|e| limits.bind(e, "OFFSET")).transpose()?;

    Ok(SelectPlan {
        table: table.map(|t| t.key.clone()),
        filter,
        preds,
        access,
        aggregate,
        aggs,
        proj,
        names: names.into(),
        group,
        having,
        order,
        count_shape,
        pk_order,
        distinct: s.distinct,
        limit,
        offset,
    })
}

/// Fills a compiled SELECT with the parameter values of one execution.
fn bind_select<'a>(db: &'a Database, p: &'a SelectPlan, params: &'a [Value]) -> Result<Run<'a>> {
    let table = p.table.as_deref().map(|k| table_of(db, k)).transpose()?;
    let (access, skip) = match (table, &p.access) {
        (Some(t), Some(a)) => a.resolve(t, &p.filter, params)?,
        _ => (Access::FullScan, 0),
    };
    let count_only = p.count_shape && table.is_some() && plan::covers_all(skip, p.filter.len());
    let (presorted, reverse) = match p.pk_order {
        _ if p.order.is_empty() => (true, false),
        Some(desc) if access.pk_ordered() => (true, desc),
        _ => (false, false),
    };
    Ok(Run {
        p,
        table,
        params,
        access,
        skip,
        count_only,
        presorted,
        reverse,
        limit: const_usize(&p.limit, params, "LIMIT")?,
        offset: const_usize(&p.offset, params, "OFFSET")?.unwrap_or(0),
    })
}

fn position(proj: &[BExpr], n: i64, clause: &str) -> Result<BExpr> {
    if n < 1 || n as usize > proj.len() {
        return Err(Error::Schema(format!("{clause} position {n} is out of range")));
    }
    Ok(proj[n as usize - 1].clone())
}

fn project(proj: &[BExpr], row: &[Value], cx: Cx) -> Result<Vec<Value>> {
    proj.iter().map(|e| eval(e, row, cx).map(Cow::into_owned)).collect()
}

fn cmp_keys<A: Borrow<Value>, B: Borrow<Value>>(a: &[A], b: &[B], dirs: &[(BExpr, bool)]) -> Ordering {
    for ((x, y), (_, desc)) in a.iter().zip(b).zip(dirs) {
        let o = x.borrow().cmp(y.borrow());
        let o = if *desc { o.reverse() } else { o };
        if o != Ordering::Equal {
            return o;
        }
    }
    Ordering::Equal
}

/// Sorts `items` by key and keeps only what OFFSET/LIMIT need. With a small
/// LIMIT a partial selection (O(n)) is done before sorting the survivors.
fn sort_window<T>(items: &mut Vec<(Vec<Value>, usize, T)>, order: &[(BExpr, bool)], need: Option<usize>) {
    let cmp = |a: &(Vec<Value>, usize, T), b: &(Vec<Value>, usize, T)| cmp_keys(&a.0, &b.0, order).then(a.1.cmp(&b.1));
    if let Some(k) = need {
        if k == 0 {
            items.clear();
            return;
        }
        if k < items.len() {
            items.select_nth_unstable_by(k - 1, cmp);
            items.truncate(k);
        }
    }
    items.sort_unstable_by(cmp);
}

/// Group key -> (first row of the group, aggregate accumulators).
type Groups<'a> = BTreeMap<Vec<Cow<'a, Value>>, (&'a [Value], Vec<AggAcc>)>;

/// Largest LIMIT + OFFSET handled with the bounded top-k buffer.
const TOP_K_BUFFER: usize = 1024;

fn window<T>(rows: Vec<T>, offset: usize, limit: Option<usize>) -> Vec<T> {
    rows.into_iter().skip(offset).take(limit.unwrap_or(usize::MAX)).collect()
}

/// The conjuncts of a WHERE clause the access path does not guarantee.
struct Residual<'a> {
    preds: &'a [Pred],
    skip: u64,
    params: &'a [Value],
    /// Some conjunct has to be evaluated. Without one every row qualifies and
    /// the check is one predictable branch per row, not a call.
    any: bool,
}

impl<'a> Residual<'a> {
    fn new(preds: &'a [Pred], skip: u64, params: &'a [Value]) -> Self {
        Residual { preds, skip, params, any: !plan::covers_all(skip, preds.len()) }
    }

    #[inline(always)]
    fn matches(&self, row: &[Value]) -> Result<bool> {
        if !self.any {
            return Ok(true);
        }
        self.test(row)
    }

    /// The conjuncts in order, like `AND`: the first FALSE decides, and a
    /// NULL conjunct makes the row not qualify.
    fn test(&self, row: &[Value]) -> Result<bool> {
        let mut all = true;
        for (i, pred) in self.preds.iter().enumerate() {
            if i < 64 && self.skip >> i & 1 == 1 {
                continue;
            }
            match pred.test(row, self.params)? {
                Some(true) => {}
                Some(false) => return Ok(false),
                None => all = false,
            }
        }
        Ok(all)
    }
}

fn execute_select(r: &Run, sink: &mut dyn RowSink) -> Result<u64> {
    let p = r.p;
    let empty_row: Vec<Value> = Vec::new();
    let filter = Residual::new(&p.preds, r.skip, r.params);
    let cx = Cx::params(r.params);
    let mut scanned: u64 = 0;
    let need = r.limit.map(|l| l.saturating_add(r.offset));

    if p.aggregate {
        let null_row = vec![Value::Null; r.table.map_or(0, |t| t.schema.columns.len())];
        // Group keys borrow column values from the table where possible; a key
        // is only copied when a new group is created.
        let mut groups: Groups = BTreeMap::new();
        let mut key: Vec<Cow<Value>> = Vec::with_capacity(p.group.len());
        match r.table {
            Some(t) if r.count_only => {
                let n = plan::count(t, &r.access) as i64;
                groups.insert(Vec::new(), (&null_row[..], p.aggs.iter().map(|_| AggAcc::counted(n)).collect()));
            }
            // One column as the group key: a row finds its group through a
            // hash table of the column's values instead of comparisons in the
            // tree of groups. The groups go into the tree at the end, so the
            // output keeps the order of the keys.
            Some(t) if matches!(p.group.as_slice(), [BExpr::Col(_)]) => {
                let [BExpr::Col(c)] = p.group.as_slice() else { unreachable!() };
                let mut slots: HashMap<ValueKey, usize, FxBuildHasher> = HashMap::default();
                let mut found: Vec<(&Value, &[Value], Vec<AggAcc>)> = Vec::new();
                for entry in plan::scan(t, &r.access, false) {
                    scanned += 1;
                    if !filter.matches(&entry.values)? {
                        continue;
                    }
                    let value = &entry.values[*c];
                    let i = match slots.get(&ValueKey(value)) {
                        Some(&i) => i,
                        None => {
                            slots.insert(ValueKey(value), found.len());
                            found.push((value, &entry.values[..], p.aggs.iter().map(AggAcc::new).collect()));
                            found.len() - 1
                        }
                    };
                    for (acc, spec) in found[i].2.iter_mut().zip(&p.aggs) {
                        acc.update(spec, &entry.values, r.params)?;
                    }
                }
                for (value, row, accs) in found {
                    groups.insert(vec![Cow::Borrowed(value)], (row, accs));
                }
            }
            Some(t) => {
                for entry in plan::scan(t, &r.access, false) {
                    scanned += 1;
                    if !filter.matches(&entry.values)? {
                        continue;
                    }
                    key.clear();
                    for g in &p.group {
                        key.push(eval(g, &entry.values, cx)?);
                    }
                    if let Some(slot) = groups.get_mut(key.as_slice()) {
                        for (acc, spec) in slot.1.iter_mut().zip(&p.aggs) {
                            acc.update(spec, &entry.values, r.params)?;
                        }
                    } else {
                        let mut accs: Vec<AggAcc> = p.aggs.iter().map(AggAcc::new).collect();
                        for (acc, spec) in accs.iter_mut().zip(&p.aggs) {
                            acc.update(spec, &entry.values, r.params)?;
                        }
                        groups.insert(key.clone(), (&entry.values[..], accs));
                    }
                }
            }
            None => {
                if filter.matches(&empty_row)? {
                    groups.insert(Vec::new(), (&empty_row[..], p.aggs.iter().map(AggAcc::new).collect()));
                }
            }
        }
        if groups.is_empty() && p.group.is_empty() {
            groups.insert(Vec::new(), (&null_row[..], p.aggs.iter().map(AggAcc::new).collect()));
        }
        let mut out: Vec<(Vec<Value>, usize, Vec<Value>)> = Vec::with_capacity(groups.len());
        let mut seen = BTreeSet::new();
        for (i, (_, (row, accs))) in groups.into_iter().enumerate() {
            let agg_values: Vec<Value> = accs.iter().zip(&p.aggs).map(|(a, s)| a.finish(s)).collect();
            let gcx = Cx { aggs: &agg_values, params: r.params };
            if let Some(h) = &p.having {
                if eval_pred(h, row, gcx)? != Some(true) {
                    continue;
                }
            }
            let values = project(&p.proj, row, gcx)?;
            if p.distinct && !seen.insert(values.clone()) {
                continue;
            }
            let keys =
                p.order.iter().map(|(e, _)| eval(e, row, gcx).map(Cow::into_owned)).collect::<Result<Vec<_>>>()?;
            out.push((keys, i, values));
        }
        if !p.order.is_empty() {
            sort_window(&mut out, &p.order, need);
        }
        sink.computed(window(out.into_iter().map(|(_, _, v)| v).collect(), r.offset, r.limit));
    } else if r.presorted && !p.distinct {
        // Streaming: stop as soon as LIMIT is satisfied.
        let mut skipped = 0;
        if r.limit != Some(0) {
            match r.table {
                Some(t) => {
                    for entry in plan::scan(t, &r.access, r.reverse) {
                        scanned += 1;
                        if !filter.matches(&entry.values)? {
                            continue;
                        }
                        if skipped < r.offset {
                            skipped += 1;
                            continue;
                        }
                        sink.project(&p.proj, &entry.values, cx)?;
                        if Some(sink.len()) == r.limit {
                            break;
                        }
                    }
                }
                None => {
                    if filter.matches(&empty_row)? && r.offset == 0 {
                        sink.project(&p.proj, &empty_row, cx)?;
                    }
                }
            }
        }
    } else if !p.distinct {
        // Sort candidate rows by reference, project only the final window.
        let mut cands: Vec<(Vec<Value>, usize, &[Value])> = Vec::new();
        let rows: Box<dyn Iterator<Item = &[Value]>> = match r.table {
            Some(t) => Box::new(plan::scan(t, &r.access, false).map(|e| &e.values[..])),
            None => Box::new(std::iter::once(&empty_row[..])),
        };
        match need {
            // ORDER BY ... LIMIT k: keep a sorted buffer of the best k rows. Most
            // rows lose against the current k-th one and are rejected without
            // copying their sort key.
            Some(k) if k <= TOP_K_BUFFER => {
                let mut key: Vec<Cow<Value>> = Vec::with_capacity(p.order.len());
                for (i, row) in rows.enumerate() {
                    scanned += 1;
                    if k == 0 {
                        break;
                    }
                    if !filter.matches(row)? {
                        continue;
                    }
                    key.clear();
                    for (e, _) in &p.order {
                        key.push(eval(e, row, cx)?);
                    }
                    // Equal keys keep scan order, so a tie with the k-th row loses.
                    if cands.len() == k && cmp_keys(&key, &cands[k - 1].0, &p.order) != Ordering::Less {
                        continue;
                    }
                    let pos = cands.partition_point(|c| cmp_keys(&c.0, &key, &p.order) != Ordering::Greater);
                    cands.insert(pos, (key.iter().map(|v| v.clone().into_owned()).collect(), i, row));
                    cands.truncate(k);
                }
            }
            _ => {
                for (i, row) in rows.enumerate() {
                    scanned += 1;
                    if !filter.matches(row)? {
                        continue;
                    }
                    let keys = p
                        .order
                        .iter()
                        .map(|(e, _)| eval(e, row, cx).map(Cow::into_owned))
                        .collect::<Result<Vec<_>>>()?;
                    cands.push((keys, i, row));
                }
                sort_window(&mut cands, &p.order, need);
            }
        }
        if r.table.is_none() {
            scanned = 0;
        }
        for (_, _, row) in window(cands, r.offset, r.limit) {
            sink.project(&p.proj, row, cx)?;
        }
    } else {
        // DISTINCT: project first, de-duplicate, then sort.
        let mut seen = BTreeSet::new();
        let mut out: Vec<(Vec<Value>, usize, Vec<Value>)> = Vec::new();
        let rows: Box<dyn Iterator<Item = &[Value]>> = match r.table {
            Some(t) => Box::new(plan::scan(t, &r.access, false).map(|e| &e.values[..])),
            None => Box::new(std::iter::once(&empty_row[..])),
        };
        for (i, row) in rows.enumerate() {
            scanned += 1;
            if !filter.matches(row)? {
                continue;
            }
            let values = project(&p.proj, row, cx)?;
            if !seen.insert(values.clone()) {
                continue;
            }
            let keys =
                p.order.iter().map(|(e, _)| eval(e, row, cx).map(Cow::into_owned)).collect::<Result<Vec<_>>>()?;
            out.push((keys, i, values));
        }
        if !p.order.is_empty() {
            sort_window(&mut out, &p.order, need);
        }
        sink.computed(window(out.into_iter().map(|(_, _, v)| v).collect(), r.offset, r.limit));
    }
    Ok(if r.table.is_some() { scanned } else { 0 })
}

fn explain_select(r: &Run) -> Vec<String> {
    let p = r.p;
    let schema = r.table.map(|t| &*t.schema);
    let d = |e: &BExpr| describe(e, schema, &p.aggs);
    let mut lines = Vec::new();
    match r.table {
        Some(t) => {
            lines.push(format!("SELECT FROM {}", t.name()));
            lines.push(format!("  ACCESS:    {}", r.access.describe(t)));
        }
        None => lines.push("SELECT (no table)".into()),
    }
    if let Some(f) = residual_text(&p.filter, r.skip, schema) {
        lines.push(format!("  FILTER:    {f}"));
    }
    if p.aggregate {
        let groups = if p.group.is_empty() {
            "single group".to_string()
        } else {
            format!("GROUP BY {}", p.group.iter().map(d).collect::<Vec<_>>().join(", "))
        };
        let aggs = p.aggs.iter().map(|a| a.label.clone()).collect::<Vec<_>>().join(", ");
        let counted = if r.count_only { " (rows counted by the access path, not read)" } else { "" };
        lines.push(format!("  AGGREGATE: {groups}; {aggs}{counted}"));
    }
    if let Some(h) = &p.having {
        lines.push(format!("  HAVING:    {}", d(h)));
    }
    if p.distinct {
        lines.push("  DISTINCT".into());
    }
    if !p.order.is_empty() {
        let keys = p
            .order
            .iter()
            .map(|(e, desc)| format!("{}{}", d(e), if *desc { " DESC" } else { "" }))
            .collect::<Vec<_>>()
            .join(", ");
        if r.presorted {
            lines.push(format!("  ORDER:     {keys} (primary key order, no sort)"));
        } else if let Some(l) = r.limit {
            lines.push(format!("  SORT:      {keys} (top-{} selection)", l + r.offset));
        } else {
            lines.push(format!("  SORT:      {keys}"));
        }
    }
    if r.limit.is_some() || r.offset > 0 {
        let limit = r.limit.map_or("ALL".to_string(), |l| l.to_string());
        lines.push(format!("  LIMIT:     {limit} OFFSET {}", r.offset));
    }
    lines.push(format!("  OUTPUT:    {}", p.names.join(", ")));
    lines
}

/// EXPLAIN text of the conjuncts the access path does not guarantee.
fn residual_text(terms: &[BExpr], skip: u64, schema: Option<&TableSchema>) -> Option<String> {
    let rest: Vec<&BExpr> =
        terms.iter().enumerate().filter(|(i, _)| *i >= 64 || skip >> i & 1 == 0).map(|(_, t)| t).collect();
    let text = |t: &BExpr| describe(t, schema, &[]);
    match rest.as_slice() {
        [] => None,
        [one] => Some(text(one)),
        many => Some(
            many.iter()
                .map(|t| match t {
                    BExpr::Bin(..)
                    | BExpr::Between(..)
                    | BExpr::InList(..)
                    | BExpr::Like { .. }
                    | BExpr::IsNull(..) => {
                        format!("({})", text(t))
                    }
                    _ => text(t),
                })
                .collect::<Vec<_>>()
                .join(" AND "),
        ),
    }
}

fn explain(db: &Database, stmt: &Statement, params: &[Value]) -> Result<QueryResult> {
    let lines = match compile(db, stmt)? {
        Some(Compiled::Select(p)) => explain_select(&bind_select(db, &p, params)?),
        Some(Compiled::Update(UpdatePlan { table, filter, access, .. }))
        | Some(Compiled::Delete(DeletePlan { table, filter, access, .. })) => {
            let t = table_of(db, &table)?;
            let (access, skip) = access.resolve(t, &filter, params)?;
            let verb = if matches!(stmt, Statement::Update(_)) { "UPDATE" } else { "DELETE FROM" };
            let mut lines = vec![format!("{verb} {}", t.name()), format!("  ACCESS:    {}", access.describe(t))];
            if let Some(f) = residual_text(&filter, skip, Some(&t.schema)) {
                lines.push(format!("  FILTER:    {f}"));
            }
            lines
        }
        _ => return Err(Error::Unsupported("EXPLAIN supports SELECT, UPDATE and DELETE".into())),
    };
    Ok(QueryResult {
        columns: vec!["plan".to_string()].into(),
        rows: lines.into_iter().map(|l| vec![Value::Text(l)]).collect(),
        message: "EXPLAIN".into(),
        ..Default::default()
    })
}

fn show_tables(db: &Database) -> QueryResult {
    let rows = db
        .tables
        .values()
        .map(|t| {
            vec![
                Value::Text(t.schema.name.clone()),
                Value::Int(t.rows.len() as i64),
                Value::Int(t.schema.columns.len() as i64),
                Value::Int(t.indexes.len() as i64),
                Value::Text(t.schema.pk_column().name.clone()),
            ]
        })
        .collect::<Vec<_>>();
    let n = rows.len();
    QueryResult {
        columns: ["table", "rows", "columns", "indexes", "primary_key"].map(String::from).to_vec().into(),
        rows,
        message: format!("SHOW TABLES {n}"),
        ..Default::default()
    }
}

fn describe_table(db: &Database, name: &str) -> Result<QueryResult> {
    let t = db.table(name)?;
    let rows = t
        .schema
        .columns
        .iter()
        .enumerate()
        .map(|(i, c)| {
            let key = if i == t.schema.pk {
                "PRIMARY KEY".to_string()
            } else {
                t.indexes
                    .iter()
                    .filter(|x| x.column == i)
                    .map(|x| format!("INDEX {}", x.name))
                    .collect::<Vec<_>>()
                    .join(", ")
            };
            vec![
                Value::Text(c.name.clone()),
                Value::Text(c.data_type.name().into()),
                Value::Bool(!c.not_null),
                Value::Text(key),
            ]
        })
        .collect::<Vec<_>>();
    let n = rows.len();
    Ok(QueryResult {
        columns: ["column", "type", "nullable", "key"].map(String::from).to_vec().into(),
        rows,
        message: format!("DESCRIBE {n}"),
        ..Default::default()
    })
}

// ----------------------------------------------------------------------
// DDL
// ----------------------------------------------------------------------

fn create_table(db: &mut Database, ct: &CreateTable) -> Result<QueryResult> {
    if db.tables.contains_key(&*catalog_key(&ct.name)) {
        if ct.if_not_exists {
            return Ok(QueryResult::command("CREATE TABLE (already exists)"));
        }
        return Err(Error::Schema(format!("table '{}' already exists", ct.name)));
    }
    let mut columns: Vec<Column> = Vec::with_capacity(ct.columns.len());
    for c in &ct.columns {
        if columns.iter().any(|x| x.name.eq_ignore_ascii_case(&c.name)) {
            return Err(Error::Schema(format!("duplicate column '{}'", c.name)));
        }
        columns.push(Column { name: c.name.clone(), data_type: c.data_type, not_null: c.not_null });
    }
    let inline: Vec<usize> = ct.columns.iter().enumerate().filter(|(_, c)| c.primary_key).map(|(i, _)| i).collect();
    let pk = match (inline.as_slice(), &ct.primary_key) {
        ([i], None) => *i,
        ([], Some(name)) => columns
            .iter()
            .position(|c| c.name.eq_ignore_ascii_case(name))
            .ok_or_else(|| Error::Schema(format!("PRIMARY KEY refers to unknown column '{name}'")))?,
        ([], None) => {
            return Err(Error::Schema(format!(
                "table '{}' must declare a PRIMARY KEY (Flat-SQL rows are addressed by key)",
                ct.name
            )))
        }
        _ => return Err(Error::Schema("a table can have only one PRIMARY KEY".into())),
    };
    columns[pk].not_null = true;
    let schema = TableSchema { name: ct.name.clone(), columns, pk };
    let stamp = db.new_stamp();
    db.write_schema(Change { stamp, mutation: Mutation::CreateTable(schema) }, true)?;
    Ok(QueryResult::command("CREATE TABLE"))
}

fn drop_table(db: &mut Database, name: &str, if_exists: bool) -> Result<QueryResult> {
    let real = match db.tables.get(&*catalog_key(name)) {
        Some(t) => t.schema.name.clone(),
        None if if_exists => return Ok(QueryResult::command("DROP TABLE (did not exist)")),
        None => return Err(Error::Schema(format!("no such table '{name}'"))),
    };
    let stamp = db.new_stamp();
    db.write_schema(Change { stamp, mutation: Mutation::DropTable(real) }, true)?;
    Ok(QueryResult::command("DROP TABLE"))
}

fn create_index(db: &mut Database, name: &str, table: &str, column: &str, if_not_exists: bool) -> Result<QueryResult> {
    if db.find_index(name).is_some() {
        if if_not_exists {
            return Ok(QueryResult::command("CREATE INDEX (already exists)"));
        }
        return Err(Error::Schema(format!("index '{name}' already exists")));
    }
    let t = db.table(table)?;
    let col = t.schema.require_column(column)?;
    if col == t.schema.pk {
        return Err(Error::Schema(format!("column '{column}' is the primary key and is already indexed")));
    }
    let mutation = Mutation::CreateIndex {
        table: t.schema.name.clone(),
        name: name.to_string(),
        column: t.schema.columns[col].name.clone(),
    };
    let stamp = db.new_stamp();
    db.write_schema(Change { stamp, mutation }, true)?;
    Ok(QueryResult::command("CREATE INDEX"))
}

fn drop_index(db: &mut Database, name: &str, if_exists: bool) -> Result<QueryResult> {
    let Some((tkey, pos)) = db.find_index(name) else {
        if if_exists {
            return Ok(QueryResult::command("DROP INDEX (did not exist)"));
        }
        return Err(Error::Schema(format!("no such index '{name}'")));
    };
    let t = &db.tables[&*tkey];
    let mutation = Mutation::DropIndex { table: t.schema.name.clone(), name: t.indexes[pos].name.clone() };
    let stamp = db.new_stamp();
    db.write_schema(Change { stamp, mutation }, true)?;
    Ok(QueryResult::command("DROP INDEX"))
}

// ----------------------------------------------------------------------
// DML
// ----------------------------------------------------------------------

/// Value of one column of an inserted row.
#[derive(Debug, Clone)]
enum Source {
    Lit(Value),
    Param(usize),
    /// Expression without column references, evaluated for every execution.
    Expr(BExpr),
}

/// A compiled INSERT: per row, the column positions and their values.
#[derive(Debug, Clone)]
pub(crate) struct InsertPlan {
    table: Arc<str>,
    schema: Arc<TableSchema>,
    rows: Vec<Vec<(usize, Source)>>,
    on_conflict: OnConflict,
}

fn compile_insert(db: &Database, ins: &Insert) -> Result<InsertPlan> {
    let t = db.table(&ins.table)?;
    let schema = t.schema.clone();
    let positions: Vec<usize> = match &ins.columns {
        Some(cols) => {
            let mut out = Vec::with_capacity(cols.len());
            for c in cols {
                let i = schema.require_column(c)?;
                if out.contains(&i) {
                    return Err(Error::Schema(format!("column '{c}' specified more than once")));
                }
                out.push(i);
            }
            out
        }
        None => (0..schema.columns.len()).collect(),
    };
    let binder = Binder::deferred(None);
    let mut rows = Vec::with_capacity(ins.rows.len());
    for row in &ins.rows {
        if row.len() != positions.len() {
            return Err(Error::Schema(format!(
                "INSERT into '{}' expects {} value(s) per row, got {}",
                schema.name,
                positions.len(),
                row.len()
            )));
        }
        let mut sources = Vec::with_capacity(row.len());
        for (e, &pos) in row.iter().zip(&positions) {
            let source = match e {
                Expr::Literal(v) => Source::Lit(v.clone()),
                Expr::Param(i) => Source::Param(*i),
                e => Source::Expr(binder.bind(e, "VALUES").map_err(|err| match err {
                    Error::Schema(m) if m.contains("without FROM") => {
                        Error::Schema("column references are not allowed in VALUES".into())
                    }
                    other => other,
                })?),
            };
            sources.push((pos, source));
        }
        rows.push(sources);
    }
    Ok(InsertPlan { table: t.key.clone(), schema, rows, on_conflict: ins.on_conflict })
}

fn insert(db: &mut Database, p: &InsertPlan, params: &[Value]) -> Result<QueryResult> {
    let schema = &p.schema;
    let mut affected = 0;
    let now = db.wall_ms();
    for row in &p.rows {
        let mut values = vec![Value::Null; schema.columns.len()];
        for (pos, source) in row {
            values[*pos] = match source {
                Source::Lit(v) => v.clone(),
                Source::Param(i) => params[*i].clone(),
                Source::Expr(b) => eval(b, &[], Cx::params(params))?.into_owned(),
            };
        }
        let values = schema.check_row(values)?;
        if table_of(db, &p.table)?.rows.contains_key(&values[schema.pk]) {
            match p.on_conflict {
                OnConflict::Abort => {
                    return Err(Error::Constraint(format!(
                        "duplicate primary key {} in table '{}'",
                        values[schema.pk].to_sql(),
                        schema.name
                    )))
                }
                OnConflict::Ignore => continue,
                OnConflict::Replace => {}
            }
        }
        let stamp = db.new_stamp_at(now);
        db.write_put(&p.table, values, stamp, true)?;
        affected += 1;
    }
    Ok(QueryResult::with_affected("INSERT", affected, 0))
}

/// A compiled UPDATE.
#[derive(Debug, Clone)]
pub(crate) struct UpdatePlan {
    table: Arc<str>,
    schema: Arc<TableSchema>,
    assigns: Vec<(usize, ValueExpr)>,
    filter: Vec<BExpr>,
    preds: Vec<Pred>,
    access: AccessPlan,
}

fn compile_update(db: &Database, u: &Update) -> Result<UpdatePlan> {
    let t = db.table(&u.table)?;
    let schema = &t.schema;
    let binder = Binder::deferred(Some(schema));
    let mut assigns: Vec<(usize, ValueExpr)> = Vec::with_capacity(u.assignments.len());
    for (col, e) in &u.assignments {
        let i = schema.require_column(col)?;
        if assigns.iter().any(|(j, _)| *j == i) {
            return Err(Error::Schema(format!("column '{col}' assigned more than once")));
        }
        assigns.push((i, compile::value(&binder.bind(e, "SET")?, schema)));
    }
    let filter = match &u.selection {
        Some(e) => binder.bind(e, "WHERE")?.into_conjuncts(),
        None => Vec::new(),
    };
    let preds = filter.iter().map(|e| compile::predicate(e, schema)).collect();
    let access = AccessPlan::new(t, &filter)?;
    Ok(UpdatePlan { table: t.key.clone(), schema: schema.clone(), assigns, filter, preds, access })
}

fn update(db: &mut Database, p: &UpdatePlan, params: &[Value]) -> Result<QueryResult> {
    if p.assigns.iter().any(|(c, _)| *c == p.schema.pk) {
        return update_key(db, p, params);
    }
    let t = table_of(db, &p.table)?;
    let schema = &p.schema;
    let (access, skip) = p.access.resolve(t, &p.filter, params)?;
    let residual = Residual::new(&p.preds, skip, params);
    let pk = schema.pk;
    // Only the assigned columns of each row: evaluated in the order of SET,
    // validated in the order of the columns (as a whole row would be).
    let mut targets = Vec::new();
    let mut scanned = 0;
    for entry in plan::scan(t, &access, false) {
        scanned += 1;
        if !residual.matches(&entry.values)? {
            continue;
        }
        let mut changes = Vec::with_capacity(p.assigns.len());
        for (c, e) in &p.assigns {
            changes.push((*c, e.eval(&entry.values, params)?));
        }
        changes.sort_unstable_by_key(|(c, _)| *c);
        for (c, v) in &mut changes {
            *v = schema.check_value(*c, std::mem::replace(v, Value::Null))?;
        }
        targets.push((entry.values[pk].clone(), changes));
    }
    let affected = targets.len() as u64;
    let now = db.wall_ms();
    for (key, changes) in targets {
        let stamp = db.new_stamp_at(now);
        db.write_update(&p.table, key, changes, stamp)?;
    }
    Ok(QueryResult::with_affected("UPDATE", affected, scanned))
}

/// UPDATE that assigns the primary key: a row whose key changes is deleted
/// and written again under the new key.
fn update_key(db: &mut Database, p: &UpdatePlan, params: &[Value]) -> Result<QueryResult> {
    let t = table_of(db, &p.table)?;
    let schema = &p.schema;
    let (access, skip) = p.access.resolve(t, &p.filter, params)?;
    let residual = Residual::new(&p.preds, skip, params);
    let pk = schema.pk;
    let mut targets = Vec::new();
    let mut scanned = 0;
    for entry in plan::scan(t, &access, false) {
        scanned += 1;
        if !residual.matches(&entry.values)? {
            continue;
        }
        let mut new = entry.values.clone();
        for (c, e) in &p.assigns {
            new[*c] = e.eval(&entry.values, params)?;
        }
        targets.push((entry.values[pk].clone(), schema.check_row(new)?));
    }
    let affected = targets.len() as u64;
    let now = db.wall_ms();
    for (old_key, new) in targets {
        if new[pk] != old_key {
            if table_of(db, &p.table)?.rows.contains_key(&new[pk]) {
                return Err(Error::Constraint(format!(
                    "duplicate primary key {} in table '{}'",
                    new[pk].to_sql(),
                    schema.name
                )));
            }
            let stamp = db.new_stamp_at(now);
            db.write_delete(&p.table, old_key, stamp, true)?;
        }
        let stamp = db.new_stamp_at(now);
        db.write_put(&p.table, new, stamp, true)?;
    }
    Ok(QueryResult::with_affected("UPDATE", affected, scanned))
}

/// A compiled DELETE.
#[derive(Debug, Clone)]
pub(crate) struct DeletePlan {
    table: Arc<str>,
    filter: Vec<BExpr>,
    preds: Vec<Pred>,
    access: AccessPlan,
}

fn compile_delete(db: &Database, d: &Delete) -> Result<DeletePlan> {
    let t = db.table(&d.table)?;
    let filter = match &d.selection {
        Some(e) => Binder::deferred(Some(&t.schema)).bind(e, "WHERE")?.into_conjuncts(),
        None => Vec::new(),
    };
    let preds = filter.iter().map(|e| compile::predicate(e, &t.schema)).collect();
    let access = AccessPlan::new(t, &filter)?;
    Ok(DeletePlan { table: t.key.clone(), filter, preds, access })
}

fn delete(db: &mut Database, p: &DeletePlan, params: &[Value]) -> Result<QueryResult> {
    let t = table_of(db, &p.table)?;
    let (access, skip) = p.access.resolve(t, &p.filter, params)?;
    let residual = Residual::new(&p.preds, skip, params);
    let pk = t.schema.pk;
    let mut keys = Vec::new();
    let mut scanned = 0;
    for entry in plan::scan(t, &access, false) {
        scanned += 1;
        if residual.matches(&entry.values)? {
            keys.push(entry.values[pk].clone());
        }
    }
    let affected = keys.len() as u64;
    let now = db.wall_ms();
    for key in keys {
        let stamp = db.new_stamp_at(now);
        db.write_delete(&p.table, key, stamp, true)?;
    }
    Ok(QueryResult::with_affected("DELETE", affected, scanned))
}
