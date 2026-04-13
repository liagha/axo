use orbyte::Orbyte;
use crate::{
    data::*,
    parser::Symbol,
    resolver::{next_identity, Type, TypeKind},
    scanner::Token,
    tracker::Span,
};

#[derive(Orbyte)]
pub struct Element<'element> {
    pub identity: Identity,
    pub kind: ElementKind<'element>,
    pub span: Span,
    pub reference: Option<Identity>,
    pub typing: Type<'element>,
}

#[derive(Orbyte)]
pub enum ElementKind<'element> {
    Literal(Box<Token<'element>>),
    Delimited(Box<Delimited<Token<'element>, Element<'element>>>),
    Unary(Box<Unary<Token<'element>, Element<'element>>>),
    Binary(Box<Binary<Element<'element>, Token<'element>, Element<'element>>>),
    Index(Box<Index<Element<'element>, Element<'element>>>),
    Invoke(Box<Invoke<Element<'element>, Element<'element>>>),
    Construct(Box<Aggregate<Element<'element>, Element<'element>>>),
    Symbolize(Box<Symbol<'element>>),
}

impl<'element> Element<'element> {
    pub fn new(kind: ElementKind<'element>, span: Span) -> Self {
        Self {
            identity: next_identity(),
            kind,
            span,
            reference: None,
            typing: Type::from(TypeKind::Unknown),
        }
    }
}

impl<'element> ElementKind<'element> {
    #[inline]
    pub fn literal(token: Token<'element>) -> Self {
        Self::Literal(Box::new(token))
    }

    #[inline]
    pub fn delimited(delimited: Delimited<Token<'element>, Element<'element>>) -> Self {
        Self::Delimited(Box::new(delimited))
    }

    #[inline]
    pub fn unary(unary: Unary<Token<'element>, Element<'element>>) -> Self {
        Self::Unary(Box::new(unary))
    }

    #[inline]
    pub fn binary(binary: Binary<Element<'element>, Token<'element>, Element<'element>>) -> Self {
        Self::Binary(Box::new(binary))
    }

    #[inline]
    pub fn index(index: Index<Element<'element>, Element<'element>>) -> Self {
        Self::Index(Box::new(index))
    }

    #[inline]
    pub fn invoke(invoke: Invoke<Element<'element>, Element<'element>>) -> Self {
        Self::Invoke(Box::new(invoke))
    }

    #[inline]
    pub fn construct(construct: Aggregate<Element<'element>, Element<'element>>) -> Self {
        Self::Construct(Box::new(construct))
    }

    #[inline]
    pub fn symbolize(symbol: Symbol<'element>) -> Self {
        Self::Symbolize(Box::new(symbol))
    }

    #[inline(always)]
    pub fn is_literal(&self) -> bool {
        matches!(self, Self::Literal(_))
    }

    #[inline(always)]
    pub fn is_delimited(&self) -> bool {
        matches!(self, Self::Delimited(_))
    }

    #[inline(always)]
    pub fn is_unary(&self) -> bool {
        matches!(self, Self::Unary(_))
    }

    #[inline(always)]
    pub fn is_binary(&self) -> bool {
        matches!(self, Self::Binary(_))
    }

    #[inline(always)]
    pub fn is_index(&self) -> bool {
        matches!(self, Self::Index(_))
    }

    #[inline(always)]
    pub fn is_invoke(&self) -> bool {
        matches!(self, Self::Invoke(_))
    }

    #[inline(always)]
    pub fn is_construct(&self) -> bool {
        matches!(self, Self::Construct(_))
    }

    #[inline(always)]
    pub fn is_symbolize(&self) -> bool {
        matches!(self, Self::Symbolize(_))
    }

    #[inline]
    #[track_caller]
    pub fn unwrap_literal(self) -> Token<'element> {
        match self {
            Self::Literal(token) => *token,
            _ => panic!("expected literal"),
        }
    }

    #[inline]
    #[track_caller]
    pub fn unwrap_delimited(self) -> Delimited<Token<'element>, Element<'element>> {
        match self {
            Self::Delimited(delimited) => *delimited,
            _ => panic!("expected delimited"),
        }
    }

    #[inline]
    #[track_caller]
    pub fn unwrap_unary(self) -> Unary<Token<'element>, Element<'element>> {
        match self {
            Self::Unary(unary) => *unary,
            _ => panic!("expected unary"),
        }
    }

    #[inline]
    #[track_caller]
    pub fn unwrap_binary(self) -> Binary<Element<'element>, Token<'element>, Element<'element>> {
        match self {
            Self::Binary(binary) => *binary,
            _ => panic!("expected binary"),
        }
    }

    #[inline]
    #[track_caller]
    pub fn unwrap_index(self) -> Index<Element<'element>, Element<'element>> {
        match self {
            Self::Index(index) => *index,
            _ => panic!("expected index"),
        }
    }

    #[inline]
    #[track_caller]
    pub fn unwrap_invoke(self) -> Invoke<Element<'element>, Element<'element>> {
        match self {
            Self::Invoke(invoke) => *invoke,
            _ => panic!("expected invoke"),
        }
    }

    #[inline]
    #[track_caller]
    pub fn unwrap_construct(self) -> Aggregate<Element<'element>, Element<'element>> {
        match self {
            Self::Construct(construct) => *construct,
            _ => panic!("expected construct"),
        }
    }

    #[inline]
    #[track_caller]
    pub fn unwrap_symbolize(self) -> Symbol<'element> {
        match self {
            Self::Symbolize(symbol) => *symbol,
            _ => panic!("expected symbolize"),
        }
    }

    #[inline(always)]
    pub fn try_unwrap_literal(&self) -> Option<&Token<'element>> {
        match self {
            Self::Literal(token) => Some(&**token),
            _ => None,
        }
    }

    #[inline(always)]
    pub fn try_unwrap_delimited(&self) -> Option<&Delimited<Token<'element>, Element<'element>>> {
        match self {
            Self::Delimited(delimited) => Some(delimited),
            _ => None,
        }
    }

    #[inline(always)]
    pub fn try_unwrap_unary(&self) -> Option<&Unary<Token<'element>, Element<'element>>> {
        match self {
            Self::Unary(unary) => Some(unary),
            _ => None,
        }
    }

    #[inline(always)]
    pub fn try_unwrap_binary(&self) -> Option<&Binary<Element<'element>, Token<'element>, Element<'element>>> {
        match self {
            Self::Binary(binary) => Some(binary),
            _ => None,
        }
    }

    #[inline(always)]
    pub fn try_unwrap_index(&self) -> Option<&Index<Element<'element>, Element<'element>>> {
        match self {
            Self::Index(index) => Some(index),
            _ => None,
        }
    }

    #[inline(always)]
    pub fn try_unwrap_invoke(&self) -> Option<&Invoke<Element<'element>, Element<'element>>> {
        match self {
            Self::Invoke(invoke) => Some(invoke),
            _ => None,
        }
    }

    #[inline(always)]
    pub fn try_unwrap_construct(&self) -> Option<&Aggregate<Element<'element>, Element<'element>>> {
        match self {
            Self::Construct(construct) => Some(construct),
            _ => None,
        }
    }

    #[inline(always)]
    pub fn try_unwrap_symbolize(&self) -> Option<&Symbol<'element>> {
        match self {
            Self::Symbolize(symbol) => Some(symbol),
            _ => None,
        }
    }

    #[inline(always)]
    pub fn try_unwrap_literal_mut(&mut self) -> Option<&mut Token<'element>> {
        match self {
            Self::Literal(token) => Some(&mut **token),
            _ => None,
        }
    }

    #[inline(always)]
    pub fn try_unwrap_delimited_mut(&mut self) -> Option<&mut Delimited<Token<'element>, Element<'element>>> {
        match self {
            Self::Delimited(delimited) => Some(delimited),
            _ => None,
        }
    }

    #[inline(always)]
    pub fn try_unwrap_unary_mut(&mut self) -> Option<&mut Unary<Token<'element>, Element<'element>>> {
        match self {
            Self::Unary(unary) => Some(unary),
            _ => None,
        }
    }

    #[inline(always)]
    pub fn try_unwrap_binary_mut(&mut self) -> Option<&mut Binary<Element<'element>, Token<'element>, Element<'element>>> {
        match self {
            Self::Binary(binary) => Some(binary),
            _ => None,
        }
    }

    #[inline(always)]
    pub fn try_unwrap_index_mut(&mut self) -> Option<&mut Index<Element<'element>, Element<'element>>> {
        match self {
            Self::Index(index) => Some(index),
            _ => None,
        }
    }

    #[inline(always)]
    pub fn try_unwrap_invoke_mut(&mut self) -> Option<&mut Invoke<Element<'element>, Element<'element>>> {
        match self {
            Self::Invoke(invoke) => Some(invoke),
            _ => None,
        }
    }

    #[inline(always)]
    pub fn try_unwrap_construct_mut(&mut self) -> Option<&mut Aggregate<Element<'element>, Element<'element>>> {
        match self {
            Self::Construct(construct) => Some(construct),
            _ => None,
        }
    }

    #[inline(always)]
    pub fn try_unwrap_symbolize_mut(&mut self) -> Option<&mut Symbol<'element>> {
        match self {
            Self::Symbolize(symbol) => Some(symbol),
            _ => None,
        }
    }
}