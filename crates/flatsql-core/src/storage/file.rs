//! File backend for native targets: `<dir>/snapshot.fsql` + `<dir>/wal.fsql`.

use std::fs::{self, File, OpenOptions};
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};

use super::Storage;
use crate::error::Result;

const SNAPSHOT: &str = "snapshot.fsql";
const SNAPSHOT_TMP: &str = "snapshot.fsql.tmp";
const WAL: &str = "wal.fsql";

/// File operations needed by [`FileStorage`]. [`OsFs`] is the real file
/// system; tests substitute a simulated one to explore the states a crash can
/// leave behind.
pub trait Vfs {
    /// Creates the directory (and its parents); returns `true` if it did not exist.
    fn create_dir_all(&mut self, dir: &Path) -> io::Result<bool>;
    /// Size of a file, `None` if it does not exist.
    fn len(&mut self, path: &Path) -> io::Result<Option<u64>>;
    /// Whole contents of a file, `None` if it does not exist.
    fn read(&mut self, path: &Path) -> io::Result<Option<Vec<u8>>>;
    /// Appends to a file, creating it if needed.
    fn append(&mut self, path: &Path, bytes: &[u8]) -> io::Result<()>;
    /// Creates or replaces a file.
    fn write(&mut self, path: &Path, bytes: &[u8]) -> io::Result<()>;
    fn truncate(&mut self, path: &Path, len: u64) -> io::Result<()>;
    /// Makes the data of a file durable (`fdatasync`).
    fn sync(&mut self, path: &Path) -> io::Result<()>;
    fn rename(&mut self, from: &Path, to: &Path) -> io::Result<()>;
    /// Makes the entries of a directory (created and renamed files) durable.
    fn sync_dir(&mut self, dir: &Path) -> io::Result<()>;
}

/// The operating system's file system. The last used file stays open, so
/// WAL appends do not reopen it.
#[derive(Debug, Default)]
pub struct OsFs {
    open: Option<(PathBuf, File)>,
}

impl OsFs {
    fn handle(&mut self, path: &Path) -> io::Result<&mut File> {
        if !self.open.as_ref().is_some_and(|(p, _)| p == path) {
            // Not opened in append mode: on Windows an append-only handle
            // cannot be truncated. Appends seek to the end instead.
            let f = OpenOptions::new().create(true).truncate(false).read(true).write(true).open(path)?;
            self.open = Some((path.to_path_buf(), f));
        }
        Ok(&mut self.open.as_mut().unwrap().1)
    }

    fn forget(&mut self, path: &Path) {
        if self.open.as_ref().is_some_and(|(p, _)| p == path) {
            self.open = None;
        }
    }
}

impl Vfs for OsFs {
    fn create_dir_all(&mut self, dir: &Path) -> io::Result<bool> {
        let existed = dir.is_dir();
        fs::create_dir_all(dir)?;
        Ok(!existed)
    }

    fn len(&mut self, path: &Path) -> io::Result<Option<u64>> {
        match fs::metadata(path) {
            Ok(m) => Ok(Some(m.len())),
            Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(None),
            Err(e) => Err(e),
        }
    }

    fn read(&mut self, path: &Path) -> io::Result<Option<Vec<u8>>> {
        let mut f = match File::open(path) {
            Ok(f) => f,
            Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(None),
            Err(e) => return Err(e),
        };
        let mut out = Vec::new();
        f.read_to_end(&mut out)?;
        Ok(Some(out))
    }

    fn append(&mut self, path: &Path, bytes: &[u8]) -> io::Result<()> {
        use std::io::{Seek, SeekFrom};
        let f = self.handle(path)?;
        f.seek(SeekFrom::End(0))?;
        f.write_all(bytes)
    }

    fn write(&mut self, path: &Path, bytes: &[u8]) -> io::Result<()> {
        self.forget(path);
        let mut f = File::create(path)?;
        f.write_all(bytes)?;
        self.open = Some((path.to_path_buf(), f));
        Ok(())
    }

    fn truncate(&mut self, path: &Path, len: u64) -> io::Result<()> {
        self.handle(path)?.set_len(len)
    }

    fn sync(&mut self, path: &Path) -> io::Result<()> {
        self.handle(path)?.sync_data()
    }

    fn rename(&mut self, from: &Path, to: &Path) -> io::Result<()> {
        self.forget(from);
        self.forget(to);
        fs::rename(from, to)
    }

    fn sync_dir(&mut self, dir: &Path) -> io::Result<()> {
        sync_dir(dir)
    }
}

/// `fsync` of a directory. On Windows a directory handle needs
/// `FILE_FLAG_BACKUP_SEMANTICS` to be opened at all and write access for
/// `FlushFileBuffers` to succeed.
pub fn sync_dir(dir: &Path) -> io::Result<()> {
    #[cfg(windows)]
    {
        use std::os::windows::fs::OpenOptionsExt;
        const FILE_FLAG_BACKUP_SEMANTICS: u32 = 0x0200_0000;
        OpenOptions::new().write(true).custom_flags(FILE_FLAG_BACKUP_SEMANTICS).open(dir)?.sync_all()
    }
    #[cfg(not(windows))]
    {
        File::open(dir)?.sync_all()
    }
}

pub struct FileStorage<F: Vfs = OsFs> {
    dir: PathBuf,
    fs: F,
    wal_size: u64,
    fsync: bool,
}

impl FileStorage<OsFs> {
    pub fn open(dir: impl AsRef<Path>) -> Result<Self> {
        Self::open_with(OsFs::default(), dir)
    }
}

impl<F: Vfs> FileStorage<F> {
    /// Opens the storage on top of any [`Vfs`].
    pub fn open_with(mut fs: F, dir: impl AsRef<Path>) -> Result<Self> {
        let dir = dir.as_ref().to_path_buf();
        // New directory entries are durable only after their directory is
        // synced; without that a crash could lose the whole WAL file.
        if fs.create_dir_all(&dir)? {
            if let Some(parent) = dir.parent().filter(|p| !p.as_os_str().is_empty()) {
                fs.sync_dir(parent)?;
            }
        }
        let wal = dir.join(WAL);
        let wal_size = match fs.len(&wal)? {
            Some(n) => n,
            None => {
                fs.write(&wal, &[])?;
                fs.sync_dir(&dir)?;
                0
            }
        };
        Ok(FileStorage { dir, fs, wal_size, fsync: true })
    }

    /// `fsync` after every WAL append (default: on). Turning it off trades
    /// durability of the last transactions for throughput; checkpoints are
    /// synced either way, so older transactions are never lost.
    pub fn with_fsync(mut self, on: bool) -> Self {
        self.fsync = on;
        self
    }

    pub fn dir(&self) -> &Path {
        &self.dir
    }

    pub fn fs(&self) -> &F {
        &self.fs
    }
}

impl<F: Vfs> Storage for FileStorage<F> {
    fn load(&mut self) -> Result<(Option<Vec<u8>>, Vec<u8>)> {
        let snapshot = self.fs.read(&self.dir.join(SNAPSHOT))?;
        let wal = self.fs.read(&self.dir.join(WAL))?.unwrap_or_default();
        Ok((snapshot, wal))
    }

    fn append_wal(&mut self, bytes: &[u8]) -> Result<()> {
        let wal = self.dir.join(WAL);
        self.fs.append(&wal, bytes)?;
        if self.fsync {
            self.fs.sync(&wal)?;
        }
        self.wal_size += bytes.len() as u64;
        Ok(())
    }

    fn write_snapshot(&mut self, snapshot: &[u8]) -> Result<()> {
        // Every step is durable before the next one starts, whatever the
        // `fsync` setting: a crash leaves either the old snapshot with the
        // whole WAL or the new snapshot, whose frames the WAL may still hold
        // (they are skipped by LSN). Truncating the WAL before the rename is
        // durable could leave the old snapshot with an empty WAL.
        let tmp = self.dir.join(SNAPSHOT_TMP);
        self.fs.write(&tmp, snapshot)?;
        self.fs.sync(&tmp)?;
        self.fs.rename(&tmp, &self.dir.join(SNAPSHOT))?;
        self.fs.sync_dir(&self.dir)?;
        let wal = self.dir.join(WAL);
        self.fs.truncate(&wal, 0)?;
        if self.fsync {
            self.fs.sync(&wal)?;
        }
        self.wal_size = 0;
        Ok(())
    }

    fn wal_size(&self) -> u64 {
        self.wal_size
    }
}
