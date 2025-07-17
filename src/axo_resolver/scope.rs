#![allow(dead_code)]

use {
    crate::{
        axo_parser::Symbol,
        hash::HashSet,
        operations::{Deref, DerefMut,},
    },
};

#[derive(Clone, Debug)]
pub struct Scope {
    pub symbols: HashSet<Symbol>,
    pub parent: Option<Box<Scope>>,
}

impl Scope {
    pub fn new() -> Self {
        Self {
            symbols: HashSet::new(),
            parent: None,
        }
    }

    pub fn with_parent(parent: Scope) -> Self {
        Self {
            symbols: HashSet::new(),
            parent: Some(Box::new(parent)),
        }
    }

    pub fn set_parent(&mut self, parent: Scope) {
        self.parent = Some(Box::new(parent));
    }

    pub fn take_parent(&mut self) -> Option<Scope> {
        self.parent.take().map(|boxed| *boxed)
    }

    pub fn insert(&mut self, symbol: Symbol) {
        self.symbols.remove(&symbol);
        self.symbols.insert(symbol);
    }

    pub fn contains(&self, symbol: &Symbol) -> bool {
        let mut current = Some(self);

        while let Some(scope) = current {
            if scope.symbols.contains(symbol) {
                return true;
            }
            current = scope.parent.as_deref();
        }

        false
    }
    
    pub fn symbols(&self) -> &HashSet<Symbol> {
        &self.symbols
    }

    pub fn gather(&self) -> HashSet<Symbol> {
        let mut symbols = HashSet::new();
        let mut current = Some(self);

        while let Some(scope) = current {
            symbols.extend(scope.symbols.iter().cloned());
            current = scope.parent.as_deref();
        }

        symbols
    }

    pub fn get(&self, symbol: &Symbol) -> Option<Symbol> {
        let mut current = Some(self);

        while let Some(scope) = current {
            if let Some(found) = scope.symbols.get(symbol) {
                return Some(found.clone());
            }

            current = scope.parent.as_deref();
        }

        None
    }
}

impl Deref for Scope {
    type Target = HashSet<Symbol>;

    fn deref(&self) -> &Self::Target {
        &self.symbols
    }
}

impl DerefMut for Scope {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.symbols
    }
}