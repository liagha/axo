use std::cmp::{max, min};
use hashbrown::HashMap;
use crate::axo_data::matcher::{create_qwerty_layout, damerau_levenshtein_distance, MatchType, SimilarityMetric, SuffixMetric};
use crate::axo_lexer::{Token, TokenKind};
use crate::axo_parser::{Expr, ExprKind, Item, ItemKind};

impl SimilarityMetric<Token, Token> for crate::axo_data::matcher::CaseInsensitiveMetric {
    fn calculate(&self, query: &Token, candidate: &Token) -> f64 {
        if query.to_string().to_lowercase() == candidate.to_string().to_lowercase() { 0.95 } else { 0.0 }
    }

    fn name(&self) -> &str {
        "CaseInsensitive"
    }
}

impl SimilarityMetric<Token, Token> for crate::axo_data::matcher::PrefixMetric {
    fn calculate(&self, query: &Token, candidate: &Token) -> f64 {
        let query_lower = query.to_string().to_lowercase();
        let candidate_lower = candidate.to_string().to_lowercase();

        if candidate_lower.starts_with(&query_lower) {
            return 0.9 * (query_lower.len() as f64 / candidate_lower.len() as f64).min(1.0);
        } else if query_lower.starts_with(&candidate_lower) {
            return 0.9 * (candidate_lower.len() as f64 / query_lower.len() as f64).min(1.0)
        }

        0.0
    }

    fn name(&self) -> &str {
        "Prefix"
    }

    fn match_type(&self, query: &Token, candidate: &Token) -> Option<MatchType> {
        let score = self.calculate(query, candidate);
        if score > 0.0 {
            Some(MatchType::Similar("Prefix".to_string()))
        } else {
            None
        }
    }
}

impl SimilarityMetric<Token, Token> for crate::axo_data::matcher::SubstringMetric {
    fn calculate(&self, query: &Token, candidate: &Token) -> f64 {
        let query_lower = query.to_string().to_lowercase();
        let candidate_lower = candidate.to_string().to_lowercase();

        if candidate_lower.contains(&query_lower) {
            return 0.8 * (query_lower.len() as f64 / candidate_lower.len() as f64).min(1.0);
        } else if query_lower.contains(&candidate_lower) {
            return 0.8 * (candidate_lower.len() as f64 / query_lower.len() as f64).min(1.0)
        }

        0.0
    }

    fn name(&self) -> &str {
        "Substring"
    }
}

impl SimilarityMetric<Token, Token> for crate::axo_data::matcher::EditDistanceMetric {
    fn calculate(&self, s1: &Token, s2: &Token) -> f64 {
        let distance = damerau_levenshtein_distance(&*s1.to_string(), &*s2.to_string());
        let max_len = max(s1.to_string().len(), s2.to_string().len());

        if max_len == 0 {
            return 1.0;
        }

        1.0 - (distance as f64 / max_len as f64)
    }

    fn name(&self) -> &str {
        "EditDistance"
    }
}

impl SimilarityMetric<Token, Token> for crate::axo_data::matcher::TokenSimilarityMetric {
    fn calculate(&self, s1: &Token, s2: &Token) -> f64 {
        let s1_lower = s1.to_string().to_lowercase();
        let s2_lower = s2.to_string().to_lowercase();

        let s1_tokens = self.split_on_separators(&s1_lower);
        let s2_tokens = self.split_on_separators(&s2_lower);

        self.token_similarity(&s1_tokens, &s2_tokens)
    }

    fn name(&self) -> &str {
        "TokenSimilarity"
    }
}

impl SimilarityMetric<Token, Token> for crate::axo_data::matcher::AcronymMetric {
    fn calculate(&self, query: &Token, candidate: &Token) -> f64 {
        if query.to_string().len() > self.max_acronym_length {
            return 0.0;
        }

        let query_lower = query.to_string().to_lowercase();
        let candidate_lower = candidate.to_string().to_lowercase();

        let tokens = self.token_metric.split_on_separators(&candidate_lower);

        if tokens.len() < query_lower.len() {
            return 0.0;
        }

        let first_letters: String = tokens.iter()
            .filter_map(|token| token.chars().next())
            .collect();

        if first_letters.contains(&query_lower) {
            return 0.75;
        }

        0.0
    }

    fn name(&self) -> &str {
        "Acronym"
    }

    fn match_type(&self, query: &Token, candidate: &Token) -> Option<MatchType> {
        let score = self.calculate(query, candidate);
        if score > 0.0 {
            Some(MatchType::Similar("Acronym".to_string()))
        } else {
            None
        }
    }
}

impl SimilarityMetric<Token, Token> for crate::axo_data::matcher::KeyboardProximityMetric {
    fn calculate(&self, s1: &Token, s2: &Token) -> f64 {
        let s1_lower = s1.to_string().to_lowercase();
        let s2_lower = s2.to_string().to_lowercase();

        if (s1_lower.len() as isize - s2_lower.len() as isize).abs() > 2 {
            return 0.0;
        }

        let s1_chars: Vec<char> = s1_lower.chars().collect();
        let s2_chars: Vec<char> = s2_lower.chars().collect();

        let edit_distance = damerau_levenshtein_distance(&s1_lower, &s2_lower);

        if edit_distance > 3 {
            return 0.0;
        }

        let mut adjacency_count = 0;
        let max_comparisons = min(s1_chars.len(), s2_chars.len());

        for i in 0..max_comparisons {
            if s1_chars[i] == s2_chars[i] {
                continue;
            }

            if let Some(neighbors) = self.keyboard_layout.get(&s1_chars[i]) {
                if neighbors.contains(&s2_chars[i]) {
                    adjacency_count += 1;
                }
            }
        }

        let differing_chars = edit_distance;

        if differing_chars == 0 {
            1.0
        } else {
            let keyboard_factor = adjacency_count as f64 / differing_chars as f64;
            let length_similarity = 1.0 - ((s1_chars.len() as isize - s2_chars.len() as isize).abs() as f64 / max(s1_chars.len(), s2_chars.len()) as f64);

            let base_similarity = 1.0 - (edit_distance as f64 / max(s1_chars.len(), s2_chars.len()) as f64);
            base_similarity * (1.0 + 0.3 * keyboard_factor) * length_similarity
        }
    }

    fn name(&self) -> &str {
        "KeyboardProximity"
    }
}

pub struct TokenKindMetric;

impl SimilarityMetric<Token, Token> for TokenKindMetric {
    fn calculate(&self, s1: &Token, s2: &Token) -> f64 {
        if s1.kind == s2.kind {
            0.2
        } else {
            0.0
        }
    }

    fn name(&self) -> &str {
        "TokenKind"
    }
}

impl SimilarityMetric<Token, Token> for SuffixMetric {
    fn calculate(&self, query: &Token, candidate: &Token) -> f64 {
        let query_lower = query.to_string().to_lowercase();
        let candidate_lower = candidate.to_string().to_lowercase();

        if candidate_lower.ends_with(&query_lower) {
            return 0.85 * (query_lower.len() as f64 / candidate_lower.len() as f64).min(1.0);
        } else if query_lower.ends_with(&candidate_lower) {
            return 0.85 * (candidate_lower.len() as f64 / query_lower.len() as f64).min(1.0)
        }

        0.0
    }

    fn name(&self) -> &str {
        "Suffix"
    }

    fn match_type(&self, query: &Token, candidate: &Token) -> Option<MatchType> {
        let score = self.calculate(query, candidate);
        if score > 0.0 {
            Some(MatchType::Similar("Suffix".to_string()))
        } else {
            None
        }
    }
}

pub trait Labeled<L> {
    fn name(&self) -> Option<L>;
}

impl Labeled<Token> for Expr {
    fn name(&self) -> Option<Token> {
        let Expr { kind, span } = self.clone();

        match kind {
            ExprKind::Literal(literal) => Some(literal),
            ExprKind::Identifier(identifier) => Some(Token {
                kind: TokenKind::Identifier(identifier),
                span,
            }),
            ExprKind::Constructor { name, .. } => name.name(),
            ExprKind::Labeled { label, .. } => label.name(),
            ExprKind::Index { expr, .. } => expr.name(),
            ExprKind::Invoke { target, .. } => target.name(),
            ExprKind::Member { object, .. } => object.name(),
            ExprKind::Item(item) => item.name(),
            ExprKind::Assignment { target, .. } => target.name(),
            _ => None,
        }
    }
}

impl Labeled<Token> for Item {
    fn name(&self) -> Option<Token> {
        let Item { kind, .. } = self.clone();
        kind.name()
    }
}

impl Labeled<Token> for ItemKind {
    fn name(&self) -> Option<Token> {
        match self {
            ItemKind::Expression(expr) => expr.name(),
            ItemKind::Trait { name, .. } => name.name(),
            ItemKind::Variable { target, .. } => target.name(),
            ItemKind::Field { name, .. } => name.name(),
            ItemKind::Structure { name, .. } => name.name(),
            ItemKind::Enum { name, .. } => name.name(),
            ItemKind::Macro { name, .. } => name.name(),
            ItemKind::Function { name, .. } => name.name(),
            _ => None,
        }
    }
}

impl SimilarityMetric<Expr, Item> for crate::axo_data::matcher::CaseInsensitiveMetric {
    fn calculate(&self, query: &Expr, candidate: &Item) -> f64 {
        match (query.name(), candidate.name()) {
            (Some(query_token), Some(candidate_token)) => {
                if query_token.to_string().to_lowercase() == candidate_token.to_string().to_lowercase() {
                    0.95
                } else {
                    0.0
                }
            }
            _ => 0.0,
        }
    }

    fn name(&self) -> &str {
        "CaseInsensitive"
    }
}

impl SimilarityMetric<Expr, Item> for crate::axo_data::matcher::PrefixMetric {
    fn calculate(&self, query: &Expr, candidate: &Item) -> f64 {
        match (query.name(), candidate.name()) {
            (Some(query_token), Some(candidate_token)) => {
                let query_lower = query_token.to_string().to_lowercase();
                let candidate_lower = candidate_token.to_string().to_lowercase();

                if candidate_lower.starts_with(&query_lower) {
                    0.9 * (query_lower.len() as f64 / candidate_lower.len() as f64).min(1.0)
                } else if query_lower.starts_with(&candidate_lower) {
                    0.9 * (candidate_lower.len() as f64 / query_lower.len() as f64).min(1.0)
                } else {
                    0.0
                }
            }
            _ => 0.0,
        }
    }

    fn name(&self) -> &str {
        "Prefix"
    }

    fn match_type(&self, query: &Expr, candidate: &Item) -> Option<MatchType> {
        let score = self.calculate(query, candidate);
        if score > 0.0 {
            Some(MatchType::Similar("Prefix".to_string()))
        } else {
            None
        }
    }
}

impl SimilarityMetric<Expr, Item> for crate::axo_data::matcher::SubstringMetric {
    fn calculate(&self, query: &Expr, candidate: &Item) -> f64 {
        match (query.name(), candidate.name()) {
            (Some(query_token), Some(candidate_token)) => {
                let query_lower = query_token.to_string().to_lowercase();
                let candidate_lower = candidate_token.to_string().to_lowercase();

                if candidate_lower.contains(&query_lower) {
                    0.8 * (query_lower.len() as f64 / candidate_lower.len() as f64).min(1.0)
                } else if query_lower.contains(&candidate_lower) {
                    0.8 * (candidate_lower.len() as f64 / query_lower.len() as f64).min(1.0)
                } else {
                    0.0
                }
            }
            _ => 0.0,
        }
    }

    fn name(&self) -> &str {
        "Substring"
    }
}

impl SimilarityMetric<Expr, Item> for crate::axo_data::matcher::EditDistanceMetric {
    fn calculate(&self, query: &Expr, candidate: &Item) -> f64 {
        match (query.name(), candidate.name()) {
            (Some(query_token), Some(candidate_token)) => {
                let distance = damerau_levenshtein_distance(&query_token.to_string(), &candidate_token.to_string());
                let max_len = max(query_token.to_string().len(), candidate_token.to_string().len());

                if max_len == 0 {
                    return 1.0;
                }

                1.0 - (distance as f64 / max_len as f64)
            }
            _ => 0.0,
        }
    }

    fn name(&self) -> &str {
        "EditDistance"
    }
}

impl SimilarityMetric<Expr, Item> for crate::axo_data::matcher::TokenSimilarityMetric {
    fn calculate(&self, query: &Expr, candidate: &Item) -> f64 {
        match (query.name(), candidate.name()) {
            (Some(query_token), Some(candidate_token)) => {
                let s1_lower = query_token.to_string().to_lowercase();
                let s2_lower = candidate_token.to_string().to_lowercase();

                let s1_tokens = self.split_on_separators(&s1_lower);
                let s2_tokens = self.split_on_separators(&s2_lower);

                self.token_similarity(&s1_tokens, &s2_tokens)
            }
            _ => 0.0,
        }
    }

    fn name(&self) -> &str {
        "TokenSimilarity"
    }
}

impl SimilarityMetric<Expr, Item> for crate::axo_data::matcher::AcronymMetric {
    fn calculate(&self, query: &Expr, candidate: &Item) -> f64 {
        match (query.name(), candidate.name()) {
            (Some(query_token), Some(candidate_token)) => {
                if query_token.to_string().len() > self.max_acronym_length {
                    return 0.0;
                }

                let query_lower = query_token.to_string().to_lowercase();
                let candidate_lower = candidate_token.to_string().to_lowercase();

                let tokens = self.token_metric.split_on_separators(&candidate_lower);

                if tokens.len() < query_lower.len() {
                    return 0.0;
                }

                let first_letters: String = tokens.iter()
                    .filter_map(|token| token.chars().next())
                    .collect();

                if first_letters.contains(&query_lower) {
                    return 0.75;
                }

                0.0
            }
            _ => 0.0,
        }
    }

    fn name(&self) -> &str {
        "Acronym"
    }

    fn match_type(&self, query: &Expr, candidate: &Item) -> Option<MatchType> {
        let score = self.calculate(query, candidate);
        if score > 0.0 {
            Some(MatchType::Similar("Acronym".to_string()))
        } else {
            None
        }
    }
}

impl SimilarityMetric<Expr, Item> for crate::axo_data::matcher::KeyboardProximityMetric {
    fn calculate(&self, query: &Expr, candidate: &Item) -> f64 {
        match (query.name(), candidate.name()) {
            (Some(query_token), Some(candidate_token)) => {
                let s1_lower = query_token.to_string().to_lowercase();
                let s2_lower = candidate_token.to_string().to_lowercase();

                if (s1_lower.len() as isize - s2_lower.len() as isize).abs() > 2 {
                    return 0.0;
                }

                let s1_chars: Vec<char> = s1_lower.chars().collect();
                let s2_chars: Vec<char> = s2_lower.chars().collect();

                let edit_distance = damerau_levenshtein_distance(&s1_lower, &s2_lower);

                if edit_distance > 3 {
                    return 0.0;
                }

                let mut adjacency_count = 0;
                let max_comparisons = min(s1_chars.len(), s2_chars.len());

                for i in 0..max_comparisons {
                    if s1_chars[i] == s2_chars[i] {
                        continue;
                    }

                    if let Some(neighbors) = self.keyboard_layout.get(&s1_chars[i]) {
                        if neighbors.contains(&s2_chars[i]) {
                            adjacency_count += 1;
                        }
                    }
                }

                let differing_chars = edit_distance;

                if differing_chars == 0 {
                    1.0
                } else {
                    let keyboard_factor = adjacency_count as f64 / differing_chars as f64;
                    let length_similarity = 1.0 - ((s1_chars.len() as isize - s2_chars.len() as isize).abs() as f64 / max(s1_chars.len(), s2_chars.len()) as f64);

                    let base_similarity = 1.0 - (edit_distance as f64 / max(s1_chars.len(), s2_chars.len()) as f64);
                    base_similarity * (1.0 + 0.3 * keyboard_factor) * length_similarity
                }
            }
            _ => 0.0,
        }
    }

    fn name(&self) -> &str {
        "KeyboardProximity"
    }
}

impl SimilarityMetric<Expr, Item> for SuffixMetric {
    fn calculate(&self, query: &Expr, candidate: &Item) -> f64 {
        match (query.name(), candidate.name()) {
            (Some(query_token), Some(candidate_token)) => {
                let query_lower = query_token.to_string().to_lowercase();
                let candidate_lower = candidate_token.to_string().to_lowercase();

                if candidate_lower.ends_with(&query_lower) {
                    0.85 * (query_lower.len() as f64 / candidate_lower.len() as f64).min(1.0)
                } else if query_lower.ends_with(&candidate_lower) {
                    0.85 * (candidate_lower.len() as f64 / query_lower.len() as f64).min(1.0)
                } else {
                    0.0
                }
            }
            _ => 0.0,
        }
    }

    fn name(&self) -> &str {
        "Suffix"
    }

    fn match_type(&self, query: &Expr, candidate: &Item) -> Option<MatchType> {
        let score = self.calculate(query, candidate);
        if score > 0.0 {
            Some(MatchType::Similar("Suffix".to_string()))
        } else {
            None
        }
    }
}

impl PartialEq<Item> for Expr {
    fn eq(&self, other: &Item) -> bool {
        let Expr { kind: expr_kind, span: expr_span} = self.clone();
        let Item { kind: item_kind, span: item_span} = other.clone();

        match (expr_kind, item_kind) {
            (ExprKind::Identifier(ident), ItemKind::Variable { target, .. }) => {
                if let Expr { kind: ExprKind::Identifier(target), .. } = *target {
                    ident == target
                } else {
                    false
                }
            }

            _ => false,
        }
    }
}