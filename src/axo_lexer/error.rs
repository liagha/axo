#![allow(dead_code)]

pub enum ErrorKind {
    InvalidChar,
    NumberParse(crate::axo_rune::ParseNumberError),
    IntParseError(IntParseError),
    FloatParseError(IntParseError),
    CharParseError(CharParseError),
    StringParseError(CharParseError),
    UnClosedChar,
    UnClosedString,
    UnClosedComment,
    InvalidOperator(String),
    InvalidPunctuation(String),
}

#[derive(Debug)]
pub enum IntParseError {
    InvalidRange,
    InvalidHexadecimal,
    InvalidOctal,
    InvalidBinary,
    InvalidScientificNotation,
}

#[derive(Debug)]
pub enum CharParseError {
    EmptyCharLiteral,
    InvalidEscapeSequence,
    UnClosedEscapeSequence,
    InvalidCharLiteral,
    UnClosedCharLiteral,
}

impl core::fmt::Display for ErrorKind {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            ErrorKind::InvalidChar => {
                write!(f, "Invalid character'")
            }
            ErrorKind::NumberParse(e) => {
                write!(f, "Failed to parse number: {}", e)
            }
            ErrorKind::IntParseError(e) => {
                write!(f, "Failed to parse int value: {:?}", e)
            }
            ErrorKind::FloatParseError(e) => {
                write!(f, "Failed to parse float value: {:?}", e)
            }
            ErrorKind::CharParseError(e) => {
                write!(f, "Failed to parse char value: {:?}", e)
            }
            ErrorKind::StringParseError(e) => {
                write!(f, "Failed to parse string value: {:?}", e)
            }
            ErrorKind::UnClosedChar => {
                write!(f, "Unclosed character")
            }
            ErrorKind::UnClosedString => {
                write!(f, "Unclosed string")
            }
            ErrorKind::InvalidOperator(e) => {
                write!(f, "Invalid operator: '{}'", e)
            }
            ErrorKind::InvalidPunctuation(e) => {
                write!(f, "Invalid punctuation: '{}'", e)
            }
            ErrorKind::UnClosedComment => {
                write!(f, "Unclosed comment")
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
