//! Crash consistency of the file storage (WAL + snapshot + checkpoint).
//!
//! The file system is simulated: every operation of `FileStorage` is recorded,
//! and a crash state is built from the operations the file system could have
//! persisted before the crash. Only `sync` (file) and `sync_dir` (directory
//! entries) force earlier operations to disk; everything else may be lost.

use std::collections::BTreeMap;
use std::io;
use std::path::Path;

use flatsql_core::storage::file::{FileStorage, OsFs, Vfs};
use flatsql_core::storage::Persistent;
use flatsql_core::Value;

const DIR: &str = "db";

#[derive(Debug, Clone)]
enum Op {
    /// A new file appears in the directory.
    Link {
        name: String,
        inode: usize,
    },
    Rename {
        from: String,
        to: String,
        inode: usize,
    },
    Write {
        inode: usize,
        offset: usize,
        bytes: Vec<u8>,
    },
    SetLen {
        inode: usize,
        len: usize,
    },
    Sync {
        inode: usize,
    },
    SyncDir,
}

impl Op {
    fn is_dir_op(&self) -> bool {
        matches!(self, Op::Link { .. } | Op::Rename { .. })
    }

    fn content_inode(&self) -> Option<usize> {
        match self {
            Op::Write { inode, .. } | Op::SetLen { inode, .. } => Some(*inode),
            _ => None,
        }
    }
}

/// One directory of files kept in memory, with the trace of operations.
#[derive(Debug, Default)]
struct SimFs {
    names: BTreeMap<String, usize>,
    data: Vec<Vec<u8>>,
    trace: Vec<Op>,
}

fn file_name(path: &Path) -> String {
    path.file_name().unwrap().to_str().unwrap().to_string()
}

fn not_found(path: &Path) -> io::Error {
    io::Error::new(io::ErrorKind::NotFound, path.display().to_string())
}

impl SimFs {
    /// A durable directory with the given files (nothing recorded yet).
    fn with_files(files: &BTreeMap<String, Vec<u8>>) -> Self {
        let mut fs = SimFs::default();
        for (name, bytes) in files {
            fs.names.insert(name.clone(), fs.data.len());
            fs.data.push(bytes.clone());
        }
        fs
    }

    fn inode(&self, path: &Path) -> io::Result<usize> {
        self.names.get(&file_name(path)).copied().ok_or_else(|| not_found(path))
    }

    fn create(&mut self, path: &Path) -> usize {
        let inode = self.data.len();
        self.data.push(Vec::new());
        self.names.insert(file_name(path), inode);
        self.trace.push(Op::Link { name: file_name(path), inode });
        inode
    }

    fn write_at(&mut self, inode: usize, offset: usize, bytes: &[u8]) {
        if bytes.is_empty() {
            return;
        }
        let d = &mut self.data[inode];
        d.truncate(offset);
        d.extend_from_slice(bytes);
        self.trace.push(Op::Write { inode, offset, bytes: bytes.to_vec() });
    }
}

impl Vfs for SimFs {
    fn create_dir_all(&mut self, _dir: &Path) -> io::Result<bool> {
        Ok(false)
    }

    fn len(&mut self, path: &Path) -> io::Result<Option<u64>> {
        Ok(self.names.get(&file_name(path)).map(|&i| self.data[i].len() as u64))
    }

    fn read(&mut self, path: &Path) -> io::Result<Option<Vec<u8>>> {
        Ok(self.names.get(&file_name(path)).map(|&i| self.data[i].clone()))
    }

    fn append(&mut self, path: &Path, bytes: &[u8]) -> io::Result<()> {
        let inode = match self.inode(path) {
            Ok(i) => i,
            Err(_) => self.create(path),
        };
        let offset = self.data[inode].len();
        self.write_at(inode, offset, bytes);
        Ok(())
    }

    fn write(&mut self, path: &Path, bytes: &[u8]) -> io::Result<()> {
        let inode = match self.inode(path) {
            Ok(i) => {
                self.data[i].clear();
                self.trace.push(Op::SetLen { inode: i, len: 0 });
                i
            }
            Err(_) => self.create(path),
        };
        self.write_at(inode, 0, bytes);
        Ok(())
    }

    fn truncate(&mut self, path: &Path, len: u64) -> io::Result<()> {
        let inode = self.inode(path)?;
        self.data[inode].resize(len as usize, 0);
        self.trace.push(Op::SetLen { inode, len: len as usize });
        Ok(())
    }

    fn sync(&mut self, path: &Path) -> io::Result<()> {
        let inode = self.inode(path)?;
        self.trace.push(Op::Sync { inode });
        Ok(())
    }

    fn rename(&mut self, from: &Path, to: &Path) -> io::Result<()> {
        let inode = self.names.remove(&file_name(from)).ok_or_else(|| not_found(from))?;
        self.names.insert(file_name(to), inode);
        self.trace.push(Op::Rename { from: file_name(from), to: file_name(to), inode });
        Ok(())
    }

    fn sync_dir(&mut self, _dir: &Path) -> io::Result<()> {
        self.trace.push(Op::SyncDir);
        Ok(())
    }
}

/// Operations of `trace[..crash]` that must be on disk: everything covered by
/// a completed `sync` of the same file or `sync_dir`.
fn forced(trace: &[Op], crash: usize) -> Vec<bool> {
    let mut out = vec![false; crash];
    for j in 0..crash {
        match &trace[j] {
            Op::Sync { inode } => {
                for i in 0..j {
                    if trace[i].content_inode() == Some(*inode) {
                        out[i] = true;
                    }
                }
            }
            Op::SyncDir => {
                for i in 0..j {
                    if trace[i].is_dir_op() {
                        out[i] = true;
                    }
                }
            }
            _ => {}
        }
    }
    out
}

/// Directory contents after a crash. `persisted[i]` is how many bytes of
/// operation `i` reached the disk (all of them for non-write operations).
fn crash_state(trace: &[Op], persisted: &[Option<usize>]) -> BTreeMap<String, Vec<u8>> {
    let mut names: BTreeMap<String, usize> = BTreeMap::new();
    let mut data: BTreeMap<usize, Vec<u8>> = BTreeMap::new();
    for (op, p) in trace.iter().zip(persisted) {
        let Some(p) = *p else { continue };
        match op {
            Op::Link { name, inode } => {
                names.insert(name.clone(), *inode);
                data.entry(*inode).or_default();
            }
            Op::Rename { from, to, inode } => {
                if names.get(from) == Some(inode) {
                    names.remove(from);
                }
                names.insert(to.clone(), *inode);
            }
            Op::Write { inode, offset, bytes } => {
                let d = data.entry(*inode).or_default();
                let end = offset + p;
                if d.len() < end {
                    d.resize(end, 0);
                }
                d[*offset..end].copy_from_slice(&bytes[..p]);
            }
            Op::SetLen { inode, len } => data.entry(*inode).or_default().resize(*len, 0),
            Op::Sync { .. } | Op::SyncDir => {}
        }
    }
    names.into_iter().map(|(name, inode)| (name, data.get(&inode).cloned().unwrap_or_default())).collect()
}

type Rows = BTreeMap<i64, String>;

/// One autocommit statement of the workload and its effect on the model.
enum Step {
    Create,
    Put(&'static [(i64, &'static str)]),
    Delete(i64),
    /// Two changes in one explicit transaction.
    Txn((i64, &'static str), i64),
    Checkpoint,
}

const WORKLOAD: &[Step] = &[
    Step::Create,
    Step::Put(&[(1, "first row"), (2, "second row")]),
    Step::Put(&[(3, "third row")]),
    Step::Checkpoint,
    Step::Put(&[(2, "second row, updated")]),
    Step::Delete(1),
    Step::Txn((4, "fourth row in a transaction"), 3),
    Step::Put(&[(5, "fifth row"), (6, "sixth row"), (7, "seventh row")]),
    Step::Delete(5),
    Step::Put(&[(1, "first row again")]),
];

/// A workload run on the simulated file system.
struct Run {
    trace: Vec<Op>,
    /// Model state after `i` transactions (`None`: no table yet).
    states: Vec<Option<Rows>>,
    /// Trace length when transaction `i` started and when it was durable.
    issued: Vec<usize>,
    durable: Vec<usize>,
}

fn run_workload(checkpoint_bytes: u64) -> Run {
    let storage = FileStorage::open_with(SimFs::default(), DIR).unwrap();
    let mut p = Persistent::open(storage, 1).unwrap().with_checkpoint_bytes(checkpoint_bytes);
    let mut model: Option<Rows> = None;
    let mut run = Run { trace: Vec::new(), states: vec![None], issued: Vec::new(), durable: Vec::new() };
    for step in WORKLOAD {
        let sql = match step {
            Step::Checkpoint => {
                p.checkpoint().unwrap();
                continue;
            }
            Step::Create => {
                model = Some(Rows::new());
                "CREATE TABLE kv (id INT PRIMARY KEY, v TEXT)".to_string()
            }
            Step::Put(rows) => {
                let m = model.as_mut().unwrap();
                let values: Vec<String> = rows
                    .iter()
                    .map(|(k, v)| {
                        m.insert(*k, v.to_string());
                        format!("({k}, '{v}')")
                    })
                    .collect();
                format!("INSERT OR REPLACE INTO kv VALUES {}", values.join(", "))
            }
            Step::Delete(k) => {
                model.as_mut().unwrap().remove(k);
                format!("DELETE FROM kv WHERE id = {k}")
            }
            Step::Txn((k, v), del) => {
                let m = model.as_mut().unwrap();
                m.insert(*k, v.to_string());
                m.remove(del);
                format!("BEGIN; INSERT OR REPLACE INTO kv VALUES ({k}, '{v}'); DELETE FROM kv WHERE id = {del}; COMMIT")
            }
        };
        run.issued.push(p.storage().fs().trace.len());
        p.execute(&sql).unwrap();
        run.durable.push(p.storage().fs().trace.len());
        run.states.push(model.clone());
    }
    run.trace = p.storage().fs().trace.clone();
    run
}

/// Opens the database left in `files` and returns its rows.
fn recover(files: &BTreeMap<String, Vec<u8>>) -> Result<Option<Rows>, String> {
    let storage = FileStorage::open_with(SimFs::with_files(files), DIR).map_err(|e| e.to_string())?;
    let mut p = Persistent::open(storage, 1).map_err(|e| e.to_string())?;
    if p.db().table_schema("kv").is_none() {
        return Ok(None);
    }
    let r = p.execute("SELECT id, v FROM kv ORDER BY id").map_err(|e| e.to_string())?;
    let rows = r
        .rows
        .into_iter()
        .map(|row| match (&row[0], &row[1]) {
            (Value::Int(k), Value::Text(v)) => (*k, v.clone()),
            other => panic!("unexpected row {other:?}"),
        })
        .collect();
    Ok(Some(rows))
}

/// The recovered state must be the state after `j` transactions, where every
/// transaction acknowledged before the crash is included (`j >= durable`) and
/// no transaction that had not started is (`j <= issued`).
fn check(run: &Run, crash: usize, files: &BTreeMap<String, Vec<u8>>) -> Result<(), String> {
    let durable = run.durable.iter().filter(|&&d| d <= crash).count();
    let issued = run.issued.iter().filter(|&&s| s < crash).count();
    let got = recover(files).map_err(|e| format!("recovery failed: {e}"))?;
    if (durable..=issued).any(|j| run.states[j] == got) {
        Ok(())
    } else {
        Err(format!("got {got:?}, expected the state after {durable}..={issued} transactions"))
    }
}

/// Crash after every operation; the file system persisted only what it was
/// forced to (synced files and synced directory entries).
#[test]
fn committed_data_survives_crash_when_only_synced_writes_persist() {
    for checkpoint_bytes in [u64::MAX, 150] {
        let run = run_workload(checkpoint_bytes);
        let checkpoints = run.trace.iter().filter(|op| matches!(op, Op::Rename { .. })).count();
        eprintln!(
            "checkpoint_bytes {checkpoint_bytes}: {} crash points, {checkpoints} checkpoints",
            run.trace.len() + 1
        );
        for crash in 0..=run.trace.len() {
            let persisted: Vec<Option<usize>> = forced(&run.trace, crash)
                .into_iter()
                .zip(&run.trace)
                .map(|(f, op)| match op {
                    Op::Write { bytes, .. } if f => Some(bytes.len()),
                    _ if f => Some(0),
                    _ => None,
                })
                .collect();
            let files = crash_state(&run.trace[..crash], &persisted);
            if let Err(e) = check(&run, crash, &files) {
                panic!(
                    "checkpoint_bytes {checkpoint_bytes}, crash after {crash} of {} ops ({:?}): {e}",
                    run.trace.len(),
                    run.trace.get(crash.wrapping_sub(1))
                );
            }
        }
    }
}

#[test]
fn directory_of_the_real_file_system_can_be_synced() {
    let dir = std::env::temp_dir().join(format!("flatsql-dirsync-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    OsFs::default().sync_dir(&dir).unwrap();
    std::fs::remove_dir_all(&dir).unwrap();
}

/// Platform experiment: on Windows a directory can be flushed only through a
/// handle opened for writing with FILE_FLAG_BACKUP_SEMANTICS.
#[cfg(windows)]
#[test]
fn windows_directory_flush_needs_write_access_and_backup_semantics() {
    use std::fs::{File, OpenOptions};
    use std::os::windows::fs::OpenOptionsExt;
    const FILE_FLAG_BACKUP_SEMANTICS: u32 = 0x0200_0000;
    const ERROR_ACCESS_DENIED: i32 = 5;

    let dir = std::env::temp_dir().join(format!("flatsql-dirflush-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let denied = |r: io::Result<()>| r.unwrap_err().raw_os_error() == Some(ERROR_ACCESS_DENIED);

    assert!(denied(File::open(&dir).map(drop)), "plain open of a directory");
    assert!(denied(OpenOptions::new().write(true).open(&dir).map(drop)), "write access without the flag");
    let read = OpenOptions::new().read(true).custom_flags(FILE_FLAG_BACKUP_SEMANTICS).open(&dir).unwrap();
    assert!(denied(read.sync_all()), "flush through a read-only handle");
    let write = OpenOptions::new().write(true).custom_flags(FILE_FLAG_BACKUP_SEMANTICS).open(&dir).unwrap();
    write.sync_all().unwrap();
    drop((read, write));
    std::fs::remove_dir_all(&dir).unwrap();
}
