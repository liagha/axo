use {
    crate::{
        format::{
            Display, Debug, 
            Formatter, Result
        },
        resolver::{
            analyzer::AnalyzeError,
            checker::{CheckError},
        },
        scanner::Token,
    }
};

#[derive(Clone, Debug)]
pub enum ErrorKind<'error> {
    UndefinedSymbol {
        query: Token<'error>,
    },
    MissingMember {
        target: Token<'error>,
        member: Token<'error>,
    },
    UndefinedMember {
        target: Token<'error>,
        member: Token<'error>,
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
                write!(f, "undefined symbol: `{}`.", query)
            },
            ErrorKind::MissingMember { target, member } => {
                write!(f, "the member `{}` is missing from `{}`.", member, target)
            }

            ErrorKind::UndefinedMember { target, member } => {
                write!(f, "the member `{}` doesn't exist in `{}`.", member, target)
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
