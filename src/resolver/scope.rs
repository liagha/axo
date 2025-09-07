use {
    super::resolver::Id,
    crate::{
        data::{Offset, Scale},
        internal::hash::Set,
        parser::Symbol,
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
}