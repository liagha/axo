use crate::{
    format::{Display, Formatter, Result, Show, Stencil},
    resolver::Type,
    scanner::Token,
};

#[derive(Clone)]
pub enum ErrorKind<'error> {
    InvalidMutation(Token<'error>, Type<'error>, Type<'error>),
    InvalidUnary(Token<'error>, Type<'error>),
    InvalidBinary(Token<'error>, Type<'error>, Type<'error>),
    InvalidType,
    InvalidTarget,
    ArityMismatch {
        name: String,
        expected: String,
        found: usize,
    },
    ContextMismatch {
        name: String,
        expected: String,
    },
}

impl<'error> Display for ErrorKind<'error> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            ErrorKind::InvalidMutation(operator, target, value) => write!(
                f,
                "cannot mutate `{}` with `{}` using `{}`.",
                target.format(Stencil::default()),
                value.format(Stencil::default()),
                operator.format(Stencil::default())
            ),
            ErrorKind::InvalidUnary(operator, operand) => write!(
                f,
                "cannot apply `{}` to `{}`.",
                operator.format(Stencil::default()),
                operand.format(Stencil::default())
            ),
            ErrorKind::InvalidBinary(operator, left, right) => write!(
                f,
                "cannot apply `{}` to `{}` and `{}`.",
                operator.format(Stencil::default()),
                left.format(Stencil::default()),
                right.format(Stencil::default())
            ),
            ErrorKind::InvalidType => write!(f, "invalid type."),
            ErrorKind::InvalidTarget => write!(f, "invalid target."),
            ErrorKind::ArityMismatch {
                name,
                expected,
                found,
            } => write!(
                f,
                "invalid arity for `{}`: expected {}, found {}.",
                name, expected, found,
            ),
            ErrorKind::ContextMismatch { name, expected } => {
                write!(f, "invalid context for `{}`: expected {}.", name, expected)
            }
        }
    }
}
