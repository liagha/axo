mod token;
mod operator;
mod punctuation;
mod keyword;
mod lexer;
mod error;
mod number;
mod handler;
mod symbol;
mod literal;
mod fmt;

pub use {
    keyword::KeywordKind,
    lexer::Lexer,
    operator::OperatorKind,
    punctuation::PunctuationKind,
    token::{Token, TokenKind},
};

use crate::{
    axo_errors::Error,
    axo_lexer::error::ErrorKind,
};

pub use crate::axo_span::span::Span;

pub type LexError = Error<ErrorKind>;
