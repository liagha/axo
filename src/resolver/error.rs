use {
    crate::{
        format::{Show, Display, Formatter, Result},
        scanner::Token,
        data::Str,
    }
};

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
    DefinedMember {
        target: Token<'error>,
        member: Token<'error>,
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

            ErrorKind::DefinedMember { target, member } => {
                format!("the member `{}` is already defined in `{}`.", member.format(verbosity), target.format(verbosity))
            }
        }.into()
    }
}

impl<'error> Display for ErrorKind<'error> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "{}", self.format(1))
    }
}