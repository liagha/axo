use {
    crate::{
        axo_scanner::{
            Token, TokenKind
        },
        axo_parser::{
            Element, ElementKind, 
            Symbol, SymbolKind
        }
    }
};

/// Labeled Trait and Implementations

/// Trait for extracting a meaningful name token from various AST elements
pub trait Branded<L> {
    fn brand(&self) -> Option<L>;
}

impl Branded<Token> for Element {
    fn brand(&self) -> Option<Token> {
        match &self.kind {
            ElementKind::Literal(literal) => Some(Token {
                kind: literal.clone(),
                span: self.span,
            }),
            ElementKind::Identifier(identifier) => Some(Token {
                kind: TokenKind::Identifier(identifier.clone()),
                span: self.span,
            }),
            ElementKind::Constructor { name, .. } => name.brand(),
            ElementKind::Labeled { label, .. } => label.brand(),
            ElementKind::Index { target: element, .. } => element.brand(),
            ElementKind::Invoke { target, .. } => target.brand(),
            ElementKind::Member { object, .. } => object.brand(),
            ElementKind::Symbolization(symbol) => symbol.brand(),
            ElementKind::Assignment { target, .. } => target.brand(),
            _ => None,
        }
    }
}

impl Branded<Token> for Symbol {
    fn brand(&self) -> Option<Token> {
        self.kind.brand()
    }
}

impl Branded<Token> for SymbolKind {
    fn brand(&self) -> Option<Token> {
        match self {
            SymbolKind::Interface { name, .. } => name.brand(),
            SymbolKind::Binding { target, .. } => target.brand(),
            SymbolKind::Structure { name, .. } => name.brand(),
            SymbolKind::Enumeration { name, .. } => name.brand(),
            SymbolKind::Function { name, .. } => name.brand(),
            _ => None,
        }
    }
}
