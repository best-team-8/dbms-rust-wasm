//! Statement execution.

use std::borrow::{Borrow, Cow};
use std::cmp::Ordering;
use std::collections::{BTreeMap, BTreeSet};

use crate::db::{catalog_key, Database};
use crate::error::{Error, Result};
use crate::expr::{describe, eval, eval_pred, matches, AggAcc, AggFunc, AggSpec, BExpr, Binder};
use crate::plan::{self, Access};
use crate::result::QueryResult;
use crate::sql::{CreateTable, Delete, Expr, Insert, OnConflict, Select, SelectItem, Statement, Update};
use crate::sync::{Change, Mutation};
use crate::table::{Column, Table, TableSchema};
use crate::value::Value;

pub(crate) fn run_read(db: &Database, stmt: &Statement, params: &[Value]) -> Result<QueryResult> {
    match stmt {
        Statement::Select(s) => execute_select(&plan_select(db, s, params)?),
        Statement::Explain(inner) => explain(db, inner, params),
        Statement::ShowTables => Ok(show_tables(db)),
        Statement::Describe(name) => describe_table(db, name),
        _ => unreachable!("not a read-only statement"),
    }
}

pub(crate) fn run_write(db: &mut Database, stmt: &Statement, params: &[Value]) -> Result<QueryResult> {
    match stmt {
        Statement::CreateTable(ct) => create_table(db, ct),
        Statement::DropTable { name, if_exists } => drop_table(db, name, *if_exists),
        Statement::CreateIndex { name, table, column, if_not_exists } => {
            create_index(db, name, table, column, *if_not_exists)
        }
        Statement::DropIndex { name, if_exists } => drop_index(db, name, *if_exists),
        Statement::Insert(i) => insert(db, i, params),
        Statement::Update(u) => update(db, u, params),
        Statement::Delete(d) => delete(db, d, params),
        _ => unreachable!("not a write statement"),
    }
}

// ----------------------------------------------------------------------
// SELECT
// ----------------------------------------------------------------------

struct SelectPlan<'a> {
    table: Option<&'a Table>,
    filter: Option<BExpr>,
    access: Access,
    aggregate: bool,
    aggs: Vec<AggSpec>,
    proj: Vec<BExpr>,
    names: Vec<String>,
    group: Vec<BExpr>,
    having: Option<BExpr>,
    order: Vec<(BExpr, bool)>,
    /// Every aggregate is `COUNT(*)` of one group and the access path
    /// guarantees the WHERE clause, so the rows are counted, not read.
    count_only: bool,
    /// Rows come from the access path in the requested order already.
    presorted: bool,
    reverse: bool,
    distinct: bool,
    limit: Option<usize>,
    offset: usize,
}

fn const_usize(e: &Option<Expr>, params: &[Value], clause: &str) -> Result<Option<usize>> {
    let Some(e) = e else { return Ok(None) };
    let b = Binder::new(None, params).bind(e, clause)?;
    match eval(&b, &[], &[])?.as_ref() {
        Value::Int(n) if *n >= 0 => Ok(Some(*n as usize)),
        Value::Null => Ok(None),
        other => Err(Error::Type(format!("{clause} must be a non-negative integer, got {}", other.to_sql()))),
    }
}

/// Whether any node of `e` satisfies `leaf` (checked on every node).
fn any_node(e: &BExpr, leaf: &impl Fn(&BExpr) -> bool) -> bool {
    leaf(e)
        || match e {
            BExpr::Agg(_) | BExpr::Lit(_) | BExpr::Col(_) => false,
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

fn plan_select<'a>(db: &'a Database, s: &Select, params: &[Value]) -> Result<SelectPlan<'a>> {
    let table = match &s.from {
        Some(name) => Some(db.table(name)?),
        None => None,
    };
    let schema = table.map(|t| &*t.schema);
    let binder = Binder::new(schema, params);

    let filter = match &s.selection {
        Some(e) => Some(binder.bind(e, "WHERE")?),
        None => None,
    };

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

    let (access, covered) = match table {
        Some(t) => plan::choose_covered(t, filter.as_ref())?,
        None => (Access::FullScan, false),
    };
    // SELECT COUNT(*) FROM t WHERE <guaranteed by the access path>: nothing
    // but the number of rows is needed, and an index stores it.
    let count_only = aggregate
        && covered
        && group.is_empty()
        && aggs.iter().all(|a| a.func == AggFunc::CountStar)
        && !proj.iter().chain(having.iter()).chain(order.iter().map(|(e, _)| e)).any(uses_columns);

    let mut presorted = order.is_empty();
    let mut reverse = false;
    if let (Some(t), [(BExpr::Col(c), desc)]) = (table, order.as_slice()) {
        if !aggregate && !s.distinct && *c == t.schema.pk && access.pk_ordered() {
            presorted = true;
            reverse = *desc;
        }
    }

    Ok(SelectPlan {
        table,
        filter,
        access,
        aggregate,
        aggs,
        proj,
        names,
        group,
        having,
        order,
        count_only,
        presorted,
        reverse,
        distinct: s.distinct,
        limit: const_usize(&s.limit, params, "LIMIT")?,
        offset: const_usize(&s.offset, params, "OFFSET")?.unwrap_or(0),
    })
}

fn position(proj: &[BExpr], n: i64, clause: &str) -> Result<BExpr> {
    if n < 1 || n as usize > proj.len() {
        return Err(Error::Schema(format!("{clause} position {n} is out of range")));
    }
    Ok(proj[n as usize - 1].clone())
}

fn project(proj: &[BExpr], row: &[Value], aggs: &[Value]) -> Result<Vec<Value>> {
    proj.iter().map(|e| eval(e, row, aggs).map(Cow::into_owned)).collect()
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

fn execute_select(p: &SelectPlan) -> Result<QueryResult> {
    let empty_row: Vec<Value> = Vec::new();
    let mut scanned: u64 = 0;
    let need = p.limit.map(|l| l.saturating_add(p.offset));

    let rows: Vec<Vec<Value>> = if p.aggregate {
        let null_row = vec![Value::Null; p.table.map_or(0, |t| t.schema.columns.len())];
        // Group keys borrow column values from the table where possible; a key
        // is only copied when a new group is created.
        let mut groups: Groups = BTreeMap::new();
        let mut key: Vec<Cow<Value>> = Vec::with_capacity(p.group.len());
        match p.table {
            Some(t) if p.count_only => {
                let n = plan::count(t, &p.access) as i64;
                groups.insert(Vec::new(), (&null_row[..], p.aggs.iter().map(|_| AggAcc::counted(n)).collect()));
            }
            Some(t) => {
                for entry in plan::scan(t, &p.access, false) {
                    scanned += 1;
                    if !matches(p.filter.as_ref(), &entry.values)? {
                        continue;
                    }
                    key.clear();
                    for g in &p.group {
                        key.push(eval(g, &entry.values, &[])?);
                    }
                    if let Some(slot) = groups.get_mut(key.as_slice()) {
                        for (acc, spec) in slot.1.iter_mut().zip(&p.aggs) {
                            acc.update(spec, &entry.values)?;
                        }
                    } else {
                        let mut accs: Vec<AggAcc> = p.aggs.iter().map(AggAcc::new).collect();
                        for (acc, spec) in accs.iter_mut().zip(&p.aggs) {
                            acc.update(spec, &entry.values)?;
                        }
                        groups.insert(key.clone(), (&entry.values[..], accs));
                    }
                }
            }
            None => {
                if matches(p.filter.as_ref(), &empty_row)? {
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
            if let Some(h) = &p.having {
                if eval_pred(h, row, &agg_values)? != Some(true) {
                    continue;
                }
            }
            let values = project(&p.proj, row, &agg_values)?;
            if p.distinct && !seen.insert(values.clone()) {
                continue;
            }
            let keys = p
                .order
                .iter()
                .map(|(e, _)| eval(e, row, &agg_values).map(Cow::into_owned))
                .collect::<Result<Vec<_>>>()?;
            out.push((keys, i, values));
        }
        if !p.order.is_empty() {
            sort_window(&mut out, &p.order, need);
        }
        window(out.into_iter().map(|(_, _, v)| v).collect(), p.offset, p.limit)
    } else if p.presorted && !p.distinct {
        // Streaming: stop as soon as LIMIT is satisfied.
        let mut out = Vec::new();
        let mut skipped = 0;
        if p.limit != Some(0) {
            match p.table {
                Some(t) => {
                    for entry in plan::scan(t, &p.access, p.reverse) {
                        scanned += 1;
                        if !matches(p.filter.as_ref(), &entry.values)? {
                            continue;
                        }
                        if skipped < p.offset {
                            skipped += 1;
                            continue;
                        }
                        out.push(project(&p.proj, &entry.values, &[])?);
                        if Some(out.len()) == p.limit {
                            break;
                        }
                    }
                }
                None => {
                    if matches(p.filter.as_ref(), &empty_row)? && p.offset == 0 {
                        out.push(project(&p.proj, &empty_row, &[])?);
                    }
                }
            }
        }
        out
    } else if !p.distinct {
        // Sort candidate rows by reference, project only the final window.
        let mut cands: Vec<(Vec<Value>, usize, &[Value])> = Vec::new();
        let rows: Box<dyn Iterator<Item = &[Value]>> = match p.table {
            Some(t) => Box::new(plan::scan(t, &p.access, false).map(|e| &e.values[..])),
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
                    if !matches(p.filter.as_ref(), row)? {
                        continue;
                    }
                    key.clear();
                    for (e, _) in &p.order {
                        key.push(eval(e, row, &[])?);
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
                    if !matches(p.filter.as_ref(), row)? {
                        continue;
                    }
                    let keys = p
                        .order
                        .iter()
                        .map(|(e, _)| eval(e, row, &[]).map(Cow::into_owned))
                        .collect::<Result<Vec<_>>>()?;
                    cands.push((keys, i, row));
                }
                sort_window(&mut cands, &p.order, need);
            }
        }
        if p.table.is_none() {
            scanned = 0;
        }
        window(cands, p.offset, p.limit)
            .into_iter()
            .map(|(_, _, row)| project(&p.proj, row, &[]))
            .collect::<Result<Vec<_>>>()?
    } else {
        // DISTINCT: project first, de-duplicate, then sort.
        let mut seen = BTreeSet::new();
        let mut out: Vec<(Vec<Value>, usize, Vec<Value>)> = Vec::new();
        let rows: Box<dyn Iterator<Item = &[Value]>> = match p.table {
            Some(t) => Box::new(plan::scan(t, &p.access, false).map(|e| &e.values[..])),
            None => Box::new(std::iter::once(&empty_row[..])),
        };
        for (i, row) in rows.enumerate() {
            scanned += 1;
            if !matches(p.filter.as_ref(), row)? {
                continue;
            }
            let values = project(&p.proj, row, &[])?;
            if !seen.insert(values.clone()) {
                continue;
            }
            let keys =
                p.order.iter().map(|(e, _)| eval(e, row, &[]).map(Cow::into_owned)).collect::<Result<Vec<_>>>()?;
            out.push((keys, i, values));
        }
        if !p.order.is_empty() {
            sort_window(&mut out, &p.order, need);
        }
        window(out.into_iter().map(|(_, _, v)| v).collect(), p.offset, p.limit)
    };

    let n = rows.len();
    Ok(QueryResult {
        columns: p.names.clone(),
        rows,
        rows_affected: 0,
        rows_scanned: if p.table.is_some() { scanned } else { 0 },
        message: format!("SELECT {n}"),
    })
}

fn explain_select(p: &SelectPlan) -> Vec<String> {
    let schema = p.table.map(|t| &*t.schema);
    let d = |e: &BExpr| describe(e, schema, &p.aggs);
    let mut lines = Vec::new();
    match p.table {
        Some(t) => {
            lines.push(format!("SELECT FROM {}", t.name()));
            lines.push(format!("  ACCESS:    {}", p.access.describe(t)));
        }
        None => lines.push("SELECT (no table)".into()),
    }
    if let Some(f) = &p.filter {
        lines.push(format!("  FILTER:    {}", d(f)));
    }
    if p.aggregate {
        let groups = if p.group.is_empty() {
            "single group".to_string()
        } else {
            format!("GROUP BY {}", p.group.iter().map(d).collect::<Vec<_>>().join(", "))
        };
        let aggs = p.aggs.iter().map(|a| a.label.clone()).collect::<Vec<_>>().join(", ");
        let counted = if p.count_only { " (rows counted by the access path, not read)" } else { "" };
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
        if p.presorted {
            lines.push(format!("  ORDER:     {keys} (primary key order, no sort)"));
        } else if let Some(l) = p.limit {
            lines.push(format!("  SORT:      {keys} (top-{} selection)", l + p.offset));
        } else {
            lines.push(format!("  SORT:      {keys}"));
        }
    }
    if p.limit.is_some() || p.offset > 0 {
        let limit = p.limit.map_or("ALL".to_string(), |l| l.to_string());
        lines.push(format!("  LIMIT:     {limit} OFFSET {}", p.offset));
    }
    lines.push(format!("  OUTPUT:    {}", p.names.join(", ")));
    lines
}

fn explain(db: &Database, stmt: &Statement, params: &[Value]) -> Result<QueryResult> {
    let lines = match stmt {
        Statement::Select(s) => explain_select(&plan_select(db, s, params)?),
        Statement::Update(Update { table, selection, .. }) | Statement::Delete(Delete { table, selection }) => {
            let t = db.table(table)?;
            let filter = match selection {
                Some(e) => Some(Binder::new(Some(&t.schema), params).bind(e, "WHERE")?),
                None => None,
            };
            let access = plan::choose(t, filter.as_ref())?;
            let verb = if matches!(stmt, Statement::Update(_)) { "UPDATE" } else { "DELETE FROM" };
            let mut lines = vec![format!("{verb} {}", t.name()), format!("  ACCESS:    {}", access.describe(t))];
            if let Some(f) = &filter {
                lines.push(format!("  FILTER:    {}", plan::describe_filter(Some(t), f)));
            }
            lines
        }
        _ => return Err(Error::Unsupported("EXPLAIN supports SELECT, UPDATE and DELETE".into())),
    };
    Ok(QueryResult {
        columns: vec!["plan".into()],
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
        columns: ["table", "rows", "columns", "indexes", "primary_key"].map(String::from).to_vec(),
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
        columns: ["column", "type", "nullable", "key"].map(String::from).to_vec(),
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

fn insert(db: &mut Database, ins: &Insert, params: &[Value]) -> Result<QueryResult> {
    let tkey = catalog_key(&ins.table);
    let schema = db.table(&ins.table)?.schema.clone();
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
    let binder = Binder::new(None, params);
    let mut affected = 0;
    for row in &ins.rows {
        if row.len() != positions.len() {
            return Err(Error::Schema(format!(
                "INSERT into '{}' expects {} value(s) per row, got {}",
                schema.name,
                positions.len(),
                row.len()
            )));
        }
        let mut values = vec![Value::Null; schema.columns.len()];
        for (e, &pos) in row.iter().zip(&positions) {
            values[pos] = match e {
                Expr::Literal(v) => v.clone(),
                Expr::Param(i) => params
                    .get(*i)
                    .cloned()
                    .ok_or_else(|| Error::Eval(format!("missing value for parameter ?{}", i + 1)))?,
                e => {
                    let b = binder.bind(e, "VALUES").map_err(|err| match err {
                        Error::Schema(m) if m.contains("without FROM") => {
                            Error::Schema("column references are not allowed in VALUES".into())
                        }
                        other => other,
                    })?;
                    eval(&b, &[], &[])?.into_owned()
                }
            };
        }
        let values = schema.check_row(values)?;
        if db.tables[&*tkey].rows.contains_key(&values[schema.pk]) {
            match ins.on_conflict {
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
        let stamp = db.new_stamp();
        db.write_put(&tkey, values, stamp, true)?;
        affected += 1;
    }
    Ok(QueryResult::with_affected("INSERT", affected, 0))
}

fn update(db: &mut Database, u: &Update, params: &[Value]) -> Result<QueryResult> {
    let tkey = catalog_key(&u.table);
    let t = db.table(&u.table)?;
    let schema = &t.schema;
    let binder = Binder::new(Some(schema), params);
    let mut assigns: Vec<(usize, BExpr)> = Vec::with_capacity(u.assignments.len());
    for (col, e) in &u.assignments {
        let i = schema.require_column(col)?;
        if assigns.iter().any(|(j, _)| *j == i) {
            return Err(Error::Schema(format!("column '{col}' assigned more than once")));
        }
        assigns.push((i, binder.bind(e, "SET")?));
    }
    let filter = match &u.selection {
        Some(e) => Some(binder.bind(e, "WHERE")?),
        None => None,
    };
    let access = plan::choose(t, filter.as_ref())?;
    let pk = schema.pk;
    let mut targets = Vec::new();
    let mut scanned = 0;
    for entry in plan::scan(t, &access, false) {
        scanned += 1;
        if !matches(filter.as_ref(), &entry.values)? {
            continue;
        }
        let mut new = entry.values.clone();
        for (c, e) in &assigns {
            new[*c] = eval(e, &entry.values, &[])?.into_owned();
        }
        targets.push((entry.values[pk].clone(), schema.check_row(new)?));
    }
    let table_name = schema.name.clone();
    let affected = targets.len() as u64;
    for (old_key, new) in targets {
        if new[pk] != old_key {
            if db.tables[&*tkey].rows.contains_key(&new[pk]) {
                return Err(Error::Constraint(format!(
                    "duplicate primary key {} in table '{}'",
                    new[pk].to_sql(),
                    table_name
                )));
            }
            let stamp = db.new_stamp();
            db.write_delete(&tkey, old_key, stamp, true)?;
        }
        let stamp = db.new_stamp();
        db.write_put(&tkey, new, stamp, true)?;
    }
    Ok(QueryResult::with_affected("UPDATE", affected, scanned))
}

fn delete(db: &mut Database, d: &Delete, params: &[Value]) -> Result<QueryResult> {
    let tkey = catalog_key(&d.table);
    let t = db.table(&d.table)?;
    let filter = match &d.selection {
        Some(e) => Some(Binder::new(Some(&t.schema), params).bind(e, "WHERE")?),
        None => None,
    };
    let access = plan::choose(t, filter.as_ref())?;
    let pk = t.schema.pk;
    let mut keys = Vec::new();
    let mut scanned = 0;
    for entry in plan::scan(t, &access, false) {
        scanned += 1;
        if matches(filter.as_ref(), &entry.values)? {
            keys.push(entry.values[pk].clone());
        }
    }
    let affected = keys.len() as u64;
    for key in keys {
        let stamp = db.new_stamp();
        db.write_delete(&tkey, key, stamp, true)?;
    }
    Ok(QueryResult::with_affected("DELETE", affected, scanned))
}
