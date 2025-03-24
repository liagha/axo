#![allow(dead_code)]

use core::fmt::Formatter;
use crate::lexer::{OperatorKind, PunctuationKind, TokenKind};

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

pub enum ParseError {
    UnexpectedToken(TokenKind, String),
    ExpectedPunctuation(PunctuationKind, String),
    ExpectedOperator(OperatorKind, String),
    ExpectedSyntax(String),
    InvalidSyntax(String),
    UnexpectedEOF,
    UnknownStatement,
}

impl core::fmt::Display for LexError {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
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

impl core::fmt::Display for ParseError {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        match self {
            ParseError::UnexpectedToken(t, m) => {
                write!(f, "Unexpected token '{}': '{}'", t, m)
            }
            ParseError::ExpectedPunctuation(t, m) => {
                write!(f, "Expected '{}' => '{}'", t, m)
            }
            ParseError::ExpectedOperator(t, m) => {
                write!(f, "Expected '{}' => '{}'", t, m)
            }
            ParseError::ExpectedSyntax(s) => {
                write!(f, "Expected '{}'", s)
            }
            ParseError::InvalidSyntax(m) => {
                write!(f, "Invalid Syntax '{}'", m)
            }
            ParseError::UnexpectedEOF => {
                write!(f, "Unexpected end of file")
            }
            ParseError::UnknownStatement => {
                write!(f, "Unknown statement")
            }
        }
    }
}

impl core::fmt::Debug for LexError {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self)
    }
}

impl core::fmt::Debug for ParseError {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self)
    }
}

impl core::error::Error for LexError {}
impl core::error::Error for ParseError {}
