use {
    super::{
        assessor::{Affinity, Aligner},
        Resolver,
        ErrorKind, ResolveError,
    },
    crate::{
        data::Scale,
        data::{Boolean, Identity},
        internal::hash::Set,
        parser::Symbol,
        parser::Element,
    },
    matchete::{Assessor, Scheme},
};

#[derive(Clone)]
pub struct Scope<Value> {
    pub symbols: Set<Value>,
    pub parent: Option<Box<Scope<Value>>>,
}

impl<'scope> Scope<Symbol<'scope>> {
    pub fn new() -> Self {
        Self {
            symbols: Set::new(),
            parent: None,
        }
    }

    pub fn is_empty(&self) -> Boolean {
        self.symbols.is_empty()
    }

    pub fn with_parent(parent: Scope<Symbol<'scope>>) -> Self {
        Self {
            symbols: Set::new(),
            parent: Some(Box::new(parent)),
        }
    }

    pub fn attach(&mut self, parent: Scope<Symbol<'scope>>) {
        self.parent = Some(Box::new(parent));
    }

    pub fn detach(&mut self) -> Option<Scope<Symbol<'scope>>> {
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

    pub fn root(&self) -> &Scope<Symbol<'scope>> {
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

    pub fn merge(&mut self, other: &Scope<Symbol<'scope>>) {
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

    pub fn get_identity(&self, target: Identity) -> Option<&Symbol<'scope>> {
        if let Some(symbol) = self.symbols.iter().find(|s| s.identity == target) {
            return Some(symbol);
        }

        self.parent.as_ref()?.get_identity(target)
    }

    pub fn lookup(
        &mut self,
        target: &Element<'scope>,
    ) -> Result<Symbol<'scope>, Vec<ResolveError<'scope>>> {
        if let Some(symbol) = Resolver::builtin(target) {
            return Ok(symbol);
        }

        let mut aligner = Aligner::new();
        let mut affinity = Affinity::new();

        let mut assessor = Assessor::new()
            .floor(0.9)
            .dimension(&mut affinity, 0.6)
            .dimension(&mut aligner, 0.4)
            .scheme(Scheme::Multiplicative);

        let candidates = &*self.all();

        let champion = assessor.champion(target, candidates);

        if let Some(champion) = champion {
            Ok(champion)
        } else if assessor.errors.is_empty() {
            Err(vec![ResolveError {
                kind: ErrorKind::UndefinedSymbol {
                    query: target.brand().unwrap().clone(),
                },
                span: target.span.clone(),
                hints: Vec::new(),
            }])
        } else {
            Err(assessor.errors.clone())
        }

    }
}
