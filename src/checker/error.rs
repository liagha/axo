use crate::{
    checker::types::Type,
    data::Str,
    format::{self, Display, Show},
    parser::Element,
    scanner::Token,
};

#[derive(Clone)]
pub enum ErrorKind<'source> {
    Mismatch(Type<'source>, Type<'source>),
    InvalidOperation(Token<'source>),
    InvalidAnnotation(Element<'source>),
}

impl<'source> Show<'source> for ErrorKind<'source> {
    type Verbosity = u8;

    fn format(&self, verbosity: Self::Verbosity) -> Str<'source> {
        match self {
            ErrorKind::Mismatch(left, right) => {
                format!("expected `{}` but got `{}`.", left.format(verbosity), right.format(verbosity)).into()
            }
            ErrorKind::InvalidOperation(token) => {
                format!("invalid operation for operand types: `{}`.", token.format(verbosity)).into()
            }
            ErrorKind::InvalidAnnotation(element) => {
                format!("invalid type annotation: `{}`.", element.format(verbosity)).into()
            }
        }
    }
}

impl Display for ErrorKind<'_> {
    fn fmt(&self, formatter: &mut format::Formatter<'_>) -> format::Result {
        write!(formatter, "{}", self.format(0))
    }
}
