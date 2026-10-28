use std::borrow::Cow;
use std::sync::Arc;

use crate::error::Result;
use crate::exec::RowSink;
use crate::expr::{eval, BExpr, Cx};
use crate::value::{write_json_string, Value};

/// Result of a single statement.
#[derive(Debug, Clone, PartialEq)]
pub struct QueryResult {
    /// Column names (empty for statements that return no rows); shared by
    /// every result of one prepared statement.
    pub columns: Arc<[String]>,
    pub rows: Vec<Vec<Value>>,
    /// Rows inserted / updated / deleted.
    pub rows_affected: u64,
    /// Rows visited by the access path (shows whether an index was used).
    pub rows_scanned: u64,
    /// Command tag of statements that change data or the schema, e.g.
    /// `INSERT 2`, `CREATE TABLE`; empty for SELECT, whose result is the rows.
    pub message: String,
}

impl Default for QueryResult {
    fn default() -> Self {
        QueryResult {
            columns: Arc::from(Vec::new()),
            rows: Vec::new(),
            rows_affected: 0,
            rows_scanned: 0,
            message: String::new(),
        }
    }
}

impl QueryResult {
    pub fn command(message: impl Into<String>) -> Self {
        QueryResult { message: message.into(), ..Default::default() }
    }

    pub fn with_affected(message: &str, affected: u64, scanned: u64) -> Self {
        QueryResult {
            message: format!("{message} {affected}"),
            rows_affected: affected,
            rows_scanned: scanned,
            ..Default::default()
        }
    }

    pub fn to_json(&self) -> String {
        let mut s = String::with_capacity(64 + self.rows.len() * 16 * self.columns.len().max(1));
        s.push_str("{\"columns\":[");
        for (i, c) in self.columns.iter().enumerate() {
            if i > 0 {
                s.push(',');
            }
            write_json_string(c, &mut s);
        }
        s.push_str("],\"rows\":[");
        for (i, row) in self.rows.iter().enumerate() {
            if i > 0 {
                s.push(',');
            }
            s.push('[');
            for (j, v) in row.iter().enumerate() {
                if j > 0 {
                    s.push(',');
                }
                v.write_json(&mut s);
            }
            s.push(']');
        }
        s.push_str("],\"rowsAffected\":");
        s.push_str(&self.rows_affected.to_string());
        s.push_str(",\"rowsScanned\":");
        s.push_str(&self.rows_scanned.to_string());
        s.push_str(",\"message\":");
        write_json_string(&self.message, &mut s);
        s.push('}');
        s
    }

    /// Plain text table used by the CLI.
    pub fn to_table_string(&self) -> String {
        if self.columns.is_empty() {
            return self.message.clone();
        }
        let cells: Vec<Vec<String>> = self.rows.iter().map(|r| r.iter().map(|v| v.to_string()).collect()).collect();
        let mut widths: Vec<usize> = self.columns.iter().map(|c| c.chars().count()).collect();
        for row in &cells {
            for (w, c) in widths.iter_mut().zip(row) {
                *w = (*w).max(c.chars().count());
            }
        }
        let line = |out: &mut String| {
            out.push('+');
            for w in &widths {
                out.push_str(&"-".repeat(w + 2));
                out.push('+');
            }
            out.push('\n');
        };
        let row_line = |out: &mut String, vals: &[String], numeric: &[bool]| {
            out.push('|');
            for (i, (v, w)) in vals.iter().zip(&widths).enumerate() {
                let pad = w - v.chars().count();
                if numeric.get(i).copied().unwrap_or(false) {
                    out.push_str(&format!(" {}{} |", " ".repeat(pad), v));
                } else {
                    out.push_str(&format!(" {}{} |", v, " ".repeat(pad)));
                }
            }
            out.push('\n');
        };
        let numeric: Vec<bool> = (0..self.columns.len())
            .map(|i| {
                self.rows.iter().any(|r| r[i].is_numeric())
                    && self.rows.iter().all(|r| r[i].is_numeric() || r[i].is_null())
            })
            .collect();
        let mut out = String::new();
        line(&mut out);
        row_line(&mut out, &self.columns, &[]);
        line(&mut out);
        for row in &cells {
            row_line(&mut out, row, &numeric);
        }
        line(&mut out);
        let n = self.rows.len();
        out.push_str(&format!("({} row{})", n, if n == 1 { "" } else { "s" }));
        out
    }
}

/// Writes a result in the binary result format, read by the WebAssembly
/// binding's JavaScript in one pass (little-endian):
///
/// ```text
///  0  u32  rows
///  4  u32  values per row
///  8  f64  rows affected
/// 16  f64  rows scanned
/// 24  u32  offset of the text area
/// 28  u32  length of the message in UTF-16 units
/// 32  values, row after row: tag [payload]
///       0 NULL, 1 FALSE, 2 TRUE,
///       3 number: f64 (INT as well, as JavaScript numbers are),
///       4 text: u32 length in UTF-16 units; the characters are in the text area
/// text area: UTF-8 of every text value in order, then the message
/// ```
///
/// All texts are decoded at once and cut by their UTF-16 lengths, so a result
/// needs one call to the text decoder instead of one per value.
pub struct ResultEncoder<'a> {
    out: &'a mut Vec<u8>,
    text: Vec<u8>,
    rows: usize,
}

const HEADER: usize = 32;

fn utf16_len(s: &str) -> u32 {
    if s.is_ascii() {
        s.len() as u32
    } else {
        s.encode_utf16().count() as u32
    }
}

impl<'a> ResultEncoder<'a> {
    pub fn new(out: &'a mut Vec<u8>, columns: usize) -> Self {
        out.clear();
        out.resize(HEADER, 0);
        out[4..8].copy_from_slice(&(columns as u32).to_le_bytes());
        ResultEncoder { out, text: Vec::new(), rows: 0 }
    }

    fn value(&mut self, v: &Value) {
        match v {
            Value::Null => self.out.push(0),
            Value::Bool(b) => self.out.push(1 + *b as u8),
            Value::Int(i) => {
                self.out.push(3);
                self.out.extend_from_slice(&(*i as f64).to_le_bytes());
            }
            Value::Float(f) => {
                self.out.push(3);
                self.out.extend_from_slice(&f.to_le_bytes());
            }
            Value::Text(s) => {
                self.out.push(4);
                self.out.extend_from_slice(&utf16_len(s).to_le_bytes());
                self.text.extend_from_slice(s.as_bytes());
            }
        }
    }

    fn row(&mut self, values: &[Value]) {
        for v in values {
            self.value(v);
        }
        self.rows += 1;
    }

    /// Completes the header and appends the text area.
    pub fn finish(self, rows_affected: u64, rows_scanned: u64, message: &str) {
        let ResultEncoder { out, mut text, rows } = self;
        let text_at = out.len() as u32;
        text.extend_from_slice(message.as_bytes());
        out.extend_from_slice(&text);
        out[0..4].copy_from_slice(&(rows as u32).to_le_bytes());
        out[8..16].copy_from_slice(&(rows_affected as f64).to_le_bytes());
        out[16..24].copy_from_slice(&(rows_scanned as f64).to_le_bytes());
        out[24..28].copy_from_slice(&text_at.to_le_bytes());
        out[28..32].copy_from_slice(&utf16_len(message).to_le_bytes());
    }
}

impl RowSink for ResultEncoder<'_> {
    fn project(&mut self, proj: &[BExpr], row: &[Value], cx: Cx) -> Result<()> {
        for e in proj {
            let v: Cow<Value> = eval(e, row, cx)?;
            self.value(&v);
        }
        self.rows += 1;
        Ok(())
    }

    fn computed(&mut self, rows: Vec<Vec<Value>>) {
        for row in &rows {
            self.row(row);
        }
    }

    fn len(&self) -> usize {
        self.rows
    }
}

impl QueryResult {
    /// The result in the binary result format (see [`ResultEncoder`]).
    pub fn encode(&self, out: &mut Vec<u8>) {
        let mut e = ResultEncoder::new(out, self.columns.len());
        for row in &self.rows {
            e.row(row);
        }
        e.finish(self.rows_affected, self.rows_scanned, &self.message);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn json_shape() {
        let r = QueryResult {
            columns: vec!["id".to_string(), "name".to_string()].into(),
            rows: vec![vec![Value::Int(1), Value::Text("a\"b".into())], vec![Value::Int(2), Value::Null]],
            rows_affected: 0,
            rows_scanned: 2,
            message: "SELECT 2".into(),
        };
        assert_eq!(
            r.to_json(),
            r#"{"columns":["id","name"],"rows":[[1,"a\"b"],[2,null]],"rowsAffected":0,"rowsScanned":2,"message":"SELECT 2"}"#
        );
    }

    #[test]
    fn table_rendering() {
        let r = QueryResult {
            columns: vec!["id".to_string(), "name".to_string()].into(),
            rows: vec![vec![Value::Int(10), Value::Text("sword".into())]],
            message: "SELECT 1".into(),
            ..Default::default()
        };
        let s = r.to_table_string();
        assert!(s.contains("| id | name  |"));
        assert!(s.contains("| 10 | sword |"));
        assert!(s.ends_with("(1 row)"));
    }
}
