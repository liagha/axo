use crate::axo_parser::ExprKind;

#[derive(Debug)]
pub enum ResolverError {
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