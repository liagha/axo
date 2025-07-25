use {
    crate::{
        axo_parser::ParseError,
        format::{
            Debug, Display, Formatter,
        },
    }
};

#[derive(Clone, Eq, Hash, PartialEq)]
pub enum ErrorKind {
    ArgumentParse(ParseError),
}

impl Display for ErrorKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ErrorKind::ArgumentParse(e) => {
                write!(f, "failed to parse arguments: {}.", e)
            },
        }
    }
}

impl Debug for ErrorKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}