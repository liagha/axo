#![allow(dead_code)]

use {
    crate::{
        hash::HashSet,
        axo_parser::DynSymbol,
    },
};

#[derive(Clone, Debug)]
pub struct Scope {
    pub symbols: HashSet<DynSymbol>,
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

    pub fn insert(&mut self, symbol: DynSymbol) {
        self.symbols.remove(&symbol);
        self.symbols.insert(symbol);
    }

    pub fn contains(&self, symbol: &DynSymbol) -> bool {
        let mut current = Some(self);

        while let Some(scope) = current {
            if scope.symbols.contains(symbol) {
                return true;
            }
            current = scope.parent.as_deref();
        }

        false
    }
    
    pub fn symbols(&self) -> &HashSet<DynSymbol> {
        &self.symbols
    }

    pub fn gather(&self) -> HashSet<DynSymbol> {
        let mut symbols = HashSet::new();
        let mut current = Some(self);

        while let Some(scope) = current {
            symbols.extend(scope.symbols.iter().cloned());
            current = scope.parent.as_deref();
        }

        symbols
    }

    pub fn get(&self, symbol: &DynSymbol) -> Option<DynSymbol> {
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