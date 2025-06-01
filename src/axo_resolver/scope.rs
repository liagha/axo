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

    pub fn contains(&self, symbol: &Item) -> bool {
        if self.contains_local(symbol) {
            return true;
        }

        if let Some(parent) = &self.parent {
            return parent.contains(symbol);
        }

        false
    }

    pub fn all_symbols(&self) -> HashSet<Item> {
        let mut all_symbols = self.symbols.clone();

        if let Some(parent) = &self.parent {
            all_symbols.extend(parent.all_symbols());
        }

        all_symbols
    }

    pub fn find(&self, symbol: &Item) -> Option<Item> {
        if let Some(found) = self.symbols.get(symbol) {
            return Some(found.clone());
        }

        if let Some(parent) = &self.parent {
            return parent.find(symbol);
        }

        None
    }
}