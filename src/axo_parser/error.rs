use {
    broccli::{
        Color, TextStyle
    },
    
    crate::{
        format::{Debug, Display, Formatter, Result},
        axo_lexer::{
            TokenKind, Token, PunctuationKind
        },
        axo_parser::Element,
    }
};

#[derive(Clone, Eq, Hash, PartialEq)]
pub enum ErrorKind {
    ExpectedCondition,
    ExpectedBody,
    PatternError,
    DanglingElse,
    MissingSeparator(TokenKind),
    UnclosedDelimiter(Token),
    UnexpectedToken(TokenKind),
}

impl Display for ErrorKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            ErrorKind::ExpectedCondition => write!(f, "expected condition"),
            ErrorKind::ExpectedBody => write!(f, "expected body"),
            ErrorKind::PatternError => write!(f, "invalid pattern syntax"),
            ErrorKind::DanglingElse => {
                write!(f, "can't have an else without conditional.")
            }
            ErrorKind::MissingSeparator(kind) => {
                write!(f, "expected separator `{:?}`.", kind)
            }
            ErrorKind::UnclosedDelimiter(delimiter) => {
                write!(f, "unclosed delimiter `{:?}`.", delimiter)
            }
            ErrorKind::UnexpectedToken(token) => {
                write!(f, "unexpected token `{:?}`.", token)
            }
        }
    }
}

impl Debug for ErrorKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "{}", self)
    }
}