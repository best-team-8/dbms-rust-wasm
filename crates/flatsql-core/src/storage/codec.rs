//! Compact binary encoding shared by the WAL, snapshots and sync batches.
//!
//! Integers use LEB128 varints (signed ones are zig-zag encoded), strings are
//! length-prefixed UTF-8, every value starts with a one byte type tag.

use crate::error::{Error, Result};
use crate::sync::Stamp;
use crate::table::{Column, TableSchema};
use crate::value::{DataType, Value};

const TAG_NULL: u8 = 0;
const TAG_FALSE: u8 = 1;
const TAG_TRUE: u8 = 2;
const TAG_INT: u8 = 3;
const TAG_FLOAT: u8 = 4;
const TAG_TEXT: u8 = 5;

/// The only NaN written to disk and to change batches (quiet, positive).
const CANONICAL_NAN: u64 = 0x7FF8_0000_0000_0000;

#[derive(Debug, Default, Clone)]
pub struct Writer {
    pub buf: Vec<u8>,
}

impl Writer {
    pub fn new() -> Self {
        Writer { buf: Vec::new() }
    }

    pub fn with_capacity(n: usize) -> Self {
        Writer { buf: Vec::with_capacity(n) }
    }

    pub fn into_inner(self) -> Vec<u8> {
        self.buf
    }

    pub fn u8(&mut self, v: u8) {
        self.buf.push(v);
    }

    pub fn u32le(&mut self, v: u32) {
        self.buf.extend_from_slice(&v.to_le_bytes());
    }

    pub fn raw(&mut self, b: &[u8]) {
        self.buf.extend_from_slice(b);
    }

    pub fn var(&mut self, mut v: u64) {
        while v >= 0x80 {
            self.buf.push((v as u8) | 0x80);
            v >>= 7;
        }
        self.buf.push(v as u8);
    }

    pub fn ivar(&mut self, v: i64) {
        self.var(((v << 1) ^ (v >> 63)) as u64);
    }

    pub fn str(&mut self, s: &str) {
        self.var(s.len() as u64);
        self.buf.extend_from_slice(s.as_bytes());
    }

    pub fn value(&mut self, v: &Value) {
        match v {
            Value::Null => self.u8(TAG_NULL),
            Value::Bool(false) => self.u8(TAG_FALSE),
            Value::Bool(true) => self.u8(TAG_TRUE),
            Value::Int(i) => {
                self.u8(TAG_INT);
                self.ivar(*i);
            }
            Value::Float(f) => {
                self.u8(TAG_FLOAT);
                // NaN bit patterns differ between platforms (WebAssembly
                // leaves them unspecified) although every NaN is the same
                // value, so all of them are written as one quiet NaN.
                let bits = if f.is_nan() { CANONICAL_NAN } else { f.to_bits() };
                self.buf.extend_from_slice(&bits.to_le_bytes());
            }
            Value::Text(s) => {
                self.u8(TAG_TEXT);
                self.str(s);
            }
        }
    }

    pub fn values(&mut self, vs: &[Value]) {
        self.var(vs.len() as u64);
        for v in vs {
            self.value(v);
        }
    }

    pub fn stamp(&mut self, s: Stamp) {
        self.var(s.time);
        self.var(s.node as u64);
    }

    pub fn schema(&mut self, s: &TableSchema) {
        self.str(&s.name);
        self.var(s.columns.len() as u64);
        for c in &s.columns {
            self.str(&c.name);
            self.u8(c.data_type.tag());
            self.u8(c.not_null as u8);
        }
        self.var(s.pk as u64);
    }
}

pub struct Reader<'a> {
    data: &'a [u8],
    pos: usize,
}

fn eof() -> Error {
    Error::Storage("unexpected end of data".into())
}

impl<'a> Reader<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        Reader { data, pos: 0 }
    }

    pub fn pos(&self) -> usize {
        self.pos
    }

    pub fn is_empty(&self) -> bool {
        self.pos >= self.data.len()
    }

    pub fn u8(&mut self) -> Result<u8> {
        let b = *self.data.get(self.pos).ok_or_else(eof)?;
        self.pos += 1;
        Ok(b)
    }

    pub fn take(&mut self, n: usize) -> Result<&'a [u8]> {
        let end = self.pos.checked_add(n).ok_or_else(eof)?;
        let s = self.data.get(self.pos..end).ok_or_else(eof)?;
        self.pos = end;
        Ok(s)
    }

    pub fn u32le(&mut self) -> Result<u32> {
        Ok(u32::from_le_bytes(self.take(4)?.try_into().unwrap()))
    }

    pub fn var(&mut self) -> Result<u64> {
        let mut out = 0u64;
        let mut shift = 0;
        loop {
            let b = self.u8()?;
            if shift >= 64 {
                return Err(Error::Storage("varint too long".into()));
            }
            out |= ((b & 0x7f) as u64) << shift;
            if b & 0x80 == 0 {
                return Ok(out);
            }
            shift += 7;
        }
    }

    pub fn len(&mut self) -> Result<usize> {
        let n = self.var()? as usize;
        // A length can never exceed the remaining input; this protects
        // allocations from corrupted data.
        if n > self.data.len() - self.pos.min(self.data.len()) {
            return Err(Error::Storage("length prefix exceeds data size".into()));
        }
        Ok(n)
    }

    pub fn ivar(&mut self) -> Result<i64> {
        let u = self.var()?;
        Ok(((u >> 1) as i64) ^ -((u & 1) as i64))
    }

    pub fn str(&mut self) -> Result<String> {
        Ok(self.str_ref()?.to_owned())
    }

    /// A string borrowed from the input, without a copy.
    pub fn str_ref(&mut self) -> Result<&'a str> {
        let n = self.len()?;
        let b = self.take(n)?;
        std::str::from_utf8(b).map_err(|_| Error::Storage("invalid UTF-8 string".into()))
    }

    pub fn value(&mut self) -> Result<Value> {
        Ok(match self.u8()? {
            TAG_NULL => Value::Null,
            TAG_FALSE => Value::Bool(false),
            TAG_TRUE => Value::Bool(true),
            TAG_INT => Value::Int(self.ivar()?),
            TAG_FLOAT => Value::Float(f64::from_le_bytes(self.take(8)?.try_into().unwrap())),
            TAG_TEXT => Value::Text(self.str_ref()?.into()),
            t => return Err(Error::Storage(format!("unknown value tag {t}"))),
        })
    }

    pub fn values(&mut self) -> Result<Vec<Value>> {
        let n = self.len()?;
        let mut out = Vec::with_capacity(n);
        for _ in 0..n {
            out.push(self.value()?);
        }
        Ok(out)
    }

    pub fn stamp(&mut self) -> Result<Stamp> {
        let time = self.var()?;
        let node = u32::try_from(self.var()?).map_err(|_| Error::Storage("node id out of range".into()))?;
        Ok(Stamp { time, node })
    }

    pub fn schema(&mut self) -> Result<TableSchema> {
        let name = self.str()?;
        let n = self.len()?;
        let mut columns = Vec::with_capacity(n);
        for _ in 0..n {
            let cname = self.str()?;
            let tag = self.u8()?;
            let data_type = DataType::from_tag(tag).ok_or_else(|| Error::Storage(format!("unknown type tag {tag}")))?;
            let not_null = self.u8()? != 0;
            columns.push(Column { name: cname, data_type, not_null });
        }
        let pk = self.var()? as usize;
        if pk >= columns.len() {
            return Err(Error::Storage(format!("invalid primary key index in schema of '{name}'")));
        }
        Ok(TableSchema { name, columns, pk })
    }
}

const CRC_TABLE: [u32; 256] = crc_table();

const fn crc_table() -> [u32; 256] {
    let mut table = [0u32; 256];
    let mut i = 0;
    while i < 256 {
        let mut c = i as u32;
        let mut k = 0;
        while k < 8 {
            c = if c & 1 != 0 { 0xEDB8_8320 ^ (c >> 1) } else { c >> 1 };
            k += 1;
        }
        table[i] = c;
        i += 1;
    }
    table
}

/// CRC-32 (IEEE 802.3), used to detect torn or corrupted records.
pub fn crc32(data: &[u8]) -> u32 {
    let mut c = !0u32;
    for &b in data {
        c = CRC_TABLE[((c ^ b as u32) & 0xff) as usize] ^ (c >> 8);
    }
    !c
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn crc32_reference_value() {
        assert_eq!(crc32(b"123456789"), 0xCBF4_3926);
    }

    #[test]
    fn roundtrip_values() {
        let vals = vec![
            Value::Null,
            Value::Bool(true),
            Value::Bool(false),
            Value::Int(0),
            Value::Int(-1),
            Value::Int(i64::MIN),
            Value::Int(i64::MAX),
            Value::Float(-2.5),
            Value::Text("zażółć gęślą jaźń".into()),
            Value::Text("".into()),
        ];
        let mut w = Writer::new();
        w.values(&vals);
        w.stamp(Stamp { time: u64::MAX, node: u32::MAX });
        let mut r = Reader::new(&w.buf);
        assert_eq!(r.values().unwrap(), vals);
        assert_eq!(r.stamp().unwrap(), Stamp { time: u64::MAX, node: u32::MAX });
        assert!(r.is_empty());
    }

    #[test]
    fn small_ints_are_compact() {
        let mut w = Writer::new();
        w.ivar(-3);
        w.var(127);
        assert_eq!(w.buf.len(), 2);
    }

    #[test]
    fn truncated_input_is_an_error() {
        let mut w = Writer::new();
        w.value(&Value::Text("hello".into()));
        let cut = &w.buf[..w.buf.len() - 1];
        assert!(Reader::new(cut).value().is_err());
        // Length prefix larger than the buffer must not allocate.
        let mut w = Writer::new();
        w.var(u32::MAX as u64);
        assert!(Reader::new(&w.buf).str().is_err());
    }
}
