//! Local storage layer.
//!
//! The engine is storage agnostic: it emits WAL bytes and snapshots, and a
//! [`Storage`] implementation decides where they live. This crate ships an
//! in-memory backend and (on native targets) a file backend; the browser
//! backends (IndexedDB, OPFS) are implemented in the JavaScript bridge on top
//! of the same byte formats.

pub mod codec;
#[cfg(not(target_arch = "wasm32"))]
pub mod file;
pub mod snapshot;
pub mod wal;

use crate::db::{Database, RecoveryReport};
use crate::error::Result;
use crate::result::QueryResult;

/// Durable home of a database: one snapshot plus an append-only WAL.
pub trait Storage {
    /// Returns the current snapshot (if any) and the WAL contents.
    fn load(&mut self) -> Result<(Option<Vec<u8>>, Vec<u8>)>;
    /// Appends complete WAL frames.
    fn append_wal(&mut self, bytes: &[u8]) -> Result<()>;
    /// Atomically replaces the snapshot and truncates the WAL.
    fn write_snapshot(&mut self, snapshot: &[u8]) -> Result<()>;
    /// Current WAL size in bytes.
    fn wal_size(&self) -> u64;
}

/// Volatile storage, useful for tests and benchmarks.
#[derive(Debug, Default, Clone)]
pub struct MemoryStorage {
    pub snapshot: Option<Vec<u8>>,
    pub wal: Vec<u8>,
}

impl Storage for MemoryStorage {
    fn load(&mut self) -> Result<(Option<Vec<u8>>, Vec<u8>)> {
        Ok((self.snapshot.clone(), self.wal.clone()))
    }

    fn append_wal(&mut self, bytes: &[u8]) -> Result<()> {
        self.wal.extend_from_slice(bytes);
        Ok(())
    }

    fn write_snapshot(&mut self, snapshot: &[u8]) -> Result<()> {
        self.snapshot = Some(snapshot.to_vec());
        self.wal.clear();
        Ok(())
    }

    fn wal_size(&self) -> u64 {
        self.wal.len() as u64
    }
}

/// A [`Database`] bound to a [`Storage`]: every statement is followed by a
/// WAL flush, and a checkpoint is taken when the WAL grows past a threshold.
pub struct Persistent<S: Storage> {
    db: Database,
    storage: S,
    checkpoint_bytes: u64,
}

impl<S: Storage> Persistent<S> {
    pub const DEFAULT_CHECKPOINT_BYTES: u64 = 4 << 20;

    pub fn open(mut storage: S, node_id: u32) -> Result<Self> {
        let (snapshot, wal) = storage.load()?;
        let db = Database::restore(snapshot.as_deref(), &wal, node_id)?;
        let mut p = Persistent { db, storage, checkpoint_bytes: Self::DEFAULT_CHECKPOINT_BYTES };
        // A torn tail must not stay in front of frames appended later, and a
        // database without a snapshot gets one right away so its node id
        // (stored in the snapshot header) stays the same across restarts.
        if snapshot.is_none() || p.db.recovery_report().is_some_and(|r| r.torn_bytes > 0) {
            p.checkpoint()?;
        }
        Ok(p)
    }

    pub fn with_checkpoint_bytes(mut self, bytes: u64) -> Self {
        self.checkpoint_bytes = bytes;
        self
    }

    /// Executes SQL and makes the committed result durable.
    pub fn execute(&mut self, sql: &str) -> Result<QueryResult> {
        let result = self.db.execute(sql);
        // Earlier statements of a failed script may have committed already.
        self.flush()?;
        result
    }

    pub fn flush(&mut self) -> Result<()> {
        let wal = self.db.take_wal();
        if !wal.is_empty() {
            if let Err(e) = self.storage.append_wal(&wal) {
                self.db.unshift_wal(wal);
                return Err(e);
            }
        }
        if self.storage.wal_size() > self.checkpoint_bytes && !self.db.in_transaction() {
            self.checkpoint()?;
        }
        Ok(())
    }

    pub fn checkpoint(&mut self) -> Result<()> {
        let wal = self.db.take_wal();
        if !wal.is_empty() {
            self.storage.append_wal(&wal)?;
        }
        let snapshot = self.db.snapshot()?;
        self.storage.write_snapshot(&snapshot)
    }

    /// [`Database::rebase_clock`] made durable with a checkpoint (the new
    /// stamps are not logged in the WAL).
    pub fn rebase_clock(&mut self) -> Result<usize> {
        if !self.db.clock_ahead() {
            return Ok(0);
        }
        let restamped = self.db.rebase_clock()?;
        self.checkpoint()?;
        Ok(restamped)
    }

    pub fn recovery_report(&self) -> Option<&RecoveryReport> {
        self.db.recovery_report()
    }

    pub fn db(&self) -> &Database {
        &self.db
    }

    pub fn db_mut(&mut self) -> &mut Database {
        &mut self.db
    }

    pub fn storage(&self) -> &S {
        &self.storage
    }

    pub fn into_parts(self) -> (Database, S) {
        (self.db, self.storage)
    }
}
