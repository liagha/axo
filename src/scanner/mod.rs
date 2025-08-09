mod character;
mod core;
mod escape;
mod format;
mod number;
mod operator;
mod punctuation;
mod scanner;
mod token;
mod error;

pub use {
    scanner::Scanner,
    character::Character,
    operator::*,
    punctuation::*,
    token::*,
};

use {
    crate::{
        reporter::Error,
        scanner::error::ErrorKind,
    }
};

pub type ScanError<'error> = Error<'error, ErrorKind>;
