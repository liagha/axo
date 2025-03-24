#![allow(dead_code)]
pub enum LexError {
    InvalidChar(String),
    IntParseError(String),
    FloatParseError(String),
    CharParseError(String),
    UnClosedChar(String),
    UnClosedString(String),
    InvalidOperator(String),
    InvalidPunctuation(String),
    StringParseError(String),
    UnClosedComment(String),
}

impl core::fmt::Display for LexError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            LexError::InvalidChar(c) => {
                write!(f, "Invalid character '{}'", c)
            }
            LexError::IntParseError(e) => {
                write!(f, "Failed to parse int value: {}", e)
            }
            LexError::FloatParseError(e) => {
                write!(f, "Failed to parse float value: {}", e)
            }
            LexError::CharParseError(e) => {
                write!(f, "Failed to parse char value: {}", e)
            }
            LexError::UnClosedChar(e) => {
                write!(f, "Unclosed character: '{}'", e)
            }
            LexError::UnClosedString(e) => {
                write!(f, "Unclosed string: '{}'", e)
            }
            LexError::InvalidOperator(e) => {
                write!(f, "Invalid operator: '{}'", e)
            }
            LexError::InvalidPunctuation(e) => {
                write!(f, "Invalid punctuation: '{}'", e)
            }
            LexError::StringParseError(e) => {
                write!(f, "Failed to parse string value: {}", e)
            }
            LexError::UnClosedComment(e) => {
                write!(f, "Unclosed comment: '{}'", e)
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
