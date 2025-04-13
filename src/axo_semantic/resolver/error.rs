use crate::axo_parser::ExprKind;

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