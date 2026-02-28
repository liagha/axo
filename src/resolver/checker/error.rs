use crate::{
    format::{self, Display},
    resolver::checker::types::Type,
    scanner::Token,
};

#[derive(Clone, Debug)]
pub enum ErrorKind<'error> {
    Mismatch(Type<'error>, Type<'error>),
    InvalidOperation(Token<'error>),
}

impl Display for ErrorKind<'_> {
    fn fmt(&self, f: &mut format::Formatter<'_>) -> format::Result {
        match self {
            ErrorKind::Mismatch(this, other) => {
                write!(f, "expected {:?} but got {:?}.", this, other)
            }
            ErrorKind::InvalidOperation(token) => {
                write!(f, "invalid operation for operand types: {}.", token)
            }
        }
    }
}
