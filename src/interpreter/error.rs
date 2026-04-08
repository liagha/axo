use crate::format::{Display, Formatter, Result};

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum ErrorKind {
    StackUnderflow,
    MemoryAccessViolation,
    TypeMismatch,
    OutOfBounds,
    InvalidFrame,
}

impl Display for ErrorKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            ErrorKind::StackUnderflow => {
                write!(f, "stack underflow: attempted to pop from an empty stack.")
            }
            ErrorKind::MemoryAccessViolation => {
                write!(f, "memory access violation: invalid memory address.")
            }
            ErrorKind::TypeMismatch => {
                write!(f, "type mismatch: invalid operation for the given types.")
            }
            ErrorKind::OutOfBounds => {
                write!(f, "out of bounds: index or mathematical operation $$>$$ bounds.")
            }
            ErrorKind::InvalidFrame => {
                write!(f, "invalid frame: attempted to return without a caller.")
            }
        }
    }
}
