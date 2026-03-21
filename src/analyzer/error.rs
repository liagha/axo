use {
    crate::{
        format::{
            Show,
            Verbosity,
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
    InvalidTarget,
    InvalidPrimitiveArity {
        name: String,
        expected: String,
        found: usize,
    },
    InvalidPrimitiveContext {
        name: String,
        expected: String,
    },
}

impl<'error> Show<'error> for ErrorKind<'error> {
    fn format(&self, verbosity: Verbosity) -> Str<'error> {
        match verbosity {
            Verbosity::Minimal => {
                match self {
                    ErrorKind::InvalidOperation(token) => {
                        format!("invalid operation token: {}.", token.format(verbosity))
                    }
                    ErrorKind::InvalidType => {
                        "invalid type.".to_string()
                    }
                    ErrorKind::InvalidTarget => {
                        "invalid target.".to_string()
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
                }
            }

            _ => {
                self.format(verbosity.fallback()).to_string()
            }
        }.into()
    }
}

impl<'error> Display for ErrorKind<'error> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "{}", self.format(Verbosity::Detailed))
    }
}