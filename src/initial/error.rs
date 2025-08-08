use {
    crate::{
        parser::ParseError,
        format::{
            Debug, Display, Formatter,
        },
    }
};

#[derive(Clone, Eq, Hash, PartialEq)]
pub enum ErrorKind<'error> {
    ArgumentParse(ParseError<'error>),
}

impl<'error> Display for ErrorKind<'error> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ErrorKind::ArgumentParse(e) => {
                write!(f, "failed to parse arguments: {}.", e)
            },
        }
    }
}

impl<'error> Debug for ErrorKind<'error> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}