use {
    super::{
        assessor::{Affinity, Aligner},
        ErrorKind, ResolveError,
    },
    crate::{
        data::Scale,
        data::{Boolean, Identity},
        internal::hash::Set,
        parser::Symbol,
        parser::Element,
        scanner::TokenKind
        ,
    },
    matchete::{Assessor, Scheme},
};
use crate::resolver::primitives::builtin;

#[derive(Clone)]
pub struct Scope<'scope> {
    pub symbols: Set<Symbol<'scope>>,
    pub parent: Option<Box<Scope<'scope>>>,
}

impl<'scope> Scope<'scope> {
    fn undefined(target: &Element<'scope>) -> Vec<ResolveError<'scope>> {
        vec![ResolveError {
            kind: ErrorKind::UndefinedSymbol {
                query: target.brand().unwrap().clone(),
            },
            span: target.span.clone(),
            hints: Vec::new(),
        }]
    }

    fn exact_lookup(target: &Element<'scope>, scope: &Scope<'scope>) -> Option<Symbol<'scope>> {
        let query = target.brand().and_then(|token| match token.kind {
            TokenKind::Identifier(name) => name.as_str().map(str::to_owned),
            _ => None,
        })?;

        let mut current = Some(scope);
        while let Some(active) = current {
            if let Some(symbol) = active.symbols.iter().find(|candidate| {
                candidate
                    .brand()
                    .and_then(|token| match token.kind {
                        TokenKind::Identifier(name) => name.as_str().map(str::to_owned),
                        _ => None,
                    })
                    .as_deref()
                    == Some(query.as_str())
            }) {
                return Some(symbol.clone());
            }

            current = active.parent.as_deref();
        }

        None
    }

    pub fn new() -> Self {
        Self {
            symbols: Set::new(),
            parent: None,
        }
    }

    pub fn is_empty(&self) -> Boolean {
        self.symbols.is_empty()
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

    pub fn get_id(&self, target: Identity) -> Option<&Symbol<'scope>> {
        if let Some(symbol) = self.symbols.iter().find(|s| s.id == target) {
            return Some(symbol);
        }

        self.parent.as_ref()?.get_id(target)
    }

    pub fn try_get(
        &mut self,
        target: &Element<'scope>,
    ) -> Result<Symbol<'scope>, Vec<ResolveError<'scope>>> {
        Self::try_lookup(target, self)
    }

    pub fn try_lookup(
        target: &Element<'scope>,
        scope: &Scope<'scope>,
    ) -> Result<Symbol<'scope>, Vec<ResolveError<'scope>>> {
        if let Some(symbol) = builtin(target, scope) {
            return Ok(symbol);
        }

        if let Some(symbol) = Self::exact_lookup(target, scope) {
            return Ok(symbol);
        }

        let mut aligner = Aligner::new();
        let mut affinity = Affinity::new();

        let mut assessor = Assessor::new()
            .floor(0.5)
            .dimension(&mut affinity, 0.6)
            .dimension(&mut aligner, 0.4)
            .scheme(Scheme::Multiplicative);

        let candidates = &*scope.all();

        let champion = assessor.champion(target, candidates);

        if champion.is_some() {
            if assessor.errors.is_empty() {
                Err(Self::undefined(target))
            } else {
                Err(assessor.errors.clone())
            }
        } else if assessor.errors.is_empty() {
            Err(Self::undefined(target))
        } else {
            Err(assessor.errors.clone())
        }
    }
}
