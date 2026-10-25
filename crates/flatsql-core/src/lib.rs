//! # Flat-SQL engine core
//!
//! A small SQL database engine designed for the Local-First model: it runs
//! inside the client (compiled to WebAssembly) or on an edge node, keeps data
//! in memory, persists it through a write-ahead log + snapshots and exchanges
//! row level changes with other replicas.
//!
//! Flat-SQL is SQL syntax over a NoSQL-like data model: every table is a flat,
//! independent key-addressed collection (mandatory PRIMARY KEY, optional
//! secondary indexes) and every query touches exactly one table — there are
//! no JOINs, foreign keys or subqueries.
//!
//! ```
//! use flatsql_core::Database;
//!
//! let mut db = Database::new(1);
//! db.execute("CREATE TABLE items (id INT PRIMARY KEY, name TEXT, qty INT)").unwrap();
//! db.execute("INSERT INTO items VALUES (1, 'sword', 1), (2, 'potion', 5)").unwrap();
//! let r = db.execute("SELECT name FROM items WHERE qty > 2").unwrap();
//! assert_eq!(r.rows[0][0].to_string(), "potion");
//! ```

pub mod compile;
pub mod db;
pub mod error;
mod exec;
pub mod expr;
mod hash;
pub mod plan;
pub mod result;
pub mod sql;
pub mod storage;
pub mod sync;
pub mod table;
pub mod value;

pub use db::{Database, DbStats, Prepared, RecoveryReport};
pub use error::{Error, Result};
pub use result::QueryResult;
pub use sync::{ApplyReport, SchemaConflict, Stamp};
pub use value::{DataType, Value};

/// Version of the engine (from Cargo.toml).
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
