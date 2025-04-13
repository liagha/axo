use std::collections::HashSet;
use core::hash::{Hash, Hasher};
use crate::axo_lexer::Span;
use crate::axo_parser::{Expr, ExprKind};
use crate::axo_semantic::ResolveError;

#[derive(Clone, Debug)]
pub struct Symbol {
    pub kind: SymbolKind,
    pub span: Span,
}

#[derive(Clone, Debug)]
pub enum SymbolKind {
    Expression(Expr),
    Field {
        name: Expr,
        field_type: Option<Expr>,
        default: Option<Expr>,
    },
    Variable {
        name: Expr,
        value: Option<Expr>,
        mutable: bool,
        ty: Option<Box<Expr>>,
    },
    Struct {
        name: Expr,
        fields: Vec<Symbol>,
    },
    Enum {
        name: Expr,
        variants: Vec<Symbol>,
    },
    Function {
        name: Expr,
        parameters: Vec<Expr>,
        body: Expr,
        return_type: Option<Box<Expr>>,
    },
    Macro {
        name: Expr,
        parameters: Vec<Expr>,
        body: Expr,
    },
    Trait {
        name: Expr,
        body: Expr,
        generic_params: Vec<Expr>,
    },
    Impl {
        trait_: Option<Box<Symbol>>,
        target: Expr,
        body: Expr,
    },
    Error(ResolveError),
}

impl PartialEq for Symbol {
    fn eq(&self, other: &Self) -> bool {
        self.kind == other.kind && self.span == other.span
    }
}

impl Eq for Symbol {}

impl Hash for Symbol {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.span.hash(state);
        self.kind.hash(state);
    }
}

impl Eq for SymbolKind {}

impl PartialEq for SymbolKind {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (SymbolKind::Expression(a), SymbolKind::Expression(b)) => a == b,

            (SymbolKind::Field { name: n1, .. }, SymbolKind::Field { name: n2, .. }) =>
                n1 == n2,

            (SymbolKind::Variable { name: n1, .. }, SymbolKind::Variable { name: n2, .. }) =>
                n1 == n2,

            (SymbolKind::Struct { name: n1, .. }, SymbolKind::Struct { name: n2, .. }) =>
                n1 == n2,

            (SymbolKind::Enum { name: n1, .. }, SymbolKind::Enum { name: n2, .. }) =>
                n1 == n2,

            (SymbolKind::Function { name: n1, parameters: p1, .. },
                SymbolKind::Function { name: n2, parameters: p2, .. }) =>
                n1 == n2 && p1 == p2,

            (SymbolKind::Macro { name: n1, parameters: p1, .. },
                SymbolKind::Macro { name: n2, parameters: p2, .. }) =>
                n1 == n2 && p1 == p2,

            (SymbolKind::Trait { name: n1, .. }, SymbolKind::Trait { name: n2, .. }) =>
                n1 == n2,

            (SymbolKind::Impl { trait_: t1, target: tg1, .. },
                SymbolKind::Impl { trait_: t2, target: tg2, .. }) =>
                match (t1, t2) {
                    (Some(t1), Some(t2)) => t1 == t2 && tg1 == tg2,
                    (None, None) => tg1 == tg2,
                    _ => false,
                },

            _ => false,
        }
    }
}

impl Hash for SymbolKind {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            SymbolKind::Expression(expr) => {
                0.hash(state);
                expr.hash(state);
            },
            SymbolKind::Field { name, .. } => {
                1.hash(state);
                name.hash(state);
            },
            SymbolKind::Variable { name, .. } => {
                3.hash(state);
                name.hash(state);
            },
            SymbolKind::Struct { name, .. } => {
                4.hash(state);
                name.hash(state);
            },
            SymbolKind::Enum { name, .. } => {
                5.hash(state);
                name.hash(state);
            },
            SymbolKind::Function { name, parameters, .. } => {
                6.hash(state);
                name.hash(state);
                parameters.hash(state);
            },
            SymbolKind::Macro { name, parameters, .. } => {
                7.hash(state);
                name.hash(state);
                parameters.hash(state);
            },
            SymbolKind::Trait { name, .. } => {
                8.hash(state);
                name.hash(state);
            },
            SymbolKind::Impl { trait_, target, .. } => {
                9.hash(state);
                if let Some(t) = trait_ {
                    t.hash(state);
                }
                target.hash(state);
            },
            SymbolKind::Error(_) => {
                10.hash(state);
            },
        }
    }
}

impl Symbol {
    pub fn get_name(&self) -> Option<String> {
        self.kind.get_name()
    }
}

impl SymbolKind {
    pub fn get_name(&self) -> Option<String> {
        match self {
            SymbolKind::Expression(_) => None,
            SymbolKind::Field { name, .. } => Some(name.to_string()),
            SymbolKind::Variable { name, .. } => Some(name.to_string()),
            SymbolKind::Struct { name, .. } => Some(name.to_string()),
            SymbolKind::Enum { name, .. } => Some(name.to_string()),
            SymbolKind::Function { name, .. } => Some(name.to_string()),
            SymbolKind::Macro { name, .. } => Some(name.to_string()),
            SymbolKind::Trait { name, .. } => Some(name.to_string()),
            _ => None,
        }
    }
}