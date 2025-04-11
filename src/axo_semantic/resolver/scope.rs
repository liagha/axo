use std::collections::HashSet;
use crate::axo_data::{MatchType, SmartMatcher};
use crate::axo_semantic::resolver::entity::Entity;
use crate::axo_semantic::resolver::error::ResolverError;

#[derive(Clone, Debug)]
pub struct Scope {
    pub entities: HashSet<Entity>,
    pub parent: Option<Box<Scope>>,
}

impl Scope {
    pub fn new() -> Self {
        Self {
            entities: HashSet::new(),
            parent: None,
        }
    }

    pub fn with_parent(parent: Scope) -> Self {
        Self {
            entities: HashSet::new(),
            parent: Some(parent.into()),
        }
    }

    pub fn insert(&mut self, entity: Entity) -> Result<(), ResolverError> {
        if let Some(name) = entity.get_name() {
            for sym in &self.entities {
                if let Some(sym_name) = sym.get_name() {
                    if sym_name == name {
                        return Err(ResolverError::AlreadyDefined(name));
                    }
                }
            }
        }

        self.entities.insert(entity);
        Ok(())
    }

    pub fn lookup(&self, name: &str) -> Option<&Entity> {
        // First try exact match in current scope
        if let Some(symbol) = self.try_exact_match(name) {
            return Some(symbol);
        }

        // Try fuzzy matching in current scope
        let matcher = SmartMatcher::default();
        let candidate_names: Vec<String> = self.entities.iter()
            .filter_map(|sym| sym.get_name())
            .collect();

        if let Some(best_match) = matcher.find_best_match(name, &candidate_names) {
            if best_match.match_type != MatchType::Exact {
                println!("No exact match found for '{}'. Did you mean '{}'? (score: {:.2})",
                         name, best_match.name, best_match.score);
            }

            // Return the matching symbol
            return self.entities.iter().find(|sym| {
                if let Some(sym_name) = sym.get_name() {
                    return sym_name == best_match.name;
                }
                false
            });
        }

        // Try parent scope if available
        if let Some(parent) = &self.parent {
            return parent.lookup(name);
        }

        None
    }

    fn try_exact_match(&self, name: &str) -> Option<&Entity> {
        self.entities.iter().find(|sym| {
            if let Some(sym_name) = sym.get_name() {
                return sym_name == name;
            }
            false
        })
    }
}
