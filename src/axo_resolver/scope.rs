#![allow(dead_code)]

use hashish::HashSet;
use crate::axo_parser::Item;

/// Represents a lexical scope with symbols and optional parent scope
#[derive(Clone, Debug)]
pub struct Scope {
    pub symbols: HashSet<Item>,
    pub parent: Option<Box<Scope>>,
}

impl Scope {
    /// Create a new empty scope with no parent
    pub fn new() -> Self {
        Self {
            symbols: HashSet::new(),
            parent: None,
        }
    }

    /// Create a new scope with the given parent
    pub fn with_parent(parent: Scope) -> Self {
        Self {
            symbols: HashSet::new(),
            parent: Some(Box::new(parent)),
        }
    }

    /// Set the parent of this scope
    pub fn set_parent(&mut self, parent: Scope) {
        self.parent = Some(Box::new(parent));
    }

    /// Take the parent scope, leaving None in its place
    pub fn take_parent(&mut self) -> Option<Scope> {
        self.parent.take().map(|boxed| *boxed)
    }

    /// Insert a symbol into this scope
    pub fn insert(&mut self, symbol: Item) {
        // First remove if it exists (to ensure we replace it correctly)
        self.symbols.remove(&symbol);
        // Then insert the new symbol
        self.symbols.insert(symbol);
    }

    /// Check if a symbol exists in this scope only (not in parent scopes)
    pub fn contains_local(&self, symbol: &Item) -> bool {
        self.symbols.contains(symbol)
    }

    /// Check if a symbol exists in this scope or any parent scope
    pub fn contains(&self, symbol: &Item) -> bool {
        if self.contains_local(symbol) {
            return true;
        }

        if let Some(parent) = &self.parent {
            return parent.contains(symbol);
        }

        false
    }

    /// Get all symbols from this scope and its parent scopes
    pub fn all_symbols(&self) -> HashSet<Item> {
        let mut all_symbols = self.symbols.clone();

        if let Some(parent) = &self.parent {
            all_symbols.extend(parent.all_symbols());
        }

        all_symbols
    }

    /// Find a symbol in this scope or parent scopes
    pub fn find(&self, symbol: &Item) -> Option<Item> {
        // First check the current scope
        if let Some(found) = self.symbols.get(symbol) {
            return Some(found.clone());
        }

        // Then check parent scopes
        if let Some(parent) = &self.parent {
            return parent.find(symbol);
        }

        None
    }
}