use crate::{
    format::{Debug, Display, Formatter},
    scanner::Token,
};
use crate::analyzer::AnalyzeError;
use crate::checker::CheckError;
use crate::data::Str;
use crate::format::Show;

#[derive(Clone)]
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
    ImportConflict {
        symbol: Token<'error>,
    },
    PrivateSymbol {
        symbol: Token<'error>,
    },
    InvalidImportPath {
        query: Token<'error>,
    },
    Analyze {
        error: AnalyzeError<'error>,
    },
    Check {
        error: CheckError<'error>,
    },
}

impl<'error> Show<'error> for ErrorKind<'error> {
    type Verbosity = u8;
    
    fn format(&self, verbosity: Self::Verbosity) -> Str<'error> {
        match self {
            ErrorKind::UndefinedSymbol { query } => {
                format!("undefined symbol: `{}`.", query.format(verbosity))
            }
            ErrorKind::MissingMember { target, member } => {
                format!("the member `{}` is missing from `{}`.", member.format(verbosity), target.format(verbosity))
            }

            ErrorKind::UndefinedMember { target, member } => {
                format!("the member `{}` doesn't exist in `{}`.", member.format(verbosity), target.format(verbosity))
            }
            ErrorKind::ImportConflict { symbol } => {
                format!("import conflict: symbol `{}` already exists in this scope.", symbol.format(verbosity))
            }
            ErrorKind::PrivateSymbol { symbol } => {
                format!(
                    "cannot access private symbol `{}` from outside its module.",
                    symbol.format(verbosity)
                )
            }
            ErrorKind::InvalidImportPath { query } => {
                format!(
                    "invalid import path: `{}`. expected `use module.member`.",
                    query.format(verbosity)
                )
            }
            ErrorKind::Analyze { error } => {
                format!("{}", error.kind)
            }
            ErrorKind::Check { error } => {
                format!("{}", error.kind)
            }
        }.into()
    }
}

impl<'error> Display for ErrorKind<'error> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        write!(f, "{}", self.format(1))
    }
}