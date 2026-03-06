use crate::{
    format::{self, Display},
    scanner::Token,
};
use crate::checker::types::Type;
use crate::data::Str;
use crate::format::Show;

#[derive(Clone)]
pub enum ErrorKind<'error> {
    Mismatch(Type<'error>, Type<'error>),
    InvalidOperation(Token<'error>),
}

impl<'error> Show<'error> for ErrorKind<'error> {
    type Verbosity = u8;

    fn format(&self, verbosity: Self::Verbosity) -> Str<'error> {
        match self {
            ErrorKind::Mismatch(this, other) => {
                format!("expected `{}` but got `{}`.", this.format(verbosity), other.format(verbosity))
            }
            ErrorKind::InvalidOperation(token) => {
                format!("invalid operation for operand types: `{}`.", token.format(verbosity))
            }
        }.into()
    }
}

impl Display for ErrorKind<'_> {
    fn fmt(&self, f: &mut format::Formatter<'_>) -> format::Result {
        write!(f, "{}", self.format(0))
    }
}
