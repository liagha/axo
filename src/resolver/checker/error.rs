use crate::format::{self, Display};
use crate::resolver::checker::types::Type;

#[derive(Clone, Debug)]
pub enum ErrorKind<'error> {
    Mismatch(Type<'error>, Type<'error>),
}

impl Display for ErrorKind<'_> {
    fn fmt(&self, f: &mut format::Formatter<'_>) -> format::Result {
        match self { 
            ErrorKind::Mismatch(this, other) => write!(f, "expected {:?} but got {:?}.", this, other),
        }
    }
}