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

#[derive(Clone)]
pub struct Symbol {
    pub kind: SymbolKind,
    pub span: Span,
}

pub enum SymbolKind {
    Inclusion {
        target: Box<Element> 
    },
    Field {
        name: Box<Element>,
        value: Option<Box<Element>>,
        ty: Option<Box<Element>>,
    },
    Formed {
        identifier: Artifact,
        form: Form<Artifact, Artifact, Artifact>,
    },
    Implement {
        element: Box<Element>,
        body: Box<Element>
    },
    Trait {
        name: Box<Element>,
        body: Box<Element>
    },
    Variable {
        target: Box<Element>,
        value: Option<Box<Element>>,
        ty: Option<Box<Element>>,
        mutable: bool,
    },
    Structure {
        name: Box<Element>,
        fields: Vec<Element>
    },
    Enumeration {
        name: Box<Element>,
        body: Box<Element>,
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