use {
    matchete::{Assessor, Scheme},
    super::resolver::Id,
    crate::{
        data::{Offset, Scale},
        internal::hash::Set,
        parser::Symbol,
        data::Str,
        parser::{Element, ElementKind, SymbolKind},
        resolver::{
            ErrorKind, ResolveError,
            matcher::{Affinity, Aligner},
        },
        scanner::{Token, TokenKind},
        tracker::Span,
        schema::*,
    },
};

#[derive(Clone, Debug)]
pub struct Scope<'scope> {
    pub symbols: Set<Symbol<'scope>>,
    pub parent: Option<Box<Scope<'scope>>>,
}

impl<'scope> Scope<'scope> {
    pub fn new() -> Self {
        Self {
            symbols: Set::new(),
            parent: None,
        }
    }

    pub fn with_parent(parent: Scope<'scope>) -> Self {
        Self {
            symbols: Set::new(),
            parent: Some(Box::new(parent)),
        }
    }

    pub fn attach(&mut self, parent: Scope<'scope>) {
        self.parent = Some(Box::new(parent));
    }

    pub fn detach(&mut self) -> Option<Scope<'scope>> {
        self.parent.take().map(|boxed| *boxed)
    }

    pub fn add(&mut self, symbol: Symbol<'scope>) {
        self.symbols.remove(&symbol);
        self.symbols.insert(symbol);
    }

    pub fn remove(&mut self, symbol: &Symbol<'scope>) -> bool {
        self.symbols.remove(symbol)
    }

    pub fn all(&self) -> Vec<Symbol<'scope>> {
        let mut symbols = Vec::new();
        let mut current = Some(self);

        while let Some(scope) = current {
            symbols.extend(scope.symbols.iter().cloned());
            current = scope.parent.as_deref();
        }

        symbols.sort();

        symbols
    }

    pub fn depth(&self) -> Scale {
        let mut depth = 0;
        let mut current = self.parent.as_deref();

        while let Some(scope) = current {
            depth += 1;
            current = scope.parent.as_deref();
        }

        depth
    }

    pub fn root(&self) -> &Scope<'scope> {
        let mut current = self;
        while let Some(parent) = current.parent.as_deref() {
            current = parent;
        }
        current
    }

    pub fn extend(&mut self, symbols: Vec<Symbol<'scope>>) {
        for symbol in symbols {
            self.add(symbol);
        }
    }

    pub fn merge(&mut self, other: &Scope<'scope>) {
        for symbol in &other.symbols {
            self.add(symbol.clone());
        }
    }

    pub fn contains(&self, symbol: &Symbol<'scope>) -> bool {
        self.symbols.contains(symbol)
    }

    pub fn replace(&mut self, old: &Symbol<'scope>, new: Symbol<'scope>) -> bool {
        if self.symbols.remove(old) {
            self.symbols.insert(new);
            true
        } else {
            false
        }
    }

    pub fn try_get(&mut self, target: &Element<'scope>) -> Result<Symbol<'scope>, Vec<ResolveError<'scope>>> {
        if let Element {
            kind: ElementKind::Literal(
                Token {
                    kind: TokenKind::Identifier(identifier),
                    ..
                }
            ),
            ..
        } = target {
            if identifier == "package" {
                let identifier = Element::new(
                    ElementKind::Literal(
                        Token::new(
                            TokenKind::Identifier(
                                *identifier,
                            ),
                            Span::void()
                        ),
                    ),
                    Span::void()
                );

                let package = Symbol::new(
                    SymbolKind::Module(
                        Module::new(Box::new(identifier))
                    ),
                    Span::void(),
                    0
                ).with_scope(
                    self.root().clone()
                );

                return Ok(
                    package
                );
            }
        }

        let candidates = self.all();

        let mut aligner = Aligner::new();
        let mut affinity = Affinity::new();

        let mut assessor = Assessor::new()
            .floor(0.5)
            .dimension(&mut affinity, 0.6)
            .dimension(&mut aligner, 0.4)
            .scheme(Scheme::Additive);

        let champion = assessor.champion(target, &candidates);

        if let Some(champion) = champion {
            Ok(champion)
        } else {
            let mut errors = assessor.errors.clone();
            if errors.is_empty() {
                errors.push(ResolveError {
                    kind: ErrorKind::UndefinedSymbol { query: target.brand().unwrap().clone() },
                    span: target.span.clone(),
                    hints: Vec::new(),
                });
            }
            Err(errors)
        }
    }

    pub fn try_lookup(target: &Element<'scope>, scope: &Scope<'scope>) -> Result<Symbol<'scope>, Vec<ResolveError<'scope>>> {
        if let Element {
            kind: ElementKind::Literal(
                Token {
                    kind: TokenKind::Identifier(identifier),
                    ..
                }
            ),
            ..
        } = target {
            if identifier == "package" {
                let identifier = Element::new(
                    ElementKind::Literal(
                        Token::new(
                            TokenKind::Identifier(
                                *identifier,
                            ),
                            Span::void()
                        ),
                    ),
                    Span::void()
                );

                let package = Symbol::new(
                    SymbolKind::Module(
                        Module::new(Box::new(identifier))
                    ),
                    Span::void(),
                    0
                ).with_scope(
                    scope.root().clone()
                );

                return Ok(
                    package
                );
            }
        }

        let mut aligner = Aligner::new();
        let mut affinity = Affinity::new();

        let mut assessor = Assessor::new()
            .floor(0.5)
            .dimension(&mut affinity, 0.6)
            .dimension(&mut aligner, 0.4)
            .scheme(Scheme::Additive);

        let champion = assessor.champion(target, &*scope.all());

        if let Some(champion) = champion {
            Ok(champion)
        } else {
            if assessor.errors.is_empty() {
                let error = ResolveError {
                    kind: ErrorKind::UndefinedSymbol { query: target.brand().unwrap().clone() },
                    span: target.span.clone(),
                    hints: Vec::new(),
                };
                Err(vec![error])
            } else {
                Err(assessor.errors.clone())
            }
        }
    }
}