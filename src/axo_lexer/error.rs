#![allow(dead_code)]

use std::fs::read_to_string;
use broccli::{Color, TextStyle};
use crate::axo_lexer::{Span};

#[derive(Debug)]
pub struct Error {
    pub kind: ErrorKind,
    pub span: Span,
    pub help: Option<String>,
}

impl Error {
    pub fn new(kind: ErrorKind, span: Span) -> Self {
        Self {
            kind,
            span,
            help: None,
        }
    }

    pub fn with_help(mut self, help: impl Into<String>) -> Self {
        self.help = Some(help.into());
        self
    }

    pub fn format(&self) -> (String, String) {
        let source_code = read_to_string(self.span.file.clone()).unwrap();
        let lines: Vec<&str> = source_code.lines().collect();
        let mut messages = String::new();
        let mut details = String::new();

        messages.push_str(&format!("error: {}", self.kind.to_string().colorize(Color::Red).bold()));

        let (line_start, column_start) = self.span.start;
        let (line_end, column_end) = self.span.end;

        details.push_str(&format!(" --> {}:{}:{}\n",
                                  self.span.file.display(),
                                  line_start,
                                  column_start
        ).colorize(Color::Blue));

        // Calculate maximum line number width safely
        let max_line_number = line_end.max(line_start).max(1); // Ensure at least 1
        let line_number_width = max_line_number.to_string().len();

        // Calculate bounds safely
        let start_line = line_start.saturating_sub(3).max(1);
        let end_line = (line_end + 3).min(lines.len()).max(1);

        for line_idx in start_line..=end_line {
            let line_content_idx = line_idx.saturating_sub(1);
            if line_content_idx >= lines.len() { break; }

            let line_content = lines[line_content_idx];
            let line_num_str = if line_content.is_empty() {
                " ".repeat(line_number_width)
            } else {
                line_idx.to_string()
            };

            details.push_str(&format!("{:>width$} | {}\n",
                                      line_num_str.colorize(Color::Blue),
                                      line_content,
                                      width = line_number_width
            ));

            if line_idx >= line_start && line_idx <= line_end {
                let line_length = line_content.chars().count();
                let start_col = if line_idx == line_start {
                    column_start.saturating_sub(1).min(line_length)
                } else { 0 };
                let end_col = if line_idx == line_end {
                    column_end.saturating_sub(1).min(line_length)
                } else { line_length };

                let caret_count = if start_col <= end_col {
                    end_col.saturating_sub(start_col) + 1
                } else {
                    1
                };

                let underline = format!("{:width$}{}",
                                        "",
                                        "^".repeat(caret_count),
                                        width = start_col
                );

                details.push_str(&format!("{:>width$} | {}\n",
                                          " ".repeat(line_number_width),
                                          underline.colorize(Color::Red).bold(),
                                          width = line_number_width
                ));
            }
        }

        if let Some(help) = &self.help {
            messages.push_str(&format!("help: {}\n", help.colorize(Color::Green)));
        }

        (messages, details)
    }
}

pub enum ErrorKind {
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
