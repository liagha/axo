mod token;
mod operator;
mod punctuation;
mod keyword;
pub mod lexer;
pub mod error;
mod span;
mod number;
mod handler;
mod symbol;
mod literal;
mod fmt;

pub use crate::axo_errors::Error as AxoError;
pub use lexer::Lexer;
pub use span::Span;
pub use token::{TokenKind, Token};
pub use keyword::KeywordKind;
pub use operator::OperatorKind;
pub use punctuation::PunctuationKind;
use crate::axo_lexer::error::ErrorKind;

pub type Error = AxoError<ErrorKind>;
