use {
    crate::{
        format::{
            Display, Debug, 
            Formatter, Result
        },

        axo_scanner::Token,
    }
};

#[derive(Clone, Debug)]
pub enum ErrorKind<'error> {
    UndefinedSymbol {
        query: Token<'error>,
    },
    BindMismatch {
        candidate: Token<'error>,
    },
}

impl<'error> Display for ErrorKind<'error> {
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
