use {
    crate::{format::Display, scanner::Token},
    std::fmt::Formatter,
};

#[derive(Clone, Debug)]
pub enum ErrorKind<'error> {
    InvalidOperation(Token<'error>),
    InvalidType,
    InvalidPrimitiveArity {
        name: String,
        expected: String,
        found: usize,
    },
    InvalidPrimitiveContext {
        name: String,
        expected: String,
    },
    Unimplemented,
}

impl Display for ErrorKind<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ErrorKind::InvalidOperation(token) => {
                write!(f, "invalid operation token: {}.", token)
            }
            ErrorKind::InvalidType => {
                write!(f, "invalid type.")
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
            ErrorKind::Unimplemented => {
                write!(f, "unimplemented operation.")
            }
        }
    }
}
