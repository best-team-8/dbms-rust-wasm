//! Abstract syntax tree produced by the Flat-SQL parser.

use std::fmt;

use crate::value::{DataType, Value};

#[derive(Debug, Clone, PartialEq)]
#[allow(clippy::large_enum_variant)]
pub enum Statement {
    CreateTable(CreateTable),
    DropTable { name: String, if_exists: bool },
    CreateIndex { name: String, table: String, column: String, if_not_exists: bool },
    DropIndex { name: String, if_exists: bool },
    Insert(Insert),
    Select(Select),
    Update(Update),
    Delete(Delete),
    Begin,
    Commit,
    Rollback,
    Explain(Box<Statement>),
    ShowTables,
    Describe(String),
}

impl Statement {
    /// True for statements that never modify data or schema.
    pub fn is_read_only(&self) -> bool {
        matches!(self, Statement::Select(_) | Statement::Explain(_) | Statement::ShowTables | Statement::Describe(_))
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ColumnDef {
    pub name: String,
    pub data_type: DataType,
    pub primary_key: bool,
    pub not_null: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CreateTable {
    pub name: String,
    pub columns: Vec<ColumnDef>,
    /// Table level `PRIMARY KEY (col)` clause.
    pub primary_key: Option<String>,
    pub if_not_exists: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OnConflict {
    Abort,
    Replace,
    Ignore,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Insert {
    pub table: String,
    pub columns: Option<Vec<String>>,
    pub rows: Vec<Vec<Expr>>,
    pub on_conflict: OnConflict,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SelectItem {
    Wildcard,
    Expr { expr: Expr, alias: Option<String> },
}

#[derive(Debug, Clone, PartialEq)]
pub struct OrderItem {
    pub expr: Expr,
    pub desc: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Select {
    pub distinct: bool,
    pub projection: Vec<SelectItem>,
    pub from: Option<String>,
    pub selection: Option<Expr>,
    pub group_by: Vec<Expr>,
    pub having: Option<Expr>,
    pub order_by: Vec<OrderItem>,
    pub limit: Option<Expr>,
    pub offset: Option<Expr>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Update {
    pub table: String,
    pub assignments: Vec<(String, Expr)>,
    pub selection: Option<Expr>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Delete {
    pub table: String,
    pub selection: Option<Expr>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnaryOp {
    Neg,
    Not,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinaryOp {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Concat,
    Eq,
    NotEq,
    Lt,
    LtEq,
    Gt,
    GtEq,
    And,
    Or,
}

impl BinaryOp {
    pub fn symbol(self) -> &'static str {
        match self {
            BinaryOp::Add => "+",
            BinaryOp::Sub => "-",
            BinaryOp::Mul => "*",
            BinaryOp::Div => "/",
            BinaryOp::Mod => "%",
            BinaryOp::Concat => "||",
            BinaryOp::Eq => "=",
            BinaryOp::NotEq => "<>",
            BinaryOp::Lt => "<",
            BinaryOp::LtEq => "<=",
            BinaryOp::Gt => ">",
            BinaryOp::GtEq => ">=",
            BinaryOp::And => "AND",
            BinaryOp::Or => "OR",
        }
    }

    pub fn is_comparison(self) -> bool {
        matches!(self, BinaryOp::Eq | BinaryOp::NotEq | BinaryOp::Lt | BinaryOp::LtEq | BinaryOp::Gt | BinaryOp::GtEq)
    }

    /// Operator with swapped operands: `a < b` <=> `b > a`.
    pub fn flipped(self) -> BinaryOp {
        match self {
            BinaryOp::Lt => BinaryOp::Gt,
            BinaryOp::LtEq => BinaryOp::GtEq,
            BinaryOp::Gt => BinaryOp::Lt,
            BinaryOp::GtEq => BinaryOp::LtEq,
            other => other,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Literal(Value),
    Column(String),
    /// Zero based index of a `?` placeholder.
    Param(usize),
    Unary {
        op: UnaryOp,
        expr: Box<Expr>,
    },
    Binary {
        op: BinaryOp,
        left: Box<Expr>,
        right: Box<Expr>,
    },
    IsNull {
        expr: Box<Expr>,
        negated: bool,
    },
    InList {
        expr: Box<Expr>,
        list: Vec<Expr>,
        negated: bool,
    },
    Between {
        expr: Box<Expr>,
        low: Box<Expr>,
        high: Box<Expr>,
        negated: bool,
    },
    Like {
        expr: Box<Expr>,
        pattern: Box<Expr>,
        negated: bool,
        case_insensitive: bool,
    },
    Function {
        name: String,
        args: Vec<Expr>,
        star: bool,
        distinct: bool,
    },
}

pub const AGGREGATES: [&str; 5] = ["COUNT", "SUM", "AVG", "MIN", "MAX"];

impl Expr {
    pub fn contains_aggregate(&self) -> bool {
        match self {
            Expr::Function { name, args, .. } => {
                AGGREGATES.contains(&name.as_str()) || args.iter().any(Expr::contains_aggregate)
            }
            Expr::Literal(_) | Expr::Column(_) | Expr::Param(_) => false,
            Expr::Unary { expr, .. } | Expr::IsNull { expr, .. } => expr.contains_aggregate(),
            Expr::Binary { left, right, .. } => left.contains_aggregate() || right.contains_aggregate(),
            Expr::InList { expr, list, .. } => expr.contains_aggregate() || list.iter().any(Expr::contains_aggregate),
            Expr::Between { expr, low, high, .. } => {
                expr.contains_aggregate() || low.contains_aggregate() || high.contains_aggregate()
            }
            Expr::Like { expr, pattern, .. } => expr.contains_aggregate() || pattern.contains_aggregate(),
        }
    }

    fn is_compound(&self) -> bool {
        matches!(
            self,
            Expr::Binary { .. } | Expr::IsNull { .. } | Expr::InList { .. } | Expr::Between { .. } | Expr::Like { .. }
        )
    }
}

struct Operand<'a>(&'a Expr);

impl fmt::Display for Operand<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.0.is_compound() {
            write!(f, "({})", self.0)
        } else {
            write!(f, "{}", self.0)
        }
    }
}

impl fmt::Display for Expr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let not = |negated: bool| if negated { "NOT " } else { "" };
        match self {
            Expr::Literal(v) => f.write_str(&v.to_sql()),
            Expr::Column(c) => f.write_str(c),
            Expr::Param(i) => write!(f, "?{}", i + 1),
            Expr::Unary { op: UnaryOp::Neg, expr } => write!(f, "-{}", Operand(expr)),
            Expr::Unary { op: UnaryOp::Not, expr } => write!(f, "NOT {}", Operand(expr)),
            Expr::Binary { op, left, right } => {
                write!(f, "{} {} {}", Operand(left), op.symbol(), Operand(right))
            }
            Expr::IsNull { expr, negated } => write!(f, "{} IS {}NULL", Operand(expr), not(*negated)),
            Expr::InList { expr, list, negated } => {
                write!(f, "{} {}IN (", Operand(expr), not(*negated))?;
                for (i, e) in list.iter().enumerate() {
                    if i > 0 {
                        f.write_str(", ")?;
                    }
                    write!(f, "{e}")?;
                }
                f.write_str(")")
            }
            Expr::Between { expr, low, high, negated } => {
                write!(f, "{} {}BETWEEN {} AND {}", Operand(expr), not(*negated), Operand(low), Operand(high))
            }
            Expr::Like { expr, pattern, negated, case_insensitive } => write!(
                f,
                "{} {}{} {}",
                Operand(expr),
                not(*negated),
                if *case_insensitive { "ILIKE" } else { "LIKE" },
                Operand(pattern)
            ),
            Expr::Function { name, args, star, distinct } => {
                write!(f, "{name}(")?;
                if *distinct {
                    f.write_str("DISTINCT ")?;
                }
                if *star {
                    f.write_str("*")?;
                }
                for (i, a) in args.iter().enumerate() {
                    if i > 0 {
                        f.write_str(", ")?;
                    }
                    write!(f, "{a}")?;
                }
                f.write_str(")")
            }
        }
    }
}
