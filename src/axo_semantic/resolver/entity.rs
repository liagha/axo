use std::collections::HashSet;
use crate::axo_parser::{Expr, ExprKind};

#[derive(Debug)]
pub enum EntityKind {
    Variable,
    Function,
    Struct,
    Enum,
    Trait,
    Impl,
    Macro,
    Field,
    Variant,
    Expression,
}

#[derive(Clone, Debug)]
pub enum Entity {
    Expression(Expr),
    Field {
        name: Expr,
        field_type: Option<Expr>,
        default: Option<Expr>,
    },
    Variant {
        name: Expr,
    },
    Variable {
        name: Expr,
        value: Option<Expr>,
        mutable: bool,
        type_annotation: Option<Box<Expr>>,
    },
    Struct {
        name: Expr,
        fields: Vec<Entity>,
        generic_params: Vec<Expr>,
    },
    Enum {
        name: Expr,
        variants: Vec<Entity>,
        generic_params: Vec<Expr>,
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
        trait_: Option<Box<Entity>>,
        target: Expr,
        body: Expr,
    }
}

impl Eq for Entity {}



impl PartialEq for Entity {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Entity::Expression(a), Entity::Expression(b)) => a == b,

            (Entity::Field { name: n1, .. }, Entity::Field { name: n2, .. }) =>
                n1 == n2,

            (Entity::Variant { name: n1 }, Entity::Variant { name: n2 }) =>
                n1 == n2,

            (Entity::Variable { name: n1, .. }, Entity::Variable { name: n2, .. }) =>
                n1 == n2,

            (Entity::Struct { name: n1, .. }, Entity::Struct { name: n2, .. }) =>
                n1 == n2,

            (Entity::Enum { name: n1, .. }, Entity::Enum { name: n2, .. }) =>
                n1 == n2,

            (Entity::Function { name: n1, parameters: p1, .. },
                Entity::Function { name: n2, parameters: p2, .. }) =>
                n1 == n2 && p1 == p2,

            (Entity::Macro { name: n1, parameters: p1, .. },
                Entity::Macro { name: n2, parameters: p2, .. }) =>
                n1 == n2 && p1 == p2,

            (Entity::Trait { name: n1, .. }, Entity::Trait { name: n2, .. }) =>
                n1 == n2,

            (Entity::Impl { trait_: t1, target: tg1, .. },
                Entity::Impl { trait_: t2, target: tg2, .. }) =>
                match (t1, t2) {
                    (Some(t1), Some(t2)) => t1 == t2 && tg1 == tg2,
                    (None, None) => tg1 == tg2,
                    _ => false,
                },

            _ => false,
        }
    }
}

impl std::hash::Hash for Entity {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        match self {
            Entity::Expression(expr) => {
                0.hash(state);
                expr.hash(state);
            },
            Entity::Field { name, .. } => {
                1.hash(state);
                name.hash(state);
            },
            Entity::Variant { name } => {
                2.hash(state);
                name.hash(state);
            },
            Entity::Variable { name, .. } => {
                3.hash(state);
                name.hash(state);
            },
            Entity::Struct { name, .. } => {
                4.hash(state);
                name.hash(state);
            },
            Entity::Enum { name, .. } => {
                5.hash(state);
                name.hash(state);
            },
            Entity::Function { name, parameters, .. } => {
                6.hash(state);
                name.hash(state);
                parameters.hash(state);
            },
            Entity::Macro { name, parameters, .. } => {
                7.hash(state);
                name.hash(state);
                parameters.hash(state);
            },
            Entity::Trait { name, .. } => {
                8.hash(state);
                name.hash(state);
            },
            Entity::Impl { trait_, target, .. } => {
                9.hash(state);
                if let Some(t) = trait_ {
                    t.hash(state);
                }
                target.hash(state);
            },
        }
    }
}

impl Entity {
    pub fn get_name(&self) -> Option<String> {
        match self {
            Entity::Expression(_) => None,
            Entity::Field { name, .. } => Some(name.to_string()),
            Entity::Variant { name } => Some(name.to_string()),
            Entity::Variable { name, .. } => Some(name.to_string()),
            Entity::Struct { name, .. } => Some(name.to_string()),
            Entity::Enum { name, .. } => Some(name.to_string()),
            Entity::Function { name, .. } => Some(name.to_string()),
            Entity::Macro { name, .. } => Some(name.to_string()),
            Entity::Trait { name, .. } => Some(name.to_string()),
            Entity::Impl { .. } => None,
        }
    }
}