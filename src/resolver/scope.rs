use {
    crate::{
        data::{Offset, Scale},
        internal::hash::Set,
        parser::Symbol,
    },
};

#[derive(Clone, Debug)]
pub struct Scope {
    pub symbols: Set<Symbol>,
    pub parent: Option<Box<Scope>>,
}

impl Scope {
    pub fn new() -> Self {
        Self {
            symbols: Set::new(),
            parent: None,
        }
    }

    pub fn child() -> Self {
        Self::new()
    }

    pub fn with_parent(parent: Scope) -> Self {
        Self {
            symbols: Set::new(),
            parent: Some(Box::new(parent)),
        }
    }

    pub fn attach(&mut self, parent: Scope) {
        self.parent = Some(Box::new(parent));
    }

    pub fn detach(&mut self) -> Option<Scope> {
        self.parent.take().map(|boxed| *boxed)
    }

    pub fn add(&mut self, symbol: Symbol) {
        self.symbols.remove(&symbol);
        self.symbols.insert(symbol);
    }

    pub fn remove(&mut self, symbol: &Symbol) -> bool {
        self.symbols.remove(symbol)
    }

    pub fn has(&self, symbol: &Symbol) -> bool {
        let mut current = Some(self);

        while let Some(scope) = current {
            if scope.symbols.contains(symbol) {
                return true;
            }
            current = scope.parent.as_deref();
        }

        false
    }

    pub fn find(&self, symbol: &Symbol) -> Option<Symbol> {
        let mut current = Some(self);

        while let Some(scope) = current {
            if let Some(found) = scope.symbols.get(symbol) {
                return Some(found.clone());
            }
            current = scope.parent.as_deref();
        }

        None
    }

    pub fn local(&self) -> &Set<Symbol> {
        &self.symbols
    }

    pub fn all(&self) -> Set<Symbol> {
        let mut symbols = Set::new();
        let mut current = Some(self);

        while let Some(scope) = current {
            symbols.extend(scope.symbols.iter().cloned());
            current = scope.parent.as_deref();
        }

        symbols
    }

    pub fn count(&self) -> Scale {
        self.symbols.len()
    }

    pub fn empty(&self) -> bool {
        self.symbols.is_empty()
    }

    pub fn clear(&mut self) {
        self.symbols.clear();
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

    pub fn root(&self) -> &Scope {
        let mut current = self;
        while let Some(parent) = current.parent.as_deref() {
            current = parent;
        }
        current
    }

    pub fn extend(&mut self, symbols: Vec<Symbol>) {
        for symbol in symbols {
            self.add(symbol);
        }
    }

    pub fn merge(&mut self, other: &Scope) {
        for symbol in &other.symbols {
            self.add(symbol.clone());
        }
    }

    pub fn contains(&self, symbol: &Symbol) -> bool {
        self.symbols.contains(symbol)
    }

    pub fn replace(&mut self, old: &Symbol, new: Symbol) -> bool {
        if self.symbols.remove(old) {
            self.symbols.insert(new);
            true
        } else {
            false
        }
    }

    pub fn retain<F>(&mut self, predicate: F)
    where
        F: FnMut(&Symbol) -> bool,
    {
        self.symbols.retain(predicate);
    }

    pub fn filter<F>(&self, predicate: F) -> Set<Symbol>
    where
        F: Fn(&Symbol) -> bool,
    {
        self.symbols.iter().filter(|s| predicate(s)).cloned().collect()
    }

    pub fn collect<F, T>(&self, mapping: F) -> Vec<T>
    where
        F: Fn(&Symbol) -> T,
    {
        self.symbols.iter().map(mapping).collect()
    }

    pub fn ancestors(&self) -> Vec<&Scope> {
        let mut scopes = Vec::new();
        let mut current = self.parent.as_deref();

        while let Some(scope) = current {
            scopes.push(scope);
            current = scope.parent.as_deref();
        }

        scopes
    }

    pub fn flatten(&self) -> Vec<Symbol> {
        self.all().into_iter().collect()
    }

    pub fn intersect(&self, other: &Scope) -> Set<Symbol> {
        self.symbols.intersection(&other.symbols).cloned().collect()
    }

    pub fn difference(&self, other: &Scope) -> Set<Symbol> {
        self.symbols.difference(&other.symbols).cloned().collect()
    }

    pub fn union(&self, other: &Scope) -> Set<Symbol> {
        self.symbols.union(&other.symbols).cloned().collect()
    }

    pub fn visible(&self, symbol: &Symbol) -> bool {
        self.has(symbol)
    }

    pub fn shadow(&mut self, symbol: Symbol) {
        self.symbols.insert(symbol);
    }

    pub fn isolate(&mut self) {
        self.parent = None;
    }

    pub fn cascade(&self) -> Vec<Set<Symbol>> {
        let mut levels = Vec::new();
        let mut current = Some(self);

        while let Some(scope) = current {
            levels.push(scope.symbols.clone());
            current = scope.parent.as_deref();
        }

        levels
    }

    pub fn nested(&self) -> bool {
        self.parent.is_some()
    }

    pub fn toplevel(&self) -> bool {
        self.parent.is_none()
    }
}