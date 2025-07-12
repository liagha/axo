use {
    crate::{
        format::{
            Debug, Display, 
            Formatter, Result
        },

        axo_text::numeral::ParseNumberError,
    },
};

#[derive(Clone, Eq, Hash, PartialEq)]
pub enum ErrorKind {
    Custom(String),
    InvalidChar,
    NumberParse(ParseNumberError),
    CharParseError(CharParseError),
    StringParseError(CharParseError),
}

#[derive(Clone, Eq, Hash, PartialEq)]
pub enum CharParseError {
    InvalidEscapeSequence,
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
        }
    }
}

impl Display for CharParseError {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            CharParseError::InvalidEscapeSequence => {
                write!(f, "invalid escape sequence")
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
