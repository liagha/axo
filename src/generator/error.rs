use crate::format::{Display, Formatter, Result};

#[derive(Clone, Debug)]
pub enum ErrorKind {
    UnsupportedAnalysis { instruction: &'static str },
    InvalidModule { reason: String },
}

impl Display for ErrorKind {
    fn fmt(&self, f: &mut Formatter) -> Result {
        match self {
            ErrorKind::UnsupportedAnalysis { instruction } => {
                write!(
                    f,
                    "unsupported instruction in schema generation: {}.",
                    instruction
                )
            }
            ErrorKind::InvalidModule { reason } => {
                write!(f, "invalid LLVM module: {}.", reason)
            }
        }
    }
}
