#![allow(dead_code)]

use crate::lexer::{KeywordKind, OperatorKind, PunctuationKind};

#[derive(Clone, PartialEq)]
pub enum TokenKind {
    Float(f64),
    Integer(i64),
    Boolean(bool),
    Str(String),
    Operator(OperatorKind),
    Identifier(String),
    Char(char),
    Punctuation(PunctuationKind),
    Keyword(KeywordKind),
    Comment(String),
    Invalid(String),
    EOF,
}

impl TokenKind {
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "true" => Some(TokenKind::Boolean(true)),
            "false" => Some(TokenKind::Boolean(false)),
            "struct" => Some(TokenKind::Keyword(KeywordKind::Struct)),
            "enum" => Some(TokenKind::Keyword(KeywordKind::Enum)),
            "impl" => Some(TokenKind::Keyword(KeywordKind::Impl)),
            "trait" => Some(TokenKind::Keyword(KeywordKind::Trait)),
            "match" => Some(TokenKind::Keyword(KeywordKind::Match)),
            "if" => Some(TokenKind::Keyword(KeywordKind::If)),
            "else" => Some(TokenKind::Keyword(KeywordKind::Else)),
            "for" => Some(TokenKind::Keyword(KeywordKind::For)),
            "while" => Some(TokenKind::Keyword(KeywordKind::While)),
            "fn" => Some(TokenKind::Keyword(KeywordKind::Fn)),
            "return" => Some(TokenKind::Keyword(KeywordKind::Return)),
            "let" => Some(TokenKind::Keyword(KeywordKind::Let)),
            "continue" => Some(TokenKind::Keyword(KeywordKind::Continue)),
            "break" => Some(TokenKind::Keyword(KeywordKind::Break)),
            "in" => Some(TokenKind::Operator(OperatorKind::In)),
            _ => None,
        }
    }

    pub fn get_operator_opt(input: Option<&TokenKind>) -> Option<OperatorKind> {
        if let Some(TokenKind::Operator(operator)) = input {
            Some(operator.clone())
        } else {
            None
        }
    }

    pub fn get_operator(input: &TokenKind) -> Option<OperatorKind> {
        if let TokenKind::Operator(operator) = input {
            Some(operator.clone())
        } else {
            None
        }
    }
}


