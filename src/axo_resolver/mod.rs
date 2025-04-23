
pub mod error;
pub mod scope;
pub mod statement;
mod fmt;
mod matcher;

use crate::{
    axo_errors::{Action, Error, Hint},
    axo_matcher::MatchType::Exact,
    axo_parser::{
        Expr, ExprKind,
        Item, ItemKind,
    },
    axo_resolver::{
        error::ErrorKind,
        scope::Scope,
        statement::ControlFlowResolver
    },
    axo_span::Span,
};

use axo_hash::HashSet;
use crate::axo_resolver::matcher::Labeled;
use crate::axo_resolver::matcher::symbol_matcher;

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
        if self.symbols().contains(&symbol) {
            self.scope.symbols.insert(symbol);
        } else {
            self.scope.symbols.remove(&symbol);

            self.scope.symbols.insert(symbol);
        }
    }

    pub fn lookup(&mut self, target: &Expr) -> Item {
        let matcher = symbol_matcher();

        let candidates: Vec<Item> = self.scope.symbols().iter().cloned().collect();

        /*
        for candidate in candidates.clone() {
            let result = matcher.analyze_match(&target, &candidate);

            println!("Match result for '{}' against '{}':", result.query, result.candidate);
            println!("Overall score: {:.2}", result.overall_score);
            println!("Match type: {:?}", result.match_type);
            println!("Is match: {}", result.is_match);

            for score in &result.metric_scores {
                println!("  {}: {:.2} (weight: {:.1}, contribution: {:.2})",
                         score.name, score.raw_score, score.weight, score.weighted_contribution);
            }
        }

        println!("\nMetric breakdown:");
        */


        let suggestion = matcher
            .find_best_match(target, &*candidates);

        if let Some(suggestion) = suggestion {
            {
                println!("Detailed Match Result:");

                for candidate in candidates {
                    let m = matcher.analyze(target, &candidate);

                    println!("  {:?}: score {}", m.candidate, m.score);
                }

                println!();
            }

            println!("Best Match: {:?} | Score: {}\n", suggestion.candidate, suggestion.score);

            let target_name = target.name().map(|name| name.to_string()).unwrap_or(target.to_string());

            let found = suggestion.candidate.name().map(|name| name.to_string()).unwrap_or(target.to_string());

            if suggestion.match_type == Exact || suggestion.score == 1.0 {
                return suggestion.candidate
            }

            let err = ResolveError {
                kind: ErrorKind::UndefinedSymbol(target_name.to_string(), None),
                span: target.span.clone(),
                note: None,
                hints: vec![
                    Hint {
                        message: format!("replace `{}` with `{}` | similarity: ({:?} | {:?})", target_name, found, suggestion.match_type, suggestion.score),
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

            ExprKind::Assignment { target, .. } => {
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