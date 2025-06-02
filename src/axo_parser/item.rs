use {
    crate::{
        Peekable,
        
        hash::{
            Hash, Hasher
        },
        
        axo_lexer::{
            Token, TokenKind,
            PunctuationKind,
            OperatorKind,
        },
        axo_parser::{
            error::ErrorKind,
            Element, ElementKind,
            ParseError, Parser
        },
        axo_span::Span,
    }
};
use crate::axo_form::former::Form;
use crate::compiler::Artifact;

#[derive(Eq, Clone)]
pub struct Item {
    pub kind: ItemKind,
    pub span: Span,
}

pub enum ItemKind {
    Use(Box<Element>),
    Formed {
        identifier: usize,
        form: Form<Box<dyn Artifact>, Box<dyn Artifact>, Box<dyn Artifact>>,
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
    Field {
        name: Box<Element>,
        value: Option<Box<Element>>,
        ty: Option<Box<Element>>,
    },
    Structure {
        name: Box<Element>,
        fields: Vec<Element>
    },
    Enum {
        name: Box<Element>,
        body: Box<Element>,
    },
    Macro {
        name: Box<Element>,
        parameters: Vec<Element>,
        body: Box<Element>
    },
    Function {
        name: Box<Element>,
        parameters: Vec<Element>,
        body: Box<Element>
    },
    Unit,
}

impl Item {
    pub fn new(kind: ItemKind, span: Span) -> Item {
        Item { kind, span }
    }
}