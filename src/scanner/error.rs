use {
    crate::{
        format::{Debug, Display, Formatter, Result},
        text::numeral::ParseNumberError,
    },
};

#[derive(Clone, Eq, Hash, PartialEq)]
pub enum ErrorKind {
    InvalidCharacter(CharacterError),
    InvalidEscape(EscapeError),
    NumberParse(ParseNumberError),
}

#[derive(Clone, Eq, Hash, PartialEq)]
pub enum CharacterError {
    Unexpected(char),
    OutOfRange,
    Surrogate,
}

#[derive(Clone, Eq, Hash, PartialEq)]
pub enum EscapeError {
    Invalid,
    Overflow,
    OutOfRange,
    Unterminated,
    Empty,
}

impl Display for ErrorKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            ErrorKind::InvalidCharacter(e) => match e {
                CharacterError::Unexpected(ch) => write!(f, "unexpected character `{}`.", ch),
                CharacterError::OutOfRange => write!(f, "character code point out of range."),
                CharacterError::Surrogate => write!(f, "character is surrogate code point."),
            },
            ErrorKind::InvalidEscape(e) => match e {
                EscapeError::Invalid => write!(f, "invalid escape sequence."),
                EscapeError::Overflow => write!(f, "escape sequence value overflow."),
                EscapeError::OutOfRange => write!(f, "escape sequence out of range."),
                EscapeError::Unterminated => write!(f, "unterminated escape sequence."),
                EscapeError::Empty => write!(f, "empty escape sequence."),
            },
            ErrorKind::NumberParse(e) => write!(f, "failed to parse number: `{}`.", e),
        }
    }
}

impl Debug for ErrorKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "{}", self)
    }
}