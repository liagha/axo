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
    UndefinedSymbol(Token),
    ParameterMismatch {
        expected: usize,
        found: usize,
    },
    FieldCountMismatch {
        expected: usize,
        found: usize,
    },
    TypeMismatch {
        expected: String,
        found: String,
    },
}

impl Display for ErrorKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            ErrorKind::UndefinedSymbol(name) => {
                write!(f, "undefined symbol: `{}`", name)
            },
            ErrorKind::ParameterMismatch { expected, found } => {
                write!(f, "parameter mismatch: expected {}, found {}", expected, found)
            }
            ErrorKind::FieldCountMismatch { expected, found } => {
                write!(f, "field count mismatch: expected {}, found {}", expected, found)
            }
            ErrorKind::TypeMismatch { expected, found } => {
                write!(f, "type mismatch: expected {}, found {}", expected, found)
            }
        }
    }
}
