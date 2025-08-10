use {
    crate::{
        format::{Debug, Display, Formatter, Result},
        scanner::TokenKind,
    }
};

#[derive(Clone, Eq, Hash, PartialEq)]
pub enum ErrorKind<'error> {
    ExpectedCondition,
    ExpectedBody,
    MissingSeparator(TokenKind<'error>),
    UnclosedDelimiter(TokenKind<'error>),
    UnexpectedPunctuation,
    UnexpectedToken(TokenKind<'error>),
}

impl<'error> Display for ErrorKind<'error> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            ErrorKind::ExpectedCondition => write!(f, "expected condition."),
            ErrorKind::UnexpectedPunctuation => write!(f, "unexpected punctuation."),
            ErrorKind::ExpectedBody => write!(f, "expected body."),
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

impl<'error> Debug for ErrorKind<'error> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "{}", self)
    }
}