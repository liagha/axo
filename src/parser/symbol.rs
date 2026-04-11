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
    pub typing: Box<Type<'symbol>>,
}

#[derive(Clone, Hash, Orbyte, PartialEq)]
pub enum SymbolKind<'symbol> {
    Binding(Box<Binding<Box<Element<'symbol>>, Box<Element<'symbol>>, Option<Box<Element<'symbol>>>>>),
    Structure(Box<Aggregate<Box<Element<'symbol>>, Symbol<'symbol>>>),
    Union(Box<Aggregate<Box<Element<'symbol>>, Symbol<'symbol>>>),
    Function(Box<Function<Box<Element<'symbol>>, Symbol<'symbol>, Option<Box<Element<'symbol>>>, Option<Box<Element<'symbol>>>>>),
    Module(Box<Module<Box<Element<'symbol>>>>),
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
            typing: Box::from(Type::from(TypeKind::Unknown)),
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
        self.scope.symbols.extend(members.into_iter().map(|member| member.identity));
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

impl<'symbol> SymbolKind<'symbol> {
    #[inline]
    pub fn binding(binding: Binding<Box<Element<'symbol>>, Box<Element<'symbol>>, Option<Box<Element<'symbol>>>>) -> Self {
        Self::Binding(Box::new(binding))
    }

    #[inline]
    pub fn structure(structure: Aggregate<Box<Element<'symbol>>, Symbol<'symbol>>) -> Self {
        Self::Structure(Box::new(structure))
    }

    #[inline]
    pub fn union(union: Aggregate<Box<Element<'symbol>>, Symbol<'symbol>>) -> Self {
        Self::Union(Box::new(union))
    }

    #[inline]
    pub fn function(function: Function<Box<Element<'symbol>>, Symbol<'symbol>, Option<Box<Element<'symbol>>>, Option<Box<Element<'symbol>>>>) -> Self {
        Self::Function(Box::new(function))
    }

    #[inline]
    pub fn module(module: Module<Box<Element<'symbol>>>) -> Self {
        Self::Module(Box::new(module))
    }

    #[inline(always)]
    pub fn is_binding(&self) -> bool {
        matches!(self, Self::Binding(_))
    }

    #[inline(always)]
    pub fn is_structure(&self) -> bool {
        matches!(self, Self::Structure(_))
    }

    #[inline(always)]
    pub fn is_union(&self) -> bool {
        matches!(self, Self::Union(_))
    }

    #[inline(always)]
    pub fn is_function(&self) -> bool {
        matches!(self, Self::Function(_))
    }

    #[inline(always)]
    pub fn is_module(&self) -> bool {
        matches!(self, Self::Module(_))
    }

    #[inline]
    #[track_caller]
    pub fn unwrap_binding(self) -> Binding<Box<Element<'symbol>>, Box<Element<'symbol>>, Option<Box<Element<'symbol>>>> {
        match self {
            Self::Binding(binding) => *binding,
            _ => panic!("expected binding"),
        }
    }

    #[inline]
    #[track_caller]
    pub fn unwrap_structure(self) -> Aggregate<Box<Element<'symbol>>, Symbol<'symbol>> {
        match self {
            Self::Structure(structure) => *structure,
            _ => panic!("expected structure"),
        }
    }

    #[inline]
    #[track_caller]
    pub fn unwrap_union(self) -> Aggregate<Box<Element<'symbol>>, Symbol<'symbol>> {
        match self {
            Self::Union(union) => *union,
            _ => panic!("expected union"),
        }
    }

    #[inline]
    #[track_caller]
    pub fn unwrap_function(self) -> Function<Box<Element<'symbol>>, Symbol<'symbol>, Option<Box<Element<'symbol>>>, Option<Box<Element<'symbol>>>> {
        match self {
            Self::Function(function) => *function,
            _ => panic!("expected function"),
        }
    }

    #[inline]
    #[track_caller]
    pub fn unwrap_module(self) -> Module<Box<Element<'symbol>>> {
        match self {
            Self::Module(module) => *module,
            _ => panic!("expected module"),
        }
    }

    #[inline(always)]
    pub fn try_unwrap_binding(&self) -> Option<&Binding<Box<Element<'symbol>>, Box<Element<'symbol>>, Option<Box<Element<'symbol>>>>> {
        match self {
            Self::Binding(binding) => Some(binding),
            _ => None,
        }
    }

    #[inline(always)]
    pub fn try_unwrap_structure(&self) -> Option<&Aggregate<Box<Element<'symbol>>, Symbol<'symbol>>> {
        match self {
            Self::Structure(structure) => Some(structure),
            _ => None,
        }
    }

    #[inline(always)]
    pub fn try_unwrap_union(&self) -> Option<&Aggregate<Box<Element<'symbol>>, Symbol<'symbol>>> {
        match self {
            Self::Union(union) => Some(union),
            _ => None,
        }
    }

    #[inline(always)]
    pub fn try_unwrap_function(&self) -> Option<&Function<Box<Element<'symbol>>, Symbol<'symbol>, Option<Box<Element<'symbol>>>, Option<Box<Element<'symbol>>>>> {
        match self {
            Self::Function(function) => Some(function),
            _ => None,
        }
    }

    #[inline(always)]
    pub fn try_unwrap_module(&self) -> Option<&Module<Box<Element<'symbol>>>> {
        match self {
            Self::Module(module) => Some(module),
            _ => None,
        }
    }

    #[inline(always)]
    pub fn try_unwrap_binding_mut(&mut self) -> Option<&mut Binding<Box<Element<'symbol>>, Box<Element<'symbol>>, Option<Box<Element<'symbol>>>>> {
        match self {
            Self::Binding(binding) => Some(binding),
            _ => None,
        }
    }

    #[inline(always)]
    pub fn try_unwrap_structure_mut(&mut self) -> Option<&mut Aggregate<Box<Element<'symbol>>, Symbol<'symbol>>> {
        match self {
            Self::Structure(structure) => Some(structure),
            _ => None,
        }
    }

    #[inline(always)]
    pub fn try_unwrap_union_mut(&mut self) -> Option<&mut Aggregate<Box<Element<'symbol>>, Symbol<'symbol>>> {
        match self {
            Self::Union(union) => Some(union),
            _ => None,
        }
    }

    #[inline(always)]
    pub fn try_unwrap_function_mut(&mut self) -> Option<&mut Function<Box<Element<'symbol>>, Symbol<'symbol>, Option<Box<Element<'symbol>>>, Option<Box<Element<'symbol>>>>> {
        match self {
            Self::Function(function) => Some(function),
            _ => None,
        }
    }

    #[inline(always)]
    pub fn try_unwrap_module_mut(&mut self) -> Option<&mut Module<Box<Element<'symbol>>>> {
        match self {
            Self::Module(module) => Some(module),
            _ => None,
        }
    }
}

impl<'symbol> Element<'symbol> {
    pub fn target(&self) -> Option<Str<'symbol>> {
        match &self.kind {
            ElementKind::Literal(literal) => match &literal.kind {
                TokenKind::Identifier(identifier) => Some(**identifier),
                _ => None,
            },
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
