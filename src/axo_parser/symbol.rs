use {
    derive_more::Unwrap,
    crate::{
        artifact::Artifact,

        hash::{
            Hash, Hasher
        },

        axo_form::form::Form,

        axo_schema::{
            Group, Sequence,
            Collection, Series,
            Bundle, Scope,
            Binary, Unary,
            Index, Invoke, Construct,
            Structure, Enumeration,
            Binding, Function, Interface, Implementation, Formation, Inclusion,
            Conditional, Repeat, Iterate,
            Label, Access, Assign,
        },

        axo_scanner::{
            Token, TokenKind,
            PunctuationKind,
            OperatorKind,
        },
        axo_parser::{
            error::ErrorKind,
            Element, ElementKind,
            ParseError, Parser
        },
        axo_cursor::Span,
    },
};

pub struct Symbol {
    pub kind: SymbolKind,
    pub span: Span,
}


#[derive(Unwrap)]
pub enum SymbolKind {
    Formation(Formation),
    Inclusion(Inclusion<Box<Element>>),
    Implementation(Implementation<Box<Element>, Box<Element>>),
    Interface(Interface<Box<Element>, Box<Element>>),
    Binding(Binding<Box<Element>, Box<Element>, Box<Element>>),
    Structure(Structure<Box<Element>, Element>),
    Enumeration(Enumeration<Box<Element>, Element>),
    Function(Function<Box<Element>, Element, Box<Element>>),
}

impl Symbol {
    pub fn new(kind: SymbolKind, span: Span) -> Symbol {
        Symbol { kind, span }
    }
}