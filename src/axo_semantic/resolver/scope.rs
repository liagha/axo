use std::collections::HashSet;
use crate::axo_data::{MatchType, Matcher};
use crate::axo_parser::{Expr, Item};
use crate::axo_semantic::Resolver;
use crate::axo_semantic::resolver::matcher::SymbolMatcher;

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

    pub fn lookup(&self, target: &Item) -> Option<Item> {
        let matcher = SymbolMatcher::default();

        let candidates: Vec<Item> = self.symbols.iter().cloned().collect();

        if let Some(best_match) = matcher.find_best_match(target, &*candidates) {
            if best_match.match_type == MatchType::Exact {
                return Some(best_match.symbol)
            }
        }

        None
    }
}
