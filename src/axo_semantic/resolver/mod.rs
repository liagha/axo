
pub mod symbol;
pub mod error;
pub mod scope;
pub mod statement;
mod fmt;
mod matcher;

use crate::{
    axo_data::matcher::Matcher,
    axo_errors::{Error, Hint, Action},
    axo_span::Span,
    axo_parser::{
        Expr, ExprKind,
        Item, ItemKind,
    },
    axo_semantic::{
        error::ErrorKind,
        scope::Scope,
        statement::ControlFlowResolver,
    },
};

use hashbrown::{HashMap, HashSet};
use crate::axo_data::matcher::{AcronymMetric, CaseInsensitiveMetric, EditDistanceMetric, ExactMatchMetric, KeyboardProximityMetric, PrefixMetric, SubstringMetric, SuffixMetric, TokenSimilarityMetric};
use crate::axo_semantic::resolver::matcher::Labeled;

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

    pub fn symbol_matcher() -> Matcher<Expr, Item> {
        Matcher::<Expr, Item>::new()
            .with_metric(ExactMatchMetric, 1.0)
            .with_metric(CaseInsensitiveMetric, 0.9)
            .with_metric(TokenSimilarityMetric::default(), 0.8)
            .with_metric(EditDistanceMetric, 0.7)
            .with_metric(PrefixMetric, 0.6)
            .with_metric(SubstringMetric, 0.5)
            .with_metric(SuffixMetric, 0.4)
            .with_metric(KeyboardProximityMetric::default(), 0.3)
            .with_metric(AcronymMetric::default(), 0.2)
            .with_threshold(0.4)
    }

    pub fn lookup(&mut self, target: &Expr) -> Item {
        if let Some(symbol) = self.scope.lookup(target) {
            return symbol;
        }

        if let Some(parent) = &self.scope.parent {
            if let Some(symbol) = parent.lookup(target) {
                return symbol;
            }
        }

        let matcher = Resolver::symbol_matcher();

        let candidates: Vec<Item> = self.scope.symbols.iter().cloned().collect();

        let suggestion = matcher
            .find_best_match(target, &*candidates);

        if let Some(suggestion) = suggestion {
            let target_name = target.name().map(|name| name.to_string()).unwrap_or(target.to_string());

            let found = suggestion.value.get_name();

            let err = ResolveError {
                kind: ErrorKind::UndefinedSymbol(target_name.to_string(), None),
                span: target.span.clone(),
                note: None,
                hints: vec![
                    Hint {
                        message: format!("replace `{}` with `{}` | similarity: {:?}", target_name, found, suggestion.match_type),
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
            let target_name = target.name().map(|name| name.to_string()).unwrap_or(target.to_string());

            self.error(ErrorKind::UndefinedSymbol(target_name, None), target.span.clone())
        }
    }

    pub fn error(&mut self, error: ErrorKind, span: Span) -> Item {
        let error = ResolveError {
            kind: error,
            span: span.clone(),
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
                self.lookup(&*target)
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

            ExprKind::Invoke { .. }
            | ExprKind::Member { .. }
            | ExprKind::Closure { .. }
            | ExprKind::Constructor { .. }
            | ExprKind::Identifier(_) => {
                self.lookup(&expr)
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