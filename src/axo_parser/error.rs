#![allow(dead_code)]

use std::fmt::Formatter;
use std::fs::read_to_string;
use broccli::{Color, TextStyle};
use crate::axo_lexer::{TokenKind, Token, Span, PunctuationKind};
use crate::axo_parser::{Expr};
use crate::axo_parser::state::{Position, Context, ContextKind};

#[derive(Clone, Eq, Hash, PartialEq)]
pub enum ErrorKind {
    DanglingElse,
    MissingSeparator(TokenKind),
    UnclosedDelimiter(Token),
    UnimplementedToken(TokenKind),
    UnexpectedToken(TokenKind),
    InvalidSyntaxPattern(String),
    ExpectedSyntax(ContextKind),
    UnexpectedEndOfFile,
}


impl core::fmt::Display for ErrorKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        match self {
            ErrorKind::DanglingElse => {
                write!(f, "can't have an else without conditional")
            }
            ErrorKind::MissingSeparator(kind) => {
                write!(f, "expected separator {}", kind)
            }
            ErrorKind::InvalidSyntaxPattern(m) => {
                write!(f, "invalid syntax pattern: '{}'", m)
            }
            ErrorKind::UnexpectedEndOfFile => {
                write!(f, "unexpected end of file")
            }
            ErrorKind::UnclosedDelimiter(delimiter) => {
                write!(f, "unclosed delimiter '{:?}'", delimiter)
            }
            ErrorKind::UnimplementedToken(token) => {
                write!(f, "unimplemented token '{}'", token)
            }
            ErrorKind::UnexpectedToken(token) => {
                write!(f, "unexpected token '{}'", token)
            }
            ErrorKind::ExpectedSyntax(kind) => {
                write!(f, "expected syntax '{:?}' but didn't get it", kind)
            }
        }
    }
}

impl core::fmt::Debug for ErrorKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self)
    }
}

impl core::error::Error for ErrorKind {}