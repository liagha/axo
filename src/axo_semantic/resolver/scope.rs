use axo_hash::HashSet;
use crate::axo_matcher::{MatchType, Matcher};
use crate::axo_parser::{Expr, Item};
use crate::axo_semantic::Resolver;
use crate::axo_semantic::resolver::matcher::symbol_matcher;

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
            parent: Some(parent.into()),
        }
    }

    pub fn symbols(&self) -> HashSet<Item> {
        let mut symbols = self.symbols.clone();
        let mut scope = self;

        while let Some(parent) = &scope.parent {
            symbols.extend(parent.symbols.clone());
            scope = parent;
        }

        symbols
    }
}
