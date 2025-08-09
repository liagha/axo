use {
    crate::{
        format::{
            self,
            Debug, Display, Formatter,
        },
        parser::ParseError,
    }
};

#[derive(Clone, Eq, Hash, PartialEq)]
pub enum ErrorKind<'error> {
    ArgumentParse(ParseError<'error>),
}

impl<'error> Display for ErrorKind<'error> {
    fn fmt(&self, f: &mut Formatter<'_>) -> format::Result {
        match self {
            ErrorKind::ArgumentParse(e) => {
                write!(f, "failed to parse arguments: {}.", e)
            },
        }
    }
}

impl<'error> Debug for ErrorKind<'error> {
    fn fmt(&self, f: &mut Formatter<'_>) -> format::Result {
        write!(f, "{}", self)
    }
}