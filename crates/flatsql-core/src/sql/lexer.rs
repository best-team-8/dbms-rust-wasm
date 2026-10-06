//! Hand written tokenizer for Flat-SQL.

use crate::error::{Error, Result};

#[derive(Debug, Clone, PartialEq)]
pub enum Tok {
    /// Unquoted identifier or keyword (keywords are resolved by the parser).
    Ident(String),
    /// `"quoted"` or `` `quoted` `` identifier, never treated as a keyword.
    QuotedIdent(String),
    Int(i64),
    Float(f64),
    Str(String),
    /// `?` positional parameter of a prepared statement.
    Param,
    Comma,
    LParen,
    RParen,
    Semicolon,
    Dot,
    Star,
    Plus,
    Minus,
    Slash,
    Percent,
    Eq,
    NotEq,
    Lt,
    LtEq,
    Gt,
    GtEq,
    Concat,
    Eof,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub tok: Tok,
    /// Byte offset of the first character of the token.
    pub pos: usize,
}

pub fn tokenize(src: &str) -> Result<Vec<Token>> {
    let b = src.as_bytes();
    let mut out = Vec::new();
    let mut i = 0;

    while i < b.len() {
        let c = b[i];
        let next = b.get(i + 1).copied();
        let start = i;
        let simple = |tok: Tok, len: usize, out: &mut Vec<Token>| {
            out.push(Token { tok, pos: start });
            start + len
        };

        i = match c {
            b' ' | b'\t' | b'\n' | b'\r' => i + 1,
            b'-' if next == Some(b'-') => match src[i..].find('\n') {
                Some(off) => i + off + 1,
                None => b.len(),
            },
            b'/' if next == Some(b'*') => match src[i + 2..].find("*/") {
                Some(off) => i + 2 + off + 2,
                None => return Err(lex_err(i, "unterminated block comment")),
            },
            b'\'' => {
                let (s, end) = read_quoted(src, i, b'\'').ok_or_else(|| lex_err(i, "unterminated string literal"))?;
                out.push(Token { tok: Tok::Str(s), pos: start });
                end
            }
            b'"' | b'`' => {
                let (s, end) = read_quoted(src, i, c).ok_or_else(|| lex_err(i, "unterminated quoted identifier"))?;
                if s.is_empty() {
                    return Err(lex_err(i, "empty quoted identifier"));
                }
                out.push(Token { tok: Tok::QuotedIdent(s), pos: start });
                end
            }
            b'0'..=b'9' => read_number(src, i, &mut out)?,
            b'.' if next.is_some_and(|n| n.is_ascii_digit()) => read_number(src, i, &mut out)?,
            c if c.is_ascii_alphabetic() || c == b'_' || c >= 0x80 => {
                let mut j = i;
                while j < b.len() && (b[j].is_ascii_alphanumeric() || b[j] == b'_' || b[j] >= 0x80) {
                    j += 1;
                }
                out.push(Token { tok: Tok::Ident(src[i..j].to_string()), pos: start });
                j
            }
            b'?' => simple(Tok::Param, 1, &mut out),
            b',' => simple(Tok::Comma, 1, &mut out),
            b'(' => simple(Tok::LParen, 1, &mut out),
            b')' => simple(Tok::RParen, 1, &mut out),
            b';' => simple(Tok::Semicolon, 1, &mut out),
            b'.' => simple(Tok::Dot, 1, &mut out),
            b'*' => simple(Tok::Star, 1, &mut out),
            b'+' => simple(Tok::Plus, 1, &mut out),
            b'-' => simple(Tok::Minus, 1, &mut out),
            b'/' => simple(Tok::Slash, 1, &mut out),
            b'%' => simple(Tok::Percent, 1, &mut out),
            b'=' if next == Some(b'=') => simple(Tok::Eq, 2, &mut out),
            b'=' => simple(Tok::Eq, 1, &mut out),
            b'!' if next == Some(b'=') => simple(Tok::NotEq, 2, &mut out),
            b'<' if next == Some(b'=') => simple(Tok::LtEq, 2, &mut out),
            b'<' if next == Some(b'>') => simple(Tok::NotEq, 2, &mut out),
            b'<' => simple(Tok::Lt, 1, &mut out),
            b'>' if next == Some(b'=') => simple(Tok::GtEq, 2, &mut out),
            b'>' => simple(Tok::Gt, 1, &mut out),
            b'|' if next == Some(b'|') => simple(Tok::Concat, 2, &mut out),
            _ => {
                let ch = src[i..].chars().next().unwrap_or('?');
                return Err(lex_err(i, &format!("unexpected character '{ch}'")));
            }
        };
    }

    out.push(Token { tok: Tok::Eof, pos: src.len() });
    Ok(out)
}

fn lex_err(pos: usize, msg: &str) -> Error {
    Error::Lex { pos, msg: msg.to_string() }
}

/// Reads a quoted string starting at `start` (which holds the quote byte).
/// A doubled quote inside the literal is an escaped quote.
fn read_quoted(src: &str, start: usize, quote: u8) -> Option<(String, usize)> {
    let b = src.as_bytes();
    let mut out = String::new();
    let mut seg = start + 1;
    let mut i = start + 1;
    while i < b.len() {
        if b[i] == quote {
            out.push_str(&src[seg..i]);
            if b.get(i + 1) == Some(&quote) {
                out.push(quote as char);
                i += 2;
                seg = i;
                continue;
            }
            return Some((out, i + 1));
        }
        i += 1;
    }
    None
}

fn read_number(src: &str, start: usize, out: &mut Vec<Token>) -> Result<usize> {
    let b = src.as_bytes();
    let mut i = start;
    let mut is_float = false;
    while i < b.len() && b[i].is_ascii_digit() {
        i += 1;
    }
    if i < b.len() && b[i] == b'.' {
        is_float = true;
        i += 1;
        while i < b.len() && b[i].is_ascii_digit() {
            i += 1;
        }
    }
    if i < b.len() && (b[i] == b'e' || b[i] == b'E') {
        let mut j = i + 1;
        if j < b.len() && (b[j] == b'+' || b[j] == b'-') {
            j += 1;
        }
        if j < b.len() && b[j].is_ascii_digit() {
            is_float = true;
            i = j;
            while i < b.len() && b[i].is_ascii_digit() {
                i += 1;
            }
        }
    }
    if i < b.len() && (b[i].is_ascii_alphabetic() || b[i] == b'_') {
        return Err(lex_err(start, "invalid numeric literal"));
    }
    let text = &src[start..i];
    let tok = if is_float {
        Tok::Float(text.parse().map_err(|_| lex_err(start, "invalid numeric literal"))?)
    } else {
        match text.parse::<i64>() {
            Ok(v) => Tok::Int(v),
            // Integers that do not fit into i64 degrade to FLOAT, like in SQLite.
            Err(_) => Tok::Float(text.parse().map_err(|_| lex_err(start, "invalid numeric literal"))?),
        }
    };
    out.push(Token { tok, pos: start });
    Ok(i)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn kinds(sql: &str) -> Vec<Tok> {
        tokenize(sql).unwrap().into_iter().map(|t| t.tok).collect()
    }

    #[test]
    fn tokenizes_basic_select() {
        assert_eq!(
            kinds("SELECT a, b FROM t WHERE x >= 1.5;"),
            vec![
                Tok::Ident("SELECT".into()),
                Tok::Ident("a".into()),
                Tok::Comma,
                Tok::Ident("b".into()),
                Tok::Ident("FROM".into()),
                Tok::Ident("t".into()),
                Tok::Ident("WHERE".into()),
                Tok::Ident("x".into()),
                Tok::GtEq,
                Tok::Float(1.5),
                Tok::Semicolon,
                Tok::Eof,
            ]
        );
    }

    #[test]
    fn strings_comments_and_operators() {
        assert_eq!(
            kinds("'it''s' -- comment\n /* block */ <> != || \"Weird Col\" ?"),
            vec![
                Tok::Str("it's".into()),
                Tok::NotEq,
                Tok::NotEq,
                Tok::Concat,
                Tok::QuotedIdent("Weird Col".into()),
                Tok::Param,
                Tok::Eof,
            ]
        );
    }

    #[test]
    fn numbers() {
        assert_eq!(
            kinds("42 .5 1e3 2.5E-1"),
            vec![Tok::Int(42), Tok::Float(0.5), Tok::Float(1000.0), Tok::Float(0.25), Tok::Eof]
        );
        assert!(tokenize("12abc").is_err());
    }

    #[test]
    fn unicode_text_and_identifiers() {
        assert_eq!(
            kinds("zażółć = 'gęślą'"),
            vec![Tok::Ident("zażółć".into()), Tok::Eq, Tok::Str("gęślą".into()), Tok::Eof]
        );
    }

    #[test]
    fn reports_position_of_errors() {
        match tokenize("SELECT #") {
            Err(Error::Lex { pos, .. }) => assert_eq!(pos, 7),
            other => panic!("unexpected {other:?}"),
        }
        assert!(tokenize("'open").is_err());
    }
}
