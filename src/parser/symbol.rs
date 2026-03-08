use {
    super::{Element, ElementKind},
    crate::{
        data::*,
        format::Debug,
        initializer::Preference,
        internal::hash::{Hash, Set},
        resolver::scope::Scope,
        scanner::{OperatorKind, Token, TokenKind},
        tracker::Span,
    },
};

pub struct Symbol<'symbol> {
    pub id: Identity,
    pub usages: Set<Identity>,
    pub kind: SymbolKind<'symbol>,
    pub span: Span<'symbol>,
    pub scope: Scope<'symbol>,
    pub visibility: Visibility,
}

#[derive(Clone, Copy, Debug)]
pub enum Visibility {
    Public,
    Private,
}

impl<'symbol> Symbol<'symbol> {
    pub fn new(id: Identity, kind: SymbolKind<'symbol>, span: Span<'symbol>, visibility: Visibility) -> Self {
        Self {
            id,
            usages: Default::default(),
            kind,
            span,
            scope: Scope::new(),
            visibility,
        }
    }

    pub fn with_members<I: IntoIterator<Item=Symbol<'symbol>>>(self, members: I) -> Self {
        Self {
            scope: Scope {
                symbols: Set::from_iter(members),
                parent: None,
            },
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

    pub fn brand(&self) -> Option<Token<'symbol>> {
        self.kind.brand()
    }
}

#[derive(Clone, PartialEq, Hash)]
pub enum SymbolKind<'symbol> {
    Binding(Binding<Box<Element<'symbol>>, Box<Element<'symbol>>, Box<Element<'symbol>>>),
    Structure(Structure<Box<Element<'symbol>>, Symbol<'symbol>>),
    Enumeration(Structure<Box<Element<'symbol>>, Symbol<'symbol>>),
    Method(
        Method<
            Box<Element<'symbol>>,
            Symbol<'symbol>,
            Box<Element<'symbol>>,
            Option<Box<Element<'symbol>>>,
        >,
    ),
    Module(Module<Box<Element<'symbol>>>),
    Preference(Preference<'symbol>),
}

impl<'symbol> SymbolKind<'symbol> {
    pub fn brand(&self) -> Option<Token<'symbol>> {
        match self {
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
            ElementKind::Binary(binary) => match binary.operator.kind {
                TokenKind::Operator(OperatorKind::Colon) => binary.left.brand().clone(),
                TokenKind::Operator(OperatorKind::Composite(ref operators))
                if operators.as_slice() == [OperatorKind::Colon, OperatorKind::Colon] =>
                    {
                        binary.right.brand().clone()
                    }
                TokenKind::Operator(OperatorKind::Equal) => binary.left.brand().clone(),
                TokenKind::Operator(OperatorKind::Dot) => binary.right.brand().clone(),
                _ => None,
            },
            _ => None,
        }
    }
}
