mod token;
mod operator;
mod punctuation;
mod lexer;
pub mod error;
mod fmt;

pub use {
    lexer::Lexer,
    operator::{OperatorKind, OperatorLexer},
    punctuation::{PunctuationKind, PunctuationLexer},
    token::{Token, TokenKind},
};

use crate::{
    axo_error::Error,
    axo_lexer::error::ErrorKind,
};

pub type LexError = Error<ErrorKind>;
