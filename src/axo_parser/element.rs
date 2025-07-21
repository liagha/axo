use {
    derive_more::{
        with_trait::{
            IsVariant, Unwrap,
        }
    },

    derive_ctor::{
        ctor
    },

    super::{
        Symbol, ParseError,
    },

    crate::{
        operations::{
            Deref, DerefMut
        },
        axo_schema::{
            Procedural,
            Group, Sequence,
            Collection, Series,
            Bundle, Block,
            Binary, Unary,
            Index, Invoke, Construct,
            Conditional, Repeat, Iterate,
            Label, Access, Assign,
        },

        axo_scanner::{
            Token, TokenKind,
            OperatorKind,
        },

        axo_cursor::Span,
    }
};

pub struct Element {
    pub kind: ElementKind,
    pub span: Span,
}

#[derive(ctor, IsVariant, Unwrap)]
pub enum ElementKind {
    Literal(TokenKind),

    Identifier(String),

    Procedural(Procedural<Box<Element>>),

    Group(Group<Element>),

    Sequence(Sequence<Element>),

    Collection(Collection<Element>),

    Series(Series<Element>),

    Bundle(Bundle<Element>),

    Block(Block<Element>),

    Unary(Unary<Token, Box<Element>>),

    Binary(Binary<Box<Element>, Token, Box<Element>>),

    Label(Label<Box<Element>, Box<Element>>),

    Access(Access<Box<Element>, Box<Element>>),

    Index(Index<Box<Element>, Element>),

    Invoke(Invoke<Box<Element>, Element>),

    Construct(Construct<Box<Element>, Element>),

    Conditional(Conditional<Box<Element>, Box<Element>, Box<Element>>),

    Repeat(Repeat<Box<Element>, Box<Element>>),

    Iterate(Iterate<Box<Element>, Box<Element>>),

    Symbolize(Symbol),

    Assign(Assign<Box<Element>, Box<Element>>),

    Produce(Option<Box<Element>>),

    Abort(Option<Box<Element>>),

    Pass(Option<Box<Element>>),
}

impl Element {
    pub fn new(kind: ElementKind, span: Span) -> Element {
        Element { kind, span }
    }
}

impl Deref for Element {
    type Target = ElementKind;

    fn deref(&self) -> &Self::Target {
        &self.kind
    }
}

impl DerefMut for Element {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.kind
    }
}