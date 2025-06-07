#![allow(dead_code)]

use {
    hashish::HashSet,
    
    crate::{
        axo_parser::Item,
    },
};

#[derive(Clone, Debug)]
pub struct Scope {
    pub symbols: HashSet<Item>,
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

    pub fn insert(&mut self, symbol: Item) {
        self.symbols.remove(&symbol);
        self.symbols.insert(symbol);
    }

    pub fn contains_local(&self, symbol: &Item) -> bool {
        self.symbols.contains(symbol)
    }

    // Stack-safe version using iteration instead of recursion
    pub fn contains(&self, symbol: &Item) -> bool {
        let mut current = Some(self);

        while let Some(scope) = current {
            if scope.symbols.contains(symbol) {
                return true;
            }
            current = scope.parent.as_deref();
        }

        false
    }

    // Stack-safe version using iteration instead of recursion
    pub fn all_symbols(&self) -> HashSet<Item> {
        let mut all_symbols = HashSet::new();
        let mut current = Some(self);

        while let Some(scope) = current {
            all_symbols.extend(scope.symbols.iter().cloned());
            current = scope.parent.as_deref();
        }

        all_symbols
    }

    // Stack-safe version using iteration instead of recursion
    pub fn find(&self, symbol: &Item) -> Option<Item> {
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
