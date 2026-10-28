use std::cmp::Ordering;
use std::fmt;
use std::hash::{Hash, Hasher};
use std::ops::Deref;

use crate::error::{Error, Result};

/// Column types supported by Flat-SQL.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DataType {
    Int,
    Float,
    Text,
    Bool,
}

impl DataType {
    pub fn name(self) -> &'static str {
        match self {
            DataType::Int => "INT",
            DataType::Float => "FLOAT",
            DataType::Text => "TEXT",
            DataType::Bool => "BOOL",
        }
    }

    /// Resolves a type name written in SQL, including the common aliases.
    pub fn parse(name: &str) -> Option<DataType> {
        match name.to_ascii_uppercase().as_str() {
            "INT" | "INTEGER" | "BIGINT" | "SMALLINT" | "TINYINT" => Some(DataType::Int),
            "FLOAT" | "REAL" | "DOUBLE" | "NUMERIC" | "DECIMAL" => Some(DataType::Float),
            "TEXT" | "STRING" | "VARCHAR" | "CHAR" => Some(DataType::Text),
            "BOOL" | "BOOLEAN" => Some(DataType::Bool),
            _ => None,
        }
    }

    pub(crate) fn tag(self) -> u8 {
        match self {
            DataType::Int => 1,
            DataType::Float => 2,
            DataType::Text => 3,
            DataType::Bool => 4,
        }
    }

    pub(crate) fn from_tag(tag: u8) -> Option<DataType> {
        match tag {
            1 => Some(DataType::Int),
            2 => Some(DataType::Float),
            3 => Some(DataType::Text),
            4 => Some(DataType::Bool),
            _ => None,
        }
    }
}

impl fmt::Display for DataType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.name())
    }
}

/// A single SQL value.
///
/// `Value` has a *total* order (see the `Ord` impl) so it can be used directly
/// as a `BTreeMap` key for primary keys and secondary indexes. Numbers compare
/// by numeric value regardless of being `Int` or `Float`.
#[derive(Debug, Clone)]
pub enum Value {
    Null,
    Bool(bool),
    Int(i64),
    Float(f64),
    Text(Text),
}

/// Bytes of text kept inside the value itself: as many as fit without making
/// `Value` larger than with a `String` (24 B on 64-bit targets, 16 B on
/// wasm32).
const INLINE: usize = if cfg!(target_pointer_width = "64") { 22 } else { 14 };

/// Text of a `Value`: up to `INLINE` bytes of UTF-8 stored in place, longer
/// text in one heap allocation. Most column values (codes, names, short
/// labels) need no allocation of their own, which saves memory and time when
/// rows are built, copied and dropped.
#[derive(Clone)]
pub struct Text(Repr);

#[derive(Clone)]
enum Repr {
    /// Invariant: the first `len` bytes are UTF-8 copied from a `str`.
    Inline(u8, [u8; INLINE]),
    Heap(Box<str>),
}

impl Text {
    pub fn as_str(&self) -> &str {
        match &self.0 {
            // SAFETY: inline bytes are only written from a `&str` in
            // `From<&str>`, whole and with their length (the invariant of
            // `Repr::Inline`).
            Repr::Inline(len, bytes) => unsafe { std::str::from_utf8_unchecked(&bytes[..*len as usize]) },
            Repr::Heap(s) => s,
        }
    }
}

impl Deref for Text {
    type Target = str;

    fn deref(&self) -> &str {
        self.as_str()
    }
}

impl From<&str> for Text {
    fn from(s: &str) -> Text {
        if s.len() <= INLINE {
            let mut bytes = [0; INLINE];
            bytes[..s.len()].copy_from_slice(s.as_bytes());
            Text(Repr::Inline(s.len() as u8, bytes))
        } else {
            Text(Repr::Heap(s.into()))
        }
    }
}

impl From<String> for Text {
    fn from(s: String) -> Text {
        if s.len() <= INLINE {
            Text::from(s.as_str())
        } else {
            Text(Repr::Heap(s.into_boxed_str()))
        }
    }
}

impl From<Text> for String {
    fn from(t: Text) -> String {
        match t.0 {
            Repr::Heap(s) => s.into_string(),
            Repr::Inline(..) => t.as_str().to_owned(),
        }
    }
}

impl PartialEq for Text {
    fn eq(&self, other: &Self) -> bool {
        self.as_str() == other.as_str()
    }
}

impl Eq for Text {}

impl PartialOrd for Text {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Text {
    // Not inlined: inlined into the searches of the trees of rows and
    // indexes, the two forms of `as_str` made inserts 11-15% slower.
    #[inline(never)]
    fn cmp(&self, other: &Self) -> Ordering {
        self.as_str().cmp(other.as_str())
    }
}

impl Hash for Text {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.as_str().hash(state);
    }
}

impl fmt::Debug for Text {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(self.as_str(), f)
    }
}

impl fmt::Display for Text {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl Value {
    pub fn is_null(&self) -> bool {
        matches!(self, Value::Null)
    }

    pub fn data_type(&self) -> Option<DataType> {
        match self {
            Value::Null => None,
            Value::Bool(_) => Some(DataType::Bool),
            Value::Int(_) => Some(DataType::Int),
            Value::Float(_) => Some(DataType::Float),
            Value::Text(_) => Some(DataType::Text),
        }
    }

    pub fn type_name(&self) -> &'static str {
        match self.data_type() {
            Some(t) => t.name(),
            None => "NULL",
        }
    }

    /// Position of the value's type in the cross-type sort order.
    fn rank(&self) -> u8 {
        match self {
            Value::Null => 0,
            Value::Bool(_) => 1,
            Value::Int(_) | Value::Float(_) => 2,
            Value::Text(_) => 3,
        }
    }

    pub fn as_f64(&self) -> Option<f64> {
        match self {
            Value::Int(i) => Some(*i as f64),
            Value::Float(f) => Some(*f),
            Value::Bool(b) => Some(if *b { 1.0 } else { 0.0 }),
            _ => None,
        }
    }

    pub fn is_numeric(&self) -> bool {
        matches!(self, Value::Int(_) | Value::Float(_))
    }

    /// Converts the value so it can be stored in a column of type `ty`.
    /// Only lossless conversions are allowed.
    pub fn coerce(self, ty: DataType) -> Result<Value> {
        match (self, ty) {
            (Value::Null, _) => Ok(Value::Null),
            (v @ Value::Int(_), DataType::Int) => Ok(v),
            (Value::Int(i), DataType::Float) => Ok(Value::Float(i as f64)),
            (Value::Float(f), DataType::Int)
                if f.is_finite() && f.fract() == 0.0 && (-9.2e18..=9.2e18).contains(&f) =>
            {
                Ok(Value::Int(f as i64))
            }
            (v @ Value::Float(_), DataType::Float) => Ok(v),
            (v @ Value::Text(_), DataType::Text) => Ok(v),
            (v @ Value::Bool(_), DataType::Bool) => Ok(v),
            (Value::Int(i), DataType::Bool) if i == 0 || i == 1 => Ok(Value::Bool(i == 1)),
            (v, ty) => {
                Err(Error::Type(format!("cannot store {} value {} in a {} column", v.type_name(), v.to_sql(), ty)))
            }
        }
    }

    /// Whether values of these two types can be compared with `<`, `=` etc.
    pub fn comparable(&self, other: &Value) -> bool {
        let numeric = |v: &Value| matches!(v, Value::Int(_) | Value::Float(_) | Value::Bool(_));
        match (self, other) {
            (Value::Null, _) | (_, Value::Null) => true,
            (Value::Text(_), Value::Text(_)) => true,
            (a, b) => numeric(a) && numeric(b),
        }
    }

    /// SQL literal representation (used by EXPLAIN and the CLI dump).
    pub fn to_sql(&self) -> String {
        match self {
            Value::Null => "NULL".into(),
            Value::Bool(true) => "TRUE".into(),
            Value::Bool(false) => "FALSE".into(),
            Value::Int(i) => i.to_string(),
            Value::Float(f) => format_float(*f),
            Value::Text(s) => format!("'{}'", s.replace('\'', "''")),
        }
    }

    /// Appends the JSON representation of the value to `out`.
    pub fn write_json(&self, out: &mut String) {
        match self {
            Value::Null => out.push_str("null"),
            Value::Bool(b) => out.push_str(if *b { "true" } else { "false" }),
            Value::Int(i) => out.push_str(&i.to_string()),
            Value::Float(f) if f.is_finite() => out.push_str(&format_float(*f)),
            Value::Float(_) => out.push_str("null"),
            Value::Text(s) => write_json_string(s, out),
        }
    }
}

/// Formats floats so that integral values keep a trailing `.0`
/// (`1.0` instead of `1`), which keeps the INT / FLOAT distinction visible.
pub fn format_float(f: f64) -> String {
    if f.is_nan() {
        "NaN".into()
    } else if f.is_infinite() {
        if f > 0.0 {
            "Infinity".into()
        } else {
            "-Infinity".into()
        }
    } else if f.fract() == 0.0 && f.abs() < 1e16 {
        format!("{f:.1}")
    } else {
        format!("{f}")
    }
}

pub fn write_json_string(s: &str, out: &mut String) {
    out.push('"');
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out.push('"');
}

/// Order of two floats used by `Value`: NaN after every number and equal to itself.
#[inline]
pub(crate) fn cmp_f64(a: f64, b: f64) -> Ordering {
    match a.partial_cmp(&b) {
        Some(o) => o,
        // NaN sorts after every number and equal to itself.
        None => a.is_nan().cmp(&b.is_nan()),
    }
}

impl Ord for Value {
    fn cmp(&self, other: &Self) -> Ordering {
        match (self, other) {
            (Value::Null, Value::Null) => Ordering::Equal,
            (Value::Bool(a), Value::Bool(b)) => a.cmp(b),
            (Value::Int(a), Value::Int(b)) => a.cmp(b),
            (Value::Float(a), Value::Float(b)) => cmp_f64(*a, *b),
            (Value::Int(a), Value::Float(b)) => cmp_f64(*a as f64, *b),
            (Value::Float(a), Value::Int(b)) => cmp_f64(*a, *b as f64),
            (Value::Text(a), Value::Text(b)) => a.cmp(b),
            _ => self.rank().cmp(&other.rank()),
        }
    }
}

impl PartialOrd for Value {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other) == Ordering::Equal
    }
}

impl Eq for Value {}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Null => f.write_str("NULL"),
            Value::Bool(b) => f.write_str(if *b { "true" } else { "false" }),
            Value::Int(i) => write!(f, "{i}"),
            Value::Float(x) => f.write_str(&format_float(*x)),
            Value::Text(s) => f.write_str(s),
        }
    }
}

impl From<i64> for Value {
    fn from(v: i64) -> Self {
        Value::Int(v)
    }
}

impl From<f64> for Value {
    fn from(v: f64) -> Self {
        Value::Float(v)
    }
}

impl From<&str> for Value {
    fn from(v: &str) -> Self {
        Value::Text(v.into())
    }
}

impl From<String> for Value {
    fn from(v: String) -> Self {
        Value::Text(v.into())
    }
}

impl From<bool> for Value {
    fn from(v: bool) -> Self {
        Value::Bool(v)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn numbers_compare_across_int_and_float() {
        assert_eq!(Value::Int(3), Value::Float(3.0));
        assert!(Value::Int(3) < Value::Float(3.5));
        assert!(Value::Float(-1.0) < Value::Int(0));
    }

    #[test]
    fn cross_type_order_is_total() {
        let mut v = [Value::Text("a".into()), Value::Int(1), Value::Null, Value::Bool(true), Value::Float(f64::NAN)];
        v.sort();
        assert!(v[0].is_null());
        assert!(matches!(v[1], Value::Bool(true)));
        assert!(matches!(v[2], Value::Int(1)));
        assert!(matches!(v[3], Value::Float(_)));
        assert!(matches!(v[4], Value::Text(_)));
    }

    #[test]
    fn coercion_rules() {
        assert_eq!(Value::Int(2).coerce(DataType::Float).unwrap(), Value::Float(2.0));
        assert_eq!(Value::Float(2.0).coerce(DataType::Int).unwrap(), Value::Int(2));
        assert!(Value::Float(2.5).coerce(DataType::Int).is_err());
        assert!(Value::Text("x".into()).coerce(DataType::Int).is_err());
        assert_eq!(Value::Int(1).coerce(DataType::Bool).unwrap(), Value::Bool(true));
        assert!(Value::Null.coerce(DataType::Text).unwrap().is_null());
    }

    #[test]
    fn text_is_kept_in_place_up_to_the_size_of_a_value() {
        assert_eq!(std::mem::size_of::<Value>(), if cfg!(target_pointer_width = "64") { 24 } else { 16 });
        for s in ["", "a", "ż", "0123456789abcdefghijkl", "0123456789abcdefghijklm", "zażółć gęślą jaźń 😀 i więcej"]
        {
            let t = Text::from(s);
            assert_eq!(t.as_str(), s);
            assert_eq!(Text::from(s.to_string()), t);
            assert_eq!(String::from(t.clone()), s);
            assert_eq!(matches!(t.0, Repr::Inline(..)), s.len() <= INLINE, "{s}");
        }
        // Order and equality are those of the text, whichever form it has.
        let long = "a".repeat(INLINE + 8);
        assert!(Text::from("ab") < Text::from("abc"));
        assert!(Text::from("b") > Text::from(long.as_str()));
        assert!(Value::from("b") > Value::from(long.as_str()));
        assert_eq!(Value::from(long.clone()), Value::from(long.as_str()));
        assert_eq!(format!("{:?}", Value::from("x")), "Text(\"x\")");
    }

    #[test]
    fn json_and_sql_rendering() {
        let mut s = String::new();
        Value::Text("a\"b\n".into()).write_json(&mut s);
        assert_eq!(s, r#""a\"b\n""#);
        assert_eq!(Value::Float(1.0).to_sql(), "1.0");
        assert_eq!(Value::Text("O'Hara".into()).to_sql(), "'O''Hara'");
        let mut n = String::new();
        Value::Float(f64::INFINITY).write_json(&mut n);
        assert_eq!(n, "null");
    }
}
