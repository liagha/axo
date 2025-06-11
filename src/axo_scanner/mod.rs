mod token;
mod operator;
mod punctuation;
mod scanner;
mod format;
pub mod error;

pub use {
    scanner::Scanner,
    operator::*,
    punctuation::*,
    token::*,
};

use crate::{
    axo_error::Error,
    axo_scanner::error::ErrorKind,
};

pub type ScanError = Error<ErrorKind>;
