use {
    broccli::{
        Color, TextStyle
    },
    crate::{
        axo_lexer::{
            TokenKind, Token, PunctuationKind
        },
        axo_parser::Element,
    }
};

#[derive(Clone, Eq, Hash, PartialEq)]
pub enum ErrorKind {
    DanglingElse,
    ExpectedToken(TokenKind),
    InvalidDelimiter,
    MissingSeparator(TokenKind),
    MissingSeparators(Vec<TokenKind>),
    MissingOperand,
    InconsistentSeparators,
    UnclosedDelimiter(Token),
    UnterminatedGroup,
    UnterminatedCollection,
    UnterminatedBlock,
    UnimplementedToken(TokenKind),
    UnexpectedToken(TokenKind),
    UnexpectedEndOfFile,
}


impl core::fmt::Display for ErrorKind {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            ErrorKind::ExpectedToken(expected) => {
                write!(f, "expected token {:?}", expected)
            }
            ErrorKind::InvalidDelimiter => {
                write!(f, "invalid delimiter")
            },
            ErrorKind::DanglingElse => {
                write!(f, "can't have an else without conditional.")
            }
            ErrorKind::MissingSeparator(kind) => {
                write!(f, "expected separator `{}`.", kind)
            }
            ErrorKind::MissingSeparators(separators) => {
                write!(f, "expected one of these separators: `{:?}`.", separators)
            }
            ErrorKind::MissingOperand => {
                write!(f, "missing operand.")
            }
            ErrorKind::InconsistentSeparators => {
                write!(f, "inconsistent separators.")
            }
            ErrorKind::UnexpectedEndOfFile => {
                write!(f, "unexpected end of file.")
            }
            ErrorKind::UnclosedDelimiter(delimiter) => {
                write!(f, "unclosed delimiter `{:?}`.", delimiter)
            }
            ErrorKind::UnterminatedGroup => {
                write!(f, "unterminated group.")
            }
            ErrorKind::UnterminatedCollection => {
                write!(f, "unterminated collection.")
            }
            ErrorKind::UnterminatedBlock => {
                write!(f, "unterminated block.")
            }
            ErrorKind::UnimplementedToken(token) => {
                write!(f, "unimplemented token `{}`.", token)
            }
            ErrorKind::UnexpectedToken(token) => {
                write!(f, "unexpected token `{}`.", token)
            }
        }
    }
}

impl core::fmt::Debug for ErrorKind {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self)
    }
}

impl core::error::Error for ErrorKind {}