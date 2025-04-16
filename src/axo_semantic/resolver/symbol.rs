use hashbrown::HashSet;
use core::hash::{Hash, Hasher};
use crate::axo_lexer::Span;
use crate::axo_parser::{Expr, ExprKind, Item, ItemKind};
use crate::axo_semantic::ResolveError;

impl PartialEq for Item {
    fn eq(&self, other: &Self) -> bool {
        self.kind == other.kind && self.span == other.span
    }
}

impl Eq for Item {}

impl Hash for Item {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.span.hash(state);
        self.kind.hash(state);
    }
}

impl Eq for ItemKind {}

impl PartialEq for ItemKind {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (ItemKind::Field { name: n1, .. }, ItemKind::Field { name: n2, .. }) =>
                n1 == n2,

            (ItemKind::Variable { target: n1, .. }, ItemKind::Variable { target: n2, .. }) =>
                n1 == n2,

            (ItemKind::Struct { name: n1, .. }, ItemKind::Struct { name: n2, .. }) =>
                n1 == n2,

            (ItemKind::Enum { name: n1, .. }, ItemKind::Enum { name: n2, .. }) =>
                n1 == n2,

            (ItemKind::Function { name: n1, parameters: p1, .. },
                ItemKind::Function { name: n2, parameters: p2, .. }) =>
                n1 == n2 && p1 == p2,

            (ItemKind::Macro { name: n1, parameters: p1, .. },
                ItemKind::Macro { name: n2, parameters: p2, .. }) =>
                n1 == n2 && p1 == p2,

            (ItemKind::Trait { name: n1, .. }, ItemKind::Trait { name: n2, .. }) =>
                n1 == n2,

            (ItemKind::Implement { expr: t1, body: tg1, .. },
                ItemKind::Implement { expr: t2, body: tg2, .. }) => t1 == t2 && tg1 == tg2,

            _ => false,
        }
    }
}

impl Hash for ItemKind {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            ItemKind::Expression(expr) => expr.hash(state),
            ItemKind::Field { name, .. } => {
                1.hash(state);
                name.hash(state);
            },
            ItemKind::Variable { target, .. } => {
                3.hash(state);
                target.hash(state);
            },
            ItemKind::Struct { name, .. } => {
                4.hash(state);
                name.hash(state);
            },
            ItemKind::Enum { name, .. } => {
                5.hash(state);
                name.hash(state);
            },
            ItemKind::Function { name, parameters, .. } => {
                6.hash(state);
                name.hash(state);
                parameters.hash(state);
            },
            ItemKind::Macro { name, parameters, .. } => {
                7.hash(state);
                name.hash(state);
                parameters.hash(state);
            },
            ItemKind::Trait { name, .. } => {
                8.hash(state);
                name.hash(state);
            },
            ItemKind::Implement { expr, body } => {
                9.hash(state);
                expr.hash(state);
                body.hash(state);
            },
            ItemKind::Use(expr) => {
                10.hash(state);

                expr.hash(state);
            }
            ItemKind::Unit => {
                11.hash(state);
            }
        }
    }
}

impl Item {
    pub fn get_name(&self) -> Option<String> {
        self.kind.get_name()
    }
}

impl ItemKind {
    pub fn get_name(&self) -> Option<String> {
        match self {
            ItemKind::Field { name, .. } => Some(name.to_string()),
            ItemKind::Variable { target, .. } => Some(target.to_string()),
            ItemKind::Struct { name, .. } => Some(name.to_string()),
            ItemKind::Enum { name, .. } => Some(name.to_string()),
            ItemKind::Function { name, .. } => Some(name.to_string()),
            ItemKind::Macro { name, .. } => Some(name.to_string()),
            ItemKind::Trait { name, .. } => Some(name.to_string()),
            _ => None,
        }
    }
}