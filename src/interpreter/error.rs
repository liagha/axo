use crate::format::{Display, Formatter, Result};

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum ErrorKind {
    StackUnderflow,
    MemoryAccessViolation,
    TypeMismatch,
    DivisionByZero,
    OutOfBounds,
    InvalidFrame,
    InvalidCall,
    InvalidAccess,
    InvalidStore,
    InvalidControl,
    MissingSymbol,
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
            ErrorKind::DivisionByZero => {
                write!(f, "division by zero: attempted to divide by zero.")
            }
            ErrorKind::OutOfBounds => {
                write!(f, "out of bounds: index exceeded the available bounds.")
            }
            ErrorKind::InvalidFrame => {
                write!(f, "invalid frame: attempted to return without a caller.")
            }
            ErrorKind::InvalidCall => {
                write!(f, "invalid call: target is not callable.")
            }
            ErrorKind::InvalidAccess => {
                write!(f, "invalid access: member access is not valid for the target.")
            }
            ErrorKind::InvalidStore => {
                write!(f, "invalid store: target cannot receive a value.")
            }
            ErrorKind::InvalidControl => {
                write!(f, "invalid control flow: statement is not valid in the current scope.")
            }
            ErrorKind::MissingSymbol => {
                write!(f, "missing symbol: target could not be resolved.")
            }
        }
    }
}
