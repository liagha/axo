use {
    crate::{
        artifact::Artifact,

        hash::{
            Hash, Hasher
        },

        axo_form::form::Form,

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
    }
};

pub struct Symbol {
    pub kind: SymbolKind,
    pub span: Span,
}

pub enum SymbolKind {
    Inclusion {
        target: Box<Element> 
    },
    Formation {
        identifier: Artifact,
        form: Form<Artifact, Artifact, Artifact>,
    },
    Implementation {
        element: Box<Element>,
        body: Box<Element>
    },
    Interface {
        name: Box<Element>,
        body: Box<Element>
    },
    Slot {
        target: Box<Element>,
        value: Option<Box<Element>>,
        ty: Option<Box<Element>>,
    },
    Binding {
        target: Box<Element>,
        value: Option<Box<Element>>,
        ty: Option<Box<Element>>,
        mutable: bool,
    },
    Structure {
        name: Box<Element>,
        fields: Vec<Element>,
    },
    Enumeration {
        name: Box<Element>,
        variants: Vec<Element>,
    },
    Function {
        name: Box<Element>,
        parameters: Vec<Element>,
        body: Box<Element>
    },
}

impl Symbol {
    pub fn new(kind: SymbolKind, span: Span) -> Symbol {
        Symbol { kind, span }
    }
}