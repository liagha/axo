use std::fmt::Formatter;
use {
    crate::{
        scanner::Token,
        format::Display,
    }
};

#[derive(Clone, Debug)]
pub enum ErrorKind<'error> {
    InvalidOperation(Token<'error>),
    InvalidType,
    UnImplemented,
}

impl Display for ErrorKind<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self { 
            ErrorKind::InvalidOperation(token) => {
                write!(f, "invalid operation token: {}.", token)
            }
            ErrorKind::InvalidType => {
                write!(f, "invalid type.")
            }
            ErrorKind::UnImplemented => {
                write!(f, "unimplemented operation.")
            }
        }
    }
}