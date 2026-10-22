use std::fmt;

/// Every fallible operation in the engine returns this error type.
#[derive(Debug, Clone, PartialEq)]
pub enum Error {
    /// Tokenizer failure (byte offset in the SQL text).
    Lex { pos: usize, msg: String },
    /// Grammar failure (byte offset in the SQL text).
    Parse { pos: usize, msg: String },
    /// Unknown / duplicated table, column or index.
    Schema(String),
    /// PRIMARY KEY or NOT NULL violation.
    Constraint(String),
    /// Value of the wrong type for a column or an operator.
    Type(String),
    /// Runtime evaluation error (division by zero, overflow, ...).
    Eval(String),
    /// Misused BEGIN / COMMIT / ROLLBACK.
    Transaction(String),
    /// Corrupted snapshot / WAL / sync batch or an I/O failure.
    Storage(String),
    /// Valid SQL that Flat-SQL deliberately does not implement (JOIN, FK, ...).
    Unsupported(String),
}

pub type Result<T> = std::result::Result<T, Error>;

impl Error {
    /// Short machine readable category, exposed to JavaScript as `error.kind`.
    pub fn kind(&self) -> &'static str {
        match self {
            Error::Lex { .. } => "lex",
            Error::Parse { .. } => "parse",
            Error::Schema(_) => "schema",
            Error::Constraint(_) => "constraint",
            Error::Type(_) => "type",
            Error::Eval(_) => "eval",
            Error::Transaction(_) => "transaction",
            Error::Storage(_) => "storage",
            Error::Unsupported(_) => "unsupported",
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Lex { pos, msg } => write!(f, "syntax error at position {pos}: {msg}"),
            Error::Parse { pos, msg } => write!(f, "syntax error at position {pos}: {msg}"),
            Error::Schema(m) => write!(f, "schema error: {m}"),
            Error::Constraint(m) => write!(f, "constraint violation: {m}"),
            Error::Type(m) => write!(f, "type error: {m}"),
            Error::Eval(m) => write!(f, "evaluation error: {m}"),
            Error::Transaction(m) => write!(f, "transaction error: {m}"),
            Error::Storage(m) => write!(f, "storage error: {m}"),
            Error::Unsupported(m) => write!(f, "not supported: {m}"),
        }
    }
}

impl std::error::Error for Error {}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Error::Storage(e.to_string())
    }
}
