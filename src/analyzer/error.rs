use crate::{
    format::{Display, Formatter, Result, Show, Stencil},
    scanner::Token,
};

#[derive(Clone)]
pub enum ErrorKind<'error> {
    InvalidOperation(Token<'error>),
    InvalidType,
    InvalidTarget,
    InvalidPrimitiveArity {
        name: String,
        expected: String,
        found: usize,
    },
    InvalidPrimitiveContext {
        name: String,
        expected: String,
    },
}

impl<'error> Display for ErrorKind<'error> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            ErrorKind::InvalidOperation(token) => {
                write!(
                    f,
                    "invalid operation token: {}.",
                    token.format(Stencil::default())
                )
            }
            ErrorKind::InvalidType => {
                write!(f, "invalid type.")
            }
            ErrorKind::InvalidTarget => {
                write!(f, "invalid target.")
            }
            ErrorKind::InvalidPrimitiveArity {
                name,
                expected,
                found,
            } => {
                write!(
                    f,
                    "invalid '{}' arity: expected {}, found {}.",
                    name, expected, found,
                )
            }
            ErrorKind::InvalidPrimitiveContext { name, expected } => {
                write!(f, "invalid '{}' usage: expected {}.", name, expected)
            }
        }
    }
}
