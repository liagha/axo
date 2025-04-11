use crate::axo_lexer::Span;
use crate::axo_parser::{Expr, ExprKind, ItemKind};
use std::collections::{HashMap, HashSet};

pub mod entity;
pub mod error;
pub mod expression;
pub mod item;
pub mod scope;
pub mod statement;
use entity::Entity;
use error::ResolverError;
use expression::ExpressionResolver;
use item::ItemResolver;
use scope::Scope;
use statement::ControlFlowResolver;

#[derive(Debug)]
pub struct Resolver {
    pub scope: Scope,
    errors: Vec<ResolverError>,
}

impl Resolver {
    pub fn new() -> Self {
        Self {
            scope: Scope::new(),
            errors: Vec::new(),
        }
    }

    pub fn resolve(&mut self, exprs: Vec<Expr>) -> Result<(), Vec<ResolverError>> {
        for expr in exprs {
            if let Err(err) = self.resolve_expr(expr) {
                self.errors.push(err);
            }
        }

        if self.errors.is_empty() {
            Ok(())
        } else {
            Err(std::mem::take(&mut self.errors))
        }
    }

    pub fn resolve_expr(&mut self, expr: Expr) -> Result<Entity, ResolverError> {
        match expr.kind {
            ExprKind::Item(item) => {
                self.resolve_item(item, expr.span)
            },

            ExprKind::Definition { target, value } => {
                self.resolve_definition(*target, value)
            },
            ExprKind::Assignment { target, value} => {
                self.resolve_assignment(*target, *value, expr.span)
            },

            ExprKind::Block(block_exprs) => {
                self.resolve_block(block_exprs, expr.span)
            },
            ExprKind::Conditional { condition, then_branch, else_branch } => {
                self.resolve_conditional(*condition, *then_branch, else_branch.map(|e| *e), expr.span)
            },
            ExprKind::While { condition, body} => {
                self.resolve_while(*condition, *body, expr.span)
            },
            ExprKind::For{ clause, body } => {
                self.resolve_for(*clause, *body, expr.span)
            },
            ExprKind::Match { target, body } => {
                self.resolve_match(*target, *body, expr.span)
            },

            ExprKind::Identifier(name) => {
                self.resolve_identifier(name)
            },
            ExprKind::Literal(_) => {
                Ok(Entity::Expression(expr))
            },
            ExprKind::Binary { left, operator, right } => {
                self.resolve_binary(*left, operator, *right, expr.span)
            },
            ExprKind::Unary { operator, operand } => {
                self.resolve_unary(operator, *operand, expr.span)
            },

            ExprKind::Invoke{ target, parameters } => {
                self.resolve_invoke(*target, parameters, expr.span)
            },
            ExprKind::Member { object, member} => {
                self.resolve_member(*object, *member, expr.span)
            },
            ExprKind::Closure { parameters, body} => {
                self.resolve_closure(parameters, *body, expr.span)
            },
            ExprKind::Struct { name, body } => {
                self.resolve_struct_instantiation(*name, *body, expr.span)
            },

            ExprKind::Error(err) => Err(ResolverError::Other(format!("Parser error: {:?}", err))),

            _ => self.resolve_expression(expr)
        }
    }
}