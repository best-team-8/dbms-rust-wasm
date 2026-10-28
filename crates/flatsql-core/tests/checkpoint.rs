//! Crash consistency of the file storage (WAL + snapshot + checkpoint).
//!
//! The file system is simulated: every operation of `FileStorage` is recorded,
//! and a crash state is built from the operations the file system could have
//! persisted before the crash. Only `sync` (file) and `sync_dir` (directory
//! entries) force earlier operations to disk; everything else may be lost.

use std::collections::{BTreeMap, HashMap};
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

    /// Current contents of the directory (what a clean reopen would see).
    fn files(&self) -> BTreeMap<String, Vec<u8>> {
        self.names.iter().map(|(name, &inode)| (name.clone(), self.data[inode].clone())).collect()
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

fn run_workload(checkpoint_bytes: u64, fsync: bool) -> Run {
    let storage = FileStorage::open_with(SimFs::default(), DIR).unwrap().with_fsync(fsync);
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
    if !fsync {
        // Without fsync a transaction is durable once a checkpoint that
        // contains it has synced the directory after renaming the snapshot.
        let synced: Vec<usize> = (0..run.trace.len())
            .filter(|&i| {
                matches!(run.trace[i], Op::SyncDir)
                    && run.trace[..i]
                        .iter()
                        .rev()
                        .take_while(|op| !matches!(op, Op::SyncDir))
                        .any(|op| matches!(op, Op::Rename { .. }))
            })
            .map(|i| i + 1)
            .collect();
        run.durable =
            run.issued.iter().map(|&s| synced.iter().copied().find(|&c| c > s).unwrap_or(usize::MAX)).collect();
    }
    run
}

fn dump(p: &mut Persistent<FileStorage<SimFs>>) -> Result<Option<Rows>, String> {
    if p.db().table_schema("kv").is_none() {
        return Ok(None);
    }
    let r = p.execute("SELECT id, v FROM kv ORDER BY id").map_err(|e| e.to_string())?;
    let rows = r
        .rows
        .into_iter()
        .map(|row| match (&row[0], &row[1]) {
            (Value::Int(k), Value::Text(v)) => (*k, v.to_string()),
            other => panic!("unexpected row {other:?}"),
        })
        .collect();
    Ok(Some(rows))
}

fn open(files: &BTreeMap<String, Vec<u8>>) -> Result<Persistent<FileStorage<SimFs>>, String> {
    let storage = FileStorage::open_with(SimFs::with_files(files), DIR).map_err(|e| e.to_string())?;
    Persistent::open(storage, 1).map_err(|e| e.to_string())
}

/// Opens the database left in `files` and returns its rows. The recovered
/// database must also keep a new write: a torn WAL tail left in front of it
/// would hide it on the next open.
fn recover(files: &BTreeMap<String, Vec<u8>>) -> Result<Option<Rows>, String> {
    let mut p = open(files)?;
    let rows = dump(&mut p)?;
    if let Some(rows) = &rows {
        p.execute("INSERT OR REPLACE INTO kv VALUES (100, 'after recovery')").map_err(|e| e.to_string())?;
        let mut again = open(&p.storage().fs().files())?;
        let mut want = rows.clone();
        want.insert(100, "after recovery".into());
        if dump(&mut again)? != Some(want) {
            return Err("a write made after recovery was lost on the next open".into());
        }
    }
    Ok(rows)
}

/// The recovered state must be the state after `j` transactions, where every
/// transaction acknowledged before the crash is included (`j >= durable`) and
/// no transaction that had not started is (`j <= issued`).
fn check(run: &Run, crash: usize, recovered: &Result<Option<Rows>, String>) -> Result<(), String> {
    let durable = run.durable.iter().filter(|&&d| d <= crash).count();
    let issued = run.issued.iter().filter(|&&s| s < crash).count();
    let got = recovered.as_ref().map_err(|e| format!("recovery failed: {e}"))?;
    if (durable..=issued).any(|j| run.states[j] == *got) {
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
        let run = run_workload(checkpoint_bytes, true);
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
            if let Err(e) = check(&run, crash, &recover(&files)) {
                panic!(
                    "checkpoint_bytes {checkpoint_bytes}, crash after {crash} of {} ops ({:?}): {e}",
                    run.trace.len(),
                    run.trace.get(crash.wrapping_sub(1))
                );
            }
        }
    }
}

/// xorshift64* – deterministic randomness without dependencies.
struct Rng(u64);

impl Rng {
    fn below(&mut self, n: u64) -> u64 {
        self.0 ^= self.0 >> 12;
        self.0 ^= self.0 << 25;
        self.0 ^= self.0 >> 27;
        self.0.wrapping_mul(0x2545_F491_4F6C_DD1D) % n
    }
}

/// How much of an operation the file system may have persisted when nothing
/// forced it: nothing or all of it; a write also half of its bytes (torn).
fn choices(op: &Op) -> Vec<Option<usize>> {
    match op {
        Op::Write { bytes, .. } if bytes.len() > 1 => vec![None, Some(bytes.len() / 2), Some(bytes.len())],
        Op::Write { bytes, .. } => vec![None, Some(bytes.len())],
        _ => vec![None, Some(0)],
    }
}

/// Crash states examined per crash point at most; above that they are drawn
/// at random with a fixed seed. `FLATSQL_CRASH_STATES` raises it for a full
/// manual run.
fn max_states_per_crash() -> u64 {
    std::env::var("FLATSQL_CRASH_STATES").ok().and_then(|v| v.parse().ok()).unwrap_or(4096)
}

/// Calls `f` for every crash state the model permits after `crash`
/// operations: forced operations are on disk, every other one is absent,
/// complete or torn. Returns how many states exist and whether they were sampled.
fn crash_states(
    trace: &[Op],
    crash: usize,
    max: u64,
    rng: &mut Rng,
    mut f: impl FnMut(Vec<Option<usize>>),
) -> (u64, bool) {
    let forced = forced(trace, crash);
    let base: Vec<Option<usize>> = (0..crash)
        .map(|i| match &trace[i] {
            Op::Write { bytes, .. } if forced[i] => Some(bytes.len()),
            _ if forced[i] => Some(0),
            _ => None,
        })
        .collect();
    // Barriers change nothing on disk, so only the other operations vary.
    let free: Vec<(usize, Vec<Option<usize>>)> = (0..crash)
        .filter(|&i| !forced[i] && !matches!(trace[i], Op::Sync { .. } | Op::SyncDir))
        .map(|i| (i, choices(&trace[i])))
        .collect();
    let total = free.iter().fold(1u64, |n, (_, c)| n.saturating_mul(c.len() as u64));
    let build = |digits: &[usize]| {
        let mut p = base.clone();
        for ((i, c), &d) in free.iter().zip(digits) {
            p[*i] = c[d];
        }
        p
    };
    if total <= max {
        let mut digits = vec![0; free.len()];
        loop {
            f(build(&digits));
            let Some(pos) = (0..free.len()).find(|&k| digits[k] + 1 < free[k].1.len()) else { break };
            digits[pos] += 1;
            digits[..pos].iter_mut().for_each(|d| *d = 0);
        }
        (total, false)
    } else {
        for _ in 0..max {
            let digits: Vec<usize> = free.iter().map(|(_, c)| rng.below(c.len() as u64) as usize).collect();
            f(build(&digits));
        }
        (total, true)
    }
}

/// Exploration in the style of ALICE: crash after every operation and, for
/// each crash point, every combination of the not yet forced operations
/// reaching the disk, including torn writes. With fsync turned off only
/// checkpoints make transactions durable.
#[test]
fn every_permitted_crash_state_recovers_to_a_committed_state() {
    let max = max_states_per_crash();
    for (checkpoint_bytes, fsync) in [(u64::MAX, true), (150, true), (u64::MAX, false), (150, false)] {
        let run = run_workload(checkpoint_bytes, fsync);
        let mut rng = Rng(0x9E37_79B9_7F4A_7C15 ^ checkpoint_bytes ^ fsync as u64);
        let mut recovered: HashMap<BTreeMap<String, Vec<u8>>, Result<Option<Rows>, String>> = HashMap::new();
        let (mut states, mut examined, mut sampled) = (0u64, 0u64, 0usize);
        for crash in 0..=run.trace.len() {
            let mut failure = None;
            let (total, was_sampled) = crash_states(&run.trace, crash, max, &mut rng, |persisted| {
                examined += 1;
                let files = crash_state(&run.trace[..crash], &persisted);
                let result = recovered.entry(files.clone()).or_insert_with(|| recover(&files));
                if let Err(e) = check(&run, crash, result) {
                    failure.get_or_insert((files, e));
                }
            });
            states = states.saturating_add(total);
            sampled += was_sampled as usize;
            if let Some((files, e)) = failure {
                let sizes: Vec<_> = files.iter().map(|(n, b)| format!("{n}: {} B", b.len())).collect();
                panic!(
                    "fsync {fsync}, checkpoint_bytes {checkpoint_bytes}, crash after {crash} of {} ops ({:?}), \
                     files {sizes:?}: {e}",
                    run.trace.len(),
                    run.trace.get(crash.wrapping_sub(1))
                );
            }
        }
        eprintln!(
            "fsync {fsync}, checkpoint_bytes {checkpoint_bytes}: {} crash points, {states} crash states, \
             {examined} examined ({sampled} points sampled), {} distinct directory contents",
            run.trace.len() + 1,
            recovered.len()
        );
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
