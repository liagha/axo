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

#[derive(Eq, Clone)]
pub struct Item {
    pub kind: ItemKind,
    pub span: Span,
}

#[derive(Eq, Clone)]
pub enum ItemKind {
    Use(Box<Element>),
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
        fields: Vec<Item>
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