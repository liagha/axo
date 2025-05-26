use {
    crate::{
        format::{Debug, Display, Formatter, Result},
        axo_rune::numeral::ParseNumberError,
    },
};

#[derive(Clone, PartialEq)]
pub enum ErrorKind {
    Custom(String),
    InvalidChar,
    NumberParse(ParseNumberError),
    CharParseError(CharParseError),
    StringParseError(CharParseError),
    UnterminatedChar,
    UnterminatedDoubleQuoteString,
    UnterminatedBackTickString,
    UnterminatedCommentBlock,
}

#[derive(Clone, PartialEq)]
pub enum CharParseError {
    InvalidToken(String),
    EmptyCharLiteral,
    InvalidEscapeSequence,
    InvalidCharLiteral,
    UnterminatedEscapeSequence,
}

impl Display for ErrorKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            ErrorKind::Custom(err) => {
                write!(f, "{}", err)
            }
            
            ErrorKind::InvalidChar => {
                write!(f, "invalid character'")
            }
            ErrorKind::NumberParse(e) => {
                write!(f, "failed to parse number: `{}`.", e)
            }
            ErrorKind::CharParseError(e) => {
                write!(f, "failed to parse character literal: `{}`.", e)
            }
            ErrorKind::StringParseError(e) => {
                write!(f, "failed to parse string literal: `{}`.", e)
            }
            ErrorKind::UnterminatedChar => {
                write!(f, "unterminated character literal.")
            }
            ErrorKind::UnterminatedBackTickString => {
                write!(f, "unterminated backtick string literal.")
            }
            ErrorKind::UnterminatedDoubleQuoteString => {
                write!(f, "unterminated double quote string literal.")
            }
            ErrorKind::UnterminatedCommentBlock => {
                write!(f, "unterminated comment block.")
            }
        }
    }
}

impl Display for CharParseError {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            CharParseError::InvalidToken(str) => {
                write!(f, "invalid token: `{}`", str)
            }
            CharParseError::EmptyCharLiteral => {
                write!(f, "empty character literal")
            }
            CharParseError::InvalidEscapeSequence => {
                write!(f, "invalid escape sequence")
            }
            CharParseError::InvalidCharLiteral => {
                write!(f, "invalid character literal")
            }
            CharParseError::UnterminatedEscapeSequence => {
                write!(f, "unterminated escape sequence")
            }
        }
    }
}

impl Debug for ErrorKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "{}", self)
    }
}
