use {
    crate::{
        format::{
            Display, Debug, 
            Formatter, Result
        },

        axo_scanner::Token,
    }
};

#[derive(Debug, Clone)]
pub enum ErrorKind {
    UndefinedSymbol(Token, Option<String>),
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
            ErrorKind::UndefinedSymbol(name, suggestion) => {
                write!(f, "undefined symbol: `{}`", name)?;
                if let Some(suggest) = suggestion {
                    write!(f, ", did you mean `{}`?", suggest)?;
                }
                Ok(())
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
