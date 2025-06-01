mod token;
mod operator;
mod punctuation;
mod lexer;
mod format;
pub mod error;

pub use {
    lexer::Lexer,
    operator::*,
    punctuation::*,
    token::*,
};

use crate::{
    axo_error::Error,
    axo_lexer::error::ErrorKind,
};

pub type LexError = Error<ErrorKind>;
