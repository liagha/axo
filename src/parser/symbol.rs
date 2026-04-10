use orbyte::Orbyte;
use crate::{
    data::*,
    format::Debug,
    internal::hash::{Hash, Set},
    parser::{Element, ElementKind},
    resolver::{next_identity, scope::Scope, Type, TypeKind},
    scanner::{OperatorKind, TokenKind},
    tracker::Span,
};

#[derive(Orbyte)]
pub struct Symbol<'symbol> {
    pub identity: Identity,
    pub usages: Set<Identity>,
    pub kind: SymbolKind<'symbol>,
    pub span: Span<'symbol>,
    pub scope: Scope,
    pub visibility: Visibility,
    pub typing: Type<'symbol>,
}

#[derive(Clone, Hash, Orbyte, PartialEq)]
pub enum SymbolKind<'symbol> {
    Binding(Binding<Box<Element<'symbol>>, Box<Element<'symbol>>, Option<Box<Element<'symbol>>>>),
    Structure(Aggregate<Box<Element<'symbol>>, Symbol<'symbol>>),
    Union(Aggregate<Box<Element<'symbol>>, Symbol<'symbol>>),
    Function(
        Function<
            Box<Element<'symbol>>,
            Symbol<'symbol>,
            Option<Box<Element<'symbol>>>,
            Option<Box<Element<'symbol>>>,
        >,
    ),
    Module(Module<Box<Element<'symbol>>>),
}

#[derive(Clone, Copy, Debug, Orbyte)]
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
            scope: Scope::new(None),
            visibility,
            typing: Type::from(TypeKind::Unknown),
        }
    }

    pub fn with_members<I: IntoIterator<Item = Symbol<'symbol>>>(self, members: I) -> Self {
        Self {
            scope: Scope {
                symbols: Set::from_iter(members.into_iter().map(|member| member.identity)),
                parent: None,
            },
            identity: self.identity,
            ..self
        }
    }

    pub fn set_members(&mut self, members: Vec<Symbol<'symbol>>) {
        self.scope
            .symbols
            .extend(members.into_iter().map(|member| member.identity));
    }

    pub fn with_scope(self, scope: Scope) -> Self {
        Self {
            scope,
            identity: self.identity,
            ..self
        }
    }

    pub fn set_scope(&mut self, scope: Scope) {
        self.scope = scope;
    }

    pub fn target(&self) -> Option<Str<'symbol>> {
        match &self.kind {
            SymbolKind::Binding(binding) => binding.target.target(),
            SymbolKind::Structure(structure) => structure.target.target(),
            SymbolKind::Union(union) => union.target.target(),
            SymbolKind::Function(function) => function.target.target(),
            SymbolKind::Module(module) => module.target.target(),
        }
    }
}

impl<'symbol> Element<'symbol> {
    pub fn target(&self) -> Option<Str<'symbol>> {
        match &self.kind {
            ElementKind::Literal(literal) => {
                if let TokenKind::Identifier(identifier) = literal.kind {
                    Some(identifier)
                } else {
                    None
                }
            }
            ElementKind::Construct(construct) => construct.target.target(),
            ElementKind::Index(index) => index.target.target(),
            ElementKind::Invoke(invoke) => invoke.target.target(),
            ElementKind::Symbolize(symbol) => symbol.target(),
            ElementKind::Binary(binary) => match binary.operator.kind {
                TokenKind::Operator(OperatorKind::Colon) => binary.left.target(),
                TokenKind::Operator(OperatorKind::Composite(ref operators))
                    if operators.as_slice() == [OperatorKind::Colon, OperatorKind::Colon] =>
                {
                    binary.right.target()
                }
                TokenKind::Operator(OperatorKind::Equal) => binary.left.target(),
                TokenKind::Operator(OperatorKind::Dot) => binary.right.target(),
                _ => None,
            },
            _ => None,
        }
    }
}
