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
mod traits;

pub use {
    scanner::Scanner,
    character::Character,
    operator::*,
    punctuation::*,
    token::*,
};

pub(super) use {
    error::*
};

use {
    crate::{
        reporter::Error,
    }
};

pub type ScanError<'error> = Error<'error, ErrorKind>;
