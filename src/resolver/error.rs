use {
    crate::{
        format::{
            Display, Debug, 
            Formatter, Result
        },

        scanner::Token,
    }
};

#[derive(Clone, Debug)]
pub enum ErrorKind {
    UndefinedSymbol {
        query: Token<'static>,
    },
    BindMismatch {
        candidate: Token<'static>,
    },
}

impl<'error> Display for ErrorKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            ErrorKind::UndefinedSymbol { query } => {
                write!(f, "undefined symbol: `{:?}`.", query)
            },
            ErrorKind::BindMismatch { candidate } => {
                write!(f, "slots of `{:?}` aren't matched correctly.", candidate)
            }
        }
    }
}
