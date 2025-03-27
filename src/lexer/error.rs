#![allow(dead_code)]
pub enum LexError {
    InvalidChar,
    NumberParse(String),
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
    InvalidEscapeSequence,
    UnClosedEscapeSequence,
    InvalidCharLiteral,
    UnClosedCharLiteral,
}

impl core::fmt::Display for LexError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            LexError::InvalidChar => {
                write!(f, "Invalid character'")
            }
            LexError::NumberParse(e) => {
                write!(f, "Failed to parse number: {}", e)
            }
            LexError::IntParseError(e) => {
                write!(f, "Failed to parse int value: {:?}", e)
            }
            LexError::FloatParseError(e) => {
                write!(f, "Failed to parse float value: {:?}", e)
            }
            LexError::CharParseError(e) => {
                write!(f, "Failed to parse char value: {:?}", e)
            }
            LexError::StringParseError(e) => {
                write!(f, "Failed to parse string value: {:?}", e)
            }
            LexError::UnClosedChar => {
                write!(f, "Unclosed character")
            }
            LexError::UnClosedString => {
                write!(f, "Unclosed string")
            }
            LexError::InvalidOperator(e) => {
                write!(f, "Invalid operator: '{}'", e)
            }
            LexError::InvalidPunctuation(e) => {
                write!(f, "Invalid punctuation: '{}'", e)
            }
            LexError::UnClosedComment => {
                write!(f, "Unclosed comment")
            }
        }
    }
}

impl core::fmt::Debug for LexError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self)
    }
}

impl core::error::Error for LexError {}
