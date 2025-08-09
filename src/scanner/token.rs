use {
    super::{
        OperatorKind, PunctuationKind
    },

    crate::{
        data::float::FloatLiteral,
        tracker::Span,
    }
};

#[derive(Clone, Eq, Hash, PartialEq)]
pub struct Token<'token> {
    pub kind: TokenKind,
    pub span: Span<'token>,
}

#[derive(Clone, Eq, Hash, PartialEq)]
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

impl<'token> Token<'token> {
    pub fn new(kind: TokenKind, span: Span<'token>) -> Self {
        Self { kind, span }
    }
}

impl TokenKind {
    pub fn float(value: FloatLiteral) -> Self {
        TokenKind::Float(value)
    }

    pub fn integer(value: i128) -> Self {
        TokenKind::Integer(value)
    }

    pub fn boolean(value: bool) -> Self {
        TokenKind::Boolean(value)
    }

    pub fn string(value: String) -> Self {
        TokenKind::String(value)
    }

    pub fn character(value: char) -> Self {
        TokenKind::Character(value)
    }

    pub fn operator(value: OperatorKind) -> Self {
        TokenKind::Operator(value)
    }

    pub fn identifier(value: String) -> Self {
        TokenKind::Identifier(value)
    }

    pub fn punctuation(value: PunctuationKind) -> Self {
        TokenKind::Punctuation(value)
    }

    pub fn comment(value: String) -> Self {
        TokenKind::Comment(value)
    }

    pub fn is_float(&self) -> bool {
        matches!(self, TokenKind::Float(_))
    }

    pub fn is_integer(&self) -> bool {
        matches!(self, TokenKind::Integer(_))
    }

    pub fn is_boolean(&self) -> bool {
        matches!(self, TokenKind::Boolean(_))
    }

    pub fn is_string(&self) -> bool {
        matches!(self, TokenKind::String(_))
    }

    pub fn is_character(&self) -> bool {
        matches!(self, TokenKind::Character(_))
    }

    pub fn is_operator(&self) -> bool {
        matches!(self, TokenKind::Operator(_))
    }

    pub fn is_identifier(&self) -> bool {
        matches!(self, TokenKind::Identifier(_))
    }

    pub fn is_punctuation(&self) -> bool {
        matches!(self, TokenKind::Punctuation(_))
    }

    pub fn is_comment(&self) -> bool {
        matches!(self, TokenKind::Comment(_))
    }

    pub fn unwrap_float(self) -> FloatLiteral {
        match self {
            TokenKind::Float(value) => value,
            _ => panic!("Called unwrap_float on non-Float variant"),
        }
    }

    pub fn unwrap_integer(self) -> i128 {
        match self {
            TokenKind::Integer(value) => value,
            _ => panic!("Called unwrap_integer on non-Integer variant"),
        }
    }

    pub fn unwrap_boolean(self) -> bool {
        match self {
            TokenKind::Boolean(value) => value,
            _ => panic!("Called unwrap_boolean on non-Boolean variant"),
        }
    }

    pub fn unwrap_string(self) -> String {
        match self {
            TokenKind::String(value) => value,
            _ => panic!("Called unwrap_string on non-String variant"),
        }
    }

    pub fn unwrap_character(self) -> char {
        match self {
            TokenKind::Character(value) => value,
            _ => panic!("Called unwrap_character on non-Character variant"),
        }
    }

    pub fn unwrap_operator(self) -> OperatorKind {
        match self {
            TokenKind::Operator(value) => value,
            _ => panic!("Called unwrap_operator on non-Operator variant"),
        }
    }

    pub fn unwrap_identifier(self) -> String {
        match self {
            TokenKind::Identifier(value) => value,
            _ => panic!("Called unwrap_identifier on non-Identifier variant"),
        }
    }

    pub fn unwrap_punctuation(self) -> PunctuationKind {
        match self {
            TokenKind::Punctuation(value) => value,
            _ => panic!("Called unwrap_punctuation on non-Punctuation variant"),
        }
    }

    pub fn unwrap_comment(self) -> String {
        match self {
            TokenKind::Comment(value) => value,
            _ => panic!("Called unwrap_comment on non-Comment variant"),
        }
    }

    pub fn try_unwrap_float(&self) -> Option<&FloatLiteral> {
        match self {
            TokenKind::Float(value) => Some(value),
            _ => None,
        }
    }

    pub fn try_unwrap_integer(&self) -> Option<&i128> {
        match self {
            TokenKind::Integer(value) => Some(value),
            _ => None,
        }
    }

    pub fn try_unwrap_boolean(&self) -> Option<&bool> {
        match self {
            TokenKind::Boolean(value) => Some(value),
            _ => None,
        }
    }

    pub fn try_unwrap_string(&self) -> Option<&String> {
        match self {
            TokenKind::String(value) => Some(value),
            _ => None,
        }
    }

    pub fn try_unwrap_character(&self) -> Option<&char> {
        match self {
            TokenKind::Character(value) => Some(value),
            _ => None,
        }
    }

    pub fn try_unwrap_operator(&self) -> Option<&OperatorKind> {
        match self {
            TokenKind::Operator(value) => Some(value),
            _ => None,
        }
    }

    pub fn try_unwrap_identifier(&self) -> Option<&String> {
        match self {
            TokenKind::Identifier(value) => Some(value),
            _ => None,
        }
    }

    pub fn try_unwrap_punctuation(&self) -> Option<&PunctuationKind> {
        match self {
            TokenKind::Punctuation(value) => Some(value),
            _ => None,
        }
    }

    pub fn try_unwrap_comment(&self) -> Option<&String> {
        match self {
            TokenKind::Comment(value) => Some(value),
            _ => None,
        }
    }

    pub fn try_unwrap_float_mut(&mut self) -> Option<&mut FloatLiteral> {
        match self {
            TokenKind::Float(value) => Some(value),
            _ => None,
        }
    }

    pub fn try_unwrap_integer_mut(&mut self) -> Option<&mut i128> {
        match self {
            TokenKind::Integer(value) => Some(value),
            _ => None,
        }
    }

    pub fn try_unwrap_boolean_mut(&mut self) -> Option<&mut bool> {
        match self {
            TokenKind::Boolean(value) => Some(value),
            _ => None,
        }
    }

    pub fn try_unwrap_string_mut(&mut self) -> Option<&mut String> {
        match self {
            TokenKind::String(value) => Some(value),
            _ => None,
        }
    }

    pub fn try_unwrap_character_mut(&mut self) -> Option<&mut char> {
        match self {
            TokenKind::Character(value) => Some(value),
            _ => None,
        }
    }

    pub fn try_unwrap_operator_mut(&mut self) -> Option<&mut OperatorKind> {
        match self {
            TokenKind::Operator(value) => Some(value),
            _ => None,
        }
    }

    pub fn try_unwrap_identifier_mut(&mut self) -> Option<&mut String> {
        match self {
            TokenKind::Identifier(value) => Some(value),
            _ => None,
        }
    }

    pub fn try_unwrap_punctuation_mut(&mut self) -> Option<&mut PunctuationKind> {
        match self {
            TokenKind::Punctuation(value) => Some(value),
            _ => None,
        }
    }

    pub fn try_unwrap_comment_mut(&mut self) -> Option<&mut String> {
        match self {
            TokenKind::Comment(value) => Some(value),
            _ => None,
        }
    }

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