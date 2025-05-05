use {
    crate::{
        axo_rune::numeral::ParseNumberError,
    },
};

#[derive(Clone)]
pub enum ErrorKind {
    InvalidChar,
    NumberParse(ParseNumberError),
    CharParseError(CharParseError),
    StringParseError(CharParseError),
    UnterminatedChar,
    UnterminatedDoubleQuoteString,
    UnterminatedBackTickString,
    UnterminatedCommentBlock,
}

#[derive(Clone)]
pub enum CharParseError {
    EmptyCharLiteral,
    InvalidEscapeSequence,
    InvalidCharLiteral,
    UnterminatedEscapeSequence,
}

impl core::fmt::Display for ErrorKind {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
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

impl core::fmt::Display for CharParseError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
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

impl core::fmt::Debug for ErrorKind {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self)
    }
}

impl core::error::Error for ErrorKind {}
