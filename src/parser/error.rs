use crate::{
    format::{
        Show,
        Display,
        Stencil,
        Formatter, Result
    },
    scanner::TokenKind,
};

#[derive(Clone, Eq, Hash, PartialEq)]
pub enum ErrorKind<'error> {
    ExpectedName,
    ExpectedHead,
    ExpectedBody,
    ExpectedAnnotation,
    MissingSeparator(TokenKind<'error>),
    UnclosedDelimiter(TokenKind<'error>),
    UnexpectedToken(TokenKind<'error>),
}

impl<'error> Display for ErrorKind<'error> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            ErrorKind::ExpectedName => {
                write!(f, "expected a name.")
            },
            ErrorKind::ExpectedHead => {
                write!(f, "expected a head.")
            },
            ErrorKind::ExpectedBody => {
                write!(f, "expected a body.")
            },
            ErrorKind::ExpectedAnnotation => {
                write!(f, "expected an annotation.")
            },
            ErrorKind::MissingSeparator(kind) => {
                write!(f, "expected separator `{}`.", kind.format(Stencil::default()))
            }
            ErrorKind::UnclosedDelimiter(delimiter) => {
                write!(f, "unclosed delimiter `{}`.", delimiter.format(Stencil::default()))
            }
            ErrorKind::UnexpectedToken(token) => {
                write!(f, "unexpected token `{}`.", token.format(Stencil::default()))
            }
        }
    }
}