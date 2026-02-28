use crate::format::Display;

#[derive(Clone, Debug)]
pub enum ErrorKind {
    UnsupportedInstruction { instruction: &'static str },
    InvalidModule { reason: String },
    OutputWriteFailure { path: String, reason: String },
}

impl Display for ErrorKind {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) -> Result<(), ::std::fmt::Error> {
        match self {
            ErrorKind::UnsupportedInstruction { instruction } => {
                write!(
                    f,
                    "unsupported instruction in code generation: {}.",
                    instruction
                )
            }
            ErrorKind::InvalidModule { reason } => {
                write!(f, "invalid LLVM module: {}.", reason)
            }
            ErrorKind::OutputWriteFailure { path, reason } => {
                write!(f, "failed to write output `{}`: {}.", path, reason)
            }
        }
    }
}
