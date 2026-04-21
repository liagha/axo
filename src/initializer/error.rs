use crate::{
    format::{self, Debug, Display, Formatter},
    parser::ParseError,
    scanner::Token,
    tracker::TrackError,
};

#[derive(Clone, Eq, Hash, PartialEq)]
pub enum ErrorKind<'error> {
    Tracking(TrackError<'error>),
    Parse(ParseError<'error>),
    Argument(Token<'error>),
}

impl<'error> Display for ErrorKind<'error> {
    fn fmt(&self, f: &mut Formatter<'_>) -> format::Result {
        match self {
            ErrorKind::Tracking(error) => write!(f, "{}", error),
            ErrorKind::Parse(error) => write!(f, "{}", error),
            ErrorKind::Argument(_) => write!(f, "invalid argument"),
        }
    }
}

impl<'error> Debug for ErrorKind<'error> {
    fn fmt(&self, f: &mut Formatter<'_>) -> format::Result {
        write!(f, "{}", self)
    }
}
