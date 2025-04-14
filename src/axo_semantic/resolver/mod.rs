
pub mod symbol;
pub mod error;
pub mod expression;
pub mod item;
pub mod scope;
pub mod statement;
mod matcher;
mod fmt;

use crate::{
    axo_data::Matcher,
    axo_errors::Error,
    axo_lexer::Span,
    axo_parser::{Expr, ExprKind, ItemKind},
    axo_semantic::{
        error::ErrorKind,
        scope::Scope,
        statement::ControlFlowResolver,
    },
};

use std::collections::{HashMap, HashSet};
use crate::axo_errors::{Action, Hint};
use crate::axo_parser::Item;
use crate::axo_semantic::expression::Expression;
use crate::axo_semantic::resolver::matcher::SymbolMatcher;

pub type ResolveError = Error<ErrorKind>;

#[derive(Debug)]
pub struct Resolver {
    pub scope: Scope,
    pub errors: Vec<ResolveError>,
}

impl Resolver {
    pub fn new() -> Self {
        Self {
            scope: Scope::new(),
            errors: Vec::new(),
        }
    }

    pub fn create_expr_symbol(&self, kind: ExprKind, span: Span) -> Item {
        let kind = ItemKind::Expression(Expr { kind, span: span.clone() }.into());

        Item {
            kind,
            span
        }
    }

    pub fn symbols(&self) -> HashSet<Item> {
        let mut set = HashSet::new();

        let mut current_scope = self.scope.clone();

        while let Some(scope) = &current_scope.parent {
            set.extend(scope.symbols.clone());
            current_scope = *scope.clone();
        }

        set.extend(current_scope.symbols);

        set
    }

    pub fn insert(&mut self, symbol: Item) {
        self.scope.symbols.insert(symbol);
    }

    pub fn lookup(&mut self, target: &Item) -> Item {
        if let Some(symbol) = self.scope.lookup(target) {
            return symbol;
        }

        if let Some(parent) = &self.scope.parent {
            if let Some(symbol) = parent.lookup(target) {
                return symbol;
            }
        }

        let matcher = SymbolMatcher::default();

        let candidates: Vec<Item> = self.scope.symbols.iter().cloned().collect();

        let suggestion = matcher
            .find_best_match(target, &*candidates);

        if let Some(suggestion) = suggestion {
            let found = if let ItemKind::Variable { target, .. } = suggestion.symbol.kind {
                target.to_string()
            } else {
                suggestion.symbol.to_string()
            };

            let err = ResolveError {
                kind: ErrorKind::UndefinedSymbol(target.to_string(), None),
                span: target.span.clone(),
                context: None,
                note: None,
                hints: vec![
                    Hint {
                        message: format!("replace `{}` with `{}` | similarity: {:?}", target, found, suggestion.match_type),
                        action: vec![
                            Action::Replace(found, target.span.clone()),
                        ],
                    }
                ],
            };

            self.errors.push(err.clone());

            Item {
                kind: ItemKind::Unit,
                span: target.span.clone(),
            }
        } else {
            self.error(ErrorKind::UndefinedSymbol(target.to_string(), None), target.span.clone())
        }
    }

    pub fn error(&mut self, error: ErrorKind, span: Span) -> Item {
        let error = ResolveError {
            kind: error,
            span: span.clone(),
            context: None,
            note: None,
            hints: vec![],
        };

        self.errors.push(error.clone());

        let kind = ItemKind::Unit;

        Item {
            kind,
            span
        }
    }

    pub fn error_with_help(&mut self, error: ErrorKind, help: String, span: Span) -> Item {
        let error = ResolveError {
            kind: error,
            span: span.clone(),
            context: None,
            note: Some(help),
            hints: vec![],
        };

        self.errors.push(error.clone());

        let kind = ItemKind::Unit;

        Item {
            kind,
            span
        }
    }

    pub fn resolve(&mut self, exprs: Vec<Expr>) {
        for expr in exprs {
            self.resolve_expr(expr);
        }
    }

    pub fn with_new_scope<F, T>(&mut self, f: F) -> T
    where
        F: FnOnce(&mut Self) -> T,
    {
        let current_scope = self.scope.clone();
        let old_scope = std::mem::replace(
            &mut self.scope,
            Scope::with_parent(current_scope.clone()),
        );

        let result = f(self);

        self.scope = old_scope;
        result
    }

    pub fn resolve_exprs(&mut self, exprs: &[Expr]) -> Vec<Item> {
        let mut results = Vec::new();

        for expr in exprs {
            results.push(self.resolve_expr(expr.clone()));
        }

        results
    }

    pub fn resolve_params(&mut self, parameters: &[Expr]) {
        for param in parameters {
            let Expr { kind, span } = param.clone();

            match &kind {
                ExprKind::Identifier(_) => {
                    let kind = ItemKind::Variable {
                        target: param.clone().into(),
                        value: None,
                        mutable: false,
                        ty: None,
                    };

                    let symbol = Item {
                        kind,
                        span
                    };

                    self.insert(symbol);
                },
                ExprKind::Labeled { label: expr, expr: ty } => {
                    if let ExprKind::Identifier(_) = expr.kind {
                        let kind = ItemKind::Variable {
                            target: expr.clone(),
                            value: None,
                            mutable: false,
                            ty: Some(ty.clone()),
                        };

                        let symbol = Item {
                            kind,
                            span
                        };

                        self.insert(symbol);
                    }
                },
                _ => {
                    self.error(ErrorKind::InvalidExpression(
                        "Expected identifier or typed identifier for parameter".to_string(),
                    ), span);
                },
            }
        }
    }

    pub fn resolve_expr(&mut self, expr: Expr) -> Item {
        let Expr { kind, span } = expr.clone();

        match kind {
            ExprKind::Item(item) => {
                let item = Item {
                    kind: item,
                    span
                };

                self.insert(item.clone());

                item
            },

            ExprKind::Assignment { target, value} => {
                self.resolve_assignment(*target, *value, span)
            },

            ExprKind::Block(block_exprs) => {
                self.resolve_block(block_exprs, span)
            },
            ExprKind::Conditional { condition, then_branch, else_branch } => {
                self.resolve_conditional(*condition, *then_branch, else_branch.map(|e| *e), span)
            },
            ExprKind::While { condition, body} => {
                self.resolve_while(*condition, *body, span)
            },
            ExprKind::For{ clause, body } => {
                self.resolve_for(*clause, *body, span)
            },
            ExprKind::Match { target, body } => {
                self.resolve_match(*target, *body, span)
            },

            ExprKind::Identifier(_) => {
                let variable = Item {
                    kind: ItemKind::Variable { target: expr.into(), value: None, mutable: false, ty: None },
                    span,
                };

                self.lookup(&variable)
            },
            ExprKind::Binary { left, operator, right } => {
                self.resolve_expr(*left.clone());
                self.resolve_expr(*right.clone());

                let kind = ItemKind::Expression(Expr {
                    kind: ExprKind::Binary { left, operator, right },
                    span: span.clone(),
                }.into());

                Item {
                    kind,
                    span
                }
            },
            ExprKind::Unary { operator, operand } => {
                self.resolve_expr(*operand.clone());

                let kind = ItemKind::Expression(Expr {
                    kind: ExprKind::Unary { operator, operand },
                    span: span.clone(),
                }.into());

                Item {
                    kind,
                    span
                }
            },

            ExprKind::Invoke { target, parameters } => {
                self.resolve_invoke(*target, parameters)
            },
            ExprKind::Member { object, member} => {
                self.resolve_member(*object, *member)
            },
            ExprKind::Closure { parameters, body} => {
                self.resolve_closure(parameters, *body, span)
            },
            ExprKind::Struct { name, body } => {
                self.resolve_struct(*name, *body)
            },

            _ => {
                let kind = ItemKind::Expression(expr.into());

                Item {
                    kind,
                    span
                }
            }
        }
    }
}