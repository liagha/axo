use crate::format::{Display, Formatter, Result};

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum ErrorKind {
    StackUnderflow,
    MemoryAccessViolation,
    InvalidBinary,
    InvalidUnary,
    InvalidCompare,
    DivisionByZero,
    OutOfBounds,
    InvalidFrame,
    InvalidCall,
    InvalidAccess,
    InvalidStore,
    InvalidCondition,
    InvalidIndex,
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
            ErrorKind::InvalidBinary => {
                write!(f, "invalid binary: operands do not match the operation.")
            }
            ErrorKind::InvalidUnary => {
                write!(f, "invalid unary: operand does not match the operation.")
            }
            ErrorKind::InvalidCompare => {
                write!(f, "invalid compare: operands cannot be compared.")
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
            ErrorKind::InvalidCondition => {
                write!(f, "invalid condition: value must be Boolean.")
            }
            ErrorKind::InvalidIndex => {
                write!(f, "invalid index: target cannot be indexed.")
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
