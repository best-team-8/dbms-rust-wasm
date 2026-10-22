//! Flat-SQL front-end: tokenizer, AST and parser.

pub mod ast;
pub mod lexer;
pub mod parser;

pub use ast::*;
pub use parser::{param_count, parse, parse_one};
