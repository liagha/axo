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
pub enum ErrorKind {
    UndefinedSymbol {
        query: Token,
    },
    SlotMismatch {
        candidate: Token,
    },
}

impl Display for ErrorKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            ErrorKind::UndefinedSymbol { query } => {
                write!(f, "undefined symbol: `{}`", query)
            },
            ErrorKind::SlotMismatch { candidate } => {
                write!(f, "slots of `{}` aren't matched correctly.", candidate)
            }
        }
    }
}
