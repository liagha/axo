use {
    derive_more::{
        with_trait::Unwrap
    },

    super::{
        SymbolKind, ParseError,
    },

    crate::{
        axo_data::tree::{
            Node, Tree
        },

        axo_schema::{
            Group, Sequence,
            Collection, Series,
            Bundle, Scope,
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

#[derive(Unwrap)]
pub enum ElementKind {
    Literal(TokenKind),

    Identifier(String),

    Procedural(Box<Element>),

    Group(Group<Element>),

    Sequence(Sequence<Element>),

    Collection(Collection<Element>),

    Series(Series<Element>),

    Bundle(Bundle<Element>),

    Scope(Scope<Element>),

    Unary(Unary<Token, Box<Element>>),

    Binary(Binary<Box<Element>, Token, Box<Element>>),

    Label(Label<Box<Element>, Box<Element>>),

    Access(Access<Box<Element>, Box<Element>>),

    Index(Index<Box<Element>, Element>),

    Invoke(Invoke<Box<Element>, Element>),

    Construct(Construct<Box<Element>, Element>),

    Domain(Tree<Box<Element>>),

    Conditional(Conditional<Box<Element>, Box<Element>, Box<Element>>),

    Repeat(Repeat<Box<Element>, Box<Element>>),

    Iterate(Iterate<Box<Element>, Box<Element>>),

    Symbolize(SymbolKind),

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