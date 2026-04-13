use crate::{
    format::{Display, Formatter, Result, Show, Stencil},
    scanner::TokenKind,
};

#[derive(Clone, Eq, Hash, PartialEq)]
pub enum ErrorKind<'error> {
    Expected(&'static str),
    ExpectedName,
    ExpectedHead,
    ExpectedBody,
    ExpectedAnnotation,
    ExpectedToken(TokenKind<'error>),
    MissingSeparator(TokenKind<'error>),
    UnclosedDelimiter(TokenKind<'error>),
    UnexpectedToken(TokenKind<'error>),
}

impl<'error> Display for ErrorKind<'error> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            ErrorKind::Expected(label) => {
                write!(f, "expected {}.", label)
            }
            ErrorKind::ExpectedName => {
                write!(f, "expected a name.")
            }
            ErrorKind::ExpectedHead => {
                write!(f, "expected a head.")
            }
            ErrorKind::ExpectedBody => {
                write!(f, "expected a body.")
            }
            ErrorKind::ExpectedAnnotation => {
                write!(f, "expected an annotation.")
            }
            ErrorKind::ExpectedToken(kind) => {
                write!(
                    f,
                    "expected `{}`.",
                    kind.format(Stencil::default())
                )
            }
            ErrorKind::MissingSeparator(kind) => {
                write!(
                    f,
                    "expected separator `{}`.",
                    kind.format(Stencil::default())
                )
            }
            ErrorKind::UnclosedDelimiter(delimiter) => {
                write!(
                    f,
                    "unclosed delimiter `{}`.",
                    delimiter.format(Stencil::default())
                )
            }
            ErrorKind::UnexpectedToken(token) => {
                write!(
                    f,
                    "unexpected token `{}`.",
                    token.format(Stencil::default())
                )
            }
        }
    }
}
