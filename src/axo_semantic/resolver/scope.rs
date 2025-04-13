use std::collections::HashSet;
use crate::axo_data::{MatchType, Matcher};
use crate::axo_parser::Expr;
use crate::axo_semantic::Resolver;
use crate::axo_semantic::resolver::matcher::SymbolMatcher;
use crate::axo_semantic::resolver::symbol::Symbol;

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
            parent: Some(parent.into()),
        }
    }

    pub fn lookup(&self, target: &Symbol) -> Option<Symbol> {
        let matcher = SymbolMatcher::default();

        let candidates: Vec<Symbol> = self.symbols.iter().cloned().collect();

        if let Some(best_match) = matcher.find_best_match(target, &*candidates) {
            if best_match.match_type == MatchType::Exact {
                println!("target: {:?} => {:?}", target, best_match.symbol);

                return Some(best_match.symbol)
            }
        }

        None
    }
}
