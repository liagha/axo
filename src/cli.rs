use core::fmt;
use core::fmt::Formatter;
use broccli::{Color, TextStyle};
use crate::tokens::{Punctuation, Token};

impl fmt::Display for Token {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Token::Boolean(b) => write!(f, "{}", b),
            Token::Float(n) => write!(f, "{}", n),
            Token::Integer(n) => write!(f, "{}", n),
            Token::Punctuation(c) => {
                if c == &Punctuation::Newline {
                    return write!(f, "\n")
                }

                write!(f, "{}", c)
            },
            Token::Operator(c) => write!(f, "{}", c),
            Token::Str(str) => write!(f, "{}", str),
            Token::Identifier(str) => write!(f, "{}", str),
            Token::Char(char) => write!(f, "'{}'", char),
            Token::Keyword(keyword) => write!(f, "{}", keyword),
            Token::Invalid(invalid) => write!(f, "{}", invalid.colorize(Color::Red)),
            Token::EOF => write!(f, "{}", "End Of File"),
        }
    }
}

impl fmt::Debug for Token {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Token::Boolean(b) => write!(f, "Boolean({})", b),
            Token::Float(n) => write!(f, "Float({})", n),
            Token::Integer(n) => write!(f, "Integer({})", n),
            Token::Operator(op) => write!(f, "Operator({:?})", op),
            Token::Punctuation(pun) => write!(f, "Punctuation({:?})", pun),
            Token::Invalid(err) => write!(f, "Invalid({})", err),
            Token::Identifier(var) => write!(f, "Identifier({})", var),
            Token::Str(str) => write!(f, "String({})", str),
            Token::Char(char) => write!(f, "Char('{}')", char),
            Token::EOF => write!(f, "{}", "End Of File"),
            Token::Keyword(keyword) => write!(f, "Keyword({:?})", keyword),
        }
    }
}