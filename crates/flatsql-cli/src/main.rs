//! `flatsql` – interactive shell and native benchmark of the Flat-SQL engine.

mod bench;

use std::io::{self, BufRead, IsTerminal, Write};
use std::time::Instant;

use flatsql_core::storage::file::FileStorage;
use flatsql_core::storage::{MemoryStorage, Persistent, Storage};
use flatsql_core::{Database, QueryResult, Result};

const HELP: &str = "\
flatsql – Flat-SQL database shell

USAGE:
    flatsql [DIR] [-e SQL] [-f FILE]     open DIR (created if missing) or an in-memory database
    flatsql bench [--rows N] [--json FILE]
    flatsql --help | --version

OPTIONS:
    -e SQL      execute SQL and exit
    -f FILE     execute a script file and exit
    --no-fsync  do not fsync the WAL after every commit

SHELL COMMANDS:
    .help                 this text
    .tables               list tables
    .schema TABLE         describe a table
    .stats                engine statistics
    .checkpoint           write a snapshot and truncate the WAL
    .timer on|off         print execution time (default on)
    .quit                 exit
";

/// Either an in-memory or a file backed database.
enum Db {
    Memory(Persistent<MemoryStorage>),
    File(Persistent<FileStorage>),
}

impl Db {
    fn execute(&mut self, sql: &str) -> Result<QueryResult> {
        match self {
            Db::Memory(p) => p.execute(sql),
            Db::File(p) => p.execute(sql),
        }
    }

    fn db(&self) -> &Database {
        match self {
            Db::Memory(p) => p.db(),
            Db::File(p) => p.db(),
        }
    }

    fn checkpoint(&mut self) -> Result<()> {
        match self {
            Db::Memory(p) => p.checkpoint(),
            Db::File(p) => p.checkpoint(),
        }
    }

    fn wal_size(&self) -> u64 {
        match self {
            Db::Memory(p) => p.storage().wal_size(),
            Db::File(p) => p.storage().wal_size(),
        }
    }
}

fn random_node_id() -> u32 {
    let t = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default();
    (t.subsec_nanos() ^ (t.as_secs() as u32).rotate_left(16) ^ std::process::id()) | 1
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.first().map(String::as_str) == Some("bench") {
        std::process::exit(bench::main(&args[1..]));
    }

    let mut dir = None;
    let mut exec = None;
    let mut file = None;
    let mut fsync = true;
    let mut it = args.into_iter();
    while let Some(a) = it.next() {
        match a.as_str() {
            "-h" | "--help" => {
                print!("{HELP}");
                return;
            }
            "-V" | "--version" => {
                println!("flatsql {}", flatsql_core::VERSION);
                return;
            }
            "-e" => exec = it.next(),
            "-f" => file = it.next(),
            "--no-fsync" => fsync = false,
            _ if a.starts_with('-') => fail(&format!("unknown option {a}")),
            _ => dir = Some(a),
        }
    }

    let mut db = match &dir {
        None => Db::Memory(Persistent::open(MemoryStorage::default(), random_node_id()).unwrap()),
        Some(d) => {
            let storage = FileStorage::open(d).unwrap_or_else(|e| fail(&e.to_string())).with_fsync(fsync);
            let p = Persistent::open(storage, random_node_id()).unwrap_or_else(|e| fail(&e.to_string()));
            if let Some(r) = p.recovery_report() {
                if r.frames_replayed > 0 || r.torn_bytes > 0 {
                    eprintln!(
                        "recovered {} WAL frame(s){}",
                        r.frames_replayed,
                        if r.torn_bytes > 0 {
                            format!(", discarded {} torn byte(s)", r.torn_bytes)
                        } else {
                            String::new()
                        }
                    );
                }
            }
            Db::File(p)
        }
    };

    if let Some(sql) = exec {
        std::process::exit(run_script(&mut db, &sql));
    }
    if let Some(path) = file {
        let sql = std::fs::read_to_string(&path).unwrap_or_else(|e| fail(&format!("{path}: {e}")));
        std::process::exit(run_script(&mut db, &sql));
    }
    repl(&mut db, dir.as_deref());
}

fn fail(msg: &str) -> ! {
    eprintln!("error: {msg}");
    std::process::exit(2);
}

fn print_result(r: &QueryResult) {
    println!("{}", r.to_table_string());
}

fn run_script(db: &mut Db, sql: &str) -> i32 {
    match db.execute(sql) {
        Ok(r) => {
            print_result(&r);
            0
        }
        Err(e) => {
            eprintln!("error: {e}");
            1
        }
    }
}

/// True when the buffer ends with `;` outside of a string literal.
fn statement_complete(buf: &str) -> bool {
    let mut in_str = false;
    let mut last = None;
    for c in buf.chars() {
        if c == '\'' {
            in_str = !in_str;
        }
        if !c.is_whitespace() {
            last = Some(c);
        }
    }
    !in_str && last == Some(';')
}

fn repl(db: &mut Db, dir: Option<&str>) {
    let interactive = io::stdin().is_terminal();
    if interactive {
        println!("flatsql {} – {}", flatsql_core::VERSION, dir.unwrap_or("in-memory database (changes are not saved)"));
        println!("Enter Flat-SQL statements terminated with ';'. Type .help for help.");
    }
    let mut timer = true;
    let mut buf = String::new();
    let stdin = io::stdin();
    let mut lines = stdin.lock().lines();
    loop {
        if interactive {
            print!("{}", if buf.is_empty() { "flatsql> " } else { "    ...> " });
            io::stdout().flush().ok();
        }
        let Some(Ok(line)) = lines.next() else { break };
        let trimmed = line.trim();
        if buf.is_empty() && trimmed.starts_with('.') {
            let mut parts = trimmed.split_whitespace();
            match (parts.next().unwrap_or(""), parts.next()) {
                (".quit" | ".exit", _) => break,
                (".help", _) => print!("{HELP}"),
                (".tables", _) => show(db, "SHOW TABLES"),
                (".schema", Some(t)) => show(db, &format!("DESCRIBE {t}")),
                (".schema", None) => eprintln!("usage: .schema TABLE"),
                (".stats", _) => {
                    let s = db.db().stats();
                    println!(
                        "node {:08x} | {} table(s), {} row(s), {} index(es), {} tombstone(s)\n\
                         lsn {} | WAL on disk {} B | sync outbox {} (acked up to {})",
                        s.node_id,
                        s.tables,
                        s.rows,
                        s.indexes,
                        s.tombstones,
                        s.lsn,
                        db.wal_size(),
                        s.outbox,
                        s.acked_seq
                    );
                }
                (".checkpoint", _) => match db.checkpoint() {
                    Ok(()) => println!("checkpoint written"),
                    Err(e) => eprintln!("error: {e}"),
                },
                (".timer", Some("on")) => timer = true,
                (".timer", Some("off")) => timer = false,
                (cmd, _) => eprintln!("unknown command {cmd} (try .help)"),
            }
            continue;
        }
        if trimmed.is_empty() && buf.is_empty() {
            continue;
        }
        buf.push_str(&line);
        buf.push('\n');
        if !statement_complete(&buf) {
            continue;
        }
        let start = Instant::now();
        match db.execute(&buf) {
            Ok(r) => {
                print_result(&r);
                if timer && interactive {
                    println!("time: {:.3} ms", start.elapsed().as_secs_f64() * 1000.0);
                }
            }
            Err(e) => eprintln!("error: {e}"),
        }
        buf.clear();
    }
}

fn show(db: &mut Db, sql: &str) {
    match db.execute(sql) {
        Ok(r) => print_result(&r),
        Err(e) => eprintln!("error: {e}"),
    }
}

#[cfg(test)]
mod tests {
    use super::statement_complete;

    #[test]
    fn detects_statement_end_outside_strings() {
        assert!(statement_complete("SELECT 1;"));
        assert!(statement_complete("SELECT 1;  \n"));
        assert!(!statement_complete("SELECT ';"));
        assert!(statement_complete("SELECT ';';"));
        assert!(!statement_complete("SELECT 1"));
    }
}
