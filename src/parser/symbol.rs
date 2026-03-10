use {
    super::{Element, ElementKind},
    crate::{
        data::*,
        format::Debug,
        internal::hash::{Hash, Set},
        resolver::scope::Scope,
        scanner::{OperatorKind, Token, TokenKind},
        tracker::Span,
    },
};

use core::sync::atomic::{AtomicUsize, Ordering};

pub static COUNTER: AtomicUsize = AtomicUsize::new(0);

pub fn next_identity() -> Identity {
    COUNTER.fetch_add(1, Ordering::Relaxed)
}

pub struct Symbol<'symbol> {
    pub identity: Identity,
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
    pub fn new(kind: SymbolKind<'symbol>, span: Span<'symbol>, visibility: Visibility) -> Self {
        Self {
            identity: next_identity(),
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
            identity: self.identity,
            ..self
        }
    }

    pub fn set_members(&mut self, members: Vec<Symbol<'symbol>>) {
        self.scope.symbols.extend(members);
    }

    pub fn with_scope(self, scope: Scope<'symbol>) -> Self {
        Self {
            scope,
            identity: self.identity,
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
    Function(
        Function<
            Box<Element<'symbol>>,
            Symbol<'symbol>,
            Box<Element<'symbol>>,
            Option<Box<Element<'symbol>>>,
        >,
    ),
    Module(Module<Box<Element<'symbol>>>),
}

impl<'symbol> SymbolKind<'symbol> {
    pub fn brand(&self) -> Option<Token<'symbol>> {
        match self {
            SymbolKind::Binding(binding) => binding.target.clone().brand(),
            SymbolKind::Structure(structure) => structure.target.clone().brand(),
            SymbolKind::Function(function) => function.target.clone().brand(),
            SymbolKind::Module(module) => module.target.brand().clone(),
        }
    }
}

impl<'symbol> Element<'symbol> {
    pub fn brand(&self) -> Option<Token<'symbol>> {
        match &self.kind {
            ElementKind::Literal(literal) => {
                    Some(literal.clone())
            },
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
