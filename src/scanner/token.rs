use {
    super::{
        OperatorKind, PunctuationKind
    },

    crate::{
        data::{
            Boolean, Char, Integer, Str, Float,
        },
        tracker::Span,
    }
};

#[derive(Clone, Eq, Hash, PartialEq)]
pub struct Token<'token> {
    pub kind: TokenKind<'token>,
    pub span: Span<'token>,
}

#[derive(Clone, Eq, Hash, PartialEq)]
pub enum TokenKind<'token> {
    Float(Float),
    Integer(Integer),
    Boolean(Boolean),
    String(Str<'token>),
    Character(Char),
    Operator(OperatorKind),
    Identifier(Str<'token>),
    Punctuation(PunctuationKind),
    Comment(Str<'token>),
}

impl<'token> Token<'token> {
    pub fn new(kind: TokenKind<'token>, span: Span<'token>) -> Self {
        Self { kind, span }
    }
}

impl<'token> TokenKind<'token> {
    #[inline]
    pub fn float(value: Float) -> Self {
        TokenKind::Float(value)
    }

    #[inline]
    pub fn integer(value: Integer) -> Self {
        TokenKind::Integer(value)
    }

    #[inline]
    pub fn boolean(value: Boolean) -> Self {
        TokenKind::Boolean(value)
    }

    #[inline]
    pub fn string(value: Str<'token>) -> Self {
        TokenKind::String(value)
    }

    #[inline]
    pub fn character(value: Char) -> Self {
        TokenKind::Character(value)
    }

    #[inline]
    pub fn operator(value: OperatorKind) -> Self {
        TokenKind::Operator(value)
    }

    #[inline]
    pub fn identifier(value: Str<'token>) -> Self {
        TokenKind::Identifier(value)
    }

    #[inline]
    pub fn punctuation(value: PunctuationKind) -> Self {
        TokenKind::Punctuation(value)
    }

    #[inline]
    pub fn comment(value: Str<'token>) -> Self {
        TokenKind::Comment(value)
    }

    #[inline(always)]
    pub fn is_float(&self) -> bool {
        matches!(self, TokenKind::Float(_))
    }

    #[inline(always)]
    pub fn is_integer(&self) -> bool {
        matches!(self, TokenKind::Integer(_))
    }

    #[inline(always)]
    pub fn is_boolean(&self) -> bool {
        matches!(self, TokenKind::Boolean(_))
    }

    #[inline(always)]
    pub fn is_string(&self) -> bool {
        matches!(self, TokenKind::String(_))
    }

    #[inline(always)]
    pub fn is_character(&self) -> bool {
        matches!(self, TokenKind::Character(_))
    }

    #[inline(always)]
    pub fn is_operator(&self) -> bool {
        matches!(self, TokenKind::Operator(_))
    }

    #[inline(always)]
    pub fn is_identifier(&self) -> bool {
        matches!(self, TokenKind::Identifier(_))
    }

    #[inline(always)]
    pub fn is_punctuation(&self) -> bool {
        matches!(self, TokenKind::Punctuation(_))
    }

    #[inline(always)]
    pub fn is_comment(&self) -> bool {
        matches!(self, TokenKind::Comment(_))
    }

    #[inline]
    #[track_caller]
    pub fn unwrap_float(self) -> Float {
        match self {
            TokenKind::Float(value) => value,
            _ => panic!("called `unwrap_float` on non-Float variant."),
        }
    }

    #[inline]
    #[track_caller]
    pub fn unwrap_integer(self) -> Integer {
        match self {
            TokenKind::Integer(value) => value,
            _ => panic!("called `unwrap_integer` on non-Integer variant."),
        }
    }

    #[inline]
    #[track_caller]
    pub fn unwrap_boolean(self) -> Boolean {
        match self {
            TokenKind::Boolean(value) => value,
            _ => panic!("called `unwrap_boolean` on non-Boolean variant."),
        }
    }

    #[inline]
    #[track_caller]
    pub fn unwrap_string(self) -> Str<'token> {
        match self {
            TokenKind::String(value) => value,
            _ => panic!("called `unwrap_string` on non-String variant."),
        }
    }

    #[inline]
    #[track_caller]
    pub fn unwrap_character(self) -> Char {
        match self {
            TokenKind::Character(value) => value,
            _ => panic!("called `unwrap_character` on non-Character variant."),
        }
    }

    #[inline]
    #[track_caller]
    pub fn unwrap_operator(self) -> OperatorKind {
        match self {
            TokenKind::Operator(value) => value,
            _ => panic!("called `unwrap_operator` on non-Operator variant."),
        }
    }

    #[inline]
    #[track_caller]
    pub fn unwrap_identifier(self) -> Str<'token> {
        match self {
            TokenKind::Identifier(value) => value,
            _ => panic!("called `unwrap_identifier` on non-Identifier variant."),
        }
    }

    #[inline]
    #[track_caller]
    pub fn unwrap_punctuation(self) -> PunctuationKind {
        match self {
            TokenKind::Punctuation(value) => value,
            _ => panic!("called `unwrap_punctuation` on non-Punctuation variant."),
        }
    }

    #[inline]
    #[track_caller]
    pub fn unwrap_comment(self) -> Str<'token> {
        match self {
            TokenKind::Comment(value) => value,
            _ => panic!("called `unwrap_comment` on non-Comment variant."),
        }
    }

    #[inline(always)]
    pub fn try_unwrap_float(&self) -> Option<&Float> {
        match self {
            TokenKind::Float(value) => Some(value),
            _ => None,
        }
    }

    #[inline(always)]
    pub fn try_unwrap_integer(&self) -> Option<&Integer> {
        match self {
            TokenKind::Integer(value) => Some(value),
            _ => None,
        }
    }

    #[inline(always)]
    pub fn try_unwrap_boolean(&self) -> Option<&Boolean> {
        match self {
            TokenKind::Boolean(value) => Some(value),
            _ => None,
        }
    }

    #[inline(always)]
    pub fn try_unwrap_string(&self) -> Option<&Str<'token>> {
        match self {
            TokenKind::String(value) => Some(value),
            _ => None,
        }
    }

    #[inline(always)]
    pub fn try_unwrap_character(&self) -> Option<&Char> {
        match self {
            TokenKind::Character(value) => Some(value),
            _ => None,
        }
    }

    #[inline(always)]
    pub fn try_unwrap_operator(&self) -> Option<&OperatorKind> {
        match self {
            TokenKind::Operator(value) => Some(value),
            _ => None,
        }
    }

    #[inline(always)]
    pub fn try_unwrap_identifier(&self) -> Option<&Str<'token>> {
        match self {
            TokenKind::Identifier(value) => Some(value),
            _ => None,
        }
    }

    #[inline(always)]
    pub fn try_unwrap_punctuation(&self) -> Option<&PunctuationKind> {
        match self {
            TokenKind::Punctuation(value) => Some(value),
            _ => None,
        }
    }

    #[inline(always)]
    pub fn try_unwrap_comment(&self) -> Option<&Str<'token>> {
        match self {
            TokenKind::Comment(value) => Some(value),
            _ => None,
        }
    }

    #[inline(always)]
    pub fn try_unwrap_float_mut(&mut self) -> Option<&mut Float> {
        match self {
            TokenKind::Float(value) => Some(value),
            _ => None,
        }
    }

    #[inline(always)]
    pub fn try_unwrap_integer_mut(&mut self) -> Option<&mut Integer> {
        match self {
            TokenKind::Integer(value) => Some(value),
            _ => None,
        }
    }

    #[inline(always)]
    pub fn try_unwrap_boolean_mut(&mut self) -> Option<&mut Boolean> {
        match self {
            TokenKind::Boolean(value) => Some(value),
            _ => None,
        }
    }

    #[inline(always)]
    pub fn try_unwrap_string_mut(&mut self) -> Option<&mut Str<'token>> {
        match self {
            TokenKind::String(value) => Some(value),
            _ => None,
        }
    }

    #[inline(always)]
    pub fn try_unwrap_character_mut(&mut self) -> Option<&mut Char> {
        match self {
            TokenKind::Character(value) => Some(value),
            _ => None,
        }
    }

    #[inline(always)]
    pub fn try_unwrap_operator_mut(&mut self) -> Option<&mut OperatorKind> {
        match self {
            TokenKind::Operator(value) => Some(value),
            _ => None,
        }
    }

    #[inline(always)]
    pub fn try_unwrap_identifier_mut(&mut self) -> Option<&mut Str<'token>> {
        match self {
            TokenKind::Identifier(value) => Some(value),
            _ => None,
        }
    }

    #[inline(always)]
    pub fn try_unwrap_punctuation_mut(&mut self) -> Option<&mut PunctuationKind> {
        match self {
            TokenKind::Punctuation(value) => Some(value),
            _ => None,
        }
    }

    #[inline(always)]
    pub fn try_unwrap_comment_mut(&mut self) -> Option<&mut Str<'token>> {
        match self {
            TokenKind::Comment(value) => Some(value),
            _ => None,
        }
    }
}