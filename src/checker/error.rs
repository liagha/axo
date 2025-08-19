use crate::{
    checker::Type,
    format::{self, Display, Formatter}
};

#[derive(Clone, Eq, Hash, PartialEq)]
pub enum ErrorKind<'error: 'static> {
    Mismatch(Type<'error>, Type<'error>),
}

impl<'error> Display for ErrorKind<'error> {
    fn fmt(&self, f: &mut Formatter<'_>) -> format::Result {
        match self { 
            ErrorKind::Mismatch(expect, found) => write!(f, "expected `{}`, `{}`", expect, found),
        }
    }
} 