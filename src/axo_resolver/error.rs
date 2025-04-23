#[derive(Debug, Clone)]
pub enum ErrorKind {
    ImmutableAssign(String),
    InvalidAssignTarget(String),
    InvalidVariant(String),
    InvalidStruct(String),
    UnknownField(String),
    UnknownVariant(String),
    ArgCountMismatch(usize, usize),
    UndefinedSymbol(String, Option<String>),
    AlreadyDefined(String),
    InvalidAssignment,
    NotProvided,
    InvalidStructField(String),
    InvalidEnumVariant(String),
    TypeMismatch(String, String),
    InvalidExpression(String),
    Other(String),
}

impl core::fmt::Display for ErrorKind {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            ErrorKind::ImmutableAssign(name) =>
                write!(f, "Cannot assign to immutable variable `{}`", name),
            ErrorKind::InvalidAssignTarget(target) =>
                write!(f, "Invalid assignment target: `{}`", target),
            ErrorKind::InvalidVariant(name) =>
                write!(f, "Invalid enum variant: `{}`", name),
            ErrorKind::InvalidStruct(name) =>
                write!(f, "Invalid struct: `{}`", name),
            ErrorKind::UnknownField(field) =>
                write!(f, "Unknown field: `{}`", field),
            ErrorKind::UnknownVariant(variant) =>
                write!(f, "Unknown variant: `{}`", variant),
            ErrorKind::ArgCountMismatch(expected, found) =>
                write!(f, "Argument count mismatch: expected {}, found {}", expected, found),
            ErrorKind::UndefinedSymbol(name, suggestion) => {
                write!(f, "Undefined symbol: `{}`", name)?;
                if let Some(suggest) = suggestion {
                    write!(f, ", did you mean `{}`?", suggest)?;
                }
                Ok(())
            },
            ErrorKind::AlreadyDefined(name) =>
                write!(f, "Symbol `{}` already defined", name),
            ErrorKind::InvalidAssignment =>
                write!(f, "Invalid assignment"),
            ErrorKind::NotProvided =>
                write!(f, "Value not provided"),
            ErrorKind::InvalidStructField(field) =>
                write!(f, "Invalid struct field: `{}`", field),
            ErrorKind::InvalidEnumVariant(variant) =>
                write!(f, "Invalid enum variant: `{}`", variant),
            ErrorKind::TypeMismatch(expected, found) =>
                write!(f, "Type mismatch: expected `{}`, found `{}`", expected, found),
            ErrorKind::InvalidExpression(expr) =>
                write!(f, "Invalid expression: `{}`", expr),
            ErrorKind::Other(msg) =>
                write!(f, "{}", msg),
        }
    }
}
