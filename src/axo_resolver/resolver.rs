use {
    super::{
        error::{
            ErrorKind,
        },
        matcher::{
            symbol_matcher,
        },
        scope::{
            Scope,
        },
        ResolveError,
    },
    crate::{
        axo_cursor::{
            Span,
        },
        axo_parser::{
            Element, ElementKind,
            Symbol
        },
        format::Debug,
        memory::replace,
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
        let mut assessor = symbol_matcher();
        let candidates: Vec<Symbol> = self.scope.all_symbols().iter().cloned().collect();
        let champion = assessor.champion(target, &candidates);
        self.errors.extend(assessor.errors);

        champion.map(|profile| profile.candidate)
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

    pub fn settle(&mut self, elements: Vec<Element>) {
        for element in elements {
            self.resolve(element.into());
        }
    }

    pub fn resolve(&mut self, element: Box<Element>) {
        let Element { kind, span } = *element.clone();

        match kind {
            ElementKind::Symbolization(symbol) => {
                let symbol = Symbol { kind: symbol, span };
                self.insert(symbol.clone());
            }

            ElementKind::Assignment { target, .. } => {
                self.lookup(&target);
            }

            ElementKind::Scope(body) => {
                self.push_scope();
                self.settle(body);
                self.pop_scope();
            }

            ElementKind::Identifier(_) => {
                self.lookup(&element);
            }

            ElementKind::Constructor { .. }
            | ElementKind::Invoke { .. }
            | ElementKind::Index { .. } => {
                self.lookup(&element);
            }

            ElementKind::Group(elements)
            | ElementKind::Collection(elements)
            | ElementKind::Bundle(elements) => {
                for element in elements {
                    self.resolve(element.into());
                }
            }

            ElementKind::Binary { left, right, .. } => {
                self.resolve(left);
                self.resolve(right);
            }

            ElementKind::Unary { operand, .. } => self.resolve(operand),

            ElementKind::Labeled {
                label,
                element: value,
            } => {
                self.resolve(label);
                self.resolve(value);
            }

            ElementKind::Conditional {
                condition,
                then: then_branch,
                alternate: else_branch,
            } => {
                self.resolve(condition);
                self.push_scope();
                self.resolve(then_branch);
                self.pop_scope();
                if let Some(else_branch) = else_branch {
                    self.push_scope();
                    self.resolve(else_branch);
                    self.pop_scope();
                }
            }

            ElementKind::Match {
                target: clause,
                body,
            } => {
                self.resolve(clause);
                self.push_scope();
                self.resolve(body);
                self.pop_scope();
            }

            ElementKind::Cycle { condition, body } => {
                if let Some(condition) = condition {
                    self.resolve(condition);
                }
                self.push_scope();
                self.resolve(body);
                self.pop_scope();
            }

            ElementKind::Iterate { clause, body } => {
                self.resolve(clause);

                let parent = replace(&mut self.scope, Scope::new());
                self.scope.set_parent(parent);

                self.resolve(body);
                self.pop_scope();
            }

            ElementKind::Return(value) | ElementKind::Break(value) | ElementKind::Skip(value) => {
                if let Some(value) = value {
                    self.resolve(value);
                }
            }

            _ => {}
        }
    }
}
