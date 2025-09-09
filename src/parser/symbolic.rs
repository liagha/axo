use {
    super::{
        Element, ElementKind,
    },
    crate::{
        scanner::{
            Token, TokenKind,
            OperatorKind,
        },
        format::{
            Debug, Formatter, Result as FormatResult,
        },
        tracker::Span,
        schema::*,
        initial::{
            Preference,
        },
        internal::{
            hash::{Hash, Hasher, Set},
        },
        resolver::{
            Id,
            scope::Scope,
        },
        data::{
            any::{Any, TypeId},
            Boolean,
        },
    }
};

pub struct Symbol<'symbol> {
    pub id: Id,
    pub kind: SymbolKind<'symbol>,
    pub span: Span<'symbol>,
    pub scope: Scope<'symbol>,
    pub specifier: Specifier,
}

#[derive(Clone, Copy, Debug)]
pub struct Specifier {
    pub entry: Boolean,
    pub interface: Interface,
    pub visibility: Visibility,
}

#[derive(Clone, Copy, Debug)]
pub enum Visibility {
    Public,
    Private,
}

#[derive(Clone, Copy, Debug)]
pub enum Interface {
    C,
    Rust,
    Axo,
    Compiler
}

impl Specifier {
    pub fn apply(&mut self, application: Element<'_>) {
        match application.kind {
            ElementKind::Literal(
                Token {
                    kind: TokenKind::Identifier(identifier),
                    ..
                }
            ) => {
                match identifier.as_str().unwrap() {
                    "public" => {
                        self.visibility = Visibility::Public;
                    }

                    "private" => {
                        self.visibility = Visibility::Private;
                    }

                    "c" => {
                        self.interface = Interface::C;
                    }

                    "rust" => {
                        self.interface = Interface::Rust;
                    }

                    "axo" => {
                        self.interface = Interface::Axo;
                    }

                    "compiler" => {
                        self.interface = Interface::Compiler;
                    }

                    "entry" => {
                        self.entry = true;
                    }

                    _ => {}
                }
            }

            _ => {}
        }
    }
}

impl Default for Specifier {
    fn default() -> Self {
        Self {
            entry: false,
            interface: Interface::Axo,
            visibility: Visibility::Public,
        }
    }
}

impl<'symbol> Symbol<'symbol> {
    pub fn new(value: SymbolKind<'symbol>, span: Span<'symbol>, id: Id) -> Self {
        Self {
            id,
            kind: value,
            span,
            scope: Scope::new(),
            specifier: Specifier::default(),
        }
    }

    pub fn with_members<I: IntoIterator<Item = Symbol<'symbol>>>(self, members: I) -> Self {
        Self {
            scope: Scope { symbols: Set::from_iter(members), parent: None },
            id: self.id,
            ..self
        }
    }

    pub fn set_members(&mut self, members: Vec<Symbol<'symbol>>) {
        self.scope.symbols.extend(members);
    }

    pub fn with_scope(self, scope: Scope<'symbol>) -> Self {
        Self {
            scope,
            id: self.id,
            ..self
        }
    }

    pub fn set_scope(&mut self, scope: Scope<'symbol>) {
        self.scope = scope;
    }

    pub fn with_specifier(self, specifier: Specifier) -> Self {
        Self {
            specifier,
            ..self
        }
    }

    pub fn brand(&self) -> Option<Token<'symbol>> {
        self.kind.brand()
    }
}

#[derive(Clone, PartialEq, Hash)]
pub enum SymbolKind<'symbol> {
    Inclusion(Inclusion<Box<Element<'symbol>>>),
    Extension(Extension<Box<Element<'symbol>>, Box<Element<'symbol>>, Symbol<'symbol>>),
    Binding(Binding<Box<Element<'symbol>>, Box<Element<'symbol>>, Box<Element<'symbol>>>),
    Structure(Structure<Box<Element<'symbol>>, Symbol<'symbol>>),
    Enumeration(Structure<Box<Element<'symbol>>, Symbol<'symbol>>),
    Method(Method<Box<Element<'symbol>>, Symbol<'symbol>, Box<Element<'symbol>>, Option<Box<Element<'symbol>>>>),
    Module(Module<Box<Element<'symbol>>>),
    Preference(Preference<'symbol>),
}

impl<'symbol> SymbolKind<'symbol> {
    pub fn brand(&self) -> Option<Token<'symbol>> {
        match self {
            SymbolKind::Inclusion(inclusion) => inclusion.target.clone().brand(),
            SymbolKind::Extension(extension) => extension.target.clone().brand(),
            SymbolKind::Binding(binding) => binding.target.clone().brand(),
            SymbolKind::Structure(structure) => structure.target.clone().brand(),
            SymbolKind::Enumeration(enumeration) => enumeration.target.clone().brand(),
            SymbolKind::Method(method) => method.target.clone().brand(),
            SymbolKind::Module(module) => module.target.brand().clone(),
            SymbolKind::Preference(preference) => Some(preference.target.clone()),
        }
    }
}

impl<'symbol> Element<'symbol> {
    pub fn brand(&self) -> Option<Token<'symbol>> {
        match &self.kind {
            ElementKind::Literal(literal) => Some(literal.clone()),
            ElementKind::Construct(construct) => construct.target.brand(),
            ElementKind::Index(index) => index.target.brand(),
            ElementKind::Invoke(invoke) => invoke.target.brand(),
            ElementKind::Symbolize(symbol) => symbol.brand(),
            ElementKind::Binary(binary) => {
                match binary.operator.kind {
                    TokenKind::Operator(OperatorKind::Colon) => binary.left.brand().clone(),
                    TokenKind::Operator(OperatorKind::Equal) => binary.left.brand().clone(),
                    TokenKind::Operator(OperatorKind::Dot) => binary.right.brand().clone(),
                    _ => None,
                }
            }
            _ => None,
        }
    }
}