//! Recursive descent parser for Flat-SQL.
//!
//! Expression precedence (lowest to highest):
//! `OR` → `AND` → `NOT` → comparison / `IS` / `IN` / `BETWEEN` / `LIKE`
//! → `||` → `+ -` → `* / %` → unary `-` → primary.

use super::ast::*;
use super::lexer::{tokenize, Tok, Token};
use crate::error::{Error, Result};
use crate::value::{DataType, Value};

/// Words that cannot be used as bare identifiers (they can still be quoted).
const RESERVED: &[&str] = &[
    "SELECT",
    "FROM",
    "WHERE",
    "GROUP",
    "BY",
    "HAVING",
    "ORDER",
    "LIMIT",
    "OFFSET",
    "INSERT",
    "INTO",
    "VALUES",
    "UPDATE",
    "SET",
    "DELETE",
    "CREATE",
    "TABLE",
    "DROP",
    "INDEX",
    "ON",
    "AND",
    "OR",
    "NOT",
    "NULL",
    "IS",
    "IN",
    "BETWEEN",
    "LIKE",
    "ILIKE",
    "AS",
    "ASC",
    "DESC",
    "DISTINCT",
    "PRIMARY",
    "TRUE",
    "FALSE",
    "JOIN",
    "INNER",
    "LEFT",
    "RIGHT",
    "OUTER",
    "CROSS",
    "UNION",
    "EXPLAIN",
    "BEGIN",
    "COMMIT",
    "ROLLBACK",
    "IF",
    "EXISTS",
    "REFERENCES",
    "FOREIGN",
];

/// Parses one or more `;` separated statements.
pub fn parse(sql: &str) -> Result<Vec<Statement>> {
    let mut p = Parser::new(sql)?;
    let mut out = Vec::new();
    loop {
        while p.eat(&Tok::Semicolon) {}
        if p.at(&Tok::Eof) {
            break;
        }
        out.push(p.statement()?);
        if !p.at(&Tok::Eof) && !p.eat(&Tok::Semicolon) {
            return Err(p.error("expected ';' or end of input"));
        }
    }
    Ok(out)
}

/// Parses exactly one statement (a trailing `;` is allowed).
pub fn parse_one(sql: &str) -> Result<Statement> {
    let mut stmts = parse(sql)?;
    match stmts.len() {
        1 => Ok(stmts.pop().unwrap()),
        0 => Err(Error::Parse { pos: 0, msg: "empty statement".into() }),
        _ => Err(Error::Parse { pos: 0, msg: "expected a single statement".into() }),
    }
}

/// Number of `?` placeholders used by a statement.
pub fn param_count(stmt: &Statement) -> usize {
    fn expr(e: &Expr, n: &mut usize) {
        match e {
            Expr::Param(i) => *n = (*n).max(i + 1),
            Expr::Literal(_) | Expr::Column(_) => {}
            Expr::Unary { expr: x, .. } | Expr::IsNull { expr: x, .. } => expr(x, n),
            Expr::Binary { left, right, .. } => {
                expr(left, n);
                expr(right, n);
            }
            Expr::InList { expr: x, list, .. } => {
                expr(x, n);
                list.iter().for_each(|e| expr(e, n));
            }
            Expr::Between { expr: x, low, high, .. } => {
                expr(x, n);
                expr(low, n);
                expr(high, n);
            }
            Expr::Like { expr: x, pattern, .. } => {
                expr(x, n);
                expr(pattern, n);
            }
            Expr::Function { args, .. } => args.iter().for_each(|e| expr(e, n)),
        }
    }
    let mut n = 0;
    let opt = |e: &Option<Expr>, n: &mut usize| {
        if let Some(e) = e {
            expr(e, n)
        }
    };
    match stmt {
        Statement::Select(s) => {
            for item in &s.projection {
                if let SelectItem::Expr { expr: e, .. } = item {
                    expr(e, &mut n);
                }
            }
            opt(&s.selection, &mut n);
            s.group_by.iter().for_each(|e| expr(e, &mut n));
            opt(&s.having, &mut n);
            s.order_by.iter().for_each(|o| expr(&o.expr, &mut n));
            opt(&s.limit, &mut n);
            opt(&s.offset, &mut n);
        }
        Statement::Insert(i) => i.rows.iter().flatten().for_each(|e| expr(e, &mut n)),
        Statement::Update(u) => {
            u.assignments.iter().for_each(|(_, e)| expr(e, &mut n));
            opt(&u.selection, &mut n);
        }
        Statement::Delete(d) => opt(&d.selection, &mut n),
        Statement::Explain(inner) => n = param_count(inner),
        _ => {}
    }
    n
}

struct Parser {
    toks: Vec<Token>,
    pos: usize,
    params: usize,
}

impl Parser {
    fn new(sql: &str) -> Result<Self> {
        Ok(Parser { toks: tokenize(sql)?, pos: 0, params: 0 })
    }

    // ---- token helpers -------------------------------------------------

    fn peek(&self) -> &Tok {
        &self.toks[self.pos].tok
    }

    fn peek_at(&self, offset: usize) -> &Tok {
        let i = (self.pos + offset).min(self.toks.len() - 1);
        &self.toks[i].tok
    }

    fn advance(&mut self) -> Tok {
        let t = self.toks[self.pos].tok.clone();
        if self.pos < self.toks.len() - 1 {
            self.pos += 1;
        }
        t
    }

    fn at(&self, t: &Tok) -> bool {
        self.peek() == t
    }

    fn eat(&mut self, t: &Tok) -> bool {
        if self.at(t) {
            self.advance();
            true
        } else {
            false
        }
    }

    fn expect(&mut self, t: &Tok, what: &str) -> Result<()> {
        if self.eat(t) {
            Ok(())
        } else {
            Err(self.error(&format!("expected {what}")))
        }
    }

    fn is_kw_at(&self, offset: usize, kw: &str) -> bool {
        matches!(self.peek_at(offset), Tok::Ident(s) if s.eq_ignore_ascii_case(kw))
    }

    fn is_kw(&self, kw: &str) -> bool {
        self.is_kw_at(0, kw)
    }

    fn eat_kw(&mut self, kw: &str) -> bool {
        if self.is_kw(kw) {
            self.advance();
            true
        } else {
            false
        }
    }

    fn expect_kw(&mut self, kw: &str) -> Result<()> {
        if self.eat_kw(kw) {
            Ok(())
        } else {
            Err(self.error(&format!("expected {kw}")))
        }
    }

    fn error(&self, msg: &str) -> Error {
        let tok = &self.toks[self.pos];
        let found = match &tok.tok {
            Tok::Eof => "end of input".to_string(),
            Tok::Ident(s) | Tok::QuotedIdent(s) => format!("'{s}'"),
            Tok::Str(s) => format!("string '{s}'"),
            Tok::Int(i) => i.to_string(),
            Tok::Float(f) => f.to_string(),
            other => format!("{other:?}"),
        };
        Error::Parse { pos: tok.pos, msg: format!("{msg}, found {found}") }
    }

    fn unsupported(&self, msg: &str) -> Error {
        Error::Unsupported(msg.to_string())
    }

    /// Identifier that is not a reserved word (quoted identifiers always pass).
    fn ident(&mut self, what: &str) -> Result<String> {
        match self.peek().clone() {
            Tok::Ident(s) if !is_reserved(&s) => {
                self.advance();
                Ok(s)
            }
            Tok::QuotedIdent(s) => {
                self.advance();
                Ok(s)
            }
            _ => Err(self.error(&format!("expected {what}"))),
        }
    }

    fn ident_list(&mut self, what: &str) -> Result<Vec<String>> {
        self.expect(&Tok::LParen, "'('")?;
        let mut out = vec![self.ident(what)?];
        while self.eat(&Tok::Comma) {
            out.push(self.ident(what)?);
        }
        self.expect(&Tok::RParen, "')'")?;
        Ok(out)
    }

    // ---- statements ------------------------------------------------------

    fn statement(&mut self) -> Result<Statement> {
        if self.eat_kw("SELECT") {
            return Ok(Statement::Select(self.select()?));
        }
        if self.is_kw("INSERT") || self.is_kw("REPLACE") {
            return self.insert();
        }
        if self.eat_kw("UPDATE") {
            return self.update();
        }
        if self.eat_kw("DELETE") {
            return self.delete();
        }
        if self.eat_kw("CREATE") {
            if self.eat_kw("TABLE") {
                return self.create_table();
            }
            if self.eat_kw("UNIQUE") {
                return Err(self.unsupported("UNIQUE secondary indexes are not implemented yet"));
            }
            if self.eat_kw("INDEX") {
                return self.create_index();
            }
            return Err(self.error("expected TABLE or INDEX after CREATE"));
        }
        if self.eat_kw("DROP") {
            if self.eat_kw("TABLE") {
                let if_exists = self.if_exists()?;
                return Ok(Statement::DropTable { name: self.ident("table name")?, if_exists });
            }
            if self.eat_kw("INDEX") {
                let if_exists = self.if_exists()?;
                let name = self.ident("index name")?;
                // `DROP INDEX name ON table` is accepted for MySQL users; index
                // names are unique per database so the table is redundant.
                if self.eat_kw("ON") {
                    self.ident("table name")?;
                }
                return Ok(Statement::DropIndex { name, if_exists });
            }
            return Err(self.error("expected TABLE or INDEX after DROP"));
        }
        if self.eat_kw("BEGIN") || (self.is_kw("START") && self.is_kw_at(1, "TRANSACTION") && self.eat_kw("START")) {
            self.eat_kw("TRANSACTION");
            return Ok(Statement::Begin);
        }
        if self.eat_kw("COMMIT") || self.eat_kw("END") {
            self.eat_kw("TRANSACTION");
            return Ok(Statement::Commit);
        }
        if self.eat_kw("ROLLBACK") {
            self.eat_kw("TRANSACTION");
            return Ok(Statement::Rollback);
        }
        if self.eat_kw("EXPLAIN") {
            let inner = self.statement()?;
            return match inner {
                Statement::Select(_) | Statement::Update(_) | Statement::Delete(_) => {
                    Ok(Statement::Explain(Box::new(inner)))
                }
                _ => Err(Error::Unsupported("EXPLAIN supports SELECT, UPDATE and DELETE".into())),
            };
        }
        if self.eat_kw("SHOW") {
            self.expect_kw("TABLES")?;
            return Ok(Statement::ShowTables);
        }
        if self.eat_kw("DESCRIBE") || self.eat_kw("DESC") {
            return Ok(Statement::Describe(self.ident("table name")?));
        }
        Err(self.error("expected a statement"))
    }

    fn if_exists(&mut self) -> Result<bool> {
        if self.eat_kw("IF") {
            self.expect_kw("EXISTS")?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn if_not_exists(&mut self) -> Result<bool> {
        if self.eat_kw("IF") {
            self.expect_kw("NOT")?;
            self.expect_kw("EXISTS")?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn create_table(&mut self) -> Result<Statement> {
        let if_not_exists = self.if_not_exists()?;
        let name = self.ident("table name")?;
        self.expect(&Tok::LParen, "'(' after table name")?;
        let mut columns = Vec::new();
        let mut primary_key = None;
        loop {
            if self.eat_kw("PRIMARY") {
                self.expect_kw("KEY")?;
                let cols = self.ident_list("column name")?;
                if cols.len() != 1 {
                    return Err(self.unsupported("composite primary keys are not supported"));
                }
                if primary_key.is_some() {
                    return Err(Error::Schema("multiple PRIMARY KEY clauses".into()));
                }
                primary_key = Some(cols.into_iter().next().unwrap());
            } else if self.is_kw("FOREIGN") || self.is_kw("CONSTRAINT") {
                return Err(
                    self.unsupported("Flat-SQL tables are flat: FOREIGN KEY / relations between tables do not exist")
                );
            } else {
                columns.push(self.column_def()?);
            }
            if !self.eat(&Tok::Comma) {
                break;
            }
        }
        self.expect(&Tok::RParen, "')' or ','")?;
        Ok(Statement::CreateTable(CreateTable { name, columns, primary_key, if_not_exists }))
    }

    fn column_def(&mut self) -> Result<ColumnDef> {
        let name = self.ident("column name")?;
        let type_pos = self.pos;
        let type_name = match self.advance() {
            Tok::Ident(s) => s,
            _ => {
                self.pos = type_pos;
                return Err(self.error("expected column type"));
            }
        };
        let data_type = DataType::parse(&type_name).ok_or_else(|| Error::Parse {
            pos: self.toks[type_pos].pos,
            msg: format!("unknown column type '{type_name}'"),
        })?;
        // Length / precision such as VARCHAR(255) or DECIMAL(10, 2) is accepted and ignored.
        if self.eat(&Tok::LParen) {
            while !self.at(&Tok::RParen) && !self.at(&Tok::Eof) {
                self.advance();
            }
            self.expect(&Tok::RParen, "')'")?;
        }
        let mut def = ColumnDef { name, data_type, primary_key: false, not_null: false };
        loop {
            if self.eat_kw("PRIMARY") {
                self.expect_kw("KEY")?;
                def.primary_key = true;
                def.not_null = true;
            } else if self.eat_kw("NOT") {
                self.expect_kw("NULL")?;
                def.not_null = true;
            } else if self.eat_kw("NULL") {
                def.not_null = false;
            } else if self.is_kw("REFERENCES") {
                return Err(
                    self.unsupported("Flat-SQL tables are flat: REFERENCES / relations between tables do not exist")
                );
            } else {
                break;
            }
        }
        Ok(def)
    }

    fn create_index(&mut self) -> Result<Statement> {
        let if_not_exists = self.if_not_exists()?;
        let name = self.ident("index name")?;
        self.expect_kw("ON")?;
        let table = self.ident("table name")?;
        let cols = self.ident_list("column name")?;
        if cols.len() != 1 {
            return Err(self.unsupported("multi-column indexes are not supported"));
        }
        Ok(Statement::CreateIndex { name, table, column: cols.into_iter().next().unwrap(), if_not_exists })
    }

    fn insert(&mut self) -> Result<Statement> {
        let mut on_conflict = OnConflict::Abort;
        if self.eat_kw("REPLACE") {
            on_conflict = OnConflict::Replace;
        } else {
            self.expect_kw("INSERT")?;
            if self.eat_kw("OR") {
                if self.eat_kw("REPLACE") {
                    on_conflict = OnConflict::Replace;
                } else if self.eat_kw("IGNORE") {
                    on_conflict = OnConflict::Ignore;
                } else {
                    return Err(self.error("expected REPLACE or IGNORE after INSERT OR"));
                }
            }
        }
        self.expect_kw("INTO")?;
        let table = self.ident("table name")?;
        let columns = if self.at(&Tok::LParen) { Some(self.ident_list("column name")?) } else { None };
        if self.is_kw("SELECT") {
            return Err(self.unsupported("INSERT ... SELECT is not supported, use VALUES"));
        }
        self.expect_kw("VALUES")?;
        let mut rows = Vec::new();
        loop {
            self.expect(&Tok::LParen, "'(' starting a VALUES row")?;
            let mut row = vec![self.expr()?];
            while self.eat(&Tok::Comma) {
                row.push(self.expr()?);
            }
            self.expect(&Tok::RParen, "')' closing a VALUES row")?;
            rows.push(row);
            if !self.eat(&Tok::Comma) {
                break;
            }
        }
        Ok(Statement::Insert(Insert { table, columns, rows, on_conflict }))
    }

    fn select(&mut self) -> Result<Select> {
        let distinct = self.eat_kw("DISTINCT");
        self.eat_kw("ALL");
        let mut projection = vec![self.select_item()?];
        while self.eat(&Tok::Comma) {
            projection.push(self.select_item()?);
        }

        let from = if self.eat_kw("FROM") {
            let t = self.ident("table name")?;
            // An optional alias is accepted (`FROM items i`) but has no meaning.
            if self.eat_kw("AS") {
                self.ident("table alias")?;
            } else if matches!(self.peek(), Tok::Ident(s) if !is_reserved(s)) {
                self.advance();
            }
            if self.at(&Tok::Comma)
                || ["JOIN", "INNER", "LEFT", "RIGHT", "OUTER", "CROSS"].iter().any(|k| self.is_kw(k))
            {
                return Err(self.unsupported("Flat-SQL does not support JOIN: every query targets exactly one table"));
            }
            Some(t)
        } else {
            None
        };

        let selection = if self.eat_kw("WHERE") { Some(self.expr()?) } else { None };

        let mut group_by = Vec::new();
        if self.eat_kw("GROUP") {
            self.expect_kw("BY")?;
            group_by.push(self.expr()?);
            while self.eat(&Tok::Comma) {
                group_by.push(self.expr()?);
            }
        }

        let having = if self.eat_kw("HAVING") { Some(self.expr()?) } else { None };

        let mut order_by = Vec::new();
        if self.eat_kw("ORDER") {
            self.expect_kw("BY")?;
            loop {
                let expr = self.expr()?;
                let desc = if self.eat_kw("DESC") {
                    true
                } else {
                    self.eat_kw("ASC");
                    false
                };
                order_by.push(OrderItem { expr, desc });
                if !self.eat(&Tok::Comma) {
                    break;
                }
            }
        }

        let (mut limit, mut offset) = (None, None);
        if self.eat_kw("LIMIT") {
            limit = Some(self.expr()?);
            if self.eat_kw("OFFSET") {
                offset = Some(self.expr()?);
            }
        }
        if self.is_kw("UNION") {
            return Err(self.unsupported("UNION is not supported"));
        }

        Ok(Select { distinct, projection, from, selection, group_by, having, order_by, limit, offset })
    }

    fn select_item(&mut self) -> Result<SelectItem> {
        if self.eat(&Tok::Star) {
            return Ok(SelectItem::Wildcard);
        }
        // `table.*`
        if matches!(self.peek(), Tok::Ident(_) | Tok::QuotedIdent(_))
            && self.peek_at(1) == &Tok::Dot
            && self.peek_at(2) == &Tok::Star
        {
            self.advance();
            self.advance();
            self.advance();
            return Ok(SelectItem::Wildcard);
        }
        let expr = self.expr()?;
        let alias = if self.eat_kw("AS") {
            Some(self.ident("alias")?)
        } else {
            match self.peek().clone() {
                Tok::Ident(s) if !is_reserved(&s) => {
                    self.advance();
                    Some(s)
                }
                Tok::QuotedIdent(s) => {
                    self.advance();
                    Some(s)
                }
                _ => None,
            }
        };
        Ok(SelectItem::Expr { expr, alias })
    }

    fn update(&mut self) -> Result<Statement> {
        let table = self.ident("table name")?;
        self.expect_kw("SET")?;
        let mut assignments = Vec::new();
        loop {
            let col = self.ident("column name")?;
            self.expect(&Tok::Eq, "'='")?;
            assignments.push((col, self.expr()?));
            if !self.eat(&Tok::Comma) {
                break;
            }
        }
        let selection = if self.eat_kw("WHERE") { Some(self.expr()?) } else { None };
        Ok(Statement::Update(Update { table, assignments, selection }))
    }

    fn delete(&mut self) -> Result<Statement> {
        self.expect_kw("FROM")?;
        let table = self.ident("table name")?;
        let selection = if self.eat_kw("WHERE") { Some(self.expr()?) } else { None };
        Ok(Statement::Delete(Delete { table, selection }))
    }

    // ---- expressions -----------------------------------------------------

    fn expr(&mut self) -> Result<Expr> {
        self.or_expr()
    }

    fn or_expr(&mut self) -> Result<Expr> {
        let mut left = self.and_expr()?;
        while self.eat_kw("OR") {
            let right = self.and_expr()?;
            left = binary(BinaryOp::Or, left, right);
        }
        Ok(left)
    }

    fn and_expr(&mut self) -> Result<Expr> {
        let mut left = self.not_expr()?;
        while self.eat_kw("AND") {
            let right = self.not_expr()?;
            left = binary(BinaryOp::And, left, right);
        }
        Ok(left)
    }

    fn not_expr(&mut self) -> Result<Expr> {
        if self.eat_kw("NOT") {
            let inner = self.not_expr()?;
            return Ok(Expr::Unary { op: UnaryOp::Not, expr: Box::new(inner) });
        }
        self.comparison()
    }

    fn comparison(&mut self) -> Result<Expr> {
        let left = self.concat()?;
        let op = match self.peek() {
            Tok::Eq => Some(BinaryOp::Eq),
            Tok::NotEq => Some(BinaryOp::NotEq),
            Tok::Lt => Some(BinaryOp::Lt),
            Tok::LtEq => Some(BinaryOp::LtEq),
            Tok::Gt => Some(BinaryOp::Gt),
            Tok::GtEq => Some(BinaryOp::GtEq),
            _ => None,
        };
        if let Some(op) = op {
            self.advance();
            let right = self.concat()?;
            return Ok(binary(op, left, right));
        }
        if self.eat_kw("IS") {
            let negated = self.eat_kw("NOT");
            self.expect_kw("NULL")?;
            return Ok(Expr::IsNull { expr: Box::new(left), negated });
        }
        let negated = if self.is_kw("NOT") && ["IN", "BETWEEN", "LIKE", "ILIKE"].iter().any(|k| self.is_kw_at(1, k)) {
            self.advance();
            true
        } else {
            false
        };
        if self.eat_kw("IN") {
            self.expect(&Tok::LParen, "'(' after IN")?;
            if self.is_kw("SELECT") {
                return Err(self.unsupported("subqueries are not supported"));
            }
            let mut list = vec![self.expr()?];
            while self.eat(&Tok::Comma) {
                list.push(self.expr()?);
            }
            self.expect(&Tok::RParen, "')'")?;
            return Ok(Expr::InList { expr: Box::new(left), list, negated });
        }
        if self.eat_kw("BETWEEN") {
            let low = self.concat()?;
            self.expect_kw("AND")?;
            let high = self.concat()?;
            return Ok(Expr::Between { expr: Box::new(left), low: Box::new(low), high: Box::new(high), negated });
        }
        let ci = self.is_kw("ILIKE");
        if self.eat_kw("LIKE") || self.eat_kw("ILIKE") {
            let pattern = self.concat()?;
            return Ok(Expr::Like { expr: Box::new(left), pattern: Box::new(pattern), negated, case_insensitive: ci });
        }
        if negated {
            return Err(self.error("expected IN, BETWEEN or LIKE after NOT"));
        }
        Ok(left)
    }

    fn concat(&mut self) -> Result<Expr> {
        let mut left = self.additive()?;
        while self.eat(&Tok::Concat) {
            let right = self.additive()?;
            left = binary(BinaryOp::Concat, left, right);
        }
        Ok(left)
    }

    fn additive(&mut self) -> Result<Expr> {
        let mut left = self.multiplicative()?;
        loop {
            let op = match self.peek() {
                Tok::Plus => BinaryOp::Add,
                Tok::Minus => BinaryOp::Sub,
                _ => return Ok(left),
            };
            self.advance();
            let right = self.multiplicative()?;
            left = binary(op, left, right);
        }
    }

    fn multiplicative(&mut self) -> Result<Expr> {
        let mut left = self.unary()?;
        loop {
            let op = match self.peek() {
                Tok::Star => BinaryOp::Mul,
                Tok::Slash => BinaryOp::Div,
                Tok::Percent => BinaryOp::Mod,
                _ => return Ok(left),
            };
            self.advance();
            let right = self.unary()?;
            left = binary(op, left, right);
        }
    }

    fn unary(&mut self) -> Result<Expr> {
        if self.eat(&Tok::Minus) {
            let inner = self.unary()?;
            // Fold negative numeric literals right away so `-5` stays a literal.
            return Ok(match inner {
                Expr::Literal(Value::Int(i)) if i != i64::MIN => Expr::Literal(Value::Int(-i)),
                Expr::Literal(Value::Float(f)) => Expr::Literal(Value::Float(-f)),
                other => Expr::Unary { op: UnaryOp::Neg, expr: Box::new(other) },
            });
        }
        if self.eat(&Tok::Plus) {
            return self.unary();
        }
        self.primary()
    }

    fn primary(&mut self) -> Result<Expr> {
        match self.peek().clone() {
            Tok::Int(i) => {
                self.advance();
                Ok(Expr::Literal(Value::Int(i)))
            }
            Tok::Float(f) => {
                self.advance();
                Ok(Expr::Literal(Value::Float(f)))
            }
            Tok::Str(s) => {
                self.advance();
                Ok(Expr::Literal(Value::Text(s.into())))
            }
            Tok::Param => {
                self.advance();
                let idx = self.params;
                self.params += 1;
                Ok(Expr::Param(idx))
            }
            Tok::LParen => {
                self.advance();
                if self.is_kw("SELECT") {
                    return Err(self.unsupported("subqueries are not supported"));
                }
                let e = self.expr()?;
                self.expect(&Tok::RParen, "')'")?;
                Ok(e)
            }
            Tok::QuotedIdent(name) => {
                self.advance();
                self.column_tail(name)
            }
            Tok::Ident(word) => {
                let upper = word.to_ascii_uppercase();
                match upper.as_str() {
                    "NULL" => {
                        self.advance();
                        return Ok(Expr::Literal(Value::Null));
                    }
                    "TRUE" => {
                        self.advance();
                        return Ok(Expr::Literal(Value::Bool(true)));
                    }
                    "FALSE" => {
                        self.advance();
                        return Ok(Expr::Literal(Value::Bool(false)));
                    }
                    "CASE" => return Err(self.unsupported("CASE expressions are not supported yet")),
                    _ => {}
                }
                if self.peek_at(1) == &Tok::LParen {
                    self.advance();
                    return self.function_call(upper);
                }
                if is_reserved(&word) {
                    return Err(self.error("expected an expression"));
                }
                self.advance();
                self.column_tail(word)
            }
            _ => Err(self.error("expected an expression")),
        }
    }

    /// Handles an optional `table.column` qualifier; the table part is ignored
    /// because a Flat-SQL query always reads exactly one table.
    fn column_tail(&mut self, first: String) -> Result<Expr> {
        if self.eat(&Tok::Dot) {
            let col = self.ident("column name")?;
            return Ok(Expr::Column(col));
        }
        Ok(Expr::Column(first))
    }

    fn function_call(&mut self, name: String) -> Result<Expr> {
        self.expect(&Tok::LParen, "'('")?;
        if self.eat(&Tok::Star) {
            self.expect(&Tok::RParen, "')'")?;
            if name != "COUNT" {
                return Err(Error::Parse {
                    pos: self.toks[self.pos].pos,
                    msg: format!("{name}(*) is not valid, only COUNT(*)"),
                });
            }
            return Ok(Expr::Function { name, args: vec![], star: true, distinct: false });
        }
        let distinct = self.eat_kw("DISTINCT");
        let mut args = Vec::new();
        if !self.at(&Tok::RParen) {
            args.push(self.expr()?);
            while self.eat(&Tok::Comma) {
                args.push(self.expr()?);
            }
        }
        self.expect(&Tok::RParen, "')' closing the function call")?;
        Ok(Expr::Function { name, args, star: false, distinct })
    }
}

fn binary(op: BinaryOp, left: Expr, right: Expr) -> Expr {
    Expr::Binary { op, left: Box::new(left), right: Box::new(right) }
}

fn is_reserved(word: &str) -> bool {
    RESERVED.iter().any(|r| r.eq_ignore_ascii_case(word))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn one(sql: &str) -> Statement {
        parse_one(sql).unwrap_or_else(|e| panic!("{sql}: {e}"))
    }

    #[test]
    fn create_table_with_types_and_constraints() {
        let Statement::CreateTable(ct) =
            one("CREATE TABLE IF NOT EXISTS items (id INT PRIMARY KEY, name VARCHAR(40) NOT NULL, price REAL)")
        else {
            panic!()
        };
        assert!(ct.if_not_exists);
        assert_eq!(ct.columns.len(), 3);
        assert!(ct.columns[0].primary_key);
        assert_eq!(ct.columns[1].data_type, DataType::Text);
        assert!(ct.columns[1].not_null);
        assert_eq!(ct.columns[2].data_type, DataType::Float);
    }

    #[test]
    fn table_level_primary_key() {
        let Statement::CreateTable(ct) = one("CREATE TABLE t (a TEXT, b INT, PRIMARY KEY (a))") else { panic!() };
        assert_eq!(ct.primary_key.as_deref(), Some("a"));
    }

    #[test]
    fn select_full_clause_set() {
        let Statement::Select(s) = one("SELECT DISTINCT category, COUNT(*) AS n, SUM(price) total FROM items \
             WHERE price > 10 AND NOT name LIKE 'x%' GROUP BY category HAVING COUNT(*) > 1 \
             ORDER BY n DESC, category LIMIT 5 OFFSET 2")
        else {
            panic!()
        };
        assert!(s.distinct);
        assert_eq!(s.projection.len(), 3);
        assert_eq!(s.from.as_deref(), Some("items"));
        assert!(s.selection.is_some());
        assert_eq!(s.group_by.len(), 1);
        assert!(s.having.is_some());
        assert_eq!(s.order_by.len(), 2);
        assert!(s.order_by[0].desc && !s.order_by[1].desc);
        assert_eq!(s.limit, Some(Expr::Literal(Value::Int(5))));
        assert_eq!(s.offset, Some(Expr::Literal(Value::Int(2))));
        match &s.projection[2] {
            SelectItem::Expr { alias, .. } => assert_eq!(alias.as_deref(), Some("total")),
            _ => panic!(),
        }
    }

    #[test]
    fn precedence_of_operators() {
        let Statement::Select(s) = one("SELECT 1 + 2 * 3 = 7 OR a AND b") else { panic!() };
        let SelectItem::Expr { expr, .. } = &s.projection[0] else { panic!() };
        assert_eq!(expr.to_string(), "((1 + (2 * 3)) = 7) OR (a AND b)");
    }

    #[test]
    fn between_in_like_and_null_checks() {
        let Statement::Select(s) =
            one("SELECT * FROM t WHERE a BETWEEN 1 AND 5 AND b NOT IN (1, 2) AND c IS NOT NULL AND d ILIKE '%x'")
        else {
            panic!()
        };
        let text = s.selection.unwrap().to_string();
        assert!(text.contains("a BETWEEN 1 AND 5"));
        assert!(text.contains("b NOT IN (1, 2)"));
        assert!(text.contains("c IS NOT NULL"));
        assert!(text.contains("d ILIKE '%x'"));
    }

    #[test]
    fn insert_variants() {
        let Statement::Insert(i) = one("INSERT OR REPLACE INTO t (a, b) VALUES (1, 'x'), (2, NULL)") else { panic!() };
        assert_eq!(i.on_conflict, OnConflict::Replace);
        assert_eq!(i.rows.len(), 2);
        let Statement::Insert(i) = one("REPLACE INTO t VALUES (-1)") else { panic!() };
        assert_eq!(i.rows[0][0], Expr::Literal(Value::Int(-1)));
        assert!(i.columns.is_none());
    }

    #[test]
    fn update_delete_and_transactions() {
        assert!(matches!(one("UPDATE t SET a = a + 1, b = 'x' WHERE id = 3"), Statement::Update(_)));
        assert!(matches!(one("DELETE FROM t WHERE id IN (1, 2)"), Statement::Delete(_)));
        assert_eq!(
            parse("BEGIN; COMMIT; ROLLBACK;").unwrap(),
            vec![Statement::Begin, Statement::Commit, Statement::Rollback]
        );
    }

    #[test]
    fn params_are_numbered() {
        let stmt = one("SELECT * FROM t WHERE a = ? AND b > ?");
        assert_eq!(param_count(&stmt), 2);
    }

    #[test]
    fn flat_sql_rejects_relations() {
        for sql in [
            "SELECT * FROM a JOIN b ON a.id = b.id",
            "SELECT * FROM a, b",
            "CREATE TABLE t (id INT PRIMARY KEY, owner INT REFERENCES users(id))",
            "CREATE TABLE t (id INT, FOREIGN KEY (id) REFERENCES u(id))",
            "SELECT * FROM t WHERE id IN (SELECT id FROM u)",
        ] {
            assert!(matches!(parse(sql), Err(Error::Unsupported(_))), "{sql}");
        }
    }

    #[test]
    fn syntax_errors_have_positions() {
        match parse("SELECT FROM t") {
            Err(Error::Parse { pos, .. }) => assert_eq!(pos, 7),
            other => panic!("{other:?}"),
        }
        assert!(parse("SELECT * FROM").is_err());
        assert!(parse("CREATE TABLE t (id BLOB)").is_err());
        assert!(parse("SELECT 1 SELECT 2").is_err());
    }

    #[test]
    fn multiple_statements() {
        let stmts = parse("CREATE TABLE t (id INT PRIMARY KEY); INSERT INTO t VALUES (1);; SELECT * FROM t").unwrap();
        assert_eq!(stmts.len(), 3);
    }
}
