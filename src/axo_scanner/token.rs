use {
    super::{
        OperatorKind, PunctuationKind
    },
    
    crate::{
        axo_data::float::FloatLiteral,
        
        axo_span::Span,
    }
};

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
    Comment(String),
}

impl Token {
    pub fn new(kind: TokenKind, span: Span) -> Self {
        Self { kind, span }
    }
    
    pub fn from(kind: TokenKind) -> Self {
        Self {
            kind,
            span: Span::default(),
        }
    }
}

impl TokenKind {
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "true" => Some(TokenKind::Boolean(true)),
            "false" => Some(TokenKind::Boolean(false)),
            "in" => Some(TokenKind::Operator(OperatorKind::In)),
            _ => {
                None
            },
        }
    }
}


