use crate::axo_lexer::{KeywordKind, OperatorKind, PunctuationKind};
use crate::axo_lexer::keyword::KeywordLexer;
use crate::axo_data::float::FloatLiteral;
use crate::axo_span::Span;

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
    String(String),
    Character(char),
    Operator(OperatorKind),
    Identifier(String),
    Punctuation(PunctuationKind),
    Keyword(KeywordKind),
    Comment(String),
}

impl Token {
    pub fn new(kind: TokenKind, span: Span) -> Self {
        Self { kind, span }
    }
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
}


