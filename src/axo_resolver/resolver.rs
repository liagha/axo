use {
    matchete::MatchType,

    crate::{
        axo_errors::{
            Action, Hint
        },
        axo_parser::{
            Element, ElementKind,
            Item, ItemKind
        },
        axo_resolver::{
            ResolveError,
            error::ErrorKind,
            matcher::{symbol_matcher, Labeled},
            scope::Scope,
        },
        axo_span::Span,
    },
};
use crate::{Token, TokenKind};

/// Resolver handles symbol resolution throughout the program
#[derive(Debug)]
pub struct Resolver {
    pub scope: Scope,
    pub errors: Vec<ResolveError>,
}

impl Resolver {
    /// Create a new Resolver with an empty root scope
    pub fn new() -> Self {
        Self {
            scope: Scope::new(),
            errors: Vec::new(),
        }
    }

    /// Create a new scope that inherits from the current scope
    pub fn push_scope(&mut self) {
        let parent_scope = core::mem::replace(&mut self.scope, Scope::new());
        self.scope.set_parent(parent_scope);
    }

    /// Pop the current scope and move back to parent scope
    pub fn pop_scope(&mut self) {
        if let Some(parent) = self.scope.take_parent() {
            self.scope = parent;
        }
    }

    /// Insert a symbol into the current scope
    pub fn insert(&mut self, symbol: Item) {
        self.scope.insert(symbol);
    }

    /// Look up a symbol across all visible scopes and validate the result
    pub fn lookup(&mut self, target: &Element) -> Item {
        let target_name = match target.name() {
            Some(name) => name,
            None => {
                self.error(
                    ErrorKind::UndefinedSymbol(
                        Token::new(TokenKind::Identifier("unnamed".to_string()), target.span.clone()),
                        None
                    ),
                    target.span.clone(),
                );
                return Item {
                    kind: ItemKind::Unit,
                    span: target.span.clone(),
                };
            }
        };

        let matcher = symbol_matcher();
        let candidates: Vec<Item> = self.scope.all_symbols().iter().cloned().collect();

        let suggestion = matcher.find_best_match(target, &*candidates);

        for candidate in candidates.clone() {
            println!("all results: \n\tcandidate: {candidate} => score: {:?}", matcher.analyze(target, &candidate).score);
        }

        if let Some(suggestion) = suggestion {
            let found = suggestion.candidate.name().map(|name| name.to_string()).unwrap_or(suggestion.candidate.to_string());

            {
                println!("Looked Up {:?} in {}", target, target.span);
                for candidate in candidates.iter() {
                    let m = matcher.analyze(target, candidate);
                    println!("  {:?}: score {}", m.candidate, m.score);
                }
                println!();
            }

            println!("Best Match: {:?} | Score: {}\n", suggestion.candidate, suggestion.score);

            self.validate(target, &suggestion.candidate);

            // If we have an exact match or high confidence, return it
            if suggestion.match_type == MatchType::Exact || suggestion.score >= 0.99 {
                return suggestion.candidate;
            }

            // If we have a close match, suggest it as a correction
            if suggestion.score > 0.4 {
                let err = ResolveError {
                    kind: ErrorKind::UndefinedSymbol(target_name.clone(), None),
                    span: target_name.span,
                    note: None,
                    hints: vec![
                        Hint {
                            message: format!("replace with `{}` | similarity: ({:?} | {:.2})",
                                             found, suggestion.match_type, suggestion.score),
                            action: vec![
                                Action::Replace(found, target.span.clone()),
                            ],
                        }
                    ],
                };

                self.errors.push(err);
            } else {
                dbg!();
                self.error(
                    ErrorKind::UndefinedSymbol(target_name.clone(), None),
                    target_name.span,
                );
            }
        } else {
            self.error(
                ErrorKind::UndefinedSymbol(target_name.clone(), None),
                target_name.span,
            );
        }

        Item {
            kind: ItemKind::Unit,
            span: target.span.clone(),
        }
    }

    /// Add a general error
    pub fn error(&mut self, error: ErrorKind, span: Span) -> Item {
        let error = ResolveError {
            kind: error,
            span: span.clone(),
            note: None,
            hints: vec![],
        };

        self.errors.push(error);

        Item {
            kind: ItemKind::Unit,
            span
        }
    }

    /// Resolve a list of expressions
    pub fn resolve(&mut self, exprs: Vec<Element>) {
        for expr in exprs {
            self.resolve_expr(expr.into());
        }
    }

    /// Resolve a single expression
    pub fn resolve_expr(&mut self, expr: Box<Element>) {
        let Element { kind, span } = *expr.clone();

        match kind {
            ElementKind::Item(item) => {
                let item = Item {
                    kind: item,
                    span
                };

                self.insert(item.clone());
            },

            ElementKind::Assignment { target, .. } => {
                self.lookup(&target);
            },

            ElementKind::Scope(body) => {
                self.push_scope();
                self.resolve(body);
                self.pop_scope();
            },

            ElementKind::Identifier(_) => {
                self.lookup(&expr);
            },

            ElementKind::Constructor { .. } | ElementKind::Invoke { .. } | ElementKind::Index { .. } => {
                self.lookup(&expr);
            },

            ElementKind::Group(elements) | ElementKind::Collection(elements) | ElementKind::Bundle(elements) => {
                for element in elements {
                    self.resolve_expr(element.into());
                }
            },

            ElementKind::Binary { left, right, .. } => {
                self.resolve_expr(left);
                self.resolve_expr(right);
            },

            ElementKind::Unary { operand, .. } => {
                self.resolve_expr(operand)
            },

            ElementKind::Bind { key, value } => {
                self.resolve_expr(key);
                self.resolve_expr(value);
            },

            ElementKind::Labeled { label, element: value } => {
                self.resolve_expr(label);
                self.resolve_expr(value);
            },

            ElementKind::Conditional { condition, then: then_branch, alternate: else_branch } => {
                self.resolve_expr(condition);

                self.push_scope();
                self.resolve_expr(then_branch);
                self.pop_scope();

                if let Some(else_branch) = else_branch {
                    self.push_scope();
                    self.resolve_expr(else_branch);
                    self.pop_scope();
                }
            },

            ElementKind::Match { target: clause, body } => {
                self.resolve_expr(clause);

                self.push_scope();
                self.resolve_expr(body);
                self.pop_scope();
            },

            ElementKind::Loop { condition, body } => {
                if let Some(condition) = condition {
                    self.resolve_expr(condition);
                }

                self.push_scope();
                self.resolve_expr(body);
                self.pop_scope();
            },

            ElementKind::Iterate { clause, body } => {
                self.resolve_expr(clause);

                self.push_scope();
                self.resolve_expr(body);
                self.pop_scope();
            },

            ElementKind::Return(value) | ElementKind::Break(value) | ElementKind::Skip(value) => {
                if let Some(value) = value {
                    self.resolve_expr(value);
                }
            },

            _ => {
            }
        }
    }
}