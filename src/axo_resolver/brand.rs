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
            ElementKind::Construct(construct) => construct.get_target().brand(),
            ElementKind::Label(label) => label.get_label().brand(),
            ElementKind::Index(index) => index.get_target().brand(),
            ElementKind::Invoke(invoke) => invoke.get_target().brand(),
            ElementKind::Access(access) => access.get_object().brand(),
            ElementKind::Symbolize(symbol) => symbol.brand(),
            ElementKind::Assign(assign) => assign.get_target().brand(),
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
            SymbolKind::Interface(interface) => interface.get_target().brand(),
            SymbolKind::Binding(binding) => binding.get_target().brand(),
            SymbolKind::Structure(structure) => structure.get_name().brand(),
            SymbolKind::Enumeration(enumeration) => enumeration.get_name().brand(),
            SymbolKind::Function(function) => function.get_name().brand(),
            _ => None,
        }
    }
}
