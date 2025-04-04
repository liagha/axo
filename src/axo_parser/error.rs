#![allow(dead_code)]

use std::fmt::Formatter;
use std::fs::read_to_string;
use broccli::{Color, TextStyle};
use crate::axo_lexer::{TokenKind, Token, Span, PunctuationKind};
use crate::axo_parser::{Expr};
use crate::axo_parser::state::{Position, Context, ContextKind};

#[derive(Debug)]
pub struct Error {
    pub kind: ErrorKind,
    pub span: Span,
    pub context: Option<Context>,
    pub help: Option<String>,
}

pub enum ErrorKind {
    ElseWithoutConditional,
    UnclosedDelimiter(Token),
    UnimplementedToken(TokenKind),
    UnexpectedToken(TokenKind),
    InvalidSyntaxPattern(String),
    ExpectedSyntax(ContextKind),
    UnexpectedEndOfFile,
}

impl Error {
    pub fn new(kind: ErrorKind, span: Span) -> Self {
        Self {
            kind,
            span,
            context: None,
            help: None,
        }
    }

    pub fn with_context(mut self, context: Context) -> Self {
        self.context = Some(context);
        self
    }

    pub fn with_help(mut self, help: impl Into<String>) -> Self {
        self.help = Some(help.into());
        self
    }

    pub fn format(&self, source_code: &str) -> String {
        let lines: Vec<&str> = source_code.lines().collect();
        let mut result = String::new();

        result.push_str(&format!("error: {}\n", self.kind.to_string().colorize(Color::Red).bold()));

        let (line_start, column_start) = self.span.start;
        let (line_end, column_end) = self.span.end;

        result.push_str(&format!(" --> {}:{}:{}\n",
                                 self.span.file.display(),
                                 line_start,
                                 column_start
        ).colorize(Color::Blue));

        if let Some(ctx) = &self.context {
            result.push_str(&format!(" note: {}\n", ctx.describe_chain().colorize(Color::Blue)));
        }

        // Calculate maximum line number width safely
        let max_line_number = line_end.max(line_end).max(1); // Ensure at least 1
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

            result.push_str(&format!("{:>width$} | {}\n",
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

                result.push_str(&format!("{:>width$} | {}\n",
                                         " ".repeat(line_number_width),
                                         underline.colorize(Color::Red).bold(),
                                         width = line_number_width
                ));
            }
        }

        if let Some(help) = &self.help {
            result.push_str(&format!("help: {}\n", help.colorize(Color::Green)));
        }

        result
    }
}

impl core::fmt::Display for ErrorKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        match self {
            ErrorKind::ElseWithoutConditional => {
                write!(f, "Cant have an else without conditional")
            }
            ErrorKind::InvalidSyntaxPattern(m) => {
                write!(f, "Invalid syntax pattern: '{}'", m)
            }
            ErrorKind::UnexpectedEndOfFile => {
                write!(f, "Unexpected end of file")
            }
            ErrorKind::UnclosedDelimiter(delimiter) => {
                write!(f, "Unclosed delimiter '{:?}'", delimiter)
            }
            ErrorKind::UnimplementedToken(token) => {
                write!(f, "Unimplemented token '{}'", token)
            }
            ErrorKind::UnexpectedToken(token) => {
                write!(f, "Unexpected token '{}'", token)
            }
            ErrorKind::ExpectedSyntax(kind) => {
                write!(f, "Expected syntax '{:?}' but didn't get it", kind)
            }
        }
    }
}

impl core::fmt::Debug for ErrorKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self)
    }
}

impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let file = read_to_string(self.span.file.clone()).unwrap();
        write!(f, "{}", self.format(file.as_str()))
    }
}

impl core::error::Error for ErrorKind {}