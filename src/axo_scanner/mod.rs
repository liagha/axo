mod token;
mod operator;
mod punctuation;
mod scanner;
mod format;
pub mod error;
mod character;

pub use {
    scanner::Scanner,
    character::Character,
    operator::*,
    punctuation::*,
    token::*,
};

use crate::{
    axo_error::Error,
    axo_scanner::error::ErrorKind,
};

pub type ScanError = Error<ErrorKind>;
