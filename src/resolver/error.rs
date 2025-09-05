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
                write!(f, "undefined symbol: `{}`.", query)
            },
            ErrorKind::MissingMember { target, members } => {
                let pretty = members.iter().map(|member| format!("{}", member)).collect::<Vec<_>>().join(", ");

                if members.len() == 1 {
                    write!(f, "the member `{}` is missing from `{}`.", pretty, target)
                } else {
                    write!(f, "the members `{}` are missing from `{}`.", pretty, target)
                }
            }

            ErrorKind::UndefinedMember { target, members } => {
                let pretty = members.iter().map(|member| format!("{}", member)).collect::<Vec<_>>().join(", ");

                if members.len() == 1 {
                    write!(f, "the member `{}` doesn't exist in `{}`.", pretty, target)
                } else {
                    write!(f, "the members `{}` don't exist in `{}`.", pretty, target)
                }
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
