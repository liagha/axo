#![allow(dead_code)]

use crate::axo_lexer::{KeywordKind, OperatorKind, PunctuationKind};
use crate::axo_lexer::keyword::KeywordLexer;
use crate::axo_lexer::Span;
use crate::float::F64;

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub enum TokenKind {
    Float(F64),
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
            "in" => Some(TokenKind::Operator(OperatorKind::In)),
            s => {
                if let Some(kw) = s.to_keyword() {
                    Some(TokenKind::Keyword(kw))
                } else {
                    None
                }
            },
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


