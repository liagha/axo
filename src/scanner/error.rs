use crate::{
    format::{Debug, Display, Formatter, Result},
    scanner::Character,
    text::numeral::ParseNumberError,
    tracker::TrackError,
};

#[derive(Clone, Eq, Hash, PartialEq)]
pub enum ErrorKind<'error> {
    Tracking(TrackError<'error>),
    Expected(&'static str),
    Unterminated(&'static str),
    InvalidCharacter(CharacterError),
    InvalidEscape(EscapeError),
    NumberParse(ParseNumberError),
}

#[derive(Clone, Eq, Hash, PartialEq)]
pub enum CharacterError {
    Unexpected(Character),
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

impl<'error> Display for ErrorKind<'error> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            ErrorKind::Tracking(tracker) => write!(f, "{}", tracker.handle().0),
            ErrorKind::Expected(label) => write!(f, "expected {}.", label),
            ErrorKind::Unterminated(label) => write!(f, "unterminated {}.", label),
            ErrorKind::InvalidCharacter(e) => match e {
                CharacterError::Unexpected(ch) => write!(f, "unexpected character `{}`.", ch.value),
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

impl<'error> Debug for ErrorKind<'error> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "{}", self)
    }
}
