use hashbrown::HashMap;
use core::cmp::{max, min};
use crate::axo_data::matcher::{MatchType, Matcher};
use crate::axo_parser::{Expr, ExprKind, Item, ItemKind};

#[derive(Debug)]
pub struct SymbolMatchInfo {
    pub score: f64,            // Overall similarity score (0.0 to 1.0)
    pub symbol: Item,        // The matched symbol
    pub match_type: MatchType, // Type of match found
}

pub struct SymbolMatcher {
    // Weights for different similarity components
    pub prefix_weight: f64,
    pub suffix_weight: f64,
    pub common_weight: f64,
    pub edit_dist_weight: f64,
    pub keyboard_dist_weight: f64,

    // Threshold below which matches are considered "not found"
    pub threshold: f64,

    // Optional keyboard layout for typo detection
    keyboard_layout: Option<HashMap<char, Vec<char>>>,

    // String matcher for name comparisons
    string_matcher: Matcher,
}

impl Default for SymbolMatcher {
    fn default() -> Self {
        SymbolMatcher {
            prefix_weight: 0.3,
            suffix_weight: 0.2,
            common_weight: 0.2,
            edit_dist_weight: 0.2,
            keyboard_dist_weight: 0.1,
            threshold: 0.4,
            keyboard_layout: Some(create_qwerty_layout()),
            string_matcher: Matcher::default(),
        }
    }
}

impl SymbolMatcher {
    pub fn new(
        prefix_weight: f64,
        suffix_weight: f64,
        common_subseq_weight: f64,
        edit_dist_weight: f64,
        keyboard_dist_weight: f64,
        threshold: f64,
    ) -> Self {
        SymbolMatcher {
            prefix_weight,
            suffix_weight,
            common_weight: common_subseq_weight,
            edit_dist_weight,
            keyboard_dist_weight,
            threshold,
            keyboard_layout: Some(create_qwerty_layout()),
            string_matcher: Matcher::new(
                prefix_weight,
                suffix_weight,
                common_subseq_weight,
                edit_dist_weight,
                keyboard_dist_weight,
                threshold,
            ),
        }
    }

    // Extract the name from a symbol based on its kind
    fn symbol_name(&self, symbol: &Item) -> Option<String> {
        match &symbol.kind {
            ItemKind::Field { name, .. } => self.expr_name(name),
            ItemKind::Variable { target, .. } => self.expr_name(target),
            ItemKind::Structure { name, .. } => self.expr_name(name),
            ItemKind::Enum { name, .. } => self.expr_name(name),
            ItemKind::Function { name, .. } => self.expr_name(name),
            ItemKind::Macro { name, .. } => self.expr_name(name),
            ItemKind::Trait { name, .. } => self.expr_name(name),
            ItemKind::Implement { expr, .. } => self.expr_name(expr),
            _ => None,
        }
    }

    // Extract the name from an expression
    fn expr_name(&self, expr: &Expr) -> Option<String> {
        match &expr.kind {
            ExprKind::Identifier(name) => Some(name.clone()),
            ExprKind::Member { object, member } => {
                let obj_name = self.expr_name(object)?;
                let mem_name = self.expr_name(member)?;
                Some(format!("{}.{}", obj_name, mem_name))
            },
            _ => None,
        }
    }

    // Find the best match for a query symbol from a list of candidates
    pub fn find_best_match<'a>(&self, query: &Item, candidates: &'a [Item]) -> Option<SymbolMatchInfo> {
        if candidates.is_empty() {
            return None;
        }

        let query_name = match self.symbol_name(query) {
            Some(name) => name,
            None => return None,
        };

        // Convert candidates to name strings for matching
        let mut named_candidates = Vec::new();
        let mut symbol_map = HashMap::new();

        for candidate in candidates {
            if let Some(name) = self.symbol_name(candidate) {
                named_candidates.push(name.clone());
                symbol_map.insert(name, candidate);
            }
        }

        // Use the string matcher to find the best match
        let string_match = self.string_matcher.find_best_match(&query_name, &named_candidates)?;

        // Get the original symbol back
        let matched_symbol = symbol_map.get(&string_match.name)?;

        Some(SymbolMatchInfo {
            score: string_match.score,
            symbol: (*matched_symbol).clone(),
            match_type: string_match.match_type,
        })
    }

    // Find all matches above a certain threshold, sorted by score
    pub fn find_all_matches(&self, query: &Item, candidates: &[Item], limit: usize) -> Vec<SymbolMatchInfo> {
        let mut matches = Vec::new();

        let query_name = match self.symbol_name(query) {
            Some(name) => name,
            None => return matches,
        };

        // Convert candidates to name strings for matching
        let mut named_candidates = Vec::new();
        let mut symbol_map = HashMap::new();

        for candidate in candidates {
            if let Some(name) = self.symbol_name(candidate) {
                named_candidates.push(name.clone());
                symbol_map.insert(name, candidate);
            }
        }

        // Use the string matcher to find all matches
        let string_matches = self.string_matcher.find_all_matches(&query_name, &named_candidates, limit);

        // Convert back to symbol matches
        for string_match in string_matches {
            if let Some(symbol) = symbol_map.get(&string_match.name) {
                matches.push(SymbolMatchInfo {
                    score: string_match.score,
                    symbol: (*symbol).clone(),
                    match_type: string_match.match_type,
                });
            }
        }

        matches
    }

    // Additional symbol-specific matching functions can be added here
    // For example, matching based on symbol kind or type information
    pub fn find_similar_kind(&self, query: &Item, candidates: &[Item]) -> Vec<SymbolMatchInfo> {
        let mut matches = Vec::new();

        let query_name = match self.symbol_name(query) {
            Some(name) => name,
            None => return matches,
        };

        let query_kind = &query.kind;

        // Convert candidates to name strings for matching
        let mut named_candidates = Vec::new();
        let mut symbol_map = HashMap::new();

        for candidate in candidates {
            if let Some(name) = self.symbol_name(candidate) {
                if &candidate.kind == query_kind {
                    named_candidates.push(name.clone());
                    symbol_map.insert(name, candidate);
                }
            }
        }

        // Use the string matcher to find all matches
        let string_matches = self.string_matcher.find_all_matches(&query_name, &named_candidates, 0);

        // Convert back to symbol matches
        for string_match in string_matches {
            if let Some(symbol) = symbol_map.get(&string_match.name) {
                matches.push(SymbolMatchInfo {
                    score: string_match.score,
                    symbol: (*symbol).clone(),
                    match_type: string_match.match_type,
                });
            }
        }

        matches
    }
}

fn create_qwerty_layout() -> HashMap<char, Vec<char>> {
    let mut layout = HashMap::new();

    // Define keyboard adjacency
    let adjacency_map = [
        ('q', vec!['w', 'a']),
        ('w', vec!['q', 'e', 'a', 's']),
        ('e', vec!['w', 'r', 's', 'd']),
        ('r', vec!['e', 't', 'd', 'f']),
        ('t', vec!['r', 'y', 'f', 'g']),
        ('y', vec!['t', 'u', 'g', 'h']),
        ('u', vec!['y', 'i', 'h', 'j']),
        ('i', vec!['u', 'o', 'j', 'k']),
        ('o', vec!['i', 'p', 'k', 'l']),
        ('p', vec!['o', 'l']),
        ('a', vec!['q', 'w', 's', 'z']),
        ('s', vec!['w', 'e', 'a', 'd', 'z', 'x']),
        ('d', vec!['e', 'r', 's', 'f', 'x', 'c']),
        ('f', vec!['r', 't', 'd', 'g', 'c', 'v']),
        ('g', vec!['t', 'y', 'f', 'h', 'v', 'b']),
        ('h', vec!['y', 'u', 'g', 'j', 'b', 'n']),
        ('j', vec!['u', 'i', 'h', 'k', 'n', 'm']),
        ('k', vec!['i', 'o', 'j', 'l', 'm']),
        ('l', vec!['o', 'p', 'k']),
        ('z', vec!['a', 's', 'x']),
        ('x', vec!['z', 's', 'd', 'c']),
        ('c', vec!['x', 'd', 'f', 'v']),
        ('v', vec!['c', 'f', 'g', 'b']),
        ('b', vec!['v', 'g', 'h', 'n']),
        ('n', vec!['b', 'h', 'j', 'm']),
        ('m', vec!['n', 'j', 'k']),
        ('1', vec!['2', '`']),
        ('2', vec!['1', '3', 'q']),
        ('3', vec!['2', '4', 'w']),
        ('4', vec!['3', '5', 'e']),
        ('5', vec!['4', '6', 'r']),
        ('6', vec!['5', '7', 't']),
        ('7', vec!['6', '8', 'y']),
        ('8', vec!['7', '9', 'u']),
        ('9', vec!['8', '0', 'i']),
        ('0', vec!['9', '-', 'o']),
        ('-', vec!['0', '=', 'p']),
        ('=', vec!['-']),
    ];

    for (key, adjacent) in adjacency_map {
        layout.insert(key, adjacent);
    }

    layout
}