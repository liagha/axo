use crate::data::Scale;
use crate::format::{Display, Formatter, Result};

#[derive(Clone, Debug)]
pub enum ErrorKind {
    InvalidModule { reason: String },
    BuilderError { reason: String },
    InvalidOperandType { side: &'static str, instruction: String },
    InvalidType,
    UnsupportedFloatWidth { width: Scale },
    SemanticError { message: String },
}

impl Display for ErrorKind {
    fn fmt(&self, f: &mut Formatter) -> Result {
        match self {
            ErrorKind::InvalidModule { reason } => {
                write!(f, "invalid LLVM module: {}.", reason)
            }
            ErrorKind::BuilderError { reason } => {
                write!(f, "builder error: {}", reason)
            }
            ErrorKind::InvalidOperandType { side, instruction } => {
                write!(f, "invalid {} type for operation '{}'", side, instruction)
            }
            ErrorKind::InvalidType => {
                write!(f, "invalid LLVM type")
            }
            ErrorKind::UnsupportedFloatWidth { width } => {
                write!(f, "invalid LLVM float width: {}", width)
            }
            ErrorKind::SemanticError { message } => {
                write!(f, "semantic error: {}", message)
            }
        }
    }
}
