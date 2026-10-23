//! In-memory table storage: rows keyed by primary key, secondary indexes and
//! tombstones used by the Local-First synchronisation.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::sync::Arc;

use crate::error::{Error, Result};
use crate::sync::Stamp;
use crate::value::{DataType, Value};

#[derive(Debug, Clone, PartialEq)]
pub struct Column {
    pub name: String,
    pub data_type: DataType,
    pub not_null: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TableSchema {
    pub name: String,
    pub columns: Vec<Column>,
    /// Index of the primary key column.
    pub pk: usize,
}

impl TableSchema {
    /// Case-insensitive column lookup.
    pub fn column_index(&self, name: &str) -> Option<usize> {
        self.columns.iter().position(|c| c.name.eq_ignore_ascii_case(name))
    }

    pub fn require_column(&self, name: &str) -> Result<usize> {
        self.column_index(name).ok_or_else(|| Error::Schema(format!("no column '{}' in table '{}'", name, self.name)))
    }

    pub fn pk_column(&self) -> &Column {
        &self.columns[self.pk]
    }

    /// Validates and coerces a full row before it is stored.
    pub fn check_row(&self, values: Vec<Value>) -> Result<Vec<Value>> {
        if values.len() != self.columns.len() {
            return Err(Error::Schema(format!(
                "table '{}' has {} columns but {} values were supplied",
                self.name,
                self.columns.len(),
                values.len()
            )));
        }
        let mut out = Vec::with_capacity(values.len());
        for (v, col) in values.into_iter().zip(&self.columns) {
            let v = v.coerce(col.data_type).map_err(|e| match e {
                Error::Type(m) => Error::Type(format!("column '{}': {m}", col.name)),
                other => other,
            })?;
            if v.is_null() && col.not_null {
                return Err(Error::Constraint(format!("NOT NULL constraint failed: {}.{}", self.name, col.name)));
            }
            out.push(v);
        }
        Ok(out)
    }
}

/// `name (column TYPE [PRIMARY KEY | NOT NULL], ...)`
impl fmt::Display for TableSchema {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} (", self.name)?;
        for (i, c) in self.columns.iter().enumerate() {
            if i > 0 {
                f.write_str(", ")?;
            }
            write!(f, "{} {}", c.name, c.data_type)?;
            if i == self.pk {
                f.write_str(" PRIMARY KEY")?;
            } else if c.not_null {
                f.write_str(" NOT NULL")?;
            }
        }
        f.write_str(")")
    }
}

impl TableSchema {
    /// How another definition of the same table relates to this one: `None`
    /// when they are the same, `Some(true)` when one only adds nullable
    /// columns at the end of the other (the difference allowed between
    /// application versions), `Some(false)` otherwise.
    pub fn compare(&self, other: &TableSchema) -> Option<bool> {
        let same = |a: &Column, b: &Column| {
            a.name.eq_ignore_ascii_case(&b.name) && a.data_type == b.data_type && a.not_null == b.not_null
        };
        let n = self.columns.len().min(other.columns.len());
        let prefix = self.pk == other.pk && self.columns.iter().zip(&other.columns).all(|(a, b)| same(a, b));
        if prefix && self.columns.len() == other.columns.len() {
            return None;
        }
        let longer = if self.columns.len() > n { self } else { other };
        Some(prefix && longer.columns[n..].iter().all(|c| !c.not_null))
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RowEntry {
    pub values: Vec<Value>,
    /// Hybrid logical clock stamp of the last write (last-writer-wins).
    pub stamp: Stamp,
}

/// Secondary index: column value -> set of primary keys.
#[derive(Debug, Clone, PartialEq)]
pub struct Index {
    pub name: String,
    pub column: usize,
    pub map: BTreeMap<Value, BTreeSet<Value>>,
}

impl Index {
    fn insert(&mut self, value: &Value, key: &Value) {
        // Look up first so an existing value does not get cloned again.
        match self.map.get_mut(value) {
            Some(keys) => {
                keys.insert(key.clone());
            }
            None => {
                self.map.insert(value.clone(), BTreeSet::from([key.clone()]));
            }
        }
    }

    fn remove(&mut self, value: &Value, key: &Value) {
        if let Some(set) = self.map.get_mut(value) {
            set.remove(key);
            if set.is_empty() {
                self.map.remove(value);
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Table {
    /// Shared so statements can hold the schema while mutating the table.
    pub schema: Arc<TableSchema>,
    /// Catalog key (lower-case name), shared by undo and outbox entries.
    pub(crate) key: Arc<str>,
    pub rows: BTreeMap<Value, RowEntry>,
    pub indexes: Vec<Index>,
    /// Keys deleted so far, with the stamp of the delete. Needed so a stale
    /// remote write cannot resurrect a row that was deleted later.
    pub tombstones: BTreeMap<Value, Stamp>,
    /// Stamp of the CREATE TABLE that produced this table.
    pub created: Stamp,
}

impl Table {
    pub fn new(schema: TableSchema, created: Stamp) -> Self {
        let key = Arc::from(schema.name.to_lowercase());
        Table {
            schema: Arc::new(schema),
            key,
            rows: BTreeMap::new(),
            indexes: Vec::new(),
            tombstones: BTreeMap::new(),
            created,
        }
    }

    pub fn name(&self) -> &str {
        &self.schema.name
    }

    pub fn index_on(&self, column: usize) -> Option<usize> {
        self.indexes.iter().position(|i| i.column == column)
    }

    pub fn index_named(&self, name: &str) -> Option<usize> {
        self.indexes.iter().position(|i| i.name.eq_ignore_ascii_case(name))
    }

    /// Inserts or replaces the row; returns the previous row and tombstone.
    pub fn put(&mut self, values: Vec<Value>, stamp: Stamp) -> (Option<RowEntry>, Option<Stamp>) {
        let key = values[self.schema.pk].clone();
        if let Some(old) = self.rows.get(&key) {
            for idx in &mut self.indexes {
                idx.remove(&old.values[idx.column], &key);
            }
        }
        for idx in &mut self.indexes {
            idx.insert(&values[idx.column], &key);
        }
        let tomb = self.tombstones.remove(&key);
        let old = self.rows.insert(key, RowEntry { values, stamp });
        (old, tomb)
    }

    /// Removes the row (if any) and records a tombstone. Returns the previous
    /// row and the previous tombstone.
    pub fn delete(&mut self, key: &Value, stamp: Stamp) -> (Option<RowEntry>, Option<Stamp>) {
        let old = self.remove_row(key);
        let tomb = self.tombstones.insert(key.clone(), stamp);
        (old, tomb)
    }

    /// Removes a row without touching tombstones (used by rollback).
    pub fn remove_row(&mut self, key: &Value) -> Option<RowEntry> {
        let old = self.rows.remove(key)?;
        for idx in &mut self.indexes {
            idx.remove(&old.values[idx.column], key);
        }
        Some(old)
    }

    /// Re-inserts a row exactly as it was (used by rollback).
    pub fn restore_row(&mut self, entry: RowEntry) {
        let key = entry.values[self.schema.pk].clone();
        for idx in &mut self.indexes {
            idx.insert(&entry.values[idx.column], &key);
        }
        self.rows.insert(key, entry);
    }

    pub fn set_tombstone(&mut self, key: &Value, tomb: Option<Stamp>) {
        match tomb {
            Some(s) => {
                self.tombstones.insert(key.clone(), s);
            }
            None => {
                self.tombstones.remove(key);
            }
        }
    }

    /// Builds a secondary index over the existing rows.
    pub fn add_index(&mut self, name: String, column: usize) {
        let mut idx = Index { name, column, map: BTreeMap::new() };
        for (key, row) in &self.rows {
            idx.insert(&row.values[column], key);
        }
        self.indexes.push(idx);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn table() -> Table {
        let schema = TableSchema {
            name: "t".into(),
            columns: vec![
                Column { name: "id".into(), data_type: DataType::Int, not_null: true },
                Column { name: "tag".into(), data_type: DataType::Text, not_null: false },
            ],
            pk: 0,
        };
        Table::new(schema, Stamp::default())
    }

    fn row(id: i64, tag: &str) -> Vec<Value> {
        vec![Value::Int(id), Value::Text(tag.into())]
    }

    #[test]
    fn index_follows_puts_and_deletes() {
        let mut t = table();
        t.add_index("ix".into(), 1);
        t.put(row(1, "a"), Stamp::default());
        t.put(row(2, "a"), Stamp::default());
        t.put(row(1, "b"), Stamp::default());
        let ix = &t.indexes[0];
        assert_eq!(ix.map[&Value::from("a")].len(), 1);
        assert_eq!(ix.map[&Value::from("b")].len(), 1);
        t.delete(&Value::Int(2), Stamp::default());
        assert!(!t.indexes[0].map.contains_key(&Value::from("a")));
        assert!(t.tombstones.contains_key(&Value::Int(2)));
    }

    #[test]
    fn index_built_over_existing_rows() {
        let mut t = table();
        for i in 0..10 {
            t.put(row(i, if i % 2 == 0 { "even" } else { "odd" }), Stamp::default());
        }
        t.add_index("ix".into(), 1);
        assert_eq!(t.indexes[0].map[&Value::from("even")].len(), 5);
    }

    #[test]
    fn definitions_compare_as_same_additive_or_incompatible() {
        let base = table().schema;
        let with = |columns: Vec<Column>, pk| TableSchema { name: "T".into(), columns, pk };
        let col = |name: &str, data_type, not_null| Column { name: name.into(), data_type, not_null };
        let id = col("ID", DataType::Int, true);
        let tag = col("tag", DataType::Text, false);
        assert_eq!(base.compare(&with(vec![id.clone(), tag.clone()], 0)), None);
        let added = with(vec![id.clone(), tag.clone(), col("note", DataType::Text, false)], 0);
        assert_eq!(base.compare(&added), Some(true));
        assert_eq!(added.compare(&base), Some(true));
        let required = with(vec![id.clone(), tag.clone(), col("note", DataType::Text, true)], 0);
        assert_eq!(base.compare(&required), Some(false), "a NOT NULL column cannot be missing in old rows");
        assert_eq!(base.compare(&with(vec![id.clone(), col("tag", DataType::Int, false)], 0)), Some(false));
        assert_eq!(
            base.compare(&with(vec![col("id", DataType::Int, false), col("tag", DataType::Text, true)], 1)),
            Some(false)
        );
        assert_eq!(base.to_string(), "t (id INT PRIMARY KEY, tag TEXT)");
    }

    #[test]
    fn check_row_coerces_and_validates() {
        let t = table();
        assert!(t.schema.check_row(vec![Value::Float(1.0), Value::Null]).is_ok());
        assert!(matches!(t.schema.check_row(vec![Value::Null, Value::Null]), Err(Error::Constraint(_))));
        assert!(matches!(t.schema.check_row(vec![Value::from("x"), Value::Null]), Err(Error::Type(_))));
        assert!(t.schema.check_row(vec![Value::Int(1)]).is_err());
    }
}
