mod token;
mod operator;
mod punctuation;
mod keyword;
mod lexer;
mod error;
mod span;
mod number;
mod handler;
mod symbol;
mod literal;
mod fmt;

pub use {
    lexer::Lexer,
    span::Span,
    token::{TokenKind, Token},
    keyword::KeywordKind,
    operator::OperatorKind,
    punctuation::PunctuationKind,
};

use crate::{
    axo_errors::Error,
    axo_lexer::error::ErrorKind,
};

pub type LexError = Error<ErrorKind>;
