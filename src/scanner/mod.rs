mod character;
mod classifier;
mod error;
mod operator;
mod punctuation;
mod scanner;
mod token;
mod traits;

pub use {character::Character, operator::*, punctuation::*, scanner::Scanner, token::*};

pub(super) use error::*;

use crate::reporter::Error;

pub type ScanError<'error> = Error<'error, ErrorKind<'error>>;
