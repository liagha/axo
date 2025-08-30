use std::fmt::Formatter;
use {
    crate::{
        scanner::Token,
        format::Display,
    }
};

pub enum ErrorKind<'error> {
    InvalidOperation(Token<'error>),
}

impl Display for ErrorKind<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self { 
            ErrorKind::InvalidOperation(token) => {
                write!(f, "invalid operation token: {}.", token)
            }
        }
    }
}