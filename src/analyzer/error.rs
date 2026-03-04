use {
    crate::{
        format::{
            Show,
            Display,
            Formatter,
            Result
        },
        data::Str,
        scanner::Token
    },
};

#[derive(Clone)]
pub enum ErrorKind<'error> {
    InvalidOperation(Token<'error>),
    InvalidType,
    InvalidPrimitiveArity {
        name: String,
        expected: String,
        found: usize,
    },
    InvalidPrimitiveContext {
        name: String,
        expected: String,
    },
    Unimplemented,
}

impl<'error> Show<'error> for ErrorKind<'error> {
    type Verbosity = u8;

    fn format(&self, verbosity: Self::Verbosity) -> Str<'error> {
        match verbosity {
            0 => {
                "".to_string()
            }

            1 => {
                match self {
                    ErrorKind::InvalidOperation(token) => {
                        format!("invalid operation token: {}.", token.format(verbosity))
                    }
                    ErrorKind::InvalidType => {
                        "invalid type.".to_string()
                    }
                    ErrorKind::InvalidPrimitiveArity {
                        name,
                        expected,
                        found,
                    } => {
                        format!(
                            "invalid '{}' arity: expected {}, found {}.",
                            name, expected, found,
                        )
                    }
                    ErrorKind::InvalidPrimitiveContext { name, expected } => {
                        format!("invalid '{}' usage: expected {}.", name, expected)
                    }
                    ErrorKind::Unimplemented => {
                        "unimplemented operation.".to_string()
                    }
                }
            }

            _ => {
                self.format(verbosity - 1).to_string()
            }
        }.into()
    }
}

impl<'error> Display for ErrorKind<'error> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "{}", self.format(1))
    }
}