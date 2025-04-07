use core::fmt;
use core::fmt::Formatter;
use broccli::{Color, TextStyle};
use crate::axo_lexer::{Span, Token};
use crate::axo_lexer::{PunctuationKind, TokenKind};

impl fmt::Display for TokenKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            TokenKind::Boolean(b) => write!(f, "{}", b),
            TokenKind::Float(n) => write!(f, "{}", n),
            TokenKind::Integer(n) => write!(f, "{}", n),
            TokenKind::Punctuation(c) => {
                if c == &PunctuationKind::Newline {
                    return write!(f, "\n")
                }
                write!(f, "{}", c)
            },
            TokenKind::Operator(c) => write!(f, "{}", c),
            TokenKind::Str(str) => write!(f, "{}", str),
            TokenKind::Identifier(str) => write!(f, "{}", str),
            TokenKind::Char(char) => write!(f, "'{}'", char),
            TokenKind::Keyword(keyword) => write!(f, "{}", keyword),
            TokenKind::Comment(comment) => write!(f, "Comment({})", comment),
            TokenKind::Invalid(invalid) => write!(f, "{}", invalid.colorize(Color::Red)),
            TokenKind::EOF => write!(f, "{}", "End Of File"),
        }
    }
}

impl fmt::Debug for TokenKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            TokenKind::Boolean(b) => write!(f, "Boolean({})", b),
            TokenKind::Float(n) => write!(f, "Float({})", n),
            TokenKind::Integer(n) => write!(f, "Integer({})", n),
            TokenKind::Operator(op) => write!(f, "Operator({:?})", op),
            TokenKind::Punctuation(pun) => write!(f, "Punctuation({:?})", pun),
            TokenKind::Invalid(err) => write!(f, "Invalid({})", err),
            TokenKind::Identifier(var) => write!(f, "Identifier({})", var),
            TokenKind::Str(str) => write!(f, "String({})", str),
            TokenKind::Char(char) => write!(f, "Char('{}')", char),
            TokenKind::Comment(comment) => write!(f, "Comment({})", comment),
            TokenKind::EOF => write!(f, "EOF"),
            TokenKind::Keyword(keyword) => write!(f, "Keyword({:?})", keyword),
        }
    }
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        if self.kind != TokenKind::EOF {
            write!(f, "{}", self.kind)
        } else {
            write!(f, "")
        }
    }
}

impl fmt::Debug for Token {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        if self.kind == TokenKind::EOF {
            write!(f, "EOF")
        } else {
            write!(f, "{:?}", self.kind)
        }
    }
}

impl fmt::Display for Span {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let Span {
            start: (start_line, start_column),
            end: (end_line, end_column), ..
        } = self;

        if start_line == end_line && start_column == end_column {
            write!(f, "{}:{}", start_line, start_column)
        } else {
            write!(f, "{}:{}-{}:{}", start_line, start_column, end_line, end_column)
        }
    }
}
