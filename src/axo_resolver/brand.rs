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
    fn name(&self) -> Option<L>;
}

impl Branded<Token> for Element {
    fn name(&self) -> Option<Token> {
        match &self.kind {
            ElementKind::Literal(literal) => Some(Token {
                kind: literal.clone(),
                span: self.span,
            }),
            ElementKind::Identifier(identifier) => Some(Token {
                kind: TokenKind::Identifier(identifier.clone()),
                span: self.span,
            }),
            ElementKind::Constructor { name, .. } => name.name(),
            ElementKind::Labeled { label, .. } => label.name(),
            ElementKind::Index { element, .. } => element.name(),
            ElementKind::Invoke { target, .. } => target.name(),
            ElementKind::Member { object, .. } => object.name(),
            ElementKind::Symbolization(symbol) => symbol.name(),
            ElementKind::Assignment { target, .. } => target.name(),
            _ => None,
        }
    }
}

impl Branded<Token> for Symbol {
    fn name(&self) -> Option<Token> {
        self.kind.name()
    }
}

impl Branded<Token> for SymbolKind {
    fn name(&self) -> Option<Token> {
        match self {
            SymbolKind::Interface { name, .. } => name.name(),
            SymbolKind::Binding { target, .. } => target.name(),
            SymbolKind::Structure { name, .. } => name.name(),
            SymbolKind::Enumeration { name, .. } => name.name(),
            SymbolKind::Function { name, .. } => name.name(),
            _ => None,
        }
    }
}
