use {
    crate::{
        scanner::{Token},
        schema::*,
        tracker::{Span},
    },
    super::Symbol,
};

pub struct Element<'element> {
    pub kind: ElementKind<'element>,
    pub span: Span<'element>,
}

pub enum ElementKind<'element> {
    Literal(Token<'element>),

    Delimited(Delimited<Token<'element>, Element<'element>>),

    Unary(Unary<Token<'element>, Box<Element<'element>>>),

    Binary(Binary<Box<Element<'element>>, Token<'element>, Box<Element<'element>>>),
    
    Closure(Closure<Element<'element>, Box<Element<'element>>>),

    Index(Index<Box<Element<'element>>, Element<'element>>),

    Invoke(Invoke<Box<Element<'element>>, Element<'element>>),

    Construct(Structure<Box<Element<'element>>, Element<'element>>),

    Symbolize(Symbol<'element>),
}

impl<'element> Element<'element> {
    pub fn new(kind: ElementKind<'element>, span: Span<'element>) -> Element<'element> {
        Element { kind, span }
    }
}

impl<'element> ElementKind<'element> {
    #[inline]
    pub fn literal(kind: Token<'element>) -> Self {
        ElementKind::Literal(kind)
    }

    #[inline]
    pub fn delimited(delimited: Delimited<Token<'element>, Element<'element>>) -> Self {
        ElementKind::Delimited(delimited)
    }

    #[inline]
    pub fn unary(unary: Unary<Token<'element>, Box<Element<'element>>>) -> Self {
        ElementKind::Unary(unary)
    }

    #[inline]
    pub fn binary(binary: Binary<Box<Element<'element>>, Token<'element>, Box<Element<'element>>>) -> Self {
        ElementKind::Binary(binary)
    }

    #[inline]
    pub fn index(index: Index<Box<Element<'element>>, Element<'element>>) -> Self {
        ElementKind::Index(index)
    }

    #[inline]
    pub fn invoke(invoke: Invoke<Box<Element<'element>>, Element<'element>>) -> Self {
        ElementKind::Invoke(invoke)
    }

    #[inline]
    pub fn construct(construct: Structure<Box<Element<'element>>, Element<'element>>) -> Self {
        ElementKind::Construct(construct)
    }

    #[inline]
    pub fn symbolize(symbol: Symbol<'element>) -> Self {
        ElementKind::Symbolize(symbol)
    }

    #[inline(always)]
    pub fn is_literal(&self) -> bool {
        matches!(self, ElementKind::Literal(_))
    }

    #[inline(always)]
    pub fn is_delimited(&self) -> bool {
        matches!(self, ElementKind::Delimited(_))
    }

    #[inline(always)]
    pub fn is_unary(&self) -> bool {
        matches!(self, ElementKind::Unary(_))
    }

    #[inline(always)]
    pub fn is_binary(&self) -> bool {
        matches!(self, ElementKind::Binary(_))
    }

    #[inline(always)]
    pub fn is_index(&self) -> bool {
        matches!(self, ElementKind::Index(_))
    }

    #[inline(always)]
    pub fn is_invoke(&self) -> bool {
        matches!(self, ElementKind::Invoke(_))
    }

    #[inline(always)]
    pub fn is_construct(&self) -> bool {
        matches!(self, ElementKind::Construct(_))
    }

    #[inline(always)]
    pub fn is_symbolize(&self) -> bool {
        matches!(self, ElementKind::Symbolize(_))
    }

    #[inline]
    #[track_caller]
    pub fn unwrap_literal(self) -> Token<'element> {
        match self {
            ElementKind::Literal(token_kind) => token_kind,
            _ => panic!("called `unwrap_literal` on non-Literal variant."),
        }
    }

    #[inline]
    #[track_caller]
    pub fn unwrap_delimited(self) -> Delimited<Token<'element>, Element<'element>> {
        match self {
            ElementKind::Delimited(delimited) => delimited,
            _ => panic!("called `unwrap_delimited` on non-Group variant."),
        }
    }

    #[inline]
    #[track_caller]
    pub fn unwrap_unary(self) -> Unary<Token<'element>, Box<Element<'element>>> {
        match self {
            ElementKind::Unary(unary) => unary,
            _ => panic!("called `unwrap_unary` on non-Unary variant."),
        }
    }

    #[inline]
    #[track_caller]
    pub fn unwrap_binary(self) -> Binary<Box<Element<'element>>, Token<'element>, Box<Element<'element>>> {
        match self {
            ElementKind::Binary(binary) => binary,
            _ => panic!("called `unwrap_binary` on non-Binary variant."),
        }
    }

    #[inline]
    #[track_caller]
    pub fn unwrap_index(self) -> Index<Box<Element<'element>>, Element<'element>> {
        match self {
            ElementKind::Index(index) => index,
            _ => panic!("called `unwrap_index` on non-Index variant."),
        }
    }

    #[inline]
    #[track_caller]
    pub fn unwrap_invoke(self) -> Invoke<Box<Element<'element>>, Element<'element>> {
        match self {
            ElementKind::Invoke(invoke) => invoke,
            _ => panic!("called `unwrap_invoke` on non-Invoke variant."),
        }
    }

    #[inline]
    #[track_caller]
    pub fn unwrap_construct(self) -> Structure<Box<Element<'element>>, Element<'element>> {
        match self {
            ElementKind::Construct(construct) => construct,
            _ => panic!("called `unwrap_construct` on non-Construct variant."),
        }
    }

    #[inline]
    #[track_caller]
    pub fn unwrap_symbolize(self) -> Symbol<'element> {
        match self {
            ElementKind::Symbolize(symbol) => symbol,
            _ => panic!("called `unwrap_symbolize` on non-Symbolize variant."),
        }
    }

    #[inline(always)]
    pub fn try_unwrap_literal(&self) -> Option<&Token<'element>> {
        match self {
            ElementKind::Literal(token_kind) => Some(token_kind),
            _ => None,
        }
    }

    #[inline(always)]
    pub fn try_unwrap_delimited(&self) -> Option<&Delimited<Token<'element>, Element<'element>>> {
        match self {
            ElementKind::Delimited(delimited) => Some(delimited),
            _ => None,
        }
    }

    #[inline(always)]
    pub fn try_unwrap_unary(&self) -> Option<&Unary<Token<'element>, Box<Element<'element>>>> {
        match self {
            ElementKind::Unary(unary) => Some(unary),
            _ => None,
        }
    }

    #[inline(always)]
    pub fn try_unwrap_binary(&self) -> Option<&Binary<Box<Element<'element>>, Token<'element>, Box<Element<'element>>>> {
        match self {
            ElementKind::Binary(binary) => Some(binary),
            _ => None,
        }
    }

    #[inline(always)]
    pub fn try_unwrap_index(&self) -> Option<&Index<Box<Element<'element>>, Element<'element>>> {
        match self {
            ElementKind::Index(index) => Some(index),
            _ => None,
        }
    }

    #[inline(always)]
    pub fn try_unwrap_invoke(&self) -> Option<&Invoke<Box<Element<'element>>, Element<'element>>> {
        match self {
            ElementKind::Invoke(invoke) => Some(invoke),
            _ => None,
        }
    }

    #[inline(always)]
    pub fn try_unwrap_construct(&self) -> Option<&Structure<Box<Element<'element>>, Element<'element>>> {
        match self {
            ElementKind::Construct(construct) => Some(construct),
            _ => None,
        }
    }

    #[inline(always)]
    pub fn try_unwrap_symbolize(&self) -> Option<&Symbol<'element>> {
        match self {
            ElementKind::Symbolize(symbol) => Some(symbol),
            _ => None,
        }
    }

    #[inline(always)]
    pub fn try_unwrap_literal_mut(&mut self) -> Option<&mut Token<'element>> {
        match self {
            ElementKind::Literal(kind) => Some(kind),
            _ => None,
        }
    }

    #[inline(always)]
    pub fn try_unwrap_delimited_mut(&mut self) -> Option<&mut Delimited<Token<'element>, Element<'element>>> {
        match self {
            ElementKind::Delimited(delimited) => Some(delimited),
            _ => None,
        }
    }

    #[inline(always)]
    pub fn try_unwrap_unary_mut(&mut self) -> Option<&mut Unary<Token<'element>, Box<Element<'element>>>> {
        match self {
            ElementKind::Unary(unary) => Some(unary),
            _ => None,
        }
    }

    #[inline(always)]
    pub fn try_unwrap_binary_mut(&mut self) -> Option<&mut Binary<Box<Element<'element>>, Token<'element>, Box<Element<'element>>>> {
        match self {
            ElementKind::Binary(binary) => Some(binary),
            _ => None,
        }
    }

    #[inline(always)]
    pub fn try_unwrap_index_mut(&mut self) -> Option<&mut Index<Box<Element<'element>>, Element<'element>>> {
        match self {
            ElementKind::Index(index) => Some(index),
            _ => None,
        }
    }

    #[inline(always)]
    pub fn try_unwrap_invoke_mut(&mut self) -> Option<&mut Invoke<Box<Element<'element>>, Element<'element>>> {
        match self {
            ElementKind::Invoke(invoke) => Some(invoke),
            _ => None,
        }
    }

    #[inline(always)]
    pub fn try_unwrap_construct_mut(&mut self) -> Option<&mut Structure<Box<Element<'element>>, Element<'element>>> {
        match self {
            ElementKind::Construct(construct) => Some(construct),
            _ => None,
        }
    }

    #[inline(always)]
    pub fn try_unwrap_symbolize_mut(&mut self) -> Option<&mut Symbol<'element>> {
        match self {
            ElementKind::Symbolize(symbol) => Some(symbol),
            _ => None,
        }
    }
}