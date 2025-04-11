#![allow(dead_code)]

#[derive(Debug, Clone)]
pub enum ErrorKind {
    // Variable and binding errors
    UndefinedVariable(String),
    DuplicateDefinition(String),
    InvalidAssignmentTarget,

    // Type errors
    TypeMismatch,
    InvalidTypeAnnotation,

    // Structural errors
    InvalidArrayElement,
    InvalidStructField,
    MissingRequiredField,

    // Control flow errors
    ReturnOutsideFunction,
    BreakOutsideLoop,
    ContinueOutsideLoop,

    // Expression errors
    InvalidBinaryOperation,
    InvalidUnaryOperation,
    InvalidFunctionCall,
    InvalidPathExpression,
    InvalidMemberAccess,

    // Item errors
    InvalidItemDeclaration,

    // Expression context errors
    ExpectedExpression,
    ExpectedStatement,

    // Other errors
    SyntaxError(String),
    Other(String),
}

impl core::fmt::Display for ErrorKind {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            ErrorKind::UndefinedVariable(var) => {
                write!(f, "Undefined variable: {}", var)
            }
            ErrorKind::DuplicateDefinition(var) => {
                write!(f, "Duplicate definition: {}", var)
            }
            ErrorKind::InvalidAssignmentTarget => {
                write!(f, "Invalid assignment target")
            }
            ErrorKind::TypeMismatch => {
                write!(f, "Type mismatch")
            }
            ErrorKind::InvalidTypeAnnotation => {
                write!(f, "Invalid type annotation")
            }
            ErrorKind::InvalidArrayElement => {
                write!(f, "Invalid array element")
            }
            ErrorKind::InvalidStructField => {
                write!(f, "Invalid struct field")
            }
            ErrorKind::MissingRequiredField => {
                write!(f, "Missing required field")
            }
            ErrorKind::ReturnOutsideFunction => {
                write!(f, "Return-outside function")
            }
            ErrorKind::BreakOutsideLoop => {
                write!(f, "Break-outside loop")
            }
            ErrorKind::ContinueOutsideLoop => {
                write!(f, "Continue-outside loop")
            }
            ErrorKind::InvalidBinaryOperation => {
                write!(f, "Invalid binary operation")
            }
            ErrorKind::InvalidUnaryOperation => {
                write!(f, "Invalid unary operation")
            }
            ErrorKind::InvalidFunctionCall => {
                write!(f, "Invalid function call")
            }
            ErrorKind::InvalidPathExpression => {
                write!(f, "Invalid path expression")
            }
            ErrorKind::InvalidMemberAccess => {
                write!(f, "Invalid member access")
            }
            ErrorKind::InvalidItemDeclaration => {
                write!(f, "Invalid item declaration")
            }
            ErrorKind::ExpectedExpression => {
                write!(f, "Expected expression")
            }
            ErrorKind::ExpectedStatement => {
                write!(f, "Expected statement")
            }
            ErrorKind::SyntaxError(msg) => {
                write!(f, "Syntax error: {}", msg)
            }
            ErrorKind::Other(msg) => {
                write!(f, "Other error: {}", msg)
            }
        }
    }
}
