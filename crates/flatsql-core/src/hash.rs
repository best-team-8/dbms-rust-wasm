//! Hashing of values for the engine's in-memory hash tables (grouping rows,
//! building indexes). The function is a multiply-rotate hash written here,
//! because the core has no dependencies; the tables are local to one
//! operation, so resistance to chosen collisions is not needed.

use std::hash::{BuildHasherDefault, Hash, Hasher};

use crate::value::Value;

/// Multiply-rotate hash of 64-bit words (the scheme of rustc's "FxHash").
#[derive(Default, Clone, Copy)]
pub(crate) struct FxHasher(u64);

const SEED: u64 = 0x51_7c_c1_b7_27_22_0a_95;

impl FxHasher {
    #[inline]
    fn add(&mut self, word: u64) {
        self.0 = (self.0.rotate_left(5) ^ word).wrapping_mul(SEED);
    }
}

impl Hasher for FxHasher {
    #[inline]
    fn write(&mut self, bytes: &[u8]) {
        let mut chunks = bytes.chunks_exact(8);
        for chunk in &mut chunks {
            self.add(u64::from_le_bytes(chunk.try_into().unwrap()));
        }
        let rest = chunks.remainder();
        if !rest.is_empty() {
            let mut word = [0u8; 8];
            word[..rest.len()].copy_from_slice(rest);
            self.add(u64::from_le_bytes(word));
        }
    }

    #[inline]
    fn write_u8(&mut self, i: u8) {
        self.add(i as u64);
    }

    #[inline]
    fn write_u64(&mut self, i: u64) {
        self.add(i);
    }

    #[inline]
    fn finish(&self) -> u64 {
        self.0
    }
}

pub(crate) type FxBuildHasher = BuildHasherDefault<FxHasher>;

/// A value as the key of a hash table. Values equal in the order of `Value`
/// hash equally: numbers hash by their value as a float (so 1 and 1.0, which
/// are equal, meet), zero without its sign and every NaN alike.
#[derive(Clone, Copy)]
pub(crate) struct ValueKey<'a>(pub &'a Value);

fn float_bits(f: f64) -> u64 {
    if f == 0.0 {
        0
    } else if f.is_nan() {
        f64::NAN.to_bits()
    } else {
        f.to_bits()
    }
}

impl Hash for ValueKey<'_> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self.0 {
            Value::Null => state.write_u8(0),
            Value::Bool(b) => {
                state.write_u8(1);
                state.write_u8(*b as u8);
            }
            Value::Int(i) => {
                state.write_u8(2);
                state.write_u64(float_bits(*i as f64));
            }
            Value::Float(f) => {
                state.write_u8(2);
                state.write_u64(float_bits(*f));
            }
            Value::Text(s) => {
                state.write_u8(3);
                s.hash(state);
            }
        }
    }
}

impl PartialEq for ValueKey<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl Eq for ValueKey<'_> {}

#[cfg(test)]
mod tests {
    use super::*;
    use std::hash::BuildHasher;

    fn hash(v: &Value) -> u64 {
        FxBuildHasher::default().hash_one(ValueKey(v))
    }

    #[test]
    fn equal_values_hash_equally() {
        assert_eq!(hash(&Value::Int(3)), hash(&Value::Float(3.0)));
        assert_eq!(hash(&Value::Float(0.0)), hash(&Value::Float(-0.0)));
        assert_eq!(hash(&Value::Float(f64::NAN)), hash(&Value::Float(-f64::NAN)));
        assert_eq!(hash(&Value::Text("abc".into())), hash(&Value::Text("abc".into())));
        assert_ne!(hash(&Value::Text("abc".into())), hash(&Value::Text("abd".into())));
        assert_ne!(hash(&Value::Bool(true)), hash(&Value::Int(1)));
    }
}
