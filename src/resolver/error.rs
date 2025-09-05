use {
    crate::{
        format::{
            Display, Debug, 
            Formatter, Result
        },

        scanner::Token,
    }
};
use crate::resolver::analyzer::AnalyzeError;
use crate::resolver::checker::{CheckError, Type};

#[derive(Clone, Debug)]
pub enum ErrorKind<'error> {
    UndefinedSymbol {
        query: Token<'error>,
    },
    MissingMember {
        target: Token<'error>,
        members: Vec<Token<'error>>,
    },
    UndefinedMember {
        target: Token<'error>,
        members: Vec<Token<'error>>,
    },
    Analyze {
        error: AnalyzeError<'error>,
    },
    Check {
        error: CheckError<'error>,
    },
}

impl<'error> Display for ErrorKind<'error> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            ErrorKind::UndefinedSymbol { query } => {
                write!(f, "undefined symbol: `{:?}`.", query)
            },

            ErrorKind::MissingMember { target, members } => {
                write!(f, "the members `{:?}` is missing from `{:?}`.", members, target)
            }

            ErrorKind::UndefinedMember { target, members } => {
                write!(f, "the members `{:?}` don't exist in `{:?}`.", members, target)
            }
            ErrorKind::Analyze { error } => {
                write!(f, "{}", error.kind)
            }
            ErrorKind::Check { error } => {
                write!(f, "{}", error.kind)
            }
        }
    }
}
