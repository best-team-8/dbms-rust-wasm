//! File backend for native targets: `<dir>/snapshot.fsql` + `<dir>/wal.fsql`.

use std::fs::{self, File, OpenOptions};
use std::io::{Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};

use super::Storage;
use crate::error::Result;

pub struct FileStorage {
    dir: PathBuf,
    wal: File,
    wal_size: u64,
    fsync: bool,
}

impl FileStorage {
    pub fn open(dir: impl AsRef<Path>) -> Result<Self> {
        let dir = dir.as_ref().to_path_buf();
        fs::create_dir_all(&dir)?;
        // Not opened in append mode: on Windows an append-only handle cannot be
        // truncated, which a checkpoint needs. Appends seek to the end instead.
        let wal = OpenOptions::new().create(true).truncate(false).read(true).write(true).open(dir.join("wal.fsql"))?;
        let wal_size = wal.metadata()?.len();
        Ok(FileStorage { dir, wal, wal_size, fsync: true })
    }

    /// `fsync` after every WAL append (default: on). Turning it off trades
    /// durability of the last transactions for throughput.
    pub fn with_fsync(mut self, on: bool) -> Self {
        self.fsync = on;
        self
    }

    pub fn dir(&self) -> &Path {
        &self.dir
    }

    fn snapshot_path(&self) -> PathBuf {
        self.dir.join("snapshot.fsql")
    }
}

impl Storage for FileStorage {
    fn load(&mut self) -> Result<(Option<Vec<u8>>, Vec<u8>)> {
        let snapshot = match fs::read(self.snapshot_path()) {
            Ok(b) => Some(b),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => None,
            Err(e) => return Err(e.into()),
        };
        let wal = fs::read(self.dir.join("wal.fsql"))?;
        Ok((snapshot, wal))
    }

    fn append_wal(&mut self, bytes: &[u8]) -> Result<()> {
        self.wal.seek(SeekFrom::End(0))?;
        self.wal.write_all(bytes)?;
        if self.fsync {
            self.wal.sync_data()?;
        }
        self.wal_size += bytes.len() as u64;
        Ok(())
    }

    fn write_snapshot(&mut self, snapshot: &[u8]) -> Result<()> {
        // write-to-temp + rename keeps the previous snapshot intact if we crash
        // in the middle; WAL frames already in the snapshot are skipped by LSN.
        let tmp = self.dir.join("snapshot.fsql.tmp");
        {
            let mut f = File::create(&tmp)?;
            f.write_all(snapshot)?;
            f.sync_all()?;
        }
        fs::rename(&tmp, self.snapshot_path())?;
        self.wal.set_len(0)?;
        if self.fsync {
            self.wal.sync_all()?;
        }
        self.wal_size = 0;
        Ok(())
    }

    fn wal_size(&self) -> u64 {
        self.wal_size
    }
}
