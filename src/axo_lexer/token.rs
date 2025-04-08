#![allow(dead_code)]

use crate::axo_lexer::{KeywordKind, OperatorKind, PunctuationKind};
use crate::axo_lexer::keyword::KeywordLexer;
use crate::axo_lexer::Span;
use crate::axo_data::float::FloatLiteral;

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub enum TokenKind {
    Float(FloatLiteral),
    Integer(i128),
    Boolean(bool),
    Str(String),
    Char(char),
    Operator(OperatorKind),
    Identifier(String),
    Punctuation(PunctuationKind),
    Keyword(KeywordKind),
    Comment(String),
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


