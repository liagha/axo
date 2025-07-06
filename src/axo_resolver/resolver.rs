use {
    matchete::MatchType,

    crate::{
        memory::replace,

        axo_error::{
            Action, Hint
        },

        axo_scanner::{
            Token, TokenKind,
        },

        axo_parser::{
            Element, ElementKind,
            Symbol
        },

        axo_resolver::{
            ResolveError,
            error::ErrorKind,
            matcher::{symbol_matcher, Labeled},
            scope::Scope,
        },

        axo_cursor::Span,
    },
};

#[derive(Clone, Debug)]
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

    pub fn push_scope(&mut self) {
        let parent_scope = replace(&mut self.scope, Scope::new());
        self.scope.set_parent(parent_scope);
    }

    pub fn pop_scope(&mut self) {
        if let Some(parent) = self.scope.take_parent() {
            self.scope = parent;
        }
    }

    pub fn insert(&mut self, symbol: Symbol) {
        self.scope.insert(symbol);
    }

    pub fn lookup(&mut self, target: &Element) -> Option<Symbol> {
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
                
                return None
            }
        };

        let matcher = symbol_matcher();
        let candidates: Vec<Symbol> = self.scope.all_symbols().iter().cloned().collect();

        let suggestion = matcher.find_best_match(target, &*candidates);

        /*{
            for candidate in candidates.clone() {
                println!(
                    "Looked Up `{:?}`:",
                    target,
                );

                println!(
                    "\t`{:?}` | Score: {:?}",
                    candidate,
                    matcher.analyze(target, &candidate).score
                );

                println!();
            }

            if let Some(suggestion) = suggestion.clone() {
                println!("Best Match: `{:?}` | Score: {}", suggestion.candidate, suggestion.score);

                println!();
            }
        }*/

        if let Some(suggestion) = suggestion {
            let found = suggestion.candidate.name().map(|name| name.to_string()).unwrap_or(suggestion.candidate.to_string());



            self.validate(target, &suggestion.candidate);

            if suggestion.match_type == MatchType::Exact || suggestion.score >= 0.99 {
                return Some(suggestion.candidate);
            }

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
                
                None
            } else {
                self.error(
                    ErrorKind::UndefinedSymbol(target_name.clone(), None),
                    target_name.span,
                );
                
                None
            }
        } else {
            self.error(
                ErrorKind::UndefinedSymbol(target_name.clone(), None),
                target_name.span,
            );
            
            None
        }
    }

    pub fn error(&mut self, error: ErrorKind, span: Span) {
        let error = ResolveError {
            kind: error,
            span: span.clone(),
            note: None,
            hints: vec![],
        };

        self.errors.push(error);
    }

    pub fn resolve(&mut self, elements: Vec<Element>) {
        for element in elements {
            self.resolve_element(element.into());
        }
    }

    pub fn resolve_element(&mut self, element: Box<Element>) {
        let Element { kind, span } = *element.clone();

        match kind {
            ElementKind::Symbolization(symbol) => {
                let symbol = Symbol {
                    kind: symbol,
                    span
                };

                self.insert(symbol.clone());
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
                self.lookup(&element);
            },

            ElementKind::Constructor { .. } | ElementKind::Invoke { .. } | ElementKind::Index { .. } => {
                self.lookup(&element);
            },

            ElementKind::Group(elements) | ElementKind::Collection(elements) | ElementKind::Bundle(elements) => {
                for element in elements {
                    self.resolve_element(element.into());
                }
            },

            ElementKind::Binary { left, right, .. } => {
                self.resolve_element(left);
                self.resolve_element(right);
            },

            ElementKind::Unary { operand, .. } => {
                self.resolve_element(operand)
            },

            ElementKind::Labeled { label, element: value } => {
                self.resolve_element(label);
                self.resolve_element(value);
            },

            ElementKind::Conditional { condition, then: then_branch, alternate: else_branch } => {
                self.resolve_element(condition);

                self.push_scope();
                self.resolve_element(then_branch);
                self.pop_scope();

                if let Some(else_branch) = else_branch {
                    self.push_scope();
                    self.resolve_element(else_branch);
                    self.pop_scope();
                }
            },

            ElementKind::Match { target: clause, body } => {
                self.resolve_element(clause);

                self.push_scope();
                self.resolve_element(body);
                self.pop_scope();
            },

            ElementKind::Cycle { condition, body } => {
                if let Some(condition) = condition {
                    self.resolve_element(condition);
                }

                self.push_scope();
                self.resolve_element(body);
                self.pop_scope();
            },

            ElementKind::Iterate { clause, body } => {
                self.resolve_element(clause);

                self.push_scope();
                self.resolve_element(body);
                self.pop_scope();
            },

            ElementKind::Return(value) | ElementKind::Break(value) | ElementKind::Skip(value) => {
                if let Some(value) = value {
                    self.resolve_element(value);
                }
            },

            _ => {
            }
        }
    }
}