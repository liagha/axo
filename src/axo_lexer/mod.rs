mod token;
mod operator;
mod punctuation;
mod lexer;
pub mod error;
mod number;
mod handler;
mod symbol;
mod literal;
mod fmt;

pub use {
    lexer::Lexer,
    operator::OperatorKind,
    punctuation::PunctuationKind,
    token::{Token, TokenKind},
};

use crate::{
    axo_errors::Error,
    axo_lexer::error::ErrorKind,
};

pub type LexError = Error<ErrorKind>;
