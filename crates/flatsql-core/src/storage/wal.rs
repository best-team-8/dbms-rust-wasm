//! Write-ahead log.
//!
//! The log is a sequence of frames, one frame per committed transaction:
//!
//! ```text
//! +-------------+-------------+--------------------------------------+
//! | len: u32 LE | crc: u32 LE | payload: lsn, op_count, op, op, ...  |
//! +-------------+-------------+--------------------------------------+
//! ```
//!
//! A frame is either replayed completely or not at all. Recovery stops at the
//! first frame that is truncated or fails the checksum (a torn write after a
//! crash or a closed browser tab); everything before it is valid.

use crate::error::{Error, Result};
use crate::storage::codec::{crc32, Reader, Writer};
use crate::sync::{decode_change, Change};

pub(crate) const OP_CHANGE: u8 = 0x10;
pub(crate) const OP_ACK: u8 = 0x20;
pub(crate) const OP_SYNC: u8 = 0x30;

#[derive(Debug, Clone, PartialEq)]
pub enum LogOp {
    /// A committed change. `seq > 0` marks a local change that belongs to the
    /// sync outbox, `seq == 0` a change received from another replica.
    Change { seq: u64, change: Change },
    /// The sync server acknowledged outbox entries up to `seq`.
    Ack(u64),
    /// Synchronisation was enabled: every row and tombstone became a pending
    /// outbox entry, numbered from `seq` on in catalog and key order.
    Sync(u64),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Frame {
    pub lsn: u64,
    pub ops: Vec<LogOp>,
}

/// Appends one frame to `out`. `ops` holds `op_count` already encoded ops.
pub(crate) fn write_frame(out: &mut Vec<u8>, lsn: u64, op_count: u64, ops: &[u8]) {
    let mut payload = Writer::with_capacity(ops.len() + 20);
    payload.var(lsn);
    payload.var(op_count);
    payload.raw(ops);
    let payload = payload.into_inner();
    out.extend_from_slice(&(payload.len() as u32).to_le_bytes());
    out.extend_from_slice(&crc32(&payload).to_le_bytes());
    out.extend_from_slice(&payload);
}

fn decode_payload(payload: &[u8]) -> Result<Frame> {
    let mut r = Reader::new(payload);
    let lsn = r.var()?;
    let count = r.var()?;
    let mut ops = Vec::with_capacity(count.min(1 << 16) as usize);
    for _ in 0..count {
        ops.push(match r.u8()? {
            OP_CHANGE => {
                let seq = r.var()?;
                LogOp::Change { seq, change: decode_change(&mut r)? }
            }
            OP_ACK => LogOp::Ack(r.var()?),
            OP_SYNC => LogOp::Sync(r.var()?),
            t => return Err(Error::Storage(format!("unknown WAL op {t:#x}"))),
        });
    }
    if !r.is_empty() {
        return Err(Error::Storage("trailing bytes in WAL frame".into()));
    }
    Ok(Frame { lsn, ops })
}

/// Iterates over the valid prefix of a WAL.
pub struct WalReader<'a> {
    data: &'a [u8],
    pos: usize,
    /// Why iteration stopped early (torn / corrupted tail), if it did.
    pub stop_reason: Option<String>,
}

impl<'a> WalReader<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        WalReader { data, pos: 0, stop_reason: None }
    }

    /// Bytes that form complete, valid frames.
    pub fn valid_len(&self) -> usize {
        self.pos
    }

    /// Bytes after the valid prefix (0 for a clean log).
    pub fn torn_bytes(&self) -> usize {
        self.data.len() - self.pos
    }
}

impl Iterator for WalReader<'_> {
    type Item = Frame;

    fn next(&mut self) -> Option<Frame> {
        if self.stop_reason.is_some() || self.pos >= self.data.len() {
            return None;
        }
        let rest = &self.data[self.pos..];
        if rest.len() < 8 {
            self.stop_reason = Some("truncated frame header".into());
            return None;
        }
        let len = u32::from_le_bytes(rest[0..4].try_into().unwrap()) as usize;
        let crc = u32::from_le_bytes(rest[4..8].try_into().unwrap());
        let Some(payload) = rest.get(8..8 + len) else {
            self.stop_reason = Some("truncated frame payload".into());
            return None;
        };
        if crc32(payload) != crc {
            self.stop_reason = Some("frame checksum mismatch".into());
            return None;
        }
        match decode_payload(payload) {
            Ok(frame) => {
                self.pos += 8 + len;
                Some(frame)
            }
            Err(e) => {
                self.stop_reason = Some(format!("undecodable frame: {e}"));
                None
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sync::{Mutation, Stamp};
    use crate::value::Value;

    fn frame_bytes(lsn: u64) -> Vec<u8> {
        let mut ops = Writer::new();
        ops.u8(OP_CHANGE);
        ops.var(lsn);
        crate::sync::encode_put(&mut ops, Stamp { time: lsn, node: 1 }, "t", &[Value::Int(lsn as i64)]);
        ops.u8(OP_ACK);
        ops.var(3);
        let mut out = Vec::new();
        write_frame(&mut out, lsn, 2, &ops.buf);
        out
    }

    #[test]
    fn reads_back_frames() {
        let mut wal = frame_bytes(1);
        wal.extend(frame_bytes(2));
        let frames: Vec<_> = WalReader::new(&wal).collect();
        assert_eq!(frames.len(), 2);
        assert_eq!(frames[1].lsn, 2);
        assert_eq!(frames[1].ops[1], LogOp::Ack(3));
        match &frames[0].ops[0] {
            LogOp::Change { seq, change } => {
                assert_eq!(*seq, 1);
                assert!(matches!(change.mutation, Mutation::Put { .. }));
            }
            other => panic!("{other:?}"),
        }
    }

    #[test]
    fn stops_at_torn_tail() {
        let mut wal = frame_bytes(1);
        let good = wal.len();
        let second = frame_bytes(2);
        // Simulate a crash in the middle of writing the second frame.
        wal.extend_from_slice(&second[..second.len() / 2]);
        let mut r = WalReader::new(&wal);
        assert_eq!(r.by_ref().count(), 1);
        assert_eq!(r.valid_len(), good);
        assert_eq!(r.torn_bytes(), second.len() / 2);
        assert!(r.stop_reason.is_some());
    }

    #[test]
    fn stops_at_corrupted_frame() {
        let mut wal = frame_bytes(1);
        wal.extend(frame_bytes(2));
        let n = wal.len();
        wal[n - 2] ^= 0x55; // flip bits inside the second payload
        let mut r = WalReader::new(&wal);
        assert_eq!(r.by_ref().count(), 1);
        assert_eq!(r.stop_reason.as_deref(), Some("frame checksum mismatch"));
    }
}
