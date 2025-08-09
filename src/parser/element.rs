use {
    crate::{
        internal::operation::{Deref, DerefMut},
        scanner::{Token, TokenKind},
        schema::{
            Access, Assign, Binary, Block, Bundle, Collection, Conditional, Construct, Group,
            Index, Invoke, Iterate, Label, Procedural, Repeat, Sequence, Series, Unary,
        },
        tracker::Span,
    },
    derive_ctor::ctor,
    derive_more::with_trait::{IsVariant, Unwrap},
    super::Symbol,
};

pub struct Element<'element> {
    pub kind: ElementKind<'element>,
    pub span: Span<'element>,
}

#[derive(ctor, IsVariant, Unwrap)]
pub enum ElementKind<'element> {
    Literal(TokenKind),

    Identifier(String),

    Procedural(Procedural<Box<Element<'element>>>),

    Group(Group<Element<'element>>),

    Sequence(Sequence<Element<'element>>),

    Collection(Collection<Element<'element>>),

    Series(Series<Element<'element>>),

    Bundle(Bundle<Element<'element>>),

    Block(Block<Element<'element>>),

    Unary(Unary<Token<'element>, Box<Element<'element>>>),

    Binary(Binary<Box<Element<'element>>, Token<'element>, Box<Element<'element>>>),

    Label(Label<Box<Element<'element>>, Box<Element<'element>>>),

    Access(Access<Box<Element<'element>>, Box<Element<'element>>>),

    Index(Index<Box<Element<'element>>, Element<'element>>),

    Invoke(Invoke<Box<Element<'element>>, Element<'element>>),

    Construct(Construct<Box<Element<'element>>, Element<'element>>),

    Conditional(Conditional<Box<Element<'element>>, Box<Element<'element>>, Box<Element<'element>>>),

    Repeat(Repeat<Box<Element<'element>>, Box<Element<'element>>>),

    Iterate(Iterate<Box<Element<'element>>, Box<Element<'element>>>),

    Symbolize(Symbol),

    Assign(Assign<Box<Element<'element>>, Box<Element<'element>>>),

    Produce(Option<Box<Element<'element>>>),

    Abort(Option<Box<Element<'element>>>),

    Pass(Option<Box<Element<'element>>>),
}

impl<'element> Element<'element> {
    pub fn new(kind: ElementKind<'element>, span: Span<'element>) -> Element<'element> {
        Element { kind, span }
    }
}

impl<'element> Deref for Element<'element> {
    type Target = ElementKind<'element>;

    fn deref(&self) -> &Self::Target {
        &self.kind
    }
}

impl<'element> DerefMut for Element<'element> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.kind
    }
}